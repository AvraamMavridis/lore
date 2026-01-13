use crate::storage::LoreStorage;
use colored::Colorize;
use std::path::PathBuf;

pub fn execute(path: Option<PathBuf>, agent_id: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let root = path.unwrap_or_else(|| std::env::current_dir().unwrap());
    let storage = LoreStorage::new(root.clone());

    match storage.init(agent_id.as_deref()) {
        Ok(()) => {
            println!("{} Initialized Lore in {}", "✓".green(), root.display());
            println!();
            println!("Next steps:");
            println!("  {} Record reasoning for your code changes", "lore record".cyan());
            println!("  {} Understand why code exists", "lore explain <file>".cyan());
            println!("  {} Search through reasoning history", "lore search <query>".cyan());
            Ok(())
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            Err(e.into())
        }
    }
}
