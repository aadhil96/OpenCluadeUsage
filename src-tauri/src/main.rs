#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod pricing;
mod session;
mod settings;
mod usage_parser;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};

use session::UsageSummary;
use settings::Settings;
use usage_parser::{FileCache, UsageEntry};

// ---- App state --------------------------------------------------------------

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub file_cache: Mutex<FileCache>,
    pub entry_cache: Mutex<HashMap<std::path::PathBuf, Vec<UsageEntry>>>,
    pub last_summary: Mutex<Option<UsageSummary>>,
    // Updated by toggle_popup each time the panel is shown. The on_window_event
    // blur handler only hides the window if 600ms have passed since last show,
    // which filters out the spurious macOS blur that fires on tray clicks.
    pub last_shown_at: Mutex<Instant>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            settings: Mutex::new(settings::load()),
            file_cache: Mutex::new(HashMap::new()),
            entry_cache: Mutex::new(HashMap::new()),
            last_summary: Mutex::new(None),
            last_shown_at: Mutex::new(Instant::now()),
        }
    }
}

// ---- Tauri commands ---------------------------------------------------------

#[tauri::command]
fn get_usage(state: State<'_, AppState>) -> Result<UsageSummary, String> {
    let settings = state.settings.lock().unwrap().clone();
    let summary = {
        let mut file_cache = state.file_cache.lock().unwrap();
        let mut entry_cache = state.entry_cache.lock().unwrap();
        let mut new_file_cache = FileCache::new();
        let entries =
            usage_parser::scan_all(&file_cache, &mut new_file_cache, &mut entry_cache);
        *file_cache = new_file_cache;
        session::aggregate(&entries, &settings)
    };
    *state.last_summary.lock().unwrap() = Some(summary.clone());
    Ok(summary)
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn save_settings(
    settings: Settings,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    crate::settings::save(&settings).map_err(|e| e.to_string())?;

    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        let autostart = app.autolaunch();
        if settings.launch_at_login {
            autostart.enable().map_err(|e| e.to_string())?;
        } else {
            autostart.disable().map_err(|e| e.to_string())?;
        }
    }

    *state.settings.lock().unwrap() = settings;
    Ok(())
}

// ---- Background polling task ------------------------------------------------

fn poll_and_update_badge(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    let entries = {
        let mut file_cache = state.file_cache.lock().unwrap();
        let mut entry_cache = state.entry_cache.lock().unwrap();
        let mut new_file_cache = FileCache::new();
        let e = usage_parser::scan_all(&file_cache, &mut new_file_cache, &mut entry_cache);
        *file_cache = new_file_cache;
        e
    };
    let summary = session::aggregate(&entries, &settings);
    let badge = session::tray_badge(&summary);
    *state.last_summary.lock().unwrap() = Some(summary.clone());
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_title(Some(&badge));
    }
    // Notify any open frontend window so it can re-render without polling.
    let _ = app.emit("usage-updated", summary);
}

fn spawn_poller(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Run immediately on startup so the tray badge is populated at launch.
        poll_and_update_badge(&app);
        loop {
            let interval = {
                let state = app.state::<AppState>();
                let val = state.settings.lock().unwrap().refresh_interval_secs;
                val
            };
            tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            poll_and_update_badge(&app);
        }
    });
}

// ---- Entry point ------------------------------------------------------------

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![get_usage, get_settings, save_settings])
        .setup(|app| {
            let icon = app
                .default_window_icon()
                .cloned()
                .map(|img| img.to_owned())
                .expect("no app icon found");

            let _tray = TrayIconBuilder::with_id("main")
                .icon(icon)
                .icon_as_template(true)
                .title("—")
                .tooltip("ClaudeUsage — Claude Code token usage")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, button_state, position, .. } = event {
                        if button == MouseButton::Left && button_state == MouseButtonState::Up {
                            toggle_popup(tray.app_handle(), position.x, position.y);
                        }
                    }
                })
                .build(app)?;

            // Hide the panel when it loses focus. We only hide if the window
            // has been visible for at least 600ms — this filters out the
            // spurious blur that macOS fires during the tray-click animation.
            let win = app.get_webview_window("main")
                .expect("main window not found");
            let win_for_blur = win.clone();
            let app_for_blur = app.handle().clone();
            win.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let state = app_for_blur.state::<AppState>();
                    let elapsed = state.last_shown_at.lock().unwrap().elapsed();
                    if elapsed.as_millis() > 600 {
                        let _ = win_for_blur.hide();
                    }
                }
            });

            spawn_poller(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ---- Window positioning -----------------------------------------------------

fn toggle_popup(app: &AppHandle, tray_x: f64, tray_y: f64) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };

    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
        return;
    }

    // Window size matches tauri.conf.json — these are LOGICAL points.
    const WIN_W: f64 = 340.0;
    const WIN_H: f64 = 510.0;
    const GAP: f64 = 8.0;

    // tray_x/tray_y are PHYSICAL pixels. Pick the monitor that contains them,
    // then convert everything to logical units so the math lines up with the
    // window's logical size.
    let monitor = win.available_monitors().ok().and_then(|monitors| {
        monitors.into_iter().find(|m| {
            let pos = m.position();
            let sz = m.size();
            let x0 = pos.x as f64;
            let y0 = pos.y as f64;
            let x1 = x0 + sz.width as f64;
            let y1 = y0 + sz.height as f64;
            tray_x >= x0 && tray_x < x1 && tray_y >= y0 && tray_y < y1
        })
    });

    let (mon_x, mon_y, screen_w, screen_h, scale) = match monitor {
        Some(m) => {
            let pos = m.position();
            let sz = m.size();
            let scale = m.scale_factor();
            (
                pos.x as f64 / scale,
                pos.y as f64 / scale,
                sz.width as f64 / scale,
                sz.height as f64 / scale,
                scale,
            )
        }
        None => (0.0, 0.0, 1440.0, 900.0, 2.0),
    };
    let tray_x_log = tray_x / scale;
    let tray_y_log = tray_y / scale;

    let x = (tray_x_log - WIN_W / 2.0)
        .max(mon_x)
        .min(mon_x + screen_w - WIN_W);
    let y = (tray_y_log + GAP).min(mon_y + screen_h - WIN_H - GAP);

    let _ = win.set_position(tauri::LogicalPosition { x, y });

    // Stamp the time before showing so the blur handler's 600ms guard is
    // measured from this exact moment.
    *app.state::<AppState>().last_shown_at.lock().unwrap() = Instant::now();

    let _ = win.show();
    let _ = win.set_focus();
}
