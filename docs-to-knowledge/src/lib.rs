pub mod crates_io;
use html2md::parse_html;

fn convert_to_markdown(html: &str) -> String {
    let markdown = parse_html(html);
    markdown
}

pub enum KnowledgeType {
    CratesIo,
}

pub struct Knowledge {
    pub repo_path: String,
    pub knowledge_type: KnowledgeType,
}

impl Knowledge {
    pub fn new(repo_path: String, knowledge_type: KnowledgeType) -> Self {
        Knowledge {
            repo_path,
            knowledge_type,
        }
    }
}

pub trait KnowledgeTrait {
    fn fetch_all(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

impl KnowledgeTrait for Knowledge {
    fn fetch_all(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self.knowledge_type {
            KnowledgeType::CratesIo => crates_io::fetch_docs(&self.repo_path),
        }
    }
}