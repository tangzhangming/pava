use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Subcommand)]
enum Commands {
    Build { file: String },
    Run { file: String },
}

#[derive(Parser)]
#[command(name = "pava")]
#[command(version = "0.1.0")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Build { file } => {
            if let Err(e) = build(&file) {
                eprintln!("Build error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Run { file } => {
            if let Err(e) = build(&file) {
                eprintln!("Build error: {}", e);
                std::process::exit(1);
            }
            if let Err(e) = run(&file) {
                eprintln!("Run error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn build(file: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(file)?;
    let output = Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Application");

    let bytecode = pava::codegen::compile(&source)?;

    let class_file = format!("{}.class", output);
    fs::write(&class_file, &bytecode)?;

    println!("Compiled: {} -> {}", file, output);
    Ok(())
}

fn run(file: &str) -> anyhow::Result<()> {
    let class_name = Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Application");

    let output = Command::new("java").arg(class_name).output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
