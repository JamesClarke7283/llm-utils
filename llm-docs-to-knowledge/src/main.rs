use clap::{Parser, Subcommand};
use docs_to_knowledge::{Knowledge, KnowledgeTrait, KnowledgeType};
use std::fs::File;
use std::io::Write;
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
        /// Package name
        #[arg(short, long)]
        name: String,

        /// Package version (optional, default is "latest")
        #[arg(short = 'v', long)]
        version: Option<String>,

        /// Source type (currently only "cratesio" is supported)
        #[arg(short = 't', long, default_value = "cratesio")]
        source_type: String,

        /// Selenium URL
        #[arg(short = 'u', long, default_value = "http://localhost:4444")]
        selenium_url: String,
    },
    /// List the available sources
    List,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Fetch {
            name,
            version,
            source_type,
            selenium_url,
        }) => {
            let knowledge_type = match source_type.as_str() {
                "cratesio" => KnowledgeType::CratesIo,
                _ => {
                    eprintln!("Unsupported source type: {}", source_type);
                    return Ok(());
                }
            };

            let knowledge = Knowledge::new(
                name.clone(),
                version.clone(),
                knowledge_type,
                selenium_url.clone(),
            );

            let markdown = knowledge.fetch_all().await?;

            let file_name = format!("{}_knowledge.md", name);
            let file_path = Path::new(&file_name);
            let mut file = File::create(file_path)?;
            file.write_all(markdown.as_bytes())?;

            println!("Markdown written to file: {}", file_name);
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