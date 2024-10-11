// ./docs-to-knowledge/src/crates_io.rs
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::env;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;
use crate::convert_to_markdown;
use std::path::{Path, PathBuf};
use tiny_http::{Server, Response};
use std::collections::HashSet;
use cargo_metadata::MetadataCommand;

const BASE_URL: &str = "http://localhost:8000";

fn fetch_page_html(client: &Client, url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let response = client.get(url).send()?;
    let html = response.text()?;
    Ok(html)
}

fn extract_main_content(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("#main-content").unwrap();
    if let Some(main_content) = document.select(&selector).next() {
        main_content.html()
    } else {
        String::new()
    }
}

pub fn fetch_docs(repo_path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Retrieve metadata to get the list of crate names
    let metadata = MetadataCommand::new()
        .manifest_path(Path::new(repo_path).join("Cargo.toml"))
        .exec()?;

    // Collect all crate (package) directory names (replace hyphens with underscores)
    let crate_dir_names: HashSet<String> = metadata.packages.iter()
        .map(|pkg| pkg.name.replace("-", "_"))
        .collect();

    // Debug: Print all crate directory names
    println!("Detected crates (as directories):");
    for name in &crate_dir_names {
        println!("- {}", name);
    }

    // Run `cargo doc` to generate documentation
    let resolved_path = fs::canonicalize(repo_path).unwrap_or_else(|_| {
        eprintln!("Failed to resolve path: {}", repo_path);
        env::current_dir().expect("Failed to get current directory")
    });

    let resolved_directory = resolved_path.display().to_string();

    let status = Command::new("cargo")
        .arg("doc")
        .arg("--no-deps")
        .arg("--document-private-items")
        .current_dir(&resolved_directory)
        .status()?;

    if !status.success() {
        return Err("Failed to generate documentation with `cargo doc`.".into());
    }

    let target_doc_path = Path::new(&resolved_directory).join("target/doc");
    if !target_doc_path.is_dir() {
        return Err("Documentation directory not found.".into());
    }

    // Start a local HTTP server to serve the documentation files
    let server = Server::http("0.0.0.0:8000")?;
    let doc_path_clone = target_doc_path.clone();
    thread::spawn(move || {
        for request in server.incoming_requests() {
            let requested_path = request.url().trim_start_matches('/');
            let full_path = doc_path_clone.join(requested_path);
            if full_path.exists() {
                if let Ok(content) = fs::read_to_string(&full_path) {
                    let response = Response::from_string(content);
                    request.respond(response).unwrap();
                } else {
                    let response = Response::from_string("500 Internal Server Error").with_status_code(500);
                    request.respond(response).unwrap();
                }
            } else {
                let response = Response::from_string("404 Not Found").with_status_code(404);
                request.respond(response).unwrap();
            }
        }
    });

    // Give the server some time to start
    thread::sleep(Duration::from_secs(2));

    let client = Client::new();
    let mut created_files = Vec::new(); // To keep track of created files

    // Ensure the .knowledgebase directory exists
    let knowledgebase_dir = Path::new(".knowledgebase");
    if !knowledgebase_dir.exists() {
        fs::create_dir(&knowledgebase_dir)?;
    }

    for entry in fs::read_dir(&target_doc_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Get the directory name
            if let Some(dir_name_osstr) = path.file_name() {
                if let Some(dir_name) = dir_name_osstr.to_str() {
                    // Check if this directory is a crate by matching with `crate_dir_names`
                    if crate_dir_names.contains(dir_name) {
                        println!("Processing crate: {}", dir_name);
                        let all_url = format!("{}/{}/all.html", BASE_URL, dir_name);
                        println!("Fetching: {}", all_url);
                        let all_html = match fetch_page_html(&client, &all_url) {
                            Ok(html) => html,
                            Err(e) => {
                                eprintln!("Failed to fetch {}: {}", all_url, e);
                                continue;
                            },
                        };
                        let document = Html::parse_document(&all_html);

                        let link_selector = Selector::parse("a").unwrap();
                        let links: Vec<String> = document
                            .select(&link_selector)
                            .filter_map(|element| element.value().attr("href"))
                            .map(|href| href.to_string())
                            .collect();

                        let mut markdown = String::new();

                        for href in links {
                            let page_url = format!("{}/{}/{}", BASE_URL, dir_name, href);
                            println!("Fetching page: {}", page_url);
                            let page_html = match fetch_page_html(&client, &page_url) {
                                Ok(html) => html,
                                Err(e) => {
                                    eprintln!("Failed to fetch {}: {}", page_url, e);
                                    continue;
                                },
                            };
                            let main_content = extract_main_content(&page_html);
                            if !main_content.is_empty() {
                                let page_markdown = convert_to_markdown(&main_content);
                                markdown.push_str(&page_markdown);
                                markdown.push_str("\n\n");
                            }
                        }

                        // Define the output file path inside .knowledgebase
                        let file_name = format!("{}_knowledge.md", dir_name);
                        let file_path = knowledgebase_dir.join(&file_name);
                        fs::write(&file_path, &markdown)?;
                        println!("Markdown written to file: {}", file_path.display());
                        created_files.push(file_path.display().to_string());
                    } else {
                        println!("Skipping non-crate directory: {}", dir_name);
                    }
                }
            }
        }
    }

    // Create a summary String listing all created files
    let summary = if created_files.is_empty() {
        "No Markdown files were created.".to_string()
    } else {
        let mut summary = String::from("Created the following Markdown files in `.knowledgebase`:\n\n");
        for file in created_files {
            summary.push_str(&format!("- {}\n", file));
        }
        summary
    };

    Ok(summary)
}
