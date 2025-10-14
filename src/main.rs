mod ast;
mod lexer;
mod parser;
mod interpreter;
mod svg_generator;

use parser::Parser;
use interpreter::Interpreter;
use svg_generator::SvgGenerator;
use std::fs;
use std::path::Path;

fn main() {
    println!("🎨 FreeERD Interpreter v0.1.0");
    println!("{}", "=".repeat(60));
    
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "parse" | "run" | "validate" => {
            if args.len() < 3 {
                eprintln!("❌ Error: Missing file path");
                print_usage();
                return;
            }
            
            let file_path = &args[2];
            
            if let Err(e) = process_file(file_path, command) {
                eprintln!("❌ Error: {}", e);
                std::process::exit(1);
            }
        }
        "svg" | "export" => {
            if args.len() < 3 {
                eprintln!("❌ Error: Missing file path");
                print_usage();
                return;
            }
            
            let file_path = &args[2];
            let output_path = if args.len() > 3 {
                args[3].clone()
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
        "example" => {
            run_example();
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        _ => {
            eprintln!("❌ Unknown command: {}", command);
            print_usage();
        }
    }
}

fn process_file(file_path: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err(format!("File not found: {}", file_path).into());
    }
    
    println!("\n📂 Reading file: {}", file_path);
    let content = fs::read_to_string(path)?;
    
    println!("🔍 Parsing...");
    let mut parser = Parser::new(&content);
    let schema = parser.parse()?;
    
    println!("✅ Parsing successful!\n");
    
    let interpreter = Interpreter::new(schema);
    
    match command {
        "validate" => {
            println!("🔍 Validating schema...");
            match interpreter.validate() {
                Ok(_) => {
                    println!("✅ Schema is valid!\n");
                    let stats = interpreter.get_statistics();
                    stats.print();
                }
                Err(errors) => {
                    println!("❌ Validation failed with {} error(s):\n", errors.len());
                    for (i, error) in errors.iter().enumerate() {
                        println!("  {}. {}", i + 1, error);
                    }
                    return Err("Validation failed".into());
                }
            }
        }
        _ => {
            // For "parse" and "run", just validate and display
            match interpreter.validate() {
                Ok(_) => {
                    interpreter.print_schema();
                    let stats = interpreter.get_statistics();
                    stats.print();
                }
                Err(errors) => {
                    println!("⚠️  Found {} validation error(s):\n", errors.len());
                    for (i, error) in errors.iter().enumerate() {
                        println!("  {}. {}", i + 1, error);
                    }
                    println!("\n📊 Displaying schema anyway...\n");
                    interpreter.print_schema();
                }
            }
        }
    }
    
    Ok(())
}

fn run_example() {
    let example = r#"title "E-Commerce Database"

table Users {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique],
  created_at: datetime [default=NOW]
}

table Products {
  id: int [pk, autoincrement],
  name: str [unique],
  description: str [nullable],
  price: float,
  stock: int [default=0]
}

table Orders {
  id: int [pk, autoincrement],
  user_id: int [fk],
  order_date: datetime [default=NOW],
  status: str [default="pending"]
}

table OrderItems {
  id: int [pk, autoincrement],
  order_id: int [fk],
  product_id: int [fk],
  quantity: int [default=1],
  price: float
}

// Relationships
Users.id > Orders.user_id
Orders.id > OrderItems.order_id
Products.id > OrderItems.product_id
"#;
    
    println!("\n📝 Running example schema:\n");
    println!("{}", example);
    println!("\n{}", "=".repeat(60));
    
    let mut parser = Parser::new(example);
    match parser.parse() {
        Ok(schema) => {
            let interpreter = Interpreter::new(schema);
            
            match interpreter.validate() {
                Ok(_) => {
                    interpreter.print_schema();
                    let stats = interpreter.get_statistics();
                    stats.print();
                }
                Err(errors) => {
                    println!("❌ Validation errors:");
                    for error in errors {
                        println!("  • {}", error);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
        }
    }
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
    let schema = parser.parse()?;
    
    println!("✅ Parsing successful!");
    
    let interpreter = Interpreter::new(schema.clone());
    
    println!("🔍 Validating schema...");
    match interpreter.validate() {
        Ok(_) => {
            println!("✅ Schema is valid!");
        }
        Err(errors) => {
            println!("⚠️  Found {} validation error(s), continuing anyway...", errors.len());
            for (i, error) in errors.iter().enumerate() {
                println!("  {}. {}", i + 1, error);
            }
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
    println!("  parse <file>              Parse and display a .frd file");
    println!("  run <file>                Parse, validate, and display a .frd file");
    println!("  validate <file>           Validate a .frd file schema");
    println!("  svg <file> [output]       Generate SVG diagram (default: <file>.svg)");
    println!("  export <file> [output]    Alias for svg command");
    println!("  example                   Run with a built-in example");
    println!("  help                      Show detailed help information\n");
    println!("Note:");
    println!("  SVG files can be converted to PNG using tools like:");
    println!("  - inkscape: inkscape diagram.svg -o diagram.png");
    println!("  - rsvg-convert: rsvg-convert -o diagram.png diagram.svg");
    println!("  - imagemagick: convert diagram.svg diagram.png\n");
}

fn print_help() {
    println!("\n🎨 FreeERD - Entity Relationship Diagram DSL Interpreter\n");
    println!("A lightweight domain-specific language for defining database schemas");
    println!("and entity relationships in a simple, human-readable format.\n");
    
    print_usage();
    
    println!("Examples:");
    println!("  free-erd example");
    println!("  free-erd parse schema.frd");
    println!("  free-erd validate database.frd");
    println!("  free-erd svg schema.frd");
    println!("  free-erd svg schema.frd diagram.svg\n");
    
    println!("File Format:");
    println!("  FreeERD files use the .frd extension and contain:");
    println!("  • Title declarations");
    println!("  • Table definitions with columns and attributes");
    println!("  • Relationship definitions\n");
    
    println!("For more information, see the README_FORAI.md file.\n");
}
