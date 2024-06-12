use glob::glob;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub fn read_gitignore(no_recursive: bool) -> Result<Option<Gitignore>, String> {
    let current_dir = env::current_dir().map_err(|e| e.to_string())?;
    let mut gitignore_builder = GitignoreBuilder::new(&current_dir);

    // Add .gitignore from current directory
    let gitignore_path = current_dir.join(".gitignore");
    if gitignore_path.is_file() {
        gitignore_builder.add(gitignore_path);
    }

    if !no_recursive {
        // Walk through subdirectories and add .gitignore files
        for entry in glob("**/.gitignore").map_err(|e| e.to_string())? {
            if let Ok(path) = entry {
                gitignore_builder.add(path);
            }
        }
    }

    gitignore_builder.build().map(Some).map_err(|e| e.to_string())
}

pub fn process_files(
    pattern: &str,
    context: Option<&str>,
    default_context: &str,
    output_file: &mut fs::File,
    ignore: Option<&Gitignore>,
) -> Result<(), String> {
    let full_pattern = format!("**/{}", pattern);
    for entry in glob(&full_pattern).map_err(|e| e.to_string())? {
        let file_path = entry.map_err(|e| e.to_string())?;
        if let Some(ignore) = ignore {
            if ignore.matched(&file_path, file_path.is_dir()).is_ignore() {
                continue;
            }
        }
        match fs::read_to_string(&file_path) {
            Ok(contents) => {
                writeln!(output_file, "// {}\n{}\n", file_path.to_string_lossy(), contents)
                    .map_err(|e| e.to_string())?;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    println!("Warning: [{}] permission denied error.", file_path.to_string_lossy());
                } else {
                    return Err(e.to_string());
                }
            }
        }
    }
    if let Some(context) = context {
        writeln!(output_file, "{}", context).map_err(|e| e.to_string())?;
    } else {
        writeln!(output_file, "{}", default_context).map_err(|e| e.to_string())?;
    }
    writeln!(output_file).map_err(|e| e.to_string())
}
