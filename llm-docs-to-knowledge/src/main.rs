// ./docs-to-knowledge/src/main.rs
use clap::{Parser, Subcommand};
use docs_to_knowledge::{Knowledge, KnowledgeType};
use docs_to_knowledge::KnowledgeTrait;
use std::path::Path;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch and generate knowledge for a package
    Fetch {
        /// Path to the local git repository
        #[arg(short, long)]
        repo_path: String,

        /// Source type (currently only "cratesio" is supported)
        #[arg(short = 't', long, default_value = "cratesio")]
        source_type: String,
    },
    /// List the available sources
    List,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Fetch {
                 repo_path,
                 source_type,
             }) => {
            let knowledge_type = match source_type.as_str() {
                "cratesio" => KnowledgeType::CratesIo,
                _ => {
                    eprintln!("Unsupported source type: {}", source_type);
                    return Ok(());
                }
            };

            let knowledge = Knowledge::new(repo_path.clone(), knowledge_type);
            let summary = knowledge.fetch_all()?;

            // Print the summary to the console
            println!("{}", summary);
        }
        Some(Commands::List) => {
            println!("Available sources:");
            println!("- cratesio");
        }
        None => {
            eprintln!("No command specified. Use --help to see available commands.");
        }
    }

    Ok(())
}
