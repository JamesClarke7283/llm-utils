use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct LLMPricing {
    pub models: HashMap<String, LLMCost>,
}

#[derive(Serialize, Deserialize)]
pub struct LLMCost {
    pub input: f64,
    pub output: f64,
    pub score: Option<i32>,
    pub context_length: Option<u32>,
    pub knowledge_cutoff: Option<i64>,
    #[serde(default)]
    pub function_calling: bool,
    #[serde(default)]
    pub languages: Vec<String>,
}

pub fn load_pricing_from_file(file_path: &PathBuf) -> LLMPricing {
    if !file_path.exists() {
        eprintln!(
            "Pricing file '{}' does not exist. Creating a new file.",
            file_path.display()
        );
        let pricing = LLMPricing {
            models: HashMap::new(),
        };
        save_pricing_to_file(&pricing, file_path);
        return pricing;
    }

    let file_open_result = File::open(file_path);
    let file = match file_open_result {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Failed to open the pricing file: {}", file_path.display());
            return LLMPricing {
                models: HashMap::new(),
            };
        }
    };

    let reader = BufReader::new(file);
    let pricing: LLMPricing = serde_json::from_reader(reader).unwrap_or_else(|err| {
        eprintln!(
            "Error parsing pricing file '{}': {}. Using an empty pricing.",
            file_path.display(),
            err
        );
        LLMPricing {
            models: HashMap::new(),
        }
    });

    pricing
}

pub fn save_pricing_to_file(pricing: &LLMPricing, file: &PathBuf) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file)
        .expect("Failed to open the pricing file for writing.");
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, pricing).expect("Failed to write the pricing file.");
}
