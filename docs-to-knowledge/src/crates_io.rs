use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::process::Command;
use crate::convert_to_markdown;

const BASE_URL: &str = "http://localhost:8000";

fn fetch_page_html(client: &Client, url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let response = client.get(url).send()?;
    let html = response.text()?;
    Ok(html)
}

fn extract_main_content(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("#main-content").unwrap();
    let main_content = document.select(&selector).next().unwrap();
    main_content.html()
}

pub fn fetch_docs(repo_path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let _ = Command::new("cargo")
        .arg("doc")
        .arg("--no-deps")
        .arg("--open")
        .current_dir(repo_path)
        .spawn()?;

    let client = Client::new();
    let all_url = format!("{}/all.html", BASE_URL);
    let all_html = fetch_page_html(&client, &all_url)?;
    let document = Html::parse_document(&all_html);

    let link_selector = Selector::parse("a").unwrap();
    let links: Vec<String> = document
        .select(&link_selector)
        .map(|element| element.value().attr("href").unwrap_or_default().to_string())
        .collect();

    let mut markdown = String::new();

    for href in links {
        let page_url = format!("{}/{}", BASE_URL, href);
        let page_html = fetch_page_html(&client, &page_url)?;
        let main_content = extract_main_content(&page_html);
        let page_markdown = convert_to_markdown(&main_content);
        markdown.push_str(&page_markdown);
        markdown.push_str("\n\n");
    }

    Ok(markdown)
}