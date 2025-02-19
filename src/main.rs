use clap::Parser;
use regex::Regex;
use serde_json::{json, Value};
use std::{env, fs, path::Path, process};

/// Generate compile_commands.json from c_cpp_properties.json.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// List of source files.
    sources: Vec<String>,

    /// Path to c_cpp_properties.json. Default is .vscode/c_cpp_properties.json.
    #[arg(short, long, default_value = ".vscode/c_cpp_properties.json")]
    directory: String,

    /// Output path for compile_commands.json. Default is ./compile_commands.json.
    #[arg(short, long, default_value = "./compile_commands.json")]
    output: String,
}

/// Remove both line comments (//) and block comments (/* ... */) from a JSON string.
fn remove_comments(json_string: &str) -> String {
    let re = Regex::new(r"//.*?$|/\*.*?\*/").unwrap();
    re.replace_all(json_string, "").to_string()
}

fn main() {
    let args = Args::parse();

    // Check if the c_cpp_properties.json file exists
    if !Path::new(&args.directory).exists() {
        eprintln!("Error: The file {} does not exist.", args.directory);
        process::exit(1);
    }

    // Read and clean the JSON file
    let content = fs::read_to_string(&args.directory).unwrap_or_else(|err| {
        eprintln!("Error reading file {}: {}", args.directory, err);
        process::exit(1);
    });
    let cleaned_content = remove_comments(&content);
    let cpp_properties: Value = serde_json::from_str(&cleaned_content).unwrap_or_else(|err| {
        eprintln!("Error: Failed to decode JSON from {}.", args.directory);
        eprintln!("Details: {}", err);
        process::exit(1);
    });

    // Prepare compile_commands structure
    let mut compile_commands = Vec::new();

    // Get configuration (assuming the first configuration for simplicity)
    let config = cpp_properties
        .get("configurations")
        .and_then(Value::as_array)
        .and_then(|arr| arr.get(0))
        .unwrap_or_else(|| {
            eprintln!("Error: No configurations found in {}.", args.directory);
            process::exit(1);
        });

    // Get include paths and defines
    let include_paths: Vec<&str> = config
        .get("includePath")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect())
        .unwrap_or_else(Vec::new);

    let defines: Vec<&str> = config
        .get("defines")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect())
        .unwrap_or_else(Vec::new);

    let compiler_path = config.get("compilerPath").and_then(Value::as_str).unwrap_or_else(|| {
        eprintln!("Error: compilerPath not found in configuration.");
        process::exit(1);
    });

    let cpp_standard = config
        .get("cppStandard")
        .and_then(Value::as_str)
        .unwrap_or("");

    // Get the current working directory
    let current_dir = env::current_dir().unwrap_or_else(|err| {
        eprintln!("Error getting current directory: {}", err);
        process::exit(1);
    });
    let current_dir_str = current_dir.to_str().unwrap_or(".");

    // Create compile_commands entries
    for source_file in args.sources {
        let include_flags = include_paths
            .iter()
            .map(|p| format!("-I{}", p))
            .collect::<Vec<_>>()
            .join(" ");
        let define_flags = defines
            .iter()
            .map(|d| format!("-D{}", d))
            .collect::<Vec<_>>()
            .join(" ");
        let command = format!(
            "{} {} {} --std={} -c {}",
            compiler_path, include_flags, define_flags, cpp_standard, source_file
        );
        let entry = json!({
            "directory": current_dir_str,
            "command": command,
            "file": source_file
        });
        compile_commands.push(entry);
    }

    // Write the compile_commands.json file
    let output_content = serde_json::to_string_pretty(&compile_commands).unwrap_or_else(|err| {
        eprintln!("Error serializing compile_commands.json: {}", err);
        process::exit(1);
    });
    fs::write(&args.output, output_content).unwrap_or_else(|err| {
        eprintln!("Error writing to {}: {}", args.output, err);
        process::exit(1);
    });

    println!("{} has been generated.", args.output);
}

