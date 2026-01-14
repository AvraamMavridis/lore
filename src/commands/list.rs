use crate::storage::{find_lore_root, LoreStorage};
use colored::Colorize;

pub struct ListOptions {
    pub json: bool,
    pub limit: Option<usize>,
}

pub fn execute(options: ListOptions) -> Result<(), Box<dyn std::error::Error>> {
    // Find lore root
    let current_dir = std::env::current_dir()?;
    let root =
        find_lore_root(&current_dir).ok_or("Lore not initialized. Run 'lore init' first.")?;

    let storage = LoreStorage::new(root);
    let mut entries = storage.get_all_entries()?;

    // Apply limit
    if let Some(limit) = options.limit {
        entries.truncate(limit);
    }

    if entries.is_empty() {
        println!("{} No entries recorded yet.", "Info:".blue());
        println!();
        println!(
            "Record reasoning with: {}",
            "lore record -m \"your message\"".cyan()
        );
        return Ok(());
    }

    if options.json {
        let json = serde_json::to_string_pretty(&entries)?;
        println!("{}", json);
    } else {
        println!();
        println!("{}", "═".repeat(70).dimmed());
        println!("{} ({} total)", "Lore Entries".bold(), entries.len());
        println!("{}", "═".repeat(70).dimmed());
        println!();

        // Header
        println!(
            "{:<40} {:<15} {:<15}",
            "FILE".bold(),
            "AGENT".bold(),
            "DATE".bold()
        );
        println!("{}", "─".repeat(70).dimmed());

        for entry in &entries {
            let file_display = if entry.target_file.len() > 38 {
                format!("...{}", &entry.target_file[entry.target_file.len() - 35..])
            } else {
                entry.target_file.clone()
            };

            let agent_display = if entry.agent_id.len() > 13 {
                format!("{}...", &entry.agent_id[..10])
            } else {
                entry.agent_id.clone()
            };

            let date = entry.timestamp.format("%Y-%m-%d").to_string();

            println!(
                "{:<40} {:<15} {:<15}",
                file_display.cyan(),
                agent_display.yellow(),
                date.dimmed()
            );
        }

        println!();
        println!("{}", "─".repeat(70).dimmed());
        println!(
            "{}",
            "Use 'lore explain <file>' to see full reasoning".dimmed()
        );
    }

    Ok(())
}
