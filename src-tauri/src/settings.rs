use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Plan {
    #[serde(rename = "pro")]
    Pro,
    #[serde(rename = "max5x")]
    Max5x,
    #[serde(rename = "max20x")]
    Max20x,
    #[serde(rename = "none")]
    None,
}

impl Default for Plan {
    fn default() -> Self {
        Plan::None
    }
}

impl Plan {
    // Approximate 5-hour session token budgets, calibrated against the
    // percentages Claude's own Settings → Usage panel shows. These are best
    // guesses — Anthropic doesn't publish exact formulas, and the real budget
    // is based on weighted compute units that vary per model. If the numbers
    // don't match what you see in claude.ai, set a custom_session_limit
    // (Settings → Custom Limits) to override.
    pub fn session_limit(&self) -> Option<u64> {
        match self {
            Plan::Pro => Some(5_000_000),
            Plan::Max5x => Some(25_000_000),
            Plan::Max20x => Some(100_000_000),
            Plan::None => None,
        }
    }

    // Approximate weekly token budgets (calendar week, resets ~Thu 03:00 local).
    pub fn weekly_limit(&self) -> Option<u64> {
        match self {
            Plan::Pro => Some(115_000_000),
            Plan::Max5x => Some(575_000_000),
            Plan::Max20x => Some(2_300_000_000),
            Plan::None => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub plan: Plan,
    #[serde(default = "default_refresh_secs")]
    pub refresh_interval_secs: u64,
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default)]
    pub custom_session_limit: Option<u64>,
    #[serde(default)]
    pub custom_weekly_limit: Option<u64>,
}

fn default_refresh_secs() -> u64 {
    30
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            plan: Plan::None,
            refresh_interval_secs: 30,
            launch_at_login: false,
            custom_session_limit: None,
            custom_weekly_limit: None,
        }
    }
}

impl Settings {
    pub fn effective_session_limit(&self) -> Option<u64> {
        self.custom_session_limit.or_else(|| self.plan.session_limit())
    }

    pub fn effective_weekly_limit(&self) -> Option<u64> {
        self.custom_weekly_limit.or_else(|| self.plan.weekly_limit())
    }
}

fn settings_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        })
        .join("ClaudeUsage")
        .join("settings.json")
}

pub fn load() -> Settings {
    let path = settings_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(settings: &Settings) -> Result<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(&path, json)?;
    Ok(())
}
