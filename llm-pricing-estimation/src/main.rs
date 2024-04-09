pub mod commands;
pub mod pricing;
pub mod utils;

use clap::Parser;
use crate::commands::{Cli, Commands};
use crate::pricing::{load_pricing_from_file, save_pricing_to_file, LLMCost, LLMPricing};
use crate::utils::{format_context_length, format_knowledge_cutoff, parse_knowledge_cutoff};
use prettytable::{row, Table};
use std::path::PathBuf;

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
                    crate::commands::SortMetric::CostScoreRatio => {
                        let ratio_a = (a.1.input + a.1.output) / (a.1.score.unwrap_or(0) as f64);
                        let ratio_b = (b.1.input + b.1.output) / (b.1.score.unwrap_or(0) as f64);
                        ratio_a.partial_cmp(&ratio_b).unwrap()
                    }
                    crate::commands::SortMetric::Price => {
                        let total_a = a.1.input + a.1.output;
                        let total_b = b.1.input + b.1.output;
                        total_a.partial_cmp(&total_b).unwrap()
                    }
                    crate::commands::SortMetric::Score => b.1.score.cmp(&a.1.score),
                    crate::commands::SortMetric::ModelName => a.0.cmp(b.0),
                    crate::commands::SortMetric::ContextLength => {
                        b.1.context_length.cmp(&a.1.context_length)
                    }
                    crate::commands::SortMetric::KnowledgeCutoff => {
                        b.1.knowledge_cutoff.cmp(&a.1.knowledge_cutoff)
                    }
                }
            });

            let mut table = Table::new();
            table.add_row(row!["Model", "Input Cost (per 1M tokens)", "Output Cost (per 1M tokens)", "Score", "Context Length", "Knowledge Cutoff"]);
            for (model_name, cost) in model_costs {
                let score = cost.score.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string());
                let context_length = format_context_length(cost.context_length);
                let knowledge_cutoff = cost
                    .knowledge_cutoff
                    .map(|k| format_knowledge_cutoff(k))
                    .unwrap_or_else(|| "-".to_string());
                table.add_row(row![model_name, format!("${:.2}", cost.input), format!("${:.2}", cost.output), score, context_length, knowledge_cutoff]);
            }
            table.printstd();
        }
        Some(Commands::Manage { command }) => match command {
            crate::commands::ManageCommands::Add {
                model_name,
                input_cost,
                output_cost,
                score,
                context_length,
                knowledge_cutoff,
            } => {
                let mut pricing = load_pricing_from_file(&PathBuf::from("llm_pricing.json"));
                let epoch = parse_knowledge_cutoff(knowledge_cutoff);
                pricing.models.insert(
                    model_name.clone(),
                    LLMCost {
                        input: input_cost,
                        output: output_cost,
                        score,
                        context_length,
                        knowledge_cutoff: epoch,
                    },
                );
                save_pricing_to_file(&pricing, &PathBuf::from("llm_pricing.json"));
                println!("Model '{}' added successfully.", model_name);
            }
            crate::commands::ManageCommands::Del { model_name } => {
                let mut pricing = load_pricing_from_file(&PathBuf::from("llm_pricing.json"));
                if pricing.models.remove(&model_name).is_some() {
                    save_pricing_to_file(&pricing, &PathBuf::from("llm_pricing.json"));
                    println!("Model '{}' deleted successfully.", model_name);
                } else {
                    eprintln!("Model '{}' not found in the pricing file.", model_name);
                }
            }
            crate::commands::ManageCommands::Set {
                model_name,
                input_cost,
                output_cost,
                score,
                context_length,
                knowledge_cutoff,
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
                    if let Some(k) = knowledge_cutoff {
                        cost.knowledge_cutoff = parse_knowledge_cutoff(Some(k));
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