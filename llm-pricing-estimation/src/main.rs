use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use prettytable::{row, Table};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Estimate the price of a given LLM
    Estimate {
        /// Name of the LLM model (optional)
        model_name: Option<String>,

        /// Number of input tokens
        #[arg(long)]
        input_tokens: u64,

        /// Number of output tokens
        #[arg(long)]
        output_tokens: u64,

        /// Path to the JSON file containing LLM pricing information
        #[arg(short, long, value_name = "FILE", default_value = "llm_pricing.json")]
        file: PathBuf,
    },

    /// List all the model names and their costs per 1M tokens
    List {
        /// Path to the JSON file containing LLM pricing information
        #[arg(short, long, value_name = "FILE", default_value = "llm_pricing.json")]
        file: PathBuf,

        /// Sort the list by the specified metric
        #[arg(long, value_enum, default_value_t = SortMetric::CostScoreRatio)]
        sort: SortMetric,
    },

    /// Manage LLM pricing information
    Manage {
        #[command(subcommand)]
        command: ManageCommands,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum SortMetric {
    CostScoreRatio,
    Price,
    Score,
    ModelName,
}

#[derive(Subcommand)]
enum ManageCommands {
    /// Add a new LLM model and its pricing information
    Add {
        /// Name of the LLM model
        model_name: String,

        /// Input cost per 1M tokens in dollars
        input_cost: f64,

        /// Output cost per 1M tokens in dollars
        output_cost: f64,

        /// Arena Elo Score of the LLM model
        score: Option<i32>,

        /// Context length of the LLM model
        context_length: Option<u32>,
    },

    /// Delete an LLM model and its pricing information
    Del {
        /// Name of the LLM model
        model_name: String,
    },

    /// Update the pricing information of an LLM model
    Set {
        /// Name of the LLM model
        model_name: String,

        /// Input cost per 1M tokens in dollars
        #[arg(long)]
        input_cost: Option<f64>,

        /// Output cost per 1M tokens in dollars
        #[arg(long)]
        output_cost: Option<f64>,

        /// Arena Elo Score of the LLM model
        #[arg(long)]
        score: Option<i32>,

        /// Context length of the LLM model
        #[arg(long)]
        context_length: Option<u32>,
    },
}

#[derive(Serialize, Deserialize)]
struct LLMPricing {
    models: std::collections::HashMap<String, LLMCost>,
}

#[derive(Serialize, Deserialize)]
struct LLMCost {
    input: f64,
    output: f64,
    score: Option<i32>,
    context_length: Option<u32>,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Estimate {
            model_name,
            input_tokens,
            output_tokens,
            file,
        }) => {
            let pricing = load_pricing_from_file(&file);
            if let Some(model_name) = model_name {
                if let Some(cost) = pricing.models.get(&model_name) {
                    let input_cost = cost.input * (input_tokens as f64) / 1_000_000.0;
                    let output_cost = cost.output * (output_tokens as f64) / 1_000_000.0;
                    let total_cost = input_cost + output_cost;
                    println!(
                        "Estimated cost for model '{}': ${:.2}",
                        model_name, total_cost
                    );
                } else {
                    eprintln!("Model '{}' not found in the pricing file.", model_name);
                }
            } else {
                let mut model_costs: Vec<(&String, &LLMCost)> = pricing.models.iter().collect();
                model_costs.sort_by(|a, b| {
                    let total_a = a.1.input + a.1.output;
                    let total_b = b.1.input + b.1.output;
                    total_b.partial_cmp(&total_a).unwrap()
                });

                let mut table = Table::new();
                table.add_row(row!["Model", "Input Cost (based on input)", "Output Cost (based on output)", "Total Cost"]);
                for (model_name, cost) in model_costs {
                    let input_cost = cost.input * (input_tokens as f64) / 1_000_000.0;
                    let output_cost = cost.output * (output_tokens as f64) / 1_000_000.0;
                    let total_cost = input_cost + output_cost;
                    table.add_row(row![model_name, format!("${:.2}", input_cost), format!("${:.2}", output_cost), format!("${:.2}", total_cost)]);
                }
                table.printstd();
            }
        }
        Some(Commands::List { file, sort }) => {
            let pricing = load_pricing_from_file(&file);
            let mut model_costs: Vec<(&String, &LLMCost)> = pricing.models.iter().collect();

            model_costs.sort_by(|a, b| {
                match sort {
                    SortMetric::CostScoreRatio => {
                        let ratio_a = (a.1.input + a.1.output) / (a.1.score.unwrap_or(0) as f64);
                        let ratio_b = (b.1.input + b.1.output) / (b.1.score.unwrap_or(0) as f64);
                        ratio_a.partial_cmp(&ratio_b).unwrap()
                    }
                    SortMetric::Price => {
                        let total_a = a.1.input + a.1.output;
                        let total_b = b.1.input + b.1.output;
                        total_a.partial_cmp(&total_b).unwrap()
                    }
                    SortMetric::Score => b.1.score.cmp(&a.1.score),
                    SortMetric::ModelName => a.0.cmp(b.0),
                }
            });

            let mut table = Table::new();
            table.add_row(row!["Model", "Input Cost (per 1M tokens)", "Output Cost (per 1M tokens)", "Score", "Context Length"]);
            for (model_name, cost) in model_costs {
                let score = cost.score.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string());
                let context_length = cost.context_length.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
                table.add_row(row![model_name, format!("${:.2}", cost.input), format!("${:.2}", cost.output), score, context_length]);
            }
            table.printstd();
        }
        Some(Commands::Manage { command }) => match command {
            ManageCommands::Add {
                model_name,
                input_cost,
                output_cost,
                score,
                context_length,
            } => {
                let mut pricing = load_pricing_from_file(&PathBuf::from("llm_pricing.json"));
                pricing.models.insert(
                    model_name.clone(),
                    LLMCost {
                        input: input_cost,
                        output: output_cost,
                        score,
                        context_length,
                    },
                );
                save_pricing_to_file(&pricing, &PathBuf::from("llm_pricing.json"));
                println!("Model '{}' added successfully.", model_name);
            }
            ManageCommands::Del { model_name } => {
                let mut pricing = load_pricing_from_file(&PathBuf::from("llm_pricing.json"));
                if pricing.models.remove(&model_name).is_some() {
                    save_pricing_to_file(&pricing, &PathBuf::from("llm_pricing.json"));
                    println!("Model '{}' deleted successfully.", model_name);
                } else {
                    eprintln!("Model '{}' not found in the pricing file.", model_name);
                }
            }
            ManageCommands::Set {
                model_name,
                input_cost,
                output_cost,
                score,
                context_length,
            } => {
                let mut pricing = load_pricing_from_file(&PathBuf::from("llm_pricing.json"));
                if let Some(cost) = pricing.models.get_mut(&model_name) {
                    if let Some(input) = input_cost {
                        cost.input = input;
                    }
                    if let Some(output) = output_cost {
                        cost.output = output;
                    }
                    if let Some(s) = score {
                        cost.score = Some(s);
                    }
                    if let Some(c) = context_length {
                        cost.context_length = Some(c);
                    }
                    save_pricing_to_file(&pricing, &PathBuf::from("llm_pricing.json"));
                    println!("Model '{}' updated successfully.", model_name);
                } else {
                    eprintln!("Model '{}' not found in the pricing file.", model_name);
                }
            }
        },
        None => {
            eprintln!("No command provided. Use --help for more information.");
        }
    }
}

fn load_pricing_from_file(file_path: &PathBuf) -> LLMPricing {
    if !file_path.exists() {
        eprintln!(
            "Pricing file '{}' does not exist. Creating a new file.",
            file_path.display()
        );
        let pricing = LLMPricing {
            models: std::collections::HashMap::new(),
        };
        save_pricing_to_file(&pricing, file_path);
        return pricing;
    }

    let file_open_result = File::open(file_path);
    let file = match file_open_result {
        Ok(file) => file,
        Err(_) => {
            eprintln!(
                "Failed to open the pricing file: {}",
                file_path.display()
            );
            return LLMPricing {
                models: std::collections::HashMap::new(),
            };
        }
    };

    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap_or_else(|_| {
        eprintln!(
            "Pricing file '{}' is empty or invalid. Using an empty pricing.",
            file_path.display()
        );
        LLMPricing {
            models: std::collections::HashMap::new(),
        }
    })
}

fn save_pricing_to_file(pricing: &LLMPricing, file: &PathBuf) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file)
        .expect("Failed to open the pricing file for writing.");
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, pricing).expect("Failed to write the pricing file.");
}