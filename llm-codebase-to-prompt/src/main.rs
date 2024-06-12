use clap::{arg, command, Parser};
use llm_codebase_to_prompt::process_files;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::env;
use log::error;
use env_logger::Builder;
use std::io::Write;

#[derive(Parser)]
#[command(name = "llm-codebase-to-prompt", version, about, long_about = None)]
struct Cli {
    #[arg(long, required = true)]
    source_files: String,

    #[arg(long, required = true)]
    instruct_files: String,

    #[arg(long)]
    source_context: Option<String>,

    #[arg(long)]
    instruct_context: Option<String>,

    #[arg(long)]
    gitignore: bool,

    #[arg(long)]
    no_recursive_gitignore: bool,

    #[arg(long)]
    watch: bool,

    #[arg(required = true)]
    working_directory: PathBuf,
}

fn main() {
    #[cfg(feature = "logging")]
    {
        use std::fs::OpenOptions;
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("prompt.log")
            .expect("Unable to open log file");
        Builder::new()
            .format(move |buf, record| {
                writeln!(buf, "{} [{}] - {}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"), record.level(), record.args())
            })
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .filter_level(log::LevelFilter::Debug)
            .init();
    }

    let args = Cli::parse();

    let original_dir = env::current_dir().expect("Failed to get current directory");

    if let Err(e) = env::set_current_dir(&args.working_directory) {
        println!("Error changing working directory: {}", e);
        #[cfg(feature = "logging")]
        error!("Error changing working directory: {}", e);
        return;
    }

    if args.watch {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx).expect("Failed to create watcher");

        // Add paths to watch
        let source_path = Path::new(&args.source_files);
        let instruct_path = Path::new(&args.instruct_files);

        watcher.watch(source_path, RecursiveMode::Recursive).expect("Failed to watch source files");
        watcher.watch(instruct_path, RecursiveMode::Recursive).expect("Failed to watch instruct files");

        loop {
            match rx.recv() {
                Ok(event) => {
                    println!("File changed: {:?}", event);
                    if let Err(e) = create_prompt(&args, &original_dir) {
                        println!("Error: {}", e);
                        #[cfg(feature = "logging")]
                        error!("Error creating prompt: {}", e);
                    }
                }
                Err(e) => {
                    println!("Watch error: {:?}", e);
                    #[cfg(feature = "logging")]
                    error!("Watch error: {:?}", e);
                },
            }
        }
    } else {
        if let Err(e) = create_prompt(&args, &original_dir) {
            println!("Error: {}", e);
            #[cfg(feature = "logging")]
            error!("Error creating prompt: {}", e);
        }
    }

    println!("Made prompt.txt file");
}

fn create_prompt(args: &Cli, original_dir: &PathBuf) -> Result<(), String> {
    let prompt_file_path = original_dir.join("prompt.txt");
    let mut output_file = File::create(&prompt_file_path).map_err(|e| e.to_string())?;
    process_files(
        &args.source_files,
        args.source_context.as_deref(),
        "The following are the relevant source code files:\n",
        &mut output_file,
        args.no_recursive_gitignore,
    )?;
    process_files(
        &args.instruct_files,
        args.instruct_context.as_deref(),
        "The following are the instructions for the project:\n",
        &mut output_file,
        args.no_recursive_gitignore,
    )
}
