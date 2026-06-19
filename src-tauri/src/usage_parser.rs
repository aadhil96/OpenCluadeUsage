use chrono::{DateTime, Utc};
use glob::glob;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct UsageEntry {
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub message_id: Option<String>,
    pub is_sidechain: bool,
}

impl UsageEntry {
    /// Tokens that count toward Anthropic's rate limit.
    /// Cache reads are excluded — they're cheap context pulls and don't
    /// count against the session or weekly quota.
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens
    }
}

// ---- Raw JSON shapes --------------------------------------------------------

#[derive(Deserialize)]
struct RawLine {
    #[serde(rename = "type")]
    entry_type: Option<String>,
    timestamp: Option<String>,
    message: Option<RawMessage>,
    #[serde(rename = "isSidechain")]
    is_sidechain: Option<bool>,
    #[serde(rename = "isApiErrorMessage")]
    is_api_error_message: Option<bool>,
}

#[derive(Deserialize)]
struct RawMessage {
    id: Option<String>,
    model: Option<String>,
    usage: Option<RawUsage>,
}

#[derive(Deserialize)]
struct RawUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cache_creation_input_tokens: Option<u64>,
    cache_read_input_tokens: Option<u64>,
    // Newer Claude Code splits cache_creation into time-bucketed fields.
    // When present use these instead of cache_creation_input_tokens.
    cache_creation: Option<RawCacheCreation>,
}

#[derive(Deserialize)]
struct RawCacheCreation {
    ephemeral_5m_input_tokens: Option<u64>,
    ephemeral_1h_input_tokens: Option<u64>,
}

// ---- File-level cache -------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub mtime: SystemTime,
    pub size: u64,
}

pub type FileCache = HashMap<PathBuf, FileMetadata>;

pub fn file_is_changed(path: &PathBuf, cache: &FileCache) -> bool {
    let Ok(meta) = fs::metadata(path) else { return true };
    let Ok(mtime) = meta.modified() else { return true };
    match cache.get(path) {
        Some(cached) => cached.mtime != mtime || cached.size != meta.len(),
        None => true,
    }
}

pub fn cache_entry(path: &PathBuf) -> Option<(PathBuf, FileMetadata)> {
    let meta = fs::metadata(path).ok()?;
    let mtime = meta.modified().ok()?;
    Some((path.clone(), FileMetadata { mtime, size: meta.len() }))
}

// ---- Scanning ---------------------------------------------------------------

fn claude_project_dirs() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".claude").join("projects"));
        paths.push(home.join(".config").join("claude").join("projects"));
    }
    paths
}

pub fn find_jsonl_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    for base in claude_project_dirs() {
        let pattern = base.join("**").join("*.jsonl");
        if let Ok(paths) = glob(&pattern.to_string_lossy()) {
            for path in paths.flatten() {
                // Canonicalize so symlinked project dirs (e.g. ~/.config/claude
                // pointing at ~/.claude) don't yield duplicate entries.
                let key = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
                if seen.insert(key) {
                    files.push(path);
                }
            }
        }
    }
    files
}

pub fn parse_file(path: &PathBuf) -> Vec<UsageEntry> {
    let Ok(file) = File::open(path) else { return vec![] };
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let Ok(line) = line else { continue };
        let line = line.trim();
        if line.is_empty() || !line.contains("\"usage\"") { continue }

        let Ok(raw): Result<RawLine, _> = serde_json::from_str(line) else { continue };
        if raw.entry_type.as_deref() != Some("assistant") { continue }
        if raw.is_api_error_message == Some(true) { continue }

        let Some(ts_str) = raw.timestamp else { continue };
        let Ok(timestamp) = ts_str.parse::<DateTime<Utc>>() else { continue };

        let Some(msg) = raw.message else { continue };
        let Some(usage) = msg.usage else { continue };
        if usage.input_tokens.is_none() && usage.output_tokens.is_none() { continue }

        let cache_creation_tokens = usage.cache_creation
            .as_ref()
            .map(|cc| {
                cc.ephemeral_5m_input_tokens.unwrap_or(0)
                    + cc.ephemeral_1h_input_tokens.unwrap_or(0)
            })
            .unwrap_or_else(|| usage.cache_creation_input_tokens.unwrap_or(0));

        entries.push(UsageEntry {
            timestamp,
            model: msg.model.unwrap_or_else(|| "unknown".to_string()),
            message_id: msg.id,
            is_sidechain: raw.is_sidechain.unwrap_or(false),
            input_tokens: usage.input_tokens.unwrap_or(0),
            output_tokens: usage.output_tokens.unwrap_or(0),
            cache_creation_tokens,
            cache_read_tokens: usage.cache_read_input_tokens.unwrap_or(0),
        });
    }

    entries
}

pub fn scan_all(
    prev_cache: &FileCache,
    new_cache: &mut FileCache,
    entry_cache: &mut HashMap<PathBuf, Vec<UsageEntry>>,
) -> Vec<UsageEntry> {
    let files = find_jsonl_files();
    let file_set: HashSet<&PathBuf> = files.iter().collect();

    for path in &files {
        if file_is_changed(path, prev_cache) {
            entry_cache.insert(path.clone(), parse_file(path));
        }
        if let Some(meta) = cache_entry(path) {
            new_cache.insert(meta.0, meta.1);
        }
    }

    entry_cache.retain(|p, _| file_set.contains(p));

    let mut all: Vec<UsageEntry> = entry_cache.values().flatten().cloned().collect();
    all.sort_by_key(|e| e.timestamp);

    // Deduplicate by message_id. The same API call can appear multiple times
    // (duplicate lines in a file, or the same message in both a session file
    // and a subagent file). When two entries share an ID, prefer the
    // non-sidechain version; otherwise keep the first seen (oldest timestamp).
    // Entries without an ID can't be deduplicated and pass through verbatim.
    let mut by_id: HashMap<String, UsageEntry> = HashMap::new();
    let mut no_id: Vec<UsageEntry> = Vec::new();

    for entry in all {
        match &entry.message_id {
            None => no_id.push(entry),
            Some(id) => match by_id.get(id) {
                None => {
                    by_id.insert(id.clone(), entry);
                }
                Some(existing) => {
                    // Replace only when upgrading a sidechain copy to a
                    // non-sidechain one. Otherwise keep the existing (older).
                    if existing.is_sidechain && !entry.is_sidechain {
                        by_id.insert(id.clone(), entry);
                    }
                }
            },
        }
    }

    let mut result: Vec<UsageEntry> = by_id.into_values().chain(no_id).collect();
    result.sort_by_key(|e| e.timestamp);
    result
}
