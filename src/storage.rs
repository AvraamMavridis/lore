use crate::models::{LoreIndex, ThoughtObject};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Lore not initialized. Run 'lore init' first.")]
    NotInitialized,

    #[error("Lore already initialized")]
    AlreadyInitialized,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

const LORE_DIR: &str = ".lore";
const ENTRIES_DIR: &str = "entries";
const INDEX_FILE: &str = "index.json";
const CONFIG_FILE: &str = "config.json";

/// Storage handler for Lore data
pub struct LoreStorage {
    root: PathBuf,
}

impl LoreStorage {
    /// Create a new storage handler at the given root path
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Get the .lore directory path
    fn lore_dir(&self) -> PathBuf {
        self.root.join(LORE_DIR)
    }

    /// Get the entries directory path
    fn entries_dir(&self) -> PathBuf {
        self.lore_dir().join(ENTRIES_DIR)
    }

    /// Get the index file path
    fn index_path(&self) -> PathBuf {
        self.lore_dir().join(INDEX_FILE)
    }

    /// Check if Lore is initialized
    pub fn is_initialized(&self) -> bool {
        self.lore_dir().exists()
    }

    /// Initialize a new Lore repository
    pub fn init(&self, agent_id: Option<&str>) -> Result<(), StorageError> {
        if self.is_initialized() {
            return Err(StorageError::AlreadyInitialized);
        }

        // Create directory structure
        fs::create_dir_all(self.entries_dir())?;

        // Create empty index
        let index = LoreIndex::new();
        self.save_index(&index)?;

        // Create config
        let config = serde_json::json!({
            "version": "0.1.0",
            "default_agent_id": agent_id.unwrap_or("unknown"),
            "created_at": chrono::Utc::now().to_rfc3339(),
        });
        let config_path = self.lore_dir().join(CONFIG_FILE);
        let mut file = fs::File::create(config_path)?;
        file.write_all(serde_json::to_string_pretty(&config)?.as_bytes())?;

        // Create .gitignore to not ignore anything (we want .lore committed)
        // But we might want to ignore some temp files
        let gitignore_path = self.lore_dir().join(".gitignore");
        fs::write(gitignore_path, "*.tmp\n*.lock\n")?;

        Ok(())
    }

    /// Load the index
    pub fn load_index(&self) -> Result<LoreIndex, StorageError> {
        if !self.is_initialized() {
            return Err(StorageError::NotInitialized);
        }

        let index_path = self.index_path();
        if !index_path.exists() {
            return Ok(LoreIndex::new());
        }

        let content = fs::read_to_string(index_path)?;
        let index: LoreIndex = serde_json::from_str(&content)?;
        Ok(index)
    }

    /// Save the index
    pub fn save_index(&self, index: &LoreIndex) -> Result<(), StorageError> {
        let index_path = self.index_path();
        let content = serde_json::to_string_pretty(index)?;
        fs::write(index_path, content)?;
        Ok(())
    }

    /// Save a thought object
    pub fn save_entry(&self, entry: &ThoughtObject) -> Result<(), StorageError> {
        if !self.is_initialized() {
            return Err(StorageError::NotInitialized);
        }

        // Save the entry
        let entry_path = self.entries_dir().join(format!("{}.json", entry.id));
        let content = serde_json::to_string_pretty(entry)?;
        fs::write(entry_path, content)?;

        // Update index
        let mut index = self.load_index()?;
        index.add_entry(&entry.target_file, &entry.id);
        self.save_index(&index)?;

        Ok(())
    }

    /// Load an entry by ID
    pub fn load_entry(&self, id: &str) -> Result<ThoughtObject, StorageError> {
        if !self.is_initialized() {
            return Err(StorageError::NotInitialized);
        }

        let entry_path = self.entries_dir().join(format!("{}.json", id));
        if !entry_path.exists() {
            return Err(StorageError::FileNotFound(id.to_string()));
        }

        let content = fs::read_to_string(entry_path)?;
        let entry: ThoughtObject = serde_json::from_str(&content)?;
        Ok(entry)
    }

    /// Get all entries for a file
    pub fn get_entries_for_file(
        &self,
        file_path: &str,
    ) -> Result<Vec<ThoughtObject>, StorageError> {
        let index = self.load_index()?;

        // Normalize the file path
        let normalized = normalize_path(file_path);

        let mut entries: Vec<ThoughtObject> = index
            .get_entries_for_file(&normalized)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.load_entry(id).ok())
                    .collect()
            })
            .unwrap_or_default();

        // Sort by timestamp, newest first
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(entries)
    }

    /// Get all entries
    pub fn get_all_entries(&self) -> Result<Vec<ThoughtObject>, StorageError> {
        if !self.is_initialized() {
            return Err(StorageError::NotInitialized);
        }

        let entries_dir = self.entries_dir();
        let mut entries = Vec::new();

        for entry in fs::read_dir(entries_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(thought) = serde_json::from_str::<ThoughtObject>(&content) {
                    entries.push(thought);
                }
            }
        }

        // Sort by timestamp, newest first
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(entries)
    }

    /// Search entries by query (searches intent and reasoning_trace)
    pub fn search(&self, query: &str) -> Result<Vec<ThoughtObject>, StorageError> {
        let all_entries = self.get_all_entries()?;
        let query_lower = query.to_lowercase();

        let matches: Vec<ThoughtObject> = all_entries
            .into_iter()
            .filter(|entry| {
                entry.intent.to_lowercase().contains(&query_lower)
                    || entry.reasoning_trace.to_lowercase().contains(&query_lower)
                    || entry
                        .rejected_alternatives
                        .iter()
                        .any(|alt| alt.name.to_lowercase().contains(&query_lower))
                    || entry
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(matches)
    }

    /// Get the default agent ID from config
    pub fn get_default_agent_id(&self) -> Result<String, StorageError> {
        let config_path = self.lore_dir().join(CONFIG_FILE);
        if !config_path.exists() {
            return Ok("unknown".to_string());
        }

        let content = fs::read_to_string(config_path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;

        Ok(config
            .get("default_agent_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string())
    }
}

/// Hash a file's contents using SHA256
pub fn hash_file(path: &Path) -> Result<String, StorageError> {
    if !path.exists() {
        return Err(StorageError::FileNotFound(
            path.to_string_lossy().to_string(),
        ));
    }

    let content = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Hash a string using SHA256
#[allow(dead_code)]
pub fn hash_string(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Normalize a file path (remove leading ./, convert to forward slashes)
pub fn normalize_path(path: &str) -> String {
    let path = path.trim_start_matches("./");
    path.replace('\\', "/")
}

/// Find the lore root by searching upward from the current directory
pub fn find_lore_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let lore_dir = current.join(LORE_DIR);
        if lore_dir.exists() {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, LoreStorage) {
        let temp_dir = TempDir::new().unwrap();
        let storage = LoreStorage::new(temp_dir.path().to_path_buf());
        (temp_dir, storage)
    }

    #[test]
    fn test_storage_not_initialized() {
        let (_temp_dir, storage) = create_test_storage();
        assert!(!storage.is_initialized());
    }

    #[test]
    fn test_storage_init() {
        let (_temp_dir, storage) = create_test_storage();

        storage.init(Some("test-agent")).unwrap();

        assert!(storage.is_initialized());
        assert!(storage.lore_dir().exists());
        assert!(storage.entries_dir().exists());
        assert!(storage.index_path().exists());
    }

    #[test]
    fn test_storage_init_with_agent_id() {
        let (_temp_dir, storage) = create_test_storage();

        storage.init(Some("my-agent")).unwrap();

        let agent_id = storage.get_default_agent_id().unwrap();
        assert_eq!(agent_id, "my-agent");
    }

    #[test]
    fn test_storage_init_without_agent_id() {
        let (_temp_dir, storage) = create_test_storage();

        storage.init(None).unwrap();

        let agent_id = storage.get_default_agent_id().unwrap();
        assert_eq!(agent_id, "unknown");
    }

    #[test]
    fn test_storage_init_already_initialized() {
        let (_temp_dir, storage) = create_test_storage();

        storage.init(None).unwrap();
        let result = storage.init(None);

        assert!(matches!(result, Err(StorageError::AlreadyInitialized)));
    }

    #[test]
    fn test_load_index_not_initialized() {
        let (_temp_dir, storage) = create_test_storage();

        let result = storage.load_index();

        assert!(matches!(result, Err(StorageError::NotInitialized)));
    }

    #[test]
    fn test_load_index_empty() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let index = storage.load_index().unwrap();

        assert_eq!(index.entry_count, 0);
        assert!(index.files.is_empty());
    }

    #[test]
    fn test_save_and_load_index() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let mut index = LoreIndex::new();
        index.add_entry("test.rs", "entry-1");
        storage.save_index(&index).unwrap();

        let loaded = storage.load_index().unwrap();
        assert_eq!(loaded.entry_count, 1);
        assert_eq!(
            loaded.get_entries_for_file("test.rs"),
            Some(&vec!["entry-1".to_string()])
        );
    }

    #[test]
    fn test_save_entry() {
        let (temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let entry = crate::models::ThoughtObject::new(
            "test.rs".to_string(),
            "hash123".to_string(),
            "test-agent".to_string(),
            "Test intent".to_string(),
            "Test reasoning".to_string(),
        );
        let entry_id = entry.id.clone();

        storage.save_entry(&entry).unwrap();

        // Verify entry was saved
        let entry_path = storage.entries_dir().join(format!("{}.json", entry_id));
        assert!(entry_path.exists());

        // Verify index was updated
        let index = storage.load_index().unwrap();
        assert_eq!(index.entry_count, 1);
    }

    #[test]
    fn test_save_entry_not_initialized() {
        let (_temp_dir, storage) = create_test_storage();

        let entry = crate::models::ThoughtObject::new(
            "test.rs".to_string(),
            "hash123".to_string(),
            "test-agent".to_string(),
            "Test".to_string(),
            "Reasoning".to_string(),
        );

        let result = storage.save_entry(&entry);
        assert!(matches!(result, Err(StorageError::NotInitialized)));
    }

    #[test]
    fn test_load_entry() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entry = crate::models::ThoughtObject::new(
            "test.rs".to_string(),
            "hash123".to_string(),
            "test-agent".to_string(),
            "Test intent".to_string(),
            "Test reasoning".to_string(),
        );
        let entry_id = entry.id.clone();

        storage.save_entry(&entry).unwrap();
        let loaded = storage.load_entry(&entry_id).unwrap();

        assert_eq!(loaded.id, entry_id);
        assert_eq!(loaded.target_file, "test.rs");
        assert_eq!(loaded.intent, "Test intent");
    }

    #[test]
    fn test_load_entry_not_found() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let result = storage.load_entry("nonexistent-id");

        assert!(matches!(result, Err(StorageError::FileNotFound(_))));
    }

    #[test]
    fn test_get_entries_for_file() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entry1 = crate::models::ThoughtObject::new(
            "test.rs".to_string(),
            "hash1".to_string(),
            "agent".to_string(),
            "Intent 1".to_string(),
            "Reasoning 1".to_string(),
        );
        let entry2 = crate::models::ThoughtObject::new(
            "test.rs".to_string(),
            "hash2".to_string(),
            "agent".to_string(),
            "Intent 2".to_string(),
            "Reasoning 2".to_string(),
        );

        storage.save_entry(&entry1).unwrap();
        storage.save_entry(&entry2).unwrap();

        let entries = storage.get_entries_for_file("test.rs").unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_get_entries_for_file_normalized_path() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entry = crate::models::ThoughtObject::new(
            "src/test.rs".to_string(),
            "hash".to_string(),
            "agent".to_string(),
            "Intent".to_string(),
            "Reasoning".to_string(),
        );
        storage.save_entry(&entry).unwrap();

        // Query with ./ prefix should still find it
        let entries = storage.get_entries_for_file("./src/test.rs").unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_get_entries_for_file_empty() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entries = storage.get_entries_for_file("nonexistent.rs").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_get_all_entries() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entry1 = crate::models::ThoughtObject::new(
            "file1.rs".to_string(),
            "hash1".to_string(),
            "agent".to_string(),
            "Intent 1".to_string(),
            "Reasoning 1".to_string(),
        );
        let entry2 = crate::models::ThoughtObject::new(
            "file2.rs".to_string(),
            "hash2".to_string(),
            "agent".to_string(),
            "Intent 2".to_string(),
            "Reasoning 2".to_string(),
        );

        storage.save_entry(&entry1).unwrap();
        storage.save_entry(&entry2).unwrap();

        let all_entries = storage.get_all_entries().unwrap();
        assert_eq!(all_entries.len(), 2);
    }

    #[test]
    fn test_get_all_entries_empty() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entries = storage.get_all_entries().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_search_by_intent() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entry = crate::models::ThoughtObject::new(
            "auth.rs".to_string(),
            "hash".to_string(),
            "agent".to_string(),
            "Implement JWT authentication".to_string(),
            "Some reasoning".to_string(),
        );
        storage.save_entry(&entry).unwrap();

        let results = storage.search("JWT").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].intent.contains("JWT"));
    }

    #[test]
    fn test_search_by_reasoning() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entry = crate::models::ThoughtObject::new(
            "auth.rs".to_string(),
            "hash".to_string(),
            "agent".to_string(),
            "Some intent".to_string(),
            "I considered using pandas but decided against it".to_string(),
        );
        storage.save_entry(&entry).unwrap();

        let results = storage.search("pandas").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_tag() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entry = crate::models::ThoughtObject::new(
            "auth.rs".to_string(),
            "hash".to_string(),
            "agent".to_string(),
            "Intent".to_string(),
            "Reasoning".to_string(),
        )
        .with_tags(vec!["security".to_string(), "auth".to_string()]);
        storage.save_entry(&entry).unwrap();

        let results = storage.search("security").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_rejected_alternative() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entry = crate::models::ThoughtObject::new(
            "auth.rs".to_string(),
            "hash".to_string(),
            "agent".to_string(),
            "Intent".to_string(),
            "Reasoning".to_string(),
        )
        .with_rejected(vec![crate::models::RejectedAlternative {
            name: "Auth0 SDK".to_string(),
            reason: None,
        }]);
        storage.save_entry(&entry).unwrap();

        let results = storage.search("Auth0").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_case_insensitive() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entry = crate::models::ThoughtObject::new(
            "auth.rs".to_string(),
            "hash".to_string(),
            "agent".to_string(),
            "Implement JWT".to_string(),
            "Reasoning".to_string(),
        );
        storage.save_entry(&entry).unwrap();

        let results = storage.search("jwt").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_no_results() {
        let (_temp_dir, storage) = create_test_storage();
        storage.init(None).unwrap();

        let entry = crate::models::ThoughtObject::new(
            "auth.rs".to_string(),
            "hash".to_string(),
            "agent".to_string(),
            "Intent".to_string(),
            "Reasoning".to_string(),
        );
        storage.save_entry(&entry).unwrap();

        let results = storage.search("nonexistent").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_hash_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "Hello, World!").unwrap();

        let hash = hash_file(&test_file).unwrap();

        // SHA256 of "Hello, World!" is known
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }

    #[test]
    fn test_hash_file_not_found() {
        let result = hash_file(Path::new("/nonexistent/file.txt"));
        assert!(matches!(result, Err(StorageError::FileNotFound(_))));
    }

    #[test]
    fn test_hash_string() {
        let hash = hash_string("Hello, World!");
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }

    #[test]
    fn test_hash_string_empty() {
        let hash = hash_string("");
        // SHA256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_normalize_path_with_dot_slash() {
        assert_eq!(normalize_path("./src/main.rs"), "src/main.rs");
    }

    #[test]
    fn test_normalize_path_with_backslashes() {
        assert_eq!(normalize_path("src\\main.rs"), "src/main.rs");
    }

    #[test]
    fn test_normalize_path_already_normalized() {
        assert_eq!(normalize_path("src/main.rs"), "src/main.rs");
    }

    #[test]
    fn test_normalize_path_complex() {
        assert_eq!(
            normalize_path("./src\\utils\\helper.rs"),
            "src/utils/helper.rs"
        );
    }

    #[test]
    fn test_find_lore_root_found() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LoreStorage::new(temp_dir.path().to_path_buf());
        storage.init(None).unwrap();

        // Create a subdirectory
        let subdir = temp_dir.path().join("src").join("utils");
        std::fs::create_dir_all(&subdir).unwrap();

        let root = find_lore_root(&subdir);
        assert!(root.is_some());
        assert_eq!(root.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_find_lore_root_not_found() {
        let temp_dir = TempDir::new().unwrap();

        let root = find_lore_root(temp_dir.path());
        assert!(root.is_none());
    }

    #[test]
    fn test_find_lore_root_at_current() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LoreStorage::new(temp_dir.path().to_path_buf());
        storage.init(None).unwrap();

        let root = find_lore_root(temp_dir.path());
        assert!(root.is_some());
        assert_eq!(root.unwrap(), temp_dir.path());
    }
}
