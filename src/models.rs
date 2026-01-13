use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A ThoughtObject represents the reasoning context behind a code change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtObject {
    /// Unique identifier for this entry
    pub id: String,

    /// The file this reasoning applies to
    pub target_file: String,

    /// Optional line range [start, end] if reasoning applies to specific lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_range: Option<(usize, usize)>,

    /// SHA256 hash of the file content at time of recording
    pub file_hash: String,

    /// Git commit hash this reasoning is associated with (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,

    /// Identifier for the agent/author that created this entry
    pub agent_id: String,

    /// When this entry was created
    pub timestamp: DateTime<Utc>,

    /// Brief description of the intent/purpose
    pub intent: String,

    /// Full reasoning trace - can be extensive chain-of-thought
    pub reasoning_trace: String,

    /// Alternatives that were considered but rejected
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_alternatives: Vec<RejectedAlternative>,

    /// Optional tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// A rejected alternative with optional reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedAlternative {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl ThoughtObject {
    pub fn new(
        target_file: String,
        file_hash: String,
        agent_id: String,
        intent: String,
        reasoning_trace: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            target_file,
            line_range: None,
            file_hash,
            commit_hash: None,
            agent_id,
            timestamp: Utc::now(),
            intent,
            reasoning_trace,
            rejected_alternatives: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_line_range(mut self, start: usize, end: usize) -> Self {
        self.line_range = Some((start, end));
        self
    }

    pub fn with_commit(mut self, commit_hash: String) -> Self {
        self.commit_hash = Some(commit_hash);
        self
    }

    pub fn with_rejected(mut self, alternatives: Vec<RejectedAlternative>) -> Self {
        self.rejected_alternatives = alternatives;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Index entry for quick lookups by file path
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoreIndex {
    /// Map of file paths to their entry IDs
    pub files: std::collections::HashMap<String, Vec<String>>,

    /// Total number of entries
    pub entry_count: usize,
}

impl LoreIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(&mut self, file_path: &str, entry_id: &str) {
        self.files
            .entry(file_path.to_string())
            .or_default()
            .push(entry_id.to_string());
        self.entry_count += 1;
    }

    pub fn get_entries_for_file(&self, file_path: &str) -> Option<&Vec<String>> {
        self.files.get(file_path)
    }
}
