use std::env;
use std::fs;
use std::io::Write;


use clap::{arg, command, Parser};
use glob::glob;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify::{RecursiveMode, Watcher};

#[derive(Parser)]
#[command(name = "llm-codebase-to-prompt", version, about, long_about = None)]
struct Cli {
    /// Filename pattern for source code files
    #[arg(long, required = true)]
    source_files: String,

    /// Filename pattern for instruction files
    #[arg(long, required = true)]
    instruct_files: String,

    /// Context string to prepend before source code
    #[arg(long)]
    source_context: Option<String>,

    /// Context string to prepend before instructions
    #[arg(long)]
    instruct_context: Option<String>,

    /// Ignore files specified in .gitignore
    #[arg(long)]
    gitignore: bool,

    /// Watch for changes in files and regenerate the prompt
    #[arg(long)]
    watch: bool,
}

fn read_gitignore() -> Option<Gitignore> {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let gitignore_path = current_dir.join(".gitignore");
    if gitignore_path.is_file() {
        let gitignore_builder = GitignoreBuilder::new(&gitignore_path);
        gitignore_builder.build().ok()
    } else {
        None
    }
}

fn process_files(
    pattern: &str,
    context: Option<&str>,
    default_context: &str,
    output_file: &mut fs::File,
    ignore: Option<&Gitignore>,
) {
    let full_pattern = format!("**/{}", pattern);
    for entry in glob(&full_pattern).expect("Failed to read glob pattern") {
        if let Ok(file_path) = entry {
            if let Some(ignore) = ignore {
                if ignore.matched(&file_path, file_path.is_dir()).is_ignore() {
                    continue;
                }
            }
            let contents = fs::read_to_string(&file_path).expect("Failed to read file");
            writeln!(
                output_file,
                "// {}\n{}\n",
                file_path.to_string_lossy(),
                contents
            )
            .expect("Failed to write to output file");
        }
    }
    if let Some(context) = context {
        writeln!(output_file, "{}", context).expect("Failed to write to output file");
    } else {
        writeln!(output_file, "{}", default_context).expect("Failed to write to output file");
    }
    writeln!(output_file).expect("Failed to write to output file");
}

fn write_prompt(args: &Cli) {
    let ignore = if args.gitignore {
        read_gitignore()
    } else {
        None
    };
    let mut output_file = fs::File::create("prompt.txt").expect("Failed to create output file");
    process_files(
        &args.source_files,
        args.source_context.as_deref(),
        "The following are the relevant source code files:\n",
        &mut output_file,
        ignore.as_ref(),
    );
    process_files(
        &args.instruct_files,
        args.instruct_context.as_deref(),
        "The following are the instructions for the project:\n",
        &mut output_file,
        ignore.as_ref(),
    );
}

fn main() {
    let args = Cli::parse();

    if args.watch {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx).unwrap();

        let ignore = if args.gitignore {
            read_gitignore()
        } else {
            None
        };

        let current_dir = env::current_dir().unwrap();
        let mut ignored_dirs = Vec::new();

        for entry in current_dir.read_dir().expect("Failed to read directory") {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(ignore) = &ignore {
                    if ignore.matched(&path, path.is_dir()).is_ignore() {
                        if path.is_dir() {
                            ignored_dirs.push(path);
                        }
                        continue;
                    }
                }
                if path.is_file() {
                    if path.file_name().unwrap() != "prompt.txt" {
                        watcher.watch(path.as_path(), RecursiveMode::NonRecursive).unwrap();
                    }
                } else if path.is_dir() {
                    if !ignored_dirs.contains(&path) {
                        watcher.watch(path.as_path(), RecursiveMode::Recursive).unwrap();
                    }
                }
            }
        }

        loop {
            match rx.recv() {
                Ok(event) => {
                    if let Ok(notify::Event { kind, paths, .. }) = event {
                        if let notify::EventKind::Modify(_) = kind {
                            if let Some(path) = paths.first() {
                                if let Some(ignore) = &ignore {
                                    if ignore.matched(&path, path.is_dir()).is_ignore() {
                                        continue;
                                    }
                                }
                            }
                            println!("File changed: {:?}", paths);
                            write_prompt(&args);
                        }
                    }
                }
                Err(e) => println!("Watch error: {:?}", e),
            }
        }
    } else {
        write_prompt(&args);
    }
}