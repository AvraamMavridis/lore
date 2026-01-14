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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thought_object_new() {
        let thought = ThoughtObject::new(
            "src/main.rs".to_string(),
            "abc123".to_string(),
            "test-agent".to_string(),
            "Test intent".to_string(),
            "Test reasoning".to_string(),
        );

        assert_eq!(thought.target_file, "src/main.rs");
        assert_eq!(thought.file_hash, "abc123");
        assert_eq!(thought.agent_id, "test-agent");
        assert_eq!(thought.intent, "Test intent");
        assert_eq!(thought.reasoning_trace, "Test reasoning");
        assert!(thought.line_range.is_none());
        assert!(thought.commit_hash.is_none());
        assert!(thought.rejected_alternatives.is_empty());
        assert!(thought.tags.is_empty());
        assert!(!thought.id.is_empty());
    }

    #[test]
    fn test_thought_object_with_line_range() {
        let thought = ThoughtObject::new(
            "src/main.rs".to_string(),
            "abc123".to_string(),
            "test-agent".to_string(),
            "Test".to_string(),
            "Reasoning".to_string(),
        )
        .with_line_range(10, 50);

        assert_eq!(thought.line_range, Some((10, 50)));
    }

    #[test]
    fn test_thought_object_with_commit() {
        let thought = ThoughtObject::new(
            "src/main.rs".to_string(),
            "abc123".to_string(),
            "test-agent".to_string(),
            "Test".to_string(),
            "Reasoning".to_string(),
        )
        .with_commit("deadbeef".to_string());

        assert_eq!(thought.commit_hash, Some("deadbeef".to_string()));
    }

    #[test]
    fn test_thought_object_with_rejected() {
        let alternatives = vec![
            RejectedAlternative {
                name: "Option A".to_string(),
                reason: Some("Too slow".to_string()),
            },
            RejectedAlternative {
                name: "Option B".to_string(),
                reason: None,
            },
        ];

        let thought = ThoughtObject::new(
            "src/main.rs".to_string(),
            "abc123".to_string(),
            "test-agent".to_string(),
            "Test".to_string(),
            "Reasoning".to_string(),
        )
        .with_rejected(alternatives);

        assert_eq!(thought.rejected_alternatives.len(), 2);
        assert_eq!(thought.rejected_alternatives[0].name, "Option A");
        assert_eq!(
            thought.rejected_alternatives[0].reason,
            Some("Too slow".to_string())
        );
        assert_eq!(thought.rejected_alternatives[1].name, "Option B");
        assert!(thought.rejected_alternatives[1].reason.is_none());
    }

    #[test]
    fn test_thought_object_with_tags() {
        let thought = ThoughtObject::new(
            "src/main.rs".to_string(),
            "abc123".to_string(),
            "test-agent".to_string(),
            "Test".to_string(),
            "Reasoning".to_string(),
        )
        .with_tags(vec!["auth".to_string(), "security".to_string()]);

        assert_eq!(thought.tags, vec!["auth", "security"]);
    }

    #[test]
    fn test_thought_object_builder_chain() {
        let thought = ThoughtObject::new(
            "src/main.rs".to_string(),
            "abc123".to_string(),
            "test-agent".to_string(),
            "Test".to_string(),
            "Reasoning".to_string(),
        )
        .with_line_range(1, 10)
        .with_commit("abc".to_string())
        .with_tags(vec!["tag1".to_string()]);

        assert_eq!(thought.line_range, Some((1, 10)));
        assert_eq!(thought.commit_hash, Some("abc".to_string()));
        assert_eq!(thought.tags, vec!["tag1"]);
    }

    #[test]
    fn test_thought_object_serialization() {
        let thought = ThoughtObject::new(
            "src/main.rs".to_string(),
            "abc123".to_string(),
            "test-agent".to_string(),
            "Test intent".to_string(),
            "Test reasoning".to_string(),
        );

        let json = serde_json::to_string(&thought).unwrap();
        let deserialized: ThoughtObject = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.target_file, thought.target_file);
        assert_eq!(deserialized.file_hash, thought.file_hash);
        assert_eq!(deserialized.agent_id, thought.agent_id);
        assert_eq!(deserialized.intent, thought.intent);
        assert_eq!(deserialized.reasoning_trace, thought.reasoning_trace);
    }

    #[test]
    fn test_lore_index_new() {
        let index = LoreIndex::new();
        assert!(index.files.is_empty());
        assert_eq!(index.entry_count, 0);
    }

    #[test]
    fn test_lore_index_add_entry() {
        let mut index = LoreIndex::new();
        index.add_entry("src/main.rs", "entry-1");

        assert_eq!(index.entry_count, 1);
        assert_eq!(
            index.get_entries_for_file("src/main.rs"),
            Some(&vec!["entry-1".to_string()])
        );
    }

    #[test]
    fn test_lore_index_add_multiple_entries_same_file() {
        let mut index = LoreIndex::new();
        index.add_entry("src/main.rs", "entry-1");
        index.add_entry("src/main.rs", "entry-2");

        assert_eq!(index.entry_count, 2);
        let entries = index.get_entries_for_file("src/main.rs").unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&"entry-1".to_string()));
        assert!(entries.contains(&"entry-2".to_string()));
    }

    #[test]
    fn test_lore_index_add_entries_different_files() {
        let mut index = LoreIndex::new();
        index.add_entry("src/main.rs", "entry-1");
        index.add_entry("src/lib.rs", "entry-2");

        assert_eq!(index.entry_count, 2);
        assert_eq!(
            index.get_entries_for_file("src/main.rs"),
            Some(&vec!["entry-1".to_string()])
        );
        assert_eq!(
            index.get_entries_for_file("src/lib.rs"),
            Some(&vec!["entry-2".to_string()])
        );
    }

    #[test]
    fn test_lore_index_get_entries_nonexistent_file() {
        let index = LoreIndex::new();
        assert!(index.get_entries_for_file("nonexistent.rs").is_none());
    }

    #[test]
    fn test_lore_index_serialization() {
        let mut index = LoreIndex::new();
        index.add_entry("src/main.rs", "entry-1");
        index.add_entry("src/lib.rs", "entry-2");

        let json = serde_json::to_string(&index).unwrap();
        let deserialized: LoreIndex = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.entry_count, 2);
        assert_eq!(
            deserialized.get_entries_for_file("src/main.rs"),
            Some(&vec!["entry-1".to_string()])
        );
    }

    #[test]
    fn test_rejected_alternative_with_reason() {
        let alt = RejectedAlternative {
            name: "Option A".to_string(),
            reason: Some("Performance issues".to_string()),
        };

        assert_eq!(alt.name, "Option A");
        assert_eq!(alt.reason, Some("Performance issues".to_string()));
    }

    #[test]
    fn test_rejected_alternative_without_reason() {
        let alt = RejectedAlternative {
            name: "Option B".to_string(),
            reason: None,
        };

        assert_eq!(alt.name, "Option B");
        assert!(alt.reason.is_none());
    }

    #[test]
    fn test_rejected_alternative_serialization() {
        let alt = RejectedAlternative {
            name: "Test".to_string(),
            reason: Some("Reason".to_string()),
        };

        let json = serde_json::to_string(&alt).unwrap();
        let deserialized: RejectedAlternative = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, alt.name);
        assert_eq!(deserialized.reason, alt.reason);
    }
}
