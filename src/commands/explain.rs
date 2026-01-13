use crate::models::ThoughtObject;
use crate::storage::{find_lore_root, normalize_path, LoreStorage};
use colored::Colorize;

pub struct ExplainOptions {
    pub file: String,
    pub all: bool,
    pub json: bool,
    pub limit: Option<usize>,
}

pub fn execute(options: ExplainOptions) -> Result<(), Box<dyn std::error::Error>> {
    // Find lore root
    let current_dir = std::env::current_dir()?;
    let root = find_lore_root(&current_dir).ok_or("Lore not initialized. Run 'lore init' first.")?;

    let storage = LoreStorage::new(root);
    let normalized = normalize_path(&options.file);

    let entries = storage.get_entries_for_file(&normalized)?;

    if entries.is_empty() {
        println!(
            "{} No reasoning found for {}",
            "Info:".blue(),
            normalized.cyan()
        );
        println!();
        println!(
            "Record reasoning with: {}",
            format!("lore record --file {} -m \"your message\"", options.file).cyan()
        );
        return Ok(());
    }

    // Limit entries if requested
    let entries: Vec<_> = if let Some(limit) = options.limit {
        entries.into_iter().take(limit).collect()
    } else if !options.all {
        // Default: show only the most recent entry
        entries.into_iter().take(1).collect()
    } else {
        entries
    };

    if options.json {
        // Output as JSON
        let json = serde_json::to_string_pretty(&entries)?;
        println!("{}", json);
    } else {
        // Pretty print
        print_entries(&normalized, &entries);
    }

    Ok(())
}

fn print_entries(file_path: &str, entries: &[ThoughtObject]) {
    println!();
    println!("{}", "═".repeat(60).dimmed());
    println!("{} {}", "Lore for:".bold(), file_path.cyan().bold());
    println!("{}", "═".repeat(60).dimmed());

    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            println!("{}", "─".repeat(60).dimmed());
        }

        // Header
        println!();
        println!(
            "{} {} {} {}",
            "Agent:".bold(),
            entry.agent_id.yellow(),
            "│".dimmed(),
            entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string().dimmed()
        );

        if let Some(commit) = &entry.commit_hash {
            println!(
                "{} {}",
                "Commit:".bold(),
                commit[..8.min(commit.len())].cyan()
            );
        }

        if let Some((start, end)) = entry.line_range {
            println!("{} Lines {}-{}", "Range:".bold(), start, end);
        }

        // Intent
        println!();
        println!("{}", "Intent:".bold().underline());
        println!("{}", entry.intent);

        // Reasoning trace
        println!();
        println!("{}", "Reasoning:".bold().underline());

        // Format reasoning trace with word wrap
        let lines: Vec<&str> = entry.reasoning_trace.lines().collect();
        for line in lines {
            println!("  {}", line);
        }

        // Rejected alternatives
        if !entry.rejected_alternatives.is_empty() {
            println!();
            println!("{}", "Rejected Alternatives:".bold().underline());
            for alt in &entry.rejected_alternatives {
                print!("  {} {}", "✗".red(), alt.name);
                if let Some(reason) = &alt.reason {
                    print!(" - {}", reason.dimmed());
                }
                println!();
            }
        }

        // Tags
        if !entry.tags.is_empty() {
            println!();
            print!("{} ", "Tags:".bold());
            for (i, tag) in entry.tags.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", format!("#{}", tag).magenta());
            }
            println!();
        }

        println!();
    }

    println!("{}", "═".repeat(60).dimmed());

    if entries.len() == 1 {
        println!(
            "{}",
            "Tip: Use --all to see complete history".dimmed()
        );
    }
}
