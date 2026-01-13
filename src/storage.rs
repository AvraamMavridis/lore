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
    pub fn get_entries_for_file(&self, file_path: &str) -> Result<Vec<ThoughtObject>, StorageError> {
        let index = self.load_index()?;

        // Normalize the file path
        let normalized = normalize_path(file_path);

        let entry_ids = index.get_entries_for_file(&normalized);

        match entry_ids {
            Some(ids) => {
                let mut entries = Vec::new();
                for id in ids {
                    if let Ok(entry) = self.load_entry(id) {
                        entries.push(entry);
                    }
                }
                // Sort by timestamp, newest first
                entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                Ok(entries)
            }
            None => Ok(Vec::new()),
        }
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

            if path.extension().map_or(false, |ext| ext == "json") {
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
