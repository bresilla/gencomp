# gencomp

`gencomp` is a command-line tool to generate `compile_commands.json` from `c_cpp_properties.json`. This is useful for integrating with tools that require a compilation database, such as `clang-tidy` and `clangd`.

## Features

- Converts `c_cpp_properties.json` to `compile_commands.json`
- Supports specifying source files directly as arguments
- Allows custom paths for input and output files
- Handles JSON files with comments

## Installation

### Prerequisites

- Python 3.6 or later

### Download and Setup

1. Clone the repository:

   ```sh
   git clone https://github.com/yourusername/gencomp.git
   cd gencomp

    Make the script executable:

    sh

    chmod +x gencomp

## Usage

```sh

    gencomp SOURCES [SOURCES ...] [-d DIRECTORY] [-o OUTPUT]

    Positional Arguments

        SOURCES: List of source files to include in the compile_commands.json.

    Optional Arguments

        -d DIRECTORY, --directory DIRECTORY: Path to c_cpp_properties.json. Default is .vscode/c_cpp_properties.json.
        -o OUTPUT, --output OUTPUT: Output path for compile_commands.json. Default is ./compile_commands.json.
```

## Examples

Generate compile_commands.json for main.cpp and another_file.cpp using the default c_cpp_properties.json location and output path:

```sh
    gencomp main.cpp another_file.cpp
```

Specify a custom c_cpp_properties.json file and output path:

```sh
    gencomp main.cpp another_file.cpp -d custom_path/c_cpp_properties.json -o custom_path/compile_commands.json
```
