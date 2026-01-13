use crate::git::GitContext;
use crate::storage::{find_lore_root, LoreStorage};
use colored::Colorize;
use std::collections::HashMap;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;

    // Check if lore is initialized
    let root = match find_lore_root(&current_dir) {
        Some(r) => r,
        None => {
            println!("{} Lore is not initialized", "Status:".yellow());
            println!();
            println!("Initialize with: {}", "lore init".cyan());
            return Ok(());
        }
    };

    let storage = LoreStorage::new(root.clone());
    let index = storage.load_index()?;

    println!();
    println!("{}", "═".repeat(50).dimmed());
    println!("{}", "Lore Status".bold());
    println!("{}", "═".repeat(50).dimmed());
    println!();

    // Repository info
    println!("{} {}", "Repository:".bold(), root.display());
    println!(
        "{} {}",
        "Total entries:".bold(),
        index.entry_count.to_string().green()
    );
    println!(
        "{} {}",
        "Files tracked:".bold(),
        index.files.len().to_string().green()
    );

    // Git status
    match GitContext::open(&root) {
        Ok(git) => {
            if let Ok(commit) = git.head_commit() {
                println!(
                    "{} {} ({})",
                    "Git HEAD:".bold(),
                    commit[..8].cyan(),
                    "tracking enabled".green()
                );
            }

            // Show changed files without lore entries
            if let Ok(changed) = git.changed_files() {
                let files_without_lore: Vec<_> = changed
                    .iter()
                    .filter(|c| !index.files.contains_key(&c.path))
                    .collect();

                if !files_without_lore.is_empty() {
                    println!();
                    println!("{}", "Changed files without reasoning:".yellow().bold());
                    for file in files_without_lore.iter().take(5) {
                        println!("  {} {}", "→".yellow(), file.path);
                    }
                    if files_without_lore.len() > 5 {
                        println!(
                            "  {} {} more...",
                            "→".yellow(),
                            files_without_lore.len() - 5
                        );
                    }
                    println!();
                    println!(
                        "{}",
                        "Consider running 'lore record' to capture your reasoning".dimmed()
                    );
                }
            }
        }
        Err(_) => {
            println!("{} {} (Git not available)", "Git:".bold(), "N/A".dimmed());
        }
    }

    // Most documented files
    if !index.files.is_empty() {
        println!();
        println!("{}", "Most documented files:".bold());

        let mut file_counts: Vec<_> = index.files.iter().collect();
        file_counts.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        for (file, entries) in file_counts.iter().take(5) {
            println!(
                "  {} ({} {})",
                file.cyan(),
                entries.len(),
                if entries.len() == 1 { "entry" } else { "entries" }
            );
        }
    }

    // Agent stats
    let entries = storage.get_all_entries()?;
    if !entries.is_empty() {
        let mut agent_counts: HashMap<&str, usize> = HashMap::new();
        for entry in &entries {
            *agent_counts.entry(&entry.agent_id).or_insert(0) += 1;
        }

        println!();
        println!("{}", "Contributors:".bold());
        for (agent, count) in agent_counts.iter() {
            println!(
                "  {} ({} {})",
                agent.yellow(),
                count,
                if *count == 1 { "entry" } else { "entries" }
            );
        }
    }

    println!();
    println!("{}", "═".repeat(50).dimmed());

    Ok(())
}
