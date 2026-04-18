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

    // 尝试解析为编译单元（支持 package 和 import）
    let unit_result = pava::parser::parse_compilation_unit(&source);

    match unit_result {
        Ok(unit) => {
            // 有 package/import 支持的新路径
            let output_dir = Path::new(file).parent().unwrap_or(Path::new("."));

            let class_files = pava::codegen::compile_unit(&unit)?;

            for (full_name, bytecode) in class_files {
                // full_name 格式如 "com/example/MyClass"
                let class_path = full_name.replace("/", std::path::MAIN_SEPARATOR_STR);
                let class_file_path = output_dir.join(format!("{}.class", class_path));

                // 创建必要的目录结构
                if let Some(parent) = class_file_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::write(&class_file_path, &bytecode)?;
                println!("Compiled: {} -> {}", file, class_file_path.display());
            }
        }
        Err(_) => {
            // 回退到旧的单类解析方式
            let output = Path::new(file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Application");

            let bytecode = pava::codegen::compile(&source)?;

            let class_file = format!("{}.class", output);
            fs::write(&class_file, &bytecode)?;

            println!("Compiled: {} -> {}", file, output);
        }
    }

    Ok(())
}

fn run(file: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(file)?;

    // 尝试解析为编译单元
    let unit_result = pava::parser::parse_compilation_unit(&source);

    let class_name = match unit_result {
        Ok(ref unit) => {
            // 使用第一个类的完整名称
            if let Some(class) = unit.classes.first() {
                class.full_name.replace("/", ".")
            } else {
                Path::new(file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Application")
                    .to_string()
            }
        }
        Err(_) => Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Application")
            .to_string(),
    };

    let output_dir = Path::new(file).parent().unwrap_or(Path::new("."));

    let output = Command::new("java")
        .arg("-cp")
        .arg(output_dir)
        .arg(&class_name)
        .output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
