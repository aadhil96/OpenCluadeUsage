// Session and weekly aggregation over raw UsageEntry slices.
//
// Session window: rolling 5 hours (matches Anthropic's Claude Code rate-limit window).
//   "Resets in X" = time until the oldest entry in the current window falls off.
// Weekly window: rolling 7 days.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::pricing::{model_display_name, model_family, pricing_for_model};
use crate::settings::Settings;
use crate::usage_parser::UsageEntry;

const SESSION_HOURS: i64 = 5;
const WEEKLY_DAYS: i64 = 7;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_tokens: u64,
}

impl TokenUsage {
    fn zero() -> Self {
        TokenUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            total_tokens: 0,
        }
    }

    fn add(&mut self, e: &UsageEntry) {
        self.input_tokens += e.input_tokens;
        self.output_tokens += e.output_tokens;
        self.cache_creation_tokens += e.cache_creation_tokens;
        self.cache_read_tokens += e.cache_read_tokens;
        self.total_tokens += e.total_tokens();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUsage {
    #[serde(flatten)]
    pub tokens: TokenUsage,
    pub session_start: Option<String>,
    pub session_end: Option<String>,
    pub minutes_until_reset: Option<i64>,
    pub limit: Option<u64>,
    pub percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyUsage {
    #[serde(flatten)]
    pub tokens: TokenUsage,
    pub week_start: String,
    pub week_end: String,
    pub limit: Option<u64>,
    pub percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub model: String,
    pub display_name: String,
    pub model_family: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub session_cost_usd: f64,
    pub weekly_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub session: SessionUsage,
    pub weekly: WeeklyUsage,
    pub model_breakdown: Vec<ModelUsage>,
    pub cost_estimate: CostEstimate,
    pub last_updated: String,
    pub plan: Option<String>,
}

pub fn aggregate(entries: &[UsageEntry], settings: &Settings) -> UsageSummary {
    let now = Utc::now();
    let session_cutoff = now - Duration::hours(SESSION_HOURS);
    let weekly_cutoff = now - Duration::days(WEEKLY_DAYS);

    let session_entries: Vec<&UsageEntry> = entries
        .iter()
        .filter(|e| e.timestamp >= session_cutoff)
        .collect();

    let weekly_entries: Vec<&UsageEntry> = entries
        .iter()
        .filter(|e| e.timestamp >= weekly_cutoff)
        .collect();

    // --- Session ---
    let mut session_tokens = TokenUsage::zero();
    for e in &session_entries {
        session_tokens.add(*e);
    }

    let oldest_in_session: Option<DateTime<Utc>> =
        session_entries.iter().map(|e| e.timestamp).min();

    let (session_start, session_end, minutes_until_reset) =
        if let Some(oldest) = oldest_in_session {
            let reset_at = oldest + Duration::hours(SESSION_HOURS);
            let remaining = (reset_at - now).num_minutes().max(0);
            (
                Some(oldest.to_rfc3339()),
                Some(reset_at.to_rfc3339()),
                Some(remaining),
            )
        } else {
            (None, None, None)
        };

    let session_limit = settings.effective_session_limit();
    let session_pct = session_limit.map(|lim| {
        (session_tokens.total_tokens as f64 / lim as f64 * 100.0).min(100.0)
    });

    let session = SessionUsage {
        tokens: session_tokens.clone(),
        session_start,
        session_end,
        minutes_until_reset,
        limit: session_limit,
        percentage: session_pct,
    };

    // --- Weekly ---
    let mut weekly_tokens = TokenUsage::zero();
    for e in &weekly_entries {
        weekly_tokens.add(*e);
    }

    let oldest_in_week: Option<DateTime<Utc>> =
        weekly_entries.iter().map(|e| e.timestamp).min();

    let week_start = weekly_cutoff.to_rfc3339();
    // "Resets" = when the oldest weekly entry falls off the rolling window.
    let week_end = oldest_in_week
        .map(|oldest| (oldest + Duration::days(WEEKLY_DAYS)).to_rfc3339())
        .unwrap_or_else(|| now.to_rfc3339());

    let weekly_limit = settings.effective_weekly_limit();
    let weekly_pct = weekly_limit.map(|lim| {
        (weekly_tokens.total_tokens as f64 / lim as f64 * 100.0).min(100.0)
    });

    let weekly = WeeklyUsage {
        tokens: weekly_tokens,
        week_start,
        week_end,
        limit: weekly_limit,
        percentage: weekly_pct,
    };

    // --- Model breakdown (session only) ---
    let mut model_map: HashMap<String, TokenUsage> = HashMap::new();
    for e in &session_entries {
        model_map.entry(e.model.clone()).or_insert_with(TokenUsage::zero).add(*e);
    }

    let mut session_cost = 0.0;
    let mut model_breakdown: Vec<ModelUsage> = model_map
        .into_iter()
        .map(|(model, tokens)| {
            let pricing = pricing_for_model(&model);
            let cost = pricing.cost_for(
                tokens.input_tokens,
                tokens.output_tokens,
                tokens.cache_creation_tokens,
                tokens.cache_read_tokens,
            );
            session_cost += cost;
            ModelUsage {
                display_name: model_display_name(&model),
                model_family: model_family(&model).to_string(),
                input_tokens: tokens.input_tokens,
                output_tokens: tokens.output_tokens,
                cache_creation_tokens: tokens.cache_creation_tokens,
                cache_read_tokens: tokens.cache_read_tokens,
                total_tokens: tokens.total_tokens,
                cost_usd: cost,
                model,
            }
        })
        .collect();

    // Sort by token count descending
    model_breakdown.sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));

    // --- Weekly cost ---
    let mut weekly_cost = 0.0;
    let mut weekly_by_model: HashMap<String, TokenUsage> = HashMap::new();
    for e in &weekly_entries {
        weekly_by_model
            .entry(e.model.clone())
            .or_insert_with(TokenUsage::zero)
            .add(*e);
    }
    for (model, tokens) in &weekly_by_model {
        let pricing = pricing_for_model(model);
        weekly_cost += pricing.cost_for(
            tokens.input_tokens,
            tokens.output_tokens,
            tokens.cache_creation_tokens,
            tokens.cache_read_tokens,
        );
    }

    let plan_label = match &settings.plan {
        crate::settings::Plan::Pro => Some("pro".to_string()),
        crate::settings::Plan::Max5x => Some("max5x".to_string()),
        crate::settings::Plan::Max20x => Some("max20x".to_string()),
        crate::settings::Plan::None => None,
    };

    UsageSummary {
        session,
        weekly,
        model_breakdown,
        cost_estimate: CostEstimate {
            session_cost_usd: session_cost,
            weekly_cost_usd: weekly_cost,
        },
        last_updated: now.to_rfc3339(),
        plan: plan_label,
    }
}

/// Returns a concise tray badge string, e.g. "42%" or "84k".
pub fn tray_badge(summary: &UsageSummary) -> String {
    if let Some(pct) = summary.session.percentage {
        format!("{:.0}%", pct)
    } else {
        let t = summary.session.tokens.total_tokens;
        if t >= 1_000_000 {
            format!("{:.1}M", t as f64 / 1_000_000.0)
        } else if t >= 1_000 {
            format!("{}k", t / 1_000)
        } else {
            format!("{}", t)
        }
    }
}
