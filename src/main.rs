mod ast;
mod lexer;
mod parser;
mod interpreter;
mod renderer;

use parser::Parser;
use crate::interpreter::Interpreter;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const VERSION: &str = "0.2.1 BETA";

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
    println!("{}", "=".repeat(107));
    
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
            if args.len() < 3 {
                eprintln!("❌ Error: Missing file path");
                eprintln!("Usage: free-erd run <filename>");
                std::process::exit(1);
            }
            
            let file_path = &args[2];
            
            if let Err(e) = open_window(file_path) {
                eprintln!("❌ Error: {}", e);
                std::process::exit(1);
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

fn print_usage() {
    println!("\nUsage:");
    println!("  free-erd <command> [arguments]\n");
    println!("Commands:");
    println!("  run <filename>               - opens the window to view the ERD");
    println!("  check <filename>             - checks the .frd file");
    println!("  help                         - Help menu");
    println!("  about                        - Information about this system\n");
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

fn open_window(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📂 Reading file: {}", file_path);
    let content = fs::read_to_string(file_path)?;
    
    println!("🔍 Parsing...");
    let mut parser = Parser::new(&content);
    let schema = match parser.parse() {
        Ok(s) => {
            println!("✅ Parsing successful!");
            s
        }
        Err(error) => {
            eprintln!("\n\x1b[1;31m❌ Parsing failed:\x1b[0m\n");
            eprint!("{}", error.format_with_source(&content, file_path));
            return Err("Parsing failed".into());
        }
    };
    
    println!("🔍 Validating schema...");
    let interpreter = Interpreter::new(schema.clone());
    if let Err(errors) = interpreter.validate() {
        eprintln!("\n\x1b[1;31m❌ Validation failed with {} error(s):\x1b[0m\n", errors.len());
        for error in errors.iter() {
            eprint!("{}", error.format_with_source(&content, file_path));
        }
        return Err("Validation failed".into());
    }
    println!("✅ Schema is valid!");
    
    // Convert schema to ERD graph
    println!("🎨 Building ERD graph...");
    let mut erd_graph = renderer::ErdGraph::new();
    
    // Add tables
    for table in &schema.tables {
        let columns: Vec<renderer::ColumnData> = table.columns.iter().map(|col| {
            renderer::ColumnData {
                name: col.name.clone(),
                data_type: col.datatype.to_string(),
                attributes: col.attributes.iter().map(|attr| attr.to_string()).collect(),
            }
        }).collect();
        
        erd_graph.add_table(renderer::TableNode {
            name: table.name.clone(),
            columns,
        });
    }
    
    // Add relationships
    for rel in &schema.relationships {
        let rel_type = match rel.relationship_type {
            ast::RelationshipType::OneToOne => renderer::RelationType::OneToOne,
            ast::RelationshipType::OneToMany => renderer::RelationType::OneToMany,
            ast::RelationshipType::ManyToOne => renderer::RelationType::ManyToOne,
            ast::RelationshipType::ManyToMany => renderer::RelationType::ManyToMany,
        };
        
        erd_graph.add_relationship(
            &rel.from_table,
            &rel.to_table,
            renderer::RelationshipEdge {
                from_field: rel.from_field.clone(),
                to_field: rel.to_field.clone(),
                relationship_type: rel_type,
            },
        )?;
    }
    
    println!("🪟 Opening window...");
    let title = schema.title.clone().unwrap_or_else(|| "Untitled Schema".to_string());
    renderer::render_window(erd_graph, title)?;
    
    Ok(())
}

