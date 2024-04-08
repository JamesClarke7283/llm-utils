use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Estimate minimum system requirements for inference of large language models
///
/// This program takes the number of parameters and precision of a model in GGUF format
/// and estimates the minimum RAM and VRAM required for inference.
///
/// The precision can be specified in the format:
/// - q[1-8]: Quantized precision, e.g., q4, q8
/// - fp(16|32): Floating-point precision, e.g., fp16, fp32
struct Cli {
    /// Number of parameters of the model
    #[arg(long, value_name = "NUM_PARAMS")]
    num_params: u64,

    /// Precision of the model (q[1-8], fp16, fp32)
    /// Examples: q4, q8, fp16, fp32
    #[arg(long, value_name = "PRECISION")]
    precision: String,
}

fn main() {
    let cli = Cli::parse();

    let num_params = cli.num_params;
    let precision = cli.precision.as_str();

    let (ram_required, vram_required) = estimate_memory_requirements(num_params, precision);

    println!("Minimum System Requirements Estimation:");
    println!("Number of Parameters: {}", num_params);
    println!("Precision: {}", precision);
    println!("RAM Required: {:.2} GB", ram_required);
    println!("VRAM Required: {:.2} GB", vram_required);
}

fn estimate_memory_requirements(num_params: u64, precision: &str) -> (f64, f64) {
    let bytes_per_param = if precision.starts_with('q') {
        let bits = precision[1..].parse::<u32>().unwrap_or_else(|_| {
            panic!("Invalid quantized precision: {}", precision);
        });
        bits as f64 / 8.0
    } else if precision.starts_with("fp") {
        let bits = precision[2..].parse::<u32>().unwrap_or_else(|_| {
            panic!("Invalid floating-point precision: {}", precision);
        });
        match bits {
            16 => 2.0,
            32 => 4.0,
            _ => panic!("Unsupported floating-point precision: {}", precision),
        }
    } else {
        panic!("Unsupported precision format: {}", precision);
    };

    let total_bytes = num_params as f64 * bytes_per_param;
    let gigabytes = total_bytes / (1024.0 * 1024.0 * 1024.0);

    (gigabytes, gigabytes)
}