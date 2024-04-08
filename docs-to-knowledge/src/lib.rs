pub mod crates_io;
use async_trait::async_trait;
use html2md::parse_html;

async fn convert_to_markdown(html: &str) -> String {
    let markdown = parse_html(html);
    markdown
}

pub enum KnowledgeType {
    CratesIo,
}

pub struct Knowledge {
    pub name: String,
    pub version: String,
    pub knowledge_type: KnowledgeType,
    pub selenium_url: String,
}

impl Knowledge {
    pub fn new(
        name: String,
        version: Option<String>,
        knowledge_type: KnowledgeType,
        selenium_url: String,
    ) -> Self {
        Knowledge {
            name,
            version: version.unwrap_or_else(|| "latest".to_string()),
            knowledge_type,
            selenium_url,
        }
    }
}

#[async_trait]
pub trait KnowledgeTrait {
    async fn fetch_all(&self) -> Result<String, Box<dyn std::error::Error>>;
}

#[async_trait]
impl KnowledgeTrait for Knowledge {
    async fn fetch_all(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self.knowledge_type {
            KnowledgeType::CratesIo => {
                crates_io::fetch_docs(&self.name, &self.version, &self.selenium_url).await
            }
        }
    }
}