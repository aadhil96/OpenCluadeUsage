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
    // Approximate 5-hour session token limits (input + output + cache_creation only;
    // cache reads don't count toward Anthropic's quota).
    pub fn session_limit(&self) -> Option<u64> {
        match self {
            Plan::Pro => Some(1_200_000),
            Plan::Max5x => Some(6_000_000),
            Plan::Max20x => Some(24_000_000),
            Plan::None => None,
        }
    }

    // Approximate 7-day weekly token limits.
    pub fn weekly_limit(&self) -> Option<u64> {
        match self {
            Plan::Pro => Some(8_000_000),
            Plan::Max5x => Some(40_000_000),
            Plan::Max20x => Some(160_000_000),
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
