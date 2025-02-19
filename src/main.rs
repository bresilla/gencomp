use clap::Parser;
use regex::Regex;
use serde_json::{json, Value};
use std::{fs, path::Path, process};

/// This tool reads a c_cpp_properties.json file, finds the compile commands
/// files referenced by each configuration, and merges their contents into a
/// single compile_commands.json.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the input c_cpp_properties.json file.
    #[arg(short, long, default_value = ".vscode/c_cpp_properties.json")]
    input: String,

    /// Output path for the merged compile_commands.json.
    #[arg(short, long, default_value = "./compile_commands.json")]
    output: String,
}

/// Remove single-line (//...) and multi-line (/*...*/) comments from a string.
fn remove_comments(text: &str) -> String {
    let re = Regex::new(r"//.*?$|/\*.*?\*/").unwrap();
    re.replace_all(text, "").to_string()
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.input).exists() {
        eprintln!("Error: The file {} does not exist.", args.input);
        process::exit(1);
    }

    // Read and (optionally) clean the c_cpp_properties.json file.
    let content = fs::read_to_string(&args.input).unwrap_or_else(|err| {
        eprintln!("Error reading {}: {}", args.input, err);
        process::exit(1);
    });
    let cleaned = remove_comments(&content);

    // Parse the c_cpp_properties.json file.
    let json: Value = serde_json::from_str(&cleaned).unwrap_or_else(|err| {
        eprintln!("Error parsing JSON from {}: {}", args.input, err);
        process::exit(1);
    });

    // Get the "configurations" array.
    let configurations = json.get("configurations").and_then(Value::as_array).unwrap_or_else(|| {
        eprintln!("Error: 'configurations' array not found in {}.", args.input);
        process::exit(1);
    });

    // This vector will hold all compile command entries from all referenced files.
    let mut all_compile_commands = Vec::new();

    // Iterate over each configuration.
    for config in configurations {
        if let Some(commands_arr) = config.get("compileCommands").and_then(Value::as_array) {
            for file_value in commands_arr {
                if let Some(command_file_path) = file_value.as_str() {
                    // Read the referenced compile commands file.
                    let file_content = fs::read_to_string(command_file_path).unwrap_or_else(|err| {
                        eprintln!("Error reading compile commands file {}: {}", command_file_path, err);
                        process::exit(1);
                    });

                    // (Optionally) remove comments.
                    let file_cleaned = remove_comments(&file_content);

                    // Parse the compile commands file.
                    let commands_json: Value = serde_json::from_str(&file_cleaned).unwrap_or_else(|err| {
                        eprintln!("Error parsing JSON from {}: {}", command_file_path, err);
                        process::exit(1);
                    });

                    // Expect a JSON array.
                    if let Some(arr) = commands_json.as_array() {
                        all_compile_commands.extend(arr.clone());
                    } else {
                        eprintln!("Error: {} does not contain a JSON array.", command_file_path);
                        process::exit(1);
                    }
                }
            }
        }
    }

    // Serialize and write the merged compile commands.
    let output_content = serde_json::to_string_pretty(&all_compile_commands).unwrap_or_else(|err| {
        eprintln!("Error serializing output JSON: {}", err);
        process::exit(1);
    });
    fs::write(&args.output, output_content).unwrap_or_else(|err| {
        eprintln!("Error writing to {}: {}", args.output, err);
        process::exit(1);
    });

    println!("{} has been generated.", args.output);
}
