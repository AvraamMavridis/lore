use crate::models::ThoughtObject;
use crate::storage::{find_lore_root, LoreStorage};
use colored::Colorize;

pub struct SearchOptions {
    pub query: String,
    pub json: bool,
    pub limit: Option<usize>,
    pub file_filter: Option<String>,
    pub agent_filter: Option<String>,
}

pub fn execute(options: SearchOptions) -> Result<(), Box<dyn std::error::Error>> {
    // Find lore root
    let current_dir = std::env::current_dir()?;
    let root =
        find_lore_root(&current_dir).ok_or("Lore not initialized. Run 'lore init' first.")?;

    let storage = LoreStorage::new(root);

    // Search for matching entries
    let mut entries = storage.search(&options.query)?;

    // Apply additional filters
    if let Some(file_filter) = &options.file_filter {
        entries.retain(|e| e.target_file.contains(file_filter));
    }

    if let Some(agent_filter) = &options.agent_filter {
        entries.retain(|e| e.agent_id.contains(agent_filter));
    }

    // Apply limit
    if let Some(limit) = options.limit {
        entries.truncate(limit);
    }

    if entries.is_empty() {
        println!(
            "{} No entries found matching '{}'",
            "Info:".blue(),
            options.query.cyan()
        );
        return Ok(());
    }

    if options.json {
        // Output as JSON
        let json = serde_json::to_string_pretty(&entries)?;
        println!("{}", json);
    } else {
        // Pretty print search results
        print_search_results(&options.query, &entries);
    }

    Ok(())
}

fn print_search_results(query: &str, entries: &[ThoughtObject]) {
    println!();
    println!("{}", "═".repeat(60).dimmed());
    println!(
        "{} {} ({} results)",
        "Search:".bold(),
        query.cyan().bold(),
        entries.len()
    );
    println!("{}", "═".repeat(60).dimmed());

    for entry in entries {
        println!();
        println!("{} {}", "File:".bold(), entry.target_file.cyan());
        println!(
            "{} {} {} {}",
            "Agent:".bold(),
            entry.agent_id.yellow(),
            "│".dimmed(),
            entry
                .timestamp
                .format("%Y-%m-%d %H:%M")
                .to_string()
                .dimmed()
        );

        // Show intent
        println!("{} {}", "Intent:".bold(), entry.intent);

        // Show snippet of reasoning trace with highlighted query
        let snippet = create_snippet(&entry.reasoning_trace, query, 150);
        if !snippet.is_empty() {
            println!("{}", "Reasoning snippet:".dimmed());
            println!("  {}", highlight_query(&snippet, query));
        }

        // Show rejected alternatives that match
        let matching_rejected: Vec<_> = entry
            .rejected_alternatives
            .iter()
            .filter(|alt| alt.name.to_lowercase().contains(&query.to_lowercase()))
            .collect();

        if !matching_rejected.is_empty() {
            println!("{}", "Rejected alternatives:".dimmed());
            for alt in matching_rejected {
                println!("  {} {}", "✗".red(), alt.name);
            }
        }

        println!("{}", "─".repeat(60).dimmed());
    }

    println!();
    println!(
        "{}",
        "Tip: Use 'lore explain <file>' for full details".dimmed()
    );
}

/// Create a snippet around the matching query
fn create_snippet(text: &str, query: &str, max_len: usize) -> String {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    if let Some(pos) = text_lower.find(&query_lower) {
        // Find snippet boundaries
        let start = pos.saturating_sub(50);
        let end = (pos + query.len() + 100).min(text.len());

        let mut snippet: String = text[start..end].to_string();

        // Clean up snippet
        snippet = snippet.replace('\n', " ");
        snippet = snippet.trim().to_string();

        // Add ellipsis if truncated
        if start > 0 {
            snippet = format!("...{}", snippet);
        }
        if end < text.len() {
            snippet = format!("{}...", snippet);
        }

        // Truncate if still too long
        if snippet.len() > max_len {
            snippet.truncate(max_len);
            snippet = format!("{}...", snippet);
        }

        snippet
    } else {
        // Just return the beginning of the text
        let mut snippet: String = text.chars().take(max_len).collect();
        snippet = snippet.replace('\n', " ");
        if text.len() > max_len {
            snippet = format!("{}...", snippet);
        }
        snippet
    }
}

/// Highlight query matches in text
fn highlight_query(text: &str, query: &str) -> String {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    let mut result = String::new();
    let mut last_end = 0;

    for (start, _) in text_lower.match_indices(&query_lower) {
        // Add text before match
        result.push_str(&text[last_end..start]);
        // Add highlighted match
        let end = start + query.len();
        result.push_str(&text[start..end].yellow().bold().to_string());
        last_end = end;
    }

    // Add remaining text
    result.push_str(&text[last_end..]);
    result
}
