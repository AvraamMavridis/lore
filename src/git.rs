use git2::{Repository, StatusOptions};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Not a git repository")]
    NotARepo,

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("No changes detected")]
    NoChanges,
}

/// Git integration for Lore
pub struct GitContext {
    repo: Repository,
}

impl GitContext {
    /// Open the git repository at the given path (or search upward)
    pub fn open(path: &Path) -> Result<Self, GitError> {
        let repo = Repository::discover(path).map_err(|_| GitError::NotARepo)?;
        Ok(Self { repo })
    }

    /// Get the current HEAD commit hash
    pub fn head_commit(&self) -> Result<String, GitError> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    /// Get list of changed files (staged and unstaged)
    pub fn changed_files(&self) -> Result<Vec<ChangedFile>, GitError> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

        let statuses = self.repo.statuses(Some(&mut opts))?;

        let mut changes = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("").to_string();

            if path.is_empty() || path.starts_with(".lore/") {
                continue;
            }

            let change_type = Self::determine_change_type(&status);
            let Some(change_type) = change_type else {
                continue;
            };

            let staged = status.is_index_new()
                || status.is_index_modified()
                || status.is_index_deleted()
                || status.is_index_renamed();

            changes.push(ChangedFile {
                path,
                change_type,
                staged,
            });
        }

        if changes.is_empty() {
            return Err(GitError::NoChanges);
        }

        Ok(changes)
    }

    /// Get the repo root directory
    pub fn workdir(&self) -> Option<&Path> {
        self.repo.workdir()
    }

    /// Check if a path is ignored by git
    pub fn is_ignored(&self, path: &str) -> bool {
        self.repo.is_path_ignored(Path::new(path)).unwrap_or(false)
    }

    /// Determine the change type from a git status
    fn determine_change_type(status: &git2::Status) -> Option<ChangeType> {
        if status.is_index_new() || status.is_wt_new() {
            Some(ChangeType::Added)
        } else if status.is_index_modified() || status.is_wt_modified() {
            Some(ChangeType::Modified)
        } else if status.is_index_deleted() || status.is_wt_deleted() {
            Some(ChangeType::Deleted)
        } else if status.is_index_renamed() || status.is_wt_renamed() {
            Some(ChangeType::Renamed)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub change_type: ChangeType,
    pub staged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Added => write!(f, "added"),
            ChangeType::Modified => write!(f, "modified"),
            ChangeType::Deleted => write!(f, "deleted"),
            ChangeType::Renamed => write!(f, "renamed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_git_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to init git repo");

        // Configure git user for commits
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to configure git email");

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to configure git name");

        temp_dir
    }

    fn create_git_repo_with_commit() -> TempDir {
        let temp_dir = create_git_repo();

        // Create and commit a file
        std::fs::write(temp_dir.path().join("initial.txt"), "initial content").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to add files");

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create commit");

        temp_dir
    }

    #[test]
    fn test_git_context_open() {
        let temp_dir = create_git_repo();

        let result = GitContext::open(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_git_context_open_not_a_repo() {
        let temp_dir = TempDir::new().unwrap();

        let result = GitContext::open(temp_dir.path());
        assert!(matches!(result, Err(GitError::NotARepo)));
    }

    #[test]
    fn test_git_context_head_commit() {
        let temp_dir = create_git_repo_with_commit();

        let git = GitContext::open(temp_dir.path()).unwrap();
        let commit = git.head_commit();

        assert!(commit.is_ok());
        let hash = commit.unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 40); // SHA1 hash length
    }

    #[test]
    fn test_git_context_workdir() {
        let temp_dir = create_git_repo();

        let git = GitContext::open(temp_dir.path()).unwrap();
        let workdir = git.workdir();

        assert!(workdir.is_some());
        // Compare canonicalized paths to handle macOS /private symlink
        let expected = temp_dir.path().canonicalize().unwrap();
        let actual = workdir.unwrap().canonicalize().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_context_changed_files_new_file() {
        let temp_dir = create_git_repo_with_commit();

        // Create a new file
        std::fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

        let git = GitContext::open(temp_dir.path()).unwrap();
        let changes = git.changed_files().unwrap();

        assert!(!changes.is_empty());
        let new_file = changes.iter().find(|c| c.path == "new_file.txt");
        assert!(new_file.is_some());
        assert_eq!(new_file.unwrap().change_type, ChangeType::Added);
    }

    #[test]
    fn test_git_context_changed_files_modified() {
        let temp_dir = create_git_repo_with_commit();

        // Modify existing file
        std::fs::write(temp_dir.path().join("initial.txt"), "modified content").unwrap();

        let git = GitContext::open(temp_dir.path()).unwrap();
        let changes = git.changed_files().unwrap();

        assert!(!changes.is_empty());
        let modified = changes.iter().find(|c| c.path == "initial.txt");
        assert!(modified.is_some());
        assert_eq!(modified.unwrap().change_type, ChangeType::Modified);
    }

    #[test]
    fn test_git_context_changed_files_no_changes() {
        let temp_dir = create_git_repo_with_commit();

        let git = GitContext::open(temp_dir.path()).unwrap();
        let result = git.changed_files();

        assert!(matches!(result, Err(GitError::NoChanges)));
    }

    #[test]
    fn test_git_context_changed_files_excludes_lore_dir() {
        let temp_dir = create_git_repo_with_commit();

        // Create .lore directory with files
        std::fs::create_dir_all(temp_dir.path().join(".lore")).unwrap();
        std::fs::write(temp_dir.path().join(".lore/index.json"), "{}").unwrap();

        // Also create a regular file
        std::fs::write(temp_dir.path().join("regular.txt"), "content").unwrap();

        let git = GitContext::open(temp_dir.path()).unwrap();
        let changes = git.changed_files().unwrap();

        // Should only contain regular.txt, not .lore files
        assert!(!changes.iter().any(|c| c.path.starts_with(".lore/")));
        assert!(changes.iter().any(|c| c.path == "regular.txt"));
    }

    #[test]
    fn test_git_context_is_ignored() {
        let temp_dir = create_git_repo();

        // Create a .gitignore
        std::fs::write(temp_dir.path().join(".gitignore"), "*.log\n").unwrap();

        let git = GitContext::open(temp_dir.path()).unwrap();

        assert!(git.is_ignored("test.log"));
        assert!(!git.is_ignored("test.txt"));
    }

    #[test]
    fn test_change_type_display() {
        assert_eq!(format!("{}", ChangeType::Added), "added");
        assert_eq!(format!("{}", ChangeType::Modified), "modified");
        assert_eq!(format!("{}", ChangeType::Deleted), "deleted");
        assert_eq!(format!("{}", ChangeType::Renamed), "renamed");
    }

    #[test]
    fn test_change_type_equality() {
        assert_eq!(ChangeType::Added, ChangeType::Added);
        assert_ne!(ChangeType::Added, ChangeType::Modified);
    }

    #[test]
    fn test_changed_file_struct() {
        let changed = ChangedFile {
            path: "src/main.rs".to_string(),
            change_type: ChangeType::Modified,
            staged: true,
        };

        assert_eq!(changed.path, "src/main.rs");
        assert_eq!(changed.change_type, ChangeType::Modified);
        assert!(changed.staged);
    }

    #[test]
    fn test_changed_file_clone() {
        let original = ChangedFile {
            path: "test.rs".to_string(),
            change_type: ChangeType::Added,
            staged: false,
        };

        let cloned = original.clone();
        assert_eq!(cloned.path, original.path);
        assert_eq!(cloned.change_type, original.change_type);
        assert_eq!(cloned.staged, original.staged);
    }

    #[test]
    fn test_git_error_display() {
        let not_a_repo = GitError::NotARepo;
        assert_eq!(format!("{}", not_a_repo), "Not a git repository");

        let no_changes = GitError::NoChanges;
        assert_eq!(format!("{}", no_changes), "No changes detected");
    }

    #[test]
    fn test_git_context_staged_files() {
        let temp_dir = create_git_repo_with_commit();

        // Create and stage a new file
        std::fs::write(temp_dir.path().join("staged.txt"), "content").unwrap();
        Command::new("git")
            .args(["add", "staged.txt"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to stage file");

        let git = GitContext::open(temp_dir.path()).unwrap();
        let changes = git.changed_files().unwrap();

        let staged_file = changes.iter().find(|c| c.path == "staged.txt");
        assert!(staged_file.is_some());
        assert!(staged_file.unwrap().staged);
    }

    #[test]
    fn test_git_context_discover_from_subdirectory() {
        let temp_dir = create_git_repo();

        // Create a subdirectory
        let subdir = temp_dir.path().join("src").join("utils");
        std::fs::create_dir_all(&subdir).unwrap();

        // Open from subdirectory should discover the repo
        let result = GitContext::open(&subdir);
        assert!(result.is_ok());
    }
}
