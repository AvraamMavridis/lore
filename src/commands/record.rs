use crate::git::{ChangeType, GitContext};
use crate::models::{RejectedAlternative, ThoughtObject};
use crate::storage::{find_lore_root, hash_file, normalize_path, LoreStorage};
use colored::Colorize;
use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;

pub struct RecordOptions {
    pub message: Option<String>,
    pub trace: Option<String>,
    pub trace_file: Option<PathBuf>,
    pub files: Vec<String>,
    pub agent_id: Option<String>,
    pub rejected: Vec<String>,
    pub tags: Vec<String>,
    pub line_range: Option<(usize, usize)>,
    pub stdin: bool,
}

pub fn execute(options: RecordOptions) -> Result<(), Box<dyn std::error::Error>> {
    // Find lore root
    let current_dir = std::env::current_dir()?;
    let root =
        find_lore_root(&current_dir).ok_or("Lore not initialized. Run 'lore init' first.")?;

    let storage = LoreStorage::new(root.clone());

    // Get agent ID
    let agent_id = options
        .agent_id
        .clone()
        .or_else(|| storage.get_default_agent_id().ok())
        .unwrap_or_else(|| "unknown".to_string());

    // Determine which files to record
    let files_to_record: Vec<(String, ChangeType)> = if !options.files.is_empty() {
        // User specified files
        options
            .files
            .iter()
            .map(|f| (f.clone(), ChangeType::Modified))
            .collect()
    } else {
        // Auto-detect from git
        match GitContext::open(&root) {
            Ok(git) => match git.changed_files() {
                Ok(changes) => changes
                    .into_iter()
                    .filter(|c| c.change_type != ChangeType::Deleted)
                    .map(|c| (c.path, c.change_type))
                    .collect(),
                Err(_) => {
                    eprintln!(
                        "{} No changed files detected. Specify files with --file or make changes first.",
                        "Warning:".yellow()
                    );
                    return Ok(());
                }
            },
            Err(_) => {
                eprintln!(
                    "{} Not a git repository and no files specified.",
                    "Error:".red()
                );
                return Err("Specify files with --file or initialize git".into());
            }
        }
    };

    if files_to_record.is_empty() {
        println!("{} No files to record reasoning for.", "Info:".blue());
        return Ok(());
    }

    // Get reasoning trace
    let reasoning_trace = get_reasoning_trace(&options)?;

    // Get intent message
    let intent = options.message.unwrap_or_else(|| {
        prompt_for_input("Enter intent/purpose (brief description):")
            .unwrap_or_else(|_| "No intent provided".to_string())
    });

    // Parse rejected alternatives
    let rejected_alternatives: Vec<RejectedAlternative> = options
        .rejected
        .into_iter()
        .map(|name| RejectedAlternative { name, reason: None })
        .collect();

    // Get commit hash if available
    let commit_hash = GitContext::open(&root)
        .ok()
        .and_then(|git| git.head_commit().ok());

    // Record entry for each file
    let mut recorded_count = 0;

    for (file_path, change_type) in &files_to_record {
        let normalized = normalize_path(file_path);
        let full_path = root.join(&normalized);

        // Skip if file doesn't exist (was deleted)
        if !full_path.exists() {
            println!("{} Skipping {} (file not found)", "→".yellow(), normalized);
            continue;
        }

        // Hash the file
        let file_hash = hash_file(&full_path)?;

        // Create thought object
        let mut entry = ThoughtObject::new(
            normalized.clone(),
            file_hash,
            agent_id.clone(),
            intent.clone(),
            reasoning_trace.clone(),
        )
        .with_rejected(rejected_alternatives.clone())
        .with_tags(options.tags.clone());

        if let Some(hash) = &commit_hash {
            entry = entry.with_commit(hash.clone());
        }

        if let Some((start, end)) = options.line_range {
            entry = entry.with_line_range(start, end);
        }

        // Save entry
        storage.save_entry(&entry)?;

        println!(
            "{} Recorded reasoning for {} ({})",
            "✓".green(),
            normalized.cyan(),
            change_type
        );
        recorded_count += 1;
    }

    println!();
    println!(
        "{} entries recorded. Use {} to review.",
        recorded_count.to_string().green(),
        "lore explain <file>".cyan()
    );

    Ok(())
}

fn get_reasoning_trace(options: &RecordOptions) -> Result<String, Box<dyn std::error::Error>> {
    // Check for trace from various sources
    if let Some(trace) = &options.trace {
        return Ok(trace.clone());
    }

    if let Some(trace_file) = &options.trace_file {
        let content = std::fs::read_to_string(trace_file)?;
        return Ok(content);
    }

    if options.stdin {
        println!(
            "{}",
            "Reading reasoning trace from stdin (Ctrl+D to end):".yellow()
        );
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        return Ok(buffer);
    }

    // Prompt for reasoning
    prompt_for_multiline_input("Enter reasoning trace (empty line to finish):")
}

fn prompt_for_input(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    print!("{} ", prompt.cyan());
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;

    Ok(line.trim().to_string())
}

fn prompt_for_multiline_input(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("{}", prompt.cyan());

    let stdin = io::stdin();
    let mut lines = Vec::new();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() {
            break;
        }
        lines.push(line);
    }

    Ok(lines.join("\n"))
}
