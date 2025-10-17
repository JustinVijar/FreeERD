mod ast;
mod lexer;
mod parser;
mod interpreter;
mod svg_generator;

use parser::Parser;
use crate::interpreter::Interpreter;
use svg_generator::SvgGenerator;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const VERSION: &str = "0.1.1 BETA";

fn main() {
    let quote = get_random_quote();
    println!("{}", r#"
/$$$$$$$$                            /$$$$$$$$ /$$$$$$$  /$$$$$$$ 
| $$_____/                           | $$_____/| $$__  $$| $$__  $$
| $$     /$$$$$$   /$$$$$$   /$$$$$$ | $$      | $$  \ $$| $$  \ $$
| $$$$$ /$$__  $$ /$$__  $$ /$$__  $$| $$$$$   | $$$$$$$/| $$  | $$
| $$__/| $$  \__/| $$$$$$$$| $$$$$$$$| $$__/   | $$__  $$| $$  | $$
| $$   | $$      | $$_____/| $$_____/| $$      | $$  \ $$| $$  | $$
| $$   | $$      |  $$$$$$$|  $$$$$$$| $$$$$$$$| $$  | $$| $$$$$$$/
|__/   |__/       \_______/ \_______/|________/|__/  |__/|_______/ 
    "#);
    println!("Version {}", VERSION);
    if let Some(q) = quote {
        println!("  {}", q);
    }
    println!("{}", "=".repeat(60));
    
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "help" | "--help" | "-h" => {
            print_help();
        }
        "check" => {
            if args.len() < 3 {
                eprintln!("❌ Error: Missing file path");
                eprintln!("Usage: free-erd check <file>");
                std::process::exit(1);
            }
            
            let file_path = &args[2];
            
            if let Err(e) = check_file(file_path) {
                eprintln!("❌ Error: {}", e);
                std::process::exit(1);
            }
        }
        "run" => {
            if args.len() < 4 {
                eprintln!("❌ Error: Missing arguments");
                eprintln!("Usage: free-erd run <file> <command> [output]");
                eprintln!("\nAvailable commands:");
                eprintln!("  svg - Generate SVG diagram");
                std::process::exit(1);
            }
            
            let file_path = &args[2];
            let subcommand = &args[3];
            
            match subcommand.as_str() {
                "svg" => {
                    let output_path = if args.len() > 4 {
                        args[4].clone()
                    } else {
                        // Generate output filename from input
                        let path = Path::new(file_path);
                        let stem = path.file_stem().unwrap().to_str().unwrap();
                        format!("{}.svg", stem)
                    };
                    
                    if let Err(e) = export_svg(file_path, &output_path) {
                        eprintln!("❌ Error: {}", e);
                        std::process::exit(1);
                    }
                }
                _ => {
                    eprintln!("❌ Unknown run command: {}", subcommand);
                    eprintln!("Available commands: svg");
                    std::process::exit(1);
                }
            }
        }
        "about" => {
            print_about();
        }
        _ => {
            eprintln!("❌ Unknown command: {}", command);
            eprintln!("Run 'free-erd help' for usage information.");
            std::process::exit(1);
        }
    }
}

fn check_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err(format!("File not found: {}", file_path).into());
    }
    
    println!("\n📂 Reading file: {}", file_path);
    let content = fs::read_to_string(path)?;
    
    println!("🔍 Parsing...");
    let mut parser = Parser::new(&content);
    let schema = match parser.parse() {
        Ok(s) => s,
        Err(e) => {
            eprint!("\n{}", e.format_with_source(&content, file_path));
            return Err(e.into());
        }
    };
    println!("✅ Parsing successful!\n");
    
    let interpreter = Interpreter::new(schema);
    
    println!("🔍 Validating schema...");
    match interpreter.validate() {
        Ok(_) => {
            println!("✅ Schema is valid!\n");
            let stats = interpreter.get_statistics();
            stats.print();
        }
        Err(errors) => {
            eprintln!("\n\x1b[1;31m❌ Validation failed with {} error(s):\x1b[0m\n", errors.len());
            for error in errors.iter() {
                eprint!("{}", error.format_with_source(&content, file_path));
            }
            return Err("Validation failed".into());
        }
    }
    
    Ok(())
}

fn export_svg(file_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err(format!("File not found: {}", file_path).into());
    }
    
    println!("\n📂 Reading file: {}", file_path);
    let content = fs::read_to_string(path)?;
    
    println!("🔍 Parsing...");
    let mut parser = Parser::new(&content);
    let schema = match parser.parse() {
        Ok(s) => s,
        Err(e) => {
            eprint!("\n{}", e.format_with_source(&content, file_path));
            return Err(e.into());
        }
    };
    
    println!("✅ Parsing successful!");
    
    let interpreter = Interpreter::new(schema.clone());
    
    println!("🔍 Validating schema...");
    match interpreter.validate() {
        Ok(_) => {
            println!("✅ Schema is valid!");
        }
        Err(errors) => {
            eprintln!("\n\x1b[1;31m❌ Validation failed with {} error(s):\x1b[0m\n", errors.len());
            for error in errors.iter() {
                eprint!("{}", error.format_with_source(&content, file_path));
            }
            eprintln!("\n\x1b[1;31m❌ Cannot generate SVG with validation errors.\x1b[0m");
            eprintln!("\x1b[1;33m💡 Fix the errors above and try again.\x1b[0m\n");
            return Err("Validation failed".into());
        }
    }
    
    println!("🎨 Generating SVG diagram...");
    let generator = SvgGenerator::new(schema);
    let svg_content = generator.generate_with_defs();
    
    println!("💾 Writing to: {}", output_path);
    fs::write(output_path, svg_content)?;
    
    println!("✅ SVG diagram created successfully!");
    println!("\n📊 Output: {}", output_path);
    
    Ok(())
}

fn print_usage() {
    println!("\nUsage:");
    println!("  free-erd <command> [options]\n");
    println!("Commands:");
    println!("  help                      Shows the help menu (this menu)");
    println!("  check <file>              Checks the .frd file if there are errors");
    println!("  run <file> <command> [output]  Runs the .frd file with specified command");
    println!("  about                     Info about this program\n");
    println!("run subcommands:");
    println!("  svg                       Outputs an SVG file of the ERD\n");
}

fn print_help() {
    println!("\n🎨 FreeERD - Entity Relationship Diagram DSL Interpreter\n");
    println!("A lightweight domain-specific language for defining database schemas");
    println!("and entity relationships in a simple, human-readable format.\n");
    
    print_usage();

}

fn print_about() {
    println!("\n🎨 FreeERD - Free Entity Relationship Diagram Tool");
    println!("Version: {}\n", VERSION);
    println!("Description:");
    println!("  A lightweight, open-source tool for creating Entity Relationship Diagrams");
    println!("  using a simple domain-specific language. FreeERD allows you to define");
    println!("  database schemas in a human-readable format and generate beautiful SVG");
    println!("  diagrams automatically.\n");
    println!("Features:");
    println!("  • Simple, intuitive syntax for defining tables and relationships");
    println!("  • Comprehensive validation with detailed error messages");
    println!("  • Automatic SVG diagram generation with smart layout");
    println!("  • Support for various data types and column attributes");
    println!("  • Multiple relationship types (1:1, 1:N, N:M)\n");
    println!("License: GNU General Public License v2.0");
    println!("  This program is free software; you can redistribute it and/or modify it");
    println!("  under the terms of the GNU General Public License version 2 as published");
    println!("  by the Free Software Foundation. This program is distributed in the hope");
    println!("  that it will be useful, but WITHOUT ANY WARRANTY; without even the implied");
    println!("  warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.");
    println!("  See the LICENSE file for more details.\n");
    println!("Repository: https://github.com/JustinVijar/FreeERD");
    println!("\nFor help and usage information, run: free-erd help\n");
}

fn get_random_quote() -> Option<String> {
    // Embed quotes.txt content at compile time
    const QUOTES: &str = include_str!("quotes.txt");
    
    let quotes: Vec<&str> = QUOTES
        .lines()
        .filter(|line| !line.trim().is_empty() && line.trim() != ".")
        .collect();
    
    if quotes.is_empty() {
        return None;
    }
    
    // Use current time as seed for pseudo-random selection
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let index = (seed as usize) % quotes.len();
    Some(quotes[index].to_string())
}
