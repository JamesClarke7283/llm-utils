use ignore::WalkBuilder;
use log::{info, warn, error};
use std::fs;
use std::io::Write;
use glob::Pattern;

pub fn process_files(
    pattern: &str,
    context: Option<&str>,
    default_context: &str,
    output_file: &mut fs::File,
    no_recursive: bool,
) -> Result<(), String> {
    #[cfg(feature = "logging")]
    info!("Starting to process files with pattern: {}", pattern);

    let mut builder = WalkBuilder::new(".");
    if no_recursive {
        builder.max_depth(Some(1));
    }
    let walker = builder.build();

    let glob_pattern = Pattern::new(pattern).map_err(|e| e.to_string())?;

    for result in walker {
        match result {
            Ok(entry) => {
                let file_path = entry.path();
                #[cfg(feature = "logging")]
                info!("Processing file: {}", file_path.to_string_lossy());

                if file_path.is_file() && glob_pattern.matches_path(file_path) {
                    match fs::read_to_string(&file_path) {
                        Ok(contents) => {
                            writeln!(output_file, "// {}\n{}\n", file_path.to_string_lossy(), contents)
                                .map_err(|e| e.to_string())?;
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::PermissionDenied {
                                warn!("Warning: [{}] permission denied error.", file_path.to_string_lossy());
                            } else {
                                error!("Error reading file [{}]: {}", file_path.to_string_lossy(), e);
                                return Err(e.to_string());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error walking directory: {}", e);
                return Err(e.to_string());
            },
        }
    }
    if let Some(context) = context {
        writeln!(output_file, "{}", context).map_err(|e| e.to_string())?;
    } else {
        writeln!(output_file, "{}", default_context).map_err(|e| e.to_string())?;
    }
    writeln!(output_file).map_err(|e| e.to_string())
}
