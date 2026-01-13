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

            let change_type = if status.is_index_new() || status.is_wt_new() {
                ChangeType::Added
            } else if status.is_index_modified() || status.is_wt_modified() {
                ChangeType::Modified
            } else if status.is_index_deleted() || status.is_wt_deleted() {
                ChangeType::Deleted
            } else if status.is_index_renamed() || status.is_wt_renamed() {
                ChangeType::Renamed
            } else {
                continue;
            };

            changes.push(ChangedFile {
                path,
                change_type,
                staged: status.is_index_new()
                    || status.is_index_modified()
                    || status.is_index_deleted()
                    || status.is_index_renamed(),
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
