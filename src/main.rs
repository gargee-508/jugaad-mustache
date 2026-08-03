#![forbid(unsafe_code)]

use clap::Parser;
use std::fs;
use serde_json::Value;

mod scanner;
mod parser;
mod renderer;
mod jugaad;
mod explainer;

#[derive(Parser, Debug)]
#[command(name = "mustache")]
#[command(about = "mustache.js ported to Rust - no Node.js required")]
struct Args {
    /// JSON data file
    data: String,

    /// Mustache template file
    template: String,

    /// Fix broken templates instead of crashing
    #[arg(long)]
    jugaad: bool,

    /// Explain what the template is doing step by step
    #[arg(long)]
    explain: bool,
}

fn main() {
    let args = Args::parse();

    // Read files
    let data_str = fs::read_to_string(&args.data)
        .unwrap_or_else(|_| panic!("Cannot read data file: {}", args.data));
    let template_str = fs::read_to_string(&args.template)
        .unwrap_or_else(|_| panic!("Cannot read template file: {}", args.template));

    // Parse JSON
    let data: Value = serde_json::from_str(&data_str)
        .unwrap_or_else(|e| panic!("Invalid JSON: {}", e));

    // Apply jugaad if requested
    let final_template = if args.jugaad {
        jugaad::print_jugaad_header();
        let result = jugaad::jugaad_fix(&template_str);
        jugaad::print_jugaad_fixes(&result.fixes);
        result.fixed_template
    } else {
        template_str
    };

    // Scan + Parse
    let tokens = scanner::Scanner::scan(&final_template);
    let nodes = parser::parse(tokens)
        .unwrap_or_else(|e| panic!("Parse error: {}", e));

    // Explain if requested
    if args.explain {
        explainer::explain(&nodes, &data);
        println!("⚙️ Executing...\n");
    }

    // Render
    let output = renderer::render(&nodes, &data);
    print!("{}", output);
}
