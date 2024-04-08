use thirtyfour::prelude::*;
use crate::convert_to_markdown;

const BASE_URL: &str = "https://docs.rs";

async fn fetch_page_markdown(driver: &WebDriver, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    driver.get(url).await?;

    let main_content = driver.find(By::Id("main-content")).await?;
    let html = main_content.inner_html().await?;

    let markdown = convert_to_markdown(&html).await;

    Ok(markdown)
}

pub async fn fetch_docs(name: &str, version: &str, selenium_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new(selenium_url, caps).await?;

    let all_url = format!("{}/{}/{}/{}/all.html", BASE_URL, name, version, name);
    driver.get(&all_url).await?;

    let main_content = driver.find(By::Id("main-content")).await?;
    let link_elements = main_content.find_all(By::Tag("a")).await?;

    let mut links = Vec::new();
    for link in link_elements {
        let href = link.attr("href").await?.unwrap_or_default();
        links.push(href);
    }

    let mut markdown = String::new();

    for href in links {
        let page_url = format!("{}/{}/{}/{}/{}", BASE_URL, name, version, name, href);
        let page_markdown = fetch_page_markdown(&driver, &page_url).await?;
        markdown.push_str(&page_markdown);
        markdown.push_str("\n\n");
    }

    driver.quit().await?;

    Ok(markdown)
}