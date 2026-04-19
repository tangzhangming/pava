use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::write::FileOptions;

use pava::codegen;
use pava::parser;
use pava::project::{generate_project_toml, ProjectConfig, PROJECT_TOML_NAME};

#[derive(Subcommand)]
enum Commands {
    /// 编译源文件为 class 文件（类似 javac）
    Compile {
        /// 源文件路径（可指定多个）
        files: Vec<String>,
        /// 输出目录（可选，覆盖 project.toml 配置）
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 打包项目为 JAR 文件
    Build {
        /// 项目根目录（默认当前目录）
        #[arg(short, long)]
        project_dir: Option<String>,
        /// 是否打成胖包（包含 vendor 依赖）
        #[arg(long)]
        fat: bool,
    },
    /// 运行项目（编译 + 执行）
    Run {
        /// 入口文件
        file: String,
    },
    /// 包管理相关命令
    #[command(subcommand)]
    Pkg(PkgCommands),
}

#[derive(Subcommand)]
enum PkgCommands {
    /// 初始化新项目
    Init {
        /// 根包名（如 com.company.project）
        package_name: Option<String>,
    },
}

#[derive(Parser)]
#[command(name = "pava")]
#[command(version = "0.2.0")]
#[command(about = "Pava 编译器 - 编译类 Java 语法到 JVM 字节码", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = Args::parse();

    if let Err(e) = match args.command {
        Commands::Compile { files, output } => cmd_compile(&files, output.as_deref()),
        Commands::Build { project_dir, fat } => cmd_build(project_dir.as_deref(), fat),
        Commands::Run { file } => cmd_run(&file),
        Commands::Pkg(pkg) => match pkg {
            PkgCommands::Init { package_name } => cmd_pkg_init(package_name.as_deref()),
        },
    } {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }
}

fn cmd_compile(files: &[String], output_override: Option<&str>) -> anyhow::Result<()> {
    if files.is_empty() {
        anyhow::bail!("请指定要编译的源文件");
    }

    let first_file = Path::new(&files[0]);
    let (config, project_root) = match ProjectConfig::find_from_entry(first_file) {
        Ok(result) => result,
        Err(_) => return compile_simple_mode(files, output_override),
    };

    let classes_dir = if let Some(out) = output_override {
        PathBuf::from(out)
    } else {
        config.get_classes_dir(&project_root)
    };

    fs::create_dir_all(&classes_dir)?;

    println!("项目根目录: {}", project_root.display());
    println!("输出目录: {}", classes_dir.display());

    for file_path in files {
        compile_file_with_project(file_path, &project_root, &classes_dir)?;
    }

    println!("编译完成！");
    Ok(())
}

fn compile_simple_mode(files: &[String], output_override: Option<&str>) -> anyhow::Result<()> {
    for file_path in files {
        let source = fs::read_to_string(file_path)?;
        let unit = match parser::parse_compilation_unit(&source) {
            Ok(u) => u,
            Err(_) => {
                let output = Path::new(file_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Application");

                let bytecode = codegen::compile(&source)?;
                let class_file = if let Some(out) = output_override {
                    format!("{}/{}.class", out, output)
                } else {
                    format!("{}.class", output)
                };

                fs::write(&class_file, &bytecode)?;
                println!("已编译: {} -> {}", file_path, class_file);
                continue;
            }
        };

        let class_files = codegen::compile_unit(&unit)?;

        for (full_name, bytecode) in class_files {
            let class_path = full_name.replace("/", std::path::MAIN_SEPARATOR_STR);
            let class_file = if let Some(out) = output_override {
                format!("{}/{}.class", out, class_path)
            } else {
                format!("{}.class", class_path)
            };

            if let Some(parent) = Path::new(&class_file).parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&class_file, &bytecode)?;
            println!("已编译: {} -> {}", file_path, class_file);
        }
    }

    Ok(())
}

fn compile_file_with_project(
    file_path: &str,
    _project_root: &Path,
    classes_dir: &Path,
) -> anyhow::Result<()> {
    let source = fs::read_to_string(file_path)?;
    let unit = parser::parse_compilation_unit(&source)?;

    let class_files = codegen::compile_unit(&unit)?;

    for (full_name, bytecode) in class_files {
        let class_path = full_name.replace("/", std::path::MAIN_SEPARATOR_STR);
        let class_file = classes_dir.join(format!("{}.class", class_path));

        if let Some(parent) = class_file.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&class_file, &bytecode)?;
        println!("  已生成: {}", class_file.display());
    }

    Ok(())
}

fn cmd_build(project_dir: Option<&str>, force_fat: bool) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    let project_root = if let Some(dir) = project_dir {
        PathBuf::from(dir)
    } else {
        current_dir.clone()
    };

    let toml_path = project_root.join(PROJECT_TOML_NAME);
    if !toml_path.exists() {
        anyhow::bail!(
            "未找到 {} 文件，请先运行 'pava pkg init' 初始化项目",
            PROJECT_TOML_NAME
        );
    }

    let config = ProjectConfig::from_file(&toml_path)?;
    let fat_jar = force_fat || config.build.fat_jar;

    println!("项目名称: {}", config.project.name);
    println!("项目版本: {}", config.project.version);
    println!("主类: {}", config.build.main_class);
    println!("胖包模式: {}", if fat_jar { "开启" } else { "关闭" });

    let classes_dir = config.get_classes_dir(&project_root);

    let mut source_files = Vec::new();
    for source_dir in &config.paths.source_dirs {
        let dir_path = project_root.join(source_dir);
        if dir_path.exists() {
            collect_pava_files(&dir_path, &mut source_files)?;
        }
    }

    if source_files.is_empty() {
        anyhow::bail!("未找到任何 .pava 源文件");
    }

    println!("找到 {} 个源文件", source_files.len());

    for file_path in &source_files {
        compile_file_with_project(&file_path.to_string_lossy(), &project_root, &classes_dir)?;
    }

    let jar_path = project_root.join(&config.paths.output_dir).join(format!(
        "{}-{}.jar",
        config.project.name, config.project.version
    ));

    if let Some(parent) = jar_path.parent() {
        fs::create_dir_all(parent)?;
    }

    create_jar(&jar_path, &classes_dir, &config, &project_root, fat_jar)?;

    println!("\n打包完成: {}", jar_path.display());
    Ok(())
}

fn collect_pava_files(dir: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_pava_files(&path, files)?;
        } else if path.extension().map_or(false, |ext| ext == "pava") {
            files.push(path);
        }
    }
    Ok(())
}

fn create_jar(
    jar_path: &Path,
    classes_dir: &Path,
    config: &ProjectConfig,
    project_root: &Path,
    fat_jar: bool,
) -> anyhow::Result<()> {
    let jar_file = fs::File::create(jar_path)?;
    let mut zip = zip::ZipWriter::new(jar_file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let main_class = config.build.main_class.replace(".", "/");
    let manifest = format!(
        "Manifest-Version: 1.0\r\n\
         Main-Class: {}\r\n\
         Created-By: Pava Compiler\r\n\
         \r\n",
        main_class
    );
    zip.start_file("META-INF/MANIFEST.MF", options)?;
    zip.write_all(manifest.as_bytes())?;

    if classes_dir.exists() {
        add_directory_to_jar(&mut zip, classes_dir, "", options)?;
    }

    if fat_jar {
        for (_prefix, jar_rel_path) in &config.vendor.dependencies {
            let jar_path = project_root.join(jar_rel_path);
            if jar_path.exists() {
                println!("  合并 vendor JAR: {}", jar_rel_path);
                merge_jar_into_jar(&mut zip, &jar_path, options)?;
            }
        }
    }

    zip.finish()?;
    Ok(())
}

fn add_directory_to_jar(
    zip: &mut zip::ZipWriter<fs::File>,
    dir: &Path,
    base_path: &str,
    options: FileOptions,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        let entry_path = if base_path.is_empty() {
            name_str.to_string()
        } else {
            format!("{}/{}", base_path, name_str)
        };

        if path.is_dir() {
            add_directory_to_jar(zip, &path, &entry_path, options)?;
        } else if path.extension().map_or(false, |ext| ext == "class") {
            zip.start_file(&entry_path, options)?;
            let content = fs::read(&path)?;
            zip.write_all(&content)?;
        }
    }
    Ok(())
}

fn merge_jar_into_jar(
    zip: &mut zip::ZipWriter<fs::File>,
    external_jar: &Path,
    options: FileOptions,
) -> anyhow::Result<()> {
    let jar_file = fs::File::open(external_jar)?;
    let mut archive = zip::ZipArchive::new(jar_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name.starts_with("META-INF/") {
            continue;
        }

        if file.is_file() {
            zip.start_file(&name, options)?;
            std::io::copy(&mut file, zip)?;
        }
    }

    Ok(())
}

fn cmd_run(file: &str) -> anyhow::Result<()> {
    cmd_compile(&[file.to_string()], None)?;

    let (config, project_root) = match ProjectConfig::find_from_entry(Path::new(file)) {
        Ok(result) => result,
        Err(_) => return run_simple_mode(file),
    };

    let classes_dir = config.get_classes_dir(&project_root);
    let main_class = &config.build.main_class;

    println!("\n运行 {}...", main_class);

    let output = Command::new("java")
        .arg("-cp")
        .arg(&classes_dir)
        .arg(main_class)
        .output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("程序运行失败");
    }

    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

fn run_simple_mode(file: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(file)?;
    let unit = parser::parse_compilation_unit(&source)?;

    let class_name = if let Some(class) = unit.classes.first() {
        class.full_name.replace("/", ".")
    } else {
        Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Application")
            .to_string()
    };

    let output_dir = Path::new(file).parent().unwrap_or(Path::new("."));

    let output = Command::new("java")
        .arg("-cp")
        .arg(output_dir)
        .arg(&class_name)
        .output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("程序运行失败");
    }

    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

fn cmd_pkg_init(package_name: Option<&str>) -> anyhow::Result<()> {
    let name = if let Some(n) = package_name {
        n.to_string()
    } else {
        let current_dir = std::env::current_dir()?;
        current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("com.example.project")
            .to_string()
    };

    let toml_path = Path::new(PROJECT_TOML_NAME);
    if toml_path.exists() {
        anyhow::bail!("{} 已存在", PROJECT_TOML_NAME);
    }

    let content = generate_project_toml(&name);
    fs::write(toml_path, &content)?;

    fs::create_dir_all("src")?;
    fs::create_dir_all("vendor")?;
    fs::create_dir_all("target/classes")?;

    let main_content = format!(
        "package {};\n\n\
         class Main {{\n    \
             public static function main(): void {{\n        \
                 println(\"Hello from {}\");\n    \
             }}\n}}\n",
        name, name
    );
    fs::write("src/Main.pava", main_content)?;

    println!("项目初始化完成!");
    println!("  配置文件: {}", PROJECT_TOML_NAME);
    println!("  根包名: {}", name);
    println!("  源码目录: src/");
    println!("  输出目录: target/");
    println!("\n开始编写代码: src/Main.pava");
    println!("编译项目: pava compile src/Main.pava");
    println!("打包项目: pava build");

    Ok(())
}
