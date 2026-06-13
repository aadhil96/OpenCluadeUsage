// Anthropic API pricing per million tokens (USD).
// Update these constants as Anthropic adjusts pricing.
// Source: https://www.anthropic.com/pricing

pub struct ModelPricing {
    pub input_per_mtok: f64,
    pub output_per_mtok: f64,
    pub cache_write_per_mtok: f64,
    pub cache_read_per_mtok: f64,
}

impl ModelPricing {
    pub fn cost_for(
        &self,
        input: u64,
        output: u64,
        cache_write: u64,
        cache_read: u64,
    ) -> f64 {
        let to_m = |n: u64| n as f64 / 1_000_000.0;
        to_m(input) * self.input_per_mtok
            + to_m(output) * self.output_per_mtok
            + to_m(cache_write) * self.cache_write_per_mtok
            + to_m(cache_read) * self.cache_read_per_mtok
    }
}

// Opus 4 / claude-opus-*
pub const OPUS: ModelPricing = ModelPricing {
    input_per_mtok: 15.0,
    output_per_mtok: 75.0,
    cache_write_per_mtok: 18.75,
    cache_read_per_mtok: 1.50,
};

// Sonnet 4.x / claude-sonnet-* / claude-3-*-sonnet-*
pub const SONNET: ModelPricing = ModelPricing {
    input_per_mtok: 3.0,
    output_per_mtok: 15.0,
    cache_write_per_mtok: 3.75,
    cache_read_per_mtok: 0.30,
};

// Haiku 4.x / claude-haiku-* / claude-3-*-haiku-*
pub const HAIKU: ModelPricing = ModelPricing {
    input_per_mtok: 0.80,
    output_per_mtok: 4.0,
    cache_write_per_mtok: 1.0,
    cache_read_per_mtok: 0.08,
};

pub fn pricing_for_model(model: &str) -> &'static ModelPricing {
    let m = model.to_lowercase();
    if m.contains("opus") {
        &OPUS
    } else if m.contains("haiku") {
        &HAIKU
    } else {
        // Sonnet is the default / fallback
        &SONNET
    }
}

pub fn model_family(model: &str) -> &'static str {
    let m = model.to_lowercase();
    if m.contains("opus") {
        "opus"
    } else if m.contains("haiku") {
        "haiku"
    } else {
        "sonnet"
    }
}

pub fn model_display_name(model: &str) -> String {
    let m = model.to_lowercase();
    // Normalise common model IDs to friendly names
    let family = if m.contains("opus") {
        "Opus"
    } else if m.contains("haiku") {
        "Haiku"
    } else {
        "Sonnet"
    };

    // Extract version suffix (e.g. "4", "4-5", "3-7")
    let version = extract_version_suffix(model);
    if version.is_empty() {
        family.to_string()
    } else {
        format!("{} {}", family, version)
    }
}

fn extract_version_suffix(model: &str) -> String {
    // claude-opus-4          -> "4"
    // claude-sonnet-4-5      -> "4.5"
    // claude-haiku-4-5       -> "4.5"
    // claude-3-5-sonnet-...  -> "3.5"
    // claude-3-7-sonnet-...  -> "3.7"
    let m = model.to_lowercase();
    let m = m.trim_start_matches("claude-");

    // Remove family name prefix
    let m = m
        .trim_start_matches("opus-")
        .trim_start_matches("sonnet-")
        .trim_start_matches("haiku-");

    // The remaining string starts with the version digits
    // Take up to the first non-version character
    let mut version = String::new();
    let mut parts = m.split('-');
    if let Some(major) = parts.next() {
        if major.chars().all(|c| c.is_ascii_digit()) {
            version.push_str(major);
            if let Some(minor) = parts.next() {
                if minor.chars().all(|c| c.is_ascii_digit()) {
                    version.push('.');
                    version.push_str(minor);
                }
            }
        }
    }
    version
}
