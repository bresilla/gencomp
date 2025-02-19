use clap::Parser;
use regex::Regex;
use serde_json::{json, Value};
use std::{env, fs, path::Path, process};

/// This tool converts a c_cpp_properties.json file into a compile_commands.json.
/// It supports two modes:
/// 1. **Merge Mode:** If no source files are provided, the tool looks for a
///    "compileCommands" field in the first configuration and merges the compile
///    command entries (either embedded or referenced via file paths).
/// 2. **Generate Mode:** If one or more source files are provided, the tool
///    uses configuration fields (like "compilerPath", "includePath", "defines",
///    "cppStandard") to generate compile command entries for each source file.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the input c_cpp_properties.json file.
    #[arg(short, long, default_value = ".vscode/c_cpp_properties.json")]
    input: String,

    /// Output path for the resulting compile_commands.json.
    #[arg(short, long, default_value = "./compile_commands.json")]
    output: String,

    /// Optional list of source files. If provided, "generate mode" is used.
    #[arg(value_name = "SOURCES")]
    sources: Vec<String>,
}

/// Remove both single-line (//…) and multi-line (/*…*/) comments from a string.
fn remove_comments(text: &str) -> String {
    let re = Regex::new(r"//.*?$|/\*.*?\*/").unwrap();
    re.replace_all(text, "").to_string()
}

fn main() {
    let args = Args::parse();

    // Read the input file.
    if !Path::new(&args.input).exists() {
        eprintln!("Error: The file {} does not exist.", args.input);
        process::exit(1);
    }
    let content = fs::read_to_string(&args.input).unwrap_or_else(|err| {
        eprintln!("Error reading {}: {}", args.input, err);
        process::exit(1);
    });
    let cleaned = remove_comments(&content);

    // Parse the JSON.
    let data: Value = serde_json::from_str(&cleaned).unwrap_or_else(|err| {
        eprintln!("Error parsing JSON from {}: {}", args.input, err);
        process::exit(1);
    });

    // Our final compile commands will be collected here.
    let mut compile_commands = Vec::new();

    // If the top-level JSON is an array, assume it’s already a list of compile commands.
    if let Some(arr) = data.as_array() {
        compile_commands = arr.clone();
    }
    // Otherwise, if it is an object with "configurations", process it.
    else if let Some(obj) = data.as_object() {
        if let Some(configs) = obj.get("configurations").and_then(Value::as_array) {
            if configs.is_empty() {
                eprintln!("Error: No configurations found in {}.", args.input);
                process::exit(1);
            }
            // For simplicity, use the first configuration.
            let config = &configs[0];

            // If the configuration contains a "compileCommands" key, use merge mode.
            if let Some(cc_field) = config.get("compileCommands") {
                // Two cases:
                // 1. If cc_field is an array and its first element is a string,
                //    treat each element as a file path.
                // 2. Otherwise, assume cc_field is directly an array of compile command objects.
                if let Some(arr) = cc_field.as_array() {
                    if !arr.is_empty() && arr[0].is_string() {
                        // Each element is a file path.
                        for file_val in arr {
                            if let Some(path_str) = file_val.as_str() {
                                if !Path::new(path_str).exists() {
                                    eprintln!("Warning: Referenced file {} does not exist. Skipping.", path_str);
                                    continue;
                                }
                                let file_content = fs::read_to_string(path_str).unwrap_or_else(|err| {
                                    eprintln!("Error reading {}: {}", path_str, err);
                                    process::exit(1);
                                });
                                let file_cleaned = remove_comments(&file_content);
                                let cc_entries: Value = serde_json::from_str(&file_cleaned).unwrap_or_else(|err| {
                                    eprintln!("Error parsing JSON from {}: {}", path_str, err);
                                    process::exit(1);
                                });
                                if let Some(entries_arr) = cc_entries.as_array() {
                                    compile_commands.extend(entries_arr.clone());
                                } else {
                                    eprintln!("Error: {} does not contain a JSON array.", path_str);
                                    process::exit(1);
                                }
                            }
                        }
                    } else {
                        // Otherwise, assume it is directly an array of compile command objects.
                        compile_commands = arr.clone();
                    }
                } else {
                    eprintln!("Error: The 'compileCommands' field is not an array.");
                    process::exit(1);
                }
            }
            // Otherwise, if no "compileCommands" field exists, use generate mode.
            else {
                if args.sources.is_empty() {
                    eprintln!("Error: No compileCommands field found and no source files were provided.");
                    process::exit(1);
                }
                // Get necessary fields from the configuration.
                let compiler_path = config.get("compilerPath").and_then(Value::as_str).unwrap_or_else(|| {
                    eprintln!("Error: 'compilerPath' not found in configuration.");
                    process::exit(1);
                });
                let include_paths = config
                    .get("includePath")
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<&str>>()
                    })
                    .unwrap_or_default();
                let defines = config
                    .get("defines")
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<&str>>()
                    })
                    .unwrap_or_default();
                let cpp_standard = config.get("cppStandard").and_then(Value::as_str).unwrap_or("");

                // Get current working directory.
                let current_dir = env::current_dir()
                    .ok()
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| ".".to_string());

                // For each source file, generate a compile command.
                for source in args.sources.iter() {
                    let include_flags = include_paths
                        .iter()
                        .map(|path| format!("-I{}", path))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let define_flags = defines
                        .iter()
                        .map(|d| format!("-D{}", d))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let command = format!(
                        "{} {} {} --std={} -c {}",
                        compiler_path, include_flags, define_flags, cpp_standard, source
                    );
                    compile_commands.push(json!({
                        "directory": current_dir,
                        "command": command,
                        "file": source
                    }));
                }
            }
        } else {
            eprintln!("Error: 'configurations' key not found in {}.", args.input);
            process::exit(1);
        }
    } else {
        eprintln!("Error: Unexpected JSON structure in {}.", args.input);
        process::exit(1);
    }

    // Write out the merged/generated compile commands.
    let output_content = serde_json::to_string_pretty(&compile_commands).unwrap_or_else(|err| {
        eprintln!("Error serializing output JSON: {}", err);
        process::exit(1);
    });
    fs::write(&args.output, output_content).unwrap_or_else(|err| {
        eprintln!("Error writing to {}: {}", args.output, err);
        process::exit(1);
    });
    println!("{} has been generated.", args.output);
}
