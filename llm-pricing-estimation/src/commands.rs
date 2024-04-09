use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub enum SortMetric {
    CostScoreRatio,
    Price,
    Score,
    ModelName,
    ContextLength,
    KnowledgeCutoff,
}

#[derive(Subcommand)]
pub enum ManageCommands {
    /// Add a new LLM model and its pricing information
    Add {
        /// Name of the LLM model
        model_name: String,

        /// Input cost per 1M tokens in dollars
        #[arg(long)]
        input_cost: f64,

        /// Output cost per 1M tokens in dollars
        #[arg(long)]
        output_cost: f64,

        /// Arena Elo Score of the LLM model
        #[arg(long)]
        score: Option<i32>,

        /// Context length of the LLM model
        #[arg(long)]
        context_length: Option<u32>,

        /// Knowledge cutoff of the LLM model (DD/MM/YYYY, "Online", or Unix epoch)
        #[arg(long)]
        knowledge_cutoff: Option<String>,
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

        /// Knowledge cutoff of the LLM model (DD/MM/YYYY, "Online", or Unix epoch)
        #[arg(long)]
        knowledge_cutoff: Option<String>,
    },
}