use crate::ast::{Schema, Table, Column, Relationship, RelationshipType, DataType, Attribute};
use std::process::{Command, Stdio};
use std::io::Write;

pub struct SvgGenerator {
    schema: Schema,
}

impl SvgGenerator {
    pub fn new(schema: Schema) -> Self {
        SvgGenerator { schema }
    }

    pub fn generate_with_defs(&self) -> String {
        // Generate DOT format
        let dot_content = self.generate_dot();
        
        println!("🎨 Generating SVG using Graphviz...");
        
        // Call Graphviz dot command to render SVG
        match self.render_svg_from_dot(&dot_content) {
            Ok(svg) => {
                println!("✅ SVG generated successfully using Graphviz!");
                svg
            },
            Err(e) => {
                eprintln!("❌ Error generating SVG: {}", e);
                String::from("<svg><text>Error generating diagram</text></svg>")
            }
        }
    }
    
    fn generate_dot(&self) -> String {
        let mut dot = String::new();
        
        // Start digraph
        dot.push_str("digraph ERD {\n");
        
        // Graph attributes
        dot.push_str("  bgcolor=\"#f8f9fa\";\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  nodesep=2.0;\n");  // Increased horizontal spacing
        dot.push_str("  ranksep=2.0;\n");  // Increased vertical spacing
        dot.push_str("  splines=ortho;\n");
        dot.push_str("  pad=0.5;\n");
        dot.push_str("  concentrate=false;\n");  // Don't merge edges
        
        // Default node attributes
        dot.push_str("  node [shape=plain, fontname=\"Arial\", fontsize=11];\n");
        
        // Default edge attributes
        dot.push_str("  edge [color=\"#34495e\", fontsize=9, fontname=\"Arial\", penwidth=1.5];\n");
        
        // Add title if present
        if let Some(title) = &self.schema.title {
            dot.push_str(&format!("  label=\"{}\n\";\n", self.escape_dot(title)));
            dot.push_str("  labelloc=t;\n");
            dot.push_str("  labeljust=c;\n");
            dot.push_str("  fontsize=24;\n");
        }
        
        dot.push_str("\n");
        
        // Add tables as nodes
        for table in &self.schema.tables {
            self.add_table_node(&mut dot, table);
        }
        
        dot.push_str("\n");
        
        // Add relationships as edges
        for relationship in &self.schema.relationships {
            self.add_relationship_edge(&mut dot, relationship);
        }
        
        dot.push_str("}\n");
        
        dot
    }
    
    fn add_table_node(&self, dot: &mut String, table: &Table) {
        let node_id = &table.name;
        
        // Build HTML-like label for better styling
        let mut label = String::from("<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\" CELLPADDING=\"8\">");
        
        // Table header row with darker background
        label.push_str(&format!(
            "<TR><TD BGCOLOR=\"#3498db\" COLSPAN=\"1\"><FONT COLOR=\"white\" POINT-SIZE=\"13\"><B>{}</B></FONT></TD></TR>",
            self.escape_html(&table.name)
        ));
        
        // Column rows with alternating colors
        for (i, col) in table.columns.iter().enumerate() {
            let bg_color = if i % 2 == 0 { "#f8f9fa" } else { "#ffffff" };
            let column_text = self.format_column_html(col);
            label.push_str(&format!(
                "<TR><TD BGCOLOR=\"{}\" ALIGN=\"LEFT\">{}</TD></TR>",
                bg_color, column_text
            ));
        }
        
        label.push_str("</TABLE>>");
        
        // Add node
        dot.push_str(&format!("  \"{}\" [label={}];\n", 
            self.escape_dot(node_id), label));
    }
    
    fn format_column_html(&self, column: &Column) -> String {
        let mut parts = Vec::new();
        
        // Column name
        parts.push(format!("<B>{}</B>", self.escape_html(&column.name)));
        
        // Data type in gray
        let type_str = self.format_datatype(&column.datatype);
        parts.push(format!("<FONT COLOR=\"#7f8c8d\">{}</FONT>", type_str));
        
        // Attributes in blue
        if !column.attributes.is_empty() {
            let attrs: Vec<String> = column.attributes.iter().map(|attr| {
                self.format_attribute(attr)
            }).collect();
            parts.push(format!("<FONT COLOR=\"#3498db\">[{}]</FONT>", attrs.join(",")));
        }
        
        parts.join(" ")
    }
    
    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
    
    
    fn format_datatype(&self, datatype: &DataType) -> String {
        match datatype {
            DataType::String => "string",
            DataType::Int => "int",
            DataType::Bool => "bool",
            DataType::Double => "double",
            DataType::Float => "float",
            DataType::Date => "date",
            DataType::Time => "time",
            DataType::DateTime => "datetime",
            DataType::Blob => "blob",
            DataType::TinyBlob => "tinyblob",
            DataType::LargeBlob => "largeblob",
            DataType::Custom(s) => s,
        }.to_string()
    }
    
    fn format_attribute(&self, attr: &Attribute) -> String {
        match attr {
            Attribute::PrimaryKey => "PK",
            Attribute::ForeignKey => "FK",
            Attribute::Unique => "UQ",
            Attribute::Nullable => "NULL",
            Attribute::AutoIncrement => "AI",
            Attribute::Default(_) => "DEF",
        }.to_string()
    }
    
    fn add_relationship_edge(&self, dot: &mut String, relationship: &Relationship) {
        let from_node = &relationship.from_table;
        let to_node = &relationship.to_table;
        
        // Create label showing the relationship fields
        let label = format!("{}.{} → {}.{}", 
            from_node, relationship.from_field,
            to_node, relationship.to_field);
        
        // Build edge with attributes and xlabel
        let mut edge = format!("  \"{}\" -> \"{}\" [", 
            self.escape_dot(from_node), self.escape_dot(to_node));
        
        // Add xlabel without background highlight
        edge.push_str(&format!(
            "xlabel=<<FONT POINT-SIZE=\"8\">{}</FONT>>, ",
            self.escape_html(&label)
        ));
        
        // Set edge style based on relationship type
        match relationship.relationship_type {
            RelationshipType::OneToOne => {
                edge.push_str("arrowhead=none, arrowtail=none, style=dashed");
            }
            RelationshipType::OneToMany => {
                edge.push_str("arrowhead=crow, arrowtail=none, dir=forward");
            }
            RelationshipType::ManyToOne => {
                edge.push_str("arrowhead=none, arrowtail=crow, dir=back");
            }
            RelationshipType::ManyToMany => {
                edge.push_str("arrowhead=crow, arrowtail=crow, dir=both");
            }
        }
        
        edge.push_str("];\n");
        dot.push_str(&edge);
    }
    
    fn render_svg_from_dot(&self, dot_content: &str) -> Result<String, String> {
        // Limit DOT content size to prevent DoS (5MB max)
        if dot_content.len() > 5_000_000 {
            return Err("DOT content too large (max 5MB)".to_string());
        }
        
        let mut child = Command::new("dot")
            .arg("-Tsvg")
            .arg("-Gmaxiter=100")  // Limit Graphviz iterations
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn dot command: {}. Is Graphviz installed?", e))?;
        
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(dot_content.as_bytes())
                .map_err(|e| format!("Failed to write to dot stdin: {}", e))?;
        }
        
        let output = child.wait_with_output()
            .map_err(|e| format!("Failed to wait for dot command: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("dot command failed: {}", stderr));
        }
        
        String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 in SVG output: {}", e))
    }
    
    fn escape_dot(&self, text: &str) -> String {
        text.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('|', "\\|")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('<', "\\<")
            .replace('>', "\\>")
    }
}

impl Clone for SvgGenerator {
    fn clone(&self) -> Self {
        SvgGenerator {
            schema: self.schema.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_schema() -> Schema {
        Schema {
            title: Some("Test Schema".to_string()),
            tables: vec![
                Table {
                    name: "Users".to_string(),
                    columns: vec![
                        Column {
                            name: "id".to_string(),
                            datatype: DataType::Int,
                            attributes: vec![Attribute::PrimaryKey, Attribute::AutoIncrement],
                            span: None,
                        },
                        Column {
                            name: "email".to_string(),
                            datatype: DataType::String,
                            attributes: vec![Attribute::Unique],
                            span: None,
                        },
                    ],
                    span: None,
                },
                Table {
                    name: "Posts".to_string(),
                    columns: vec![
                        Column {
                            name: "id".to_string(),
                            datatype: DataType::Int,
                            attributes: vec![Attribute::PrimaryKey],
                            span: None,
                        },
                        Column {
                            name: "user_id".to_string(),
                            datatype: DataType::Int,
                            attributes: vec![Attribute::ForeignKey],
                            span: None,
                        },
                    ],
                    span: None,
                },
            ],
            relationships: vec![
                Relationship {
                    from_table: "Users".to_string(),
                    from_field: "id".to_string(),
                    to_table: "Posts".to_string(),
                    to_field: "user_id".to_string(),
                    relationship_type: RelationshipType::OneToMany,
                    span: None,
                },
            ],
        }
    }

    #[test]
    fn test_generator_creation() {
        let schema = create_test_schema();
        let generator = SvgGenerator::new(schema.clone());
        assert_eq!(generator.schema.tables.len(), 2);
        assert_eq!(generator.schema.relationships.len(), 1);
    }

    #[test]
    fn test_dot_generation() {
        let schema = create_test_schema();
        let generator = SvgGenerator::new(schema);
        let dot = generator.generate_dot();
        
        // Check DOT structure
        assert!(dot.contains("digraph ERD"));
        assert!(dot.contains("Users"));
        assert!(dot.contains("Posts"));
        assert!(dot.contains("->"));
    }

    #[test]
    fn test_escape_dot() {
        let schema = Schema::new();
        let generator = SvgGenerator::new(schema);
        
        assert_eq!(generator.escape_dot("test\"quote"), "test\\\"quote");
        assert_eq!(generator.escape_dot("test\\slash"), "test\\\\slash");
        assert_eq!(generator.escape_dot("test\nline"), "test\\nline");
        assert_eq!(generator.escape_dot("test|pipe"), "test\\|pipe");
    }

    #[test]
    fn test_escape_html() {
        let schema = Schema::new();
        let generator = SvgGenerator::new(schema);
        
        assert_eq!(generator.escape_html("test<tag>"), "test&lt;tag&gt;");
        assert_eq!(generator.escape_html("test&amp;"), "test&amp;amp;");
        assert_eq!(generator.escape_html("test\"quote"), "test&quot;quote");
    }

    #[test]
    fn test_format_datatype() {
        let schema = Schema::new();
        let generator = SvgGenerator::new(schema);
        
        assert_eq!(generator.format_datatype(&DataType::String), "string");
        assert_eq!(generator.format_datatype(&DataType::Int), "int");
        assert_eq!(generator.format_datatype(&DataType::DateTime), "datetime");
        assert_eq!(generator.format_datatype(&DataType::Custom("uuid".to_string())), "uuid");
    }

    #[test]
    fn test_format_attribute() {
        let schema = Schema::new();
        let generator = SvgGenerator::new(schema);
        
        assert_eq!(generator.format_attribute(&Attribute::PrimaryKey), "PK");
        assert_eq!(generator.format_attribute(&Attribute::ForeignKey), "FK");
        assert_eq!(generator.format_attribute(&Attribute::Unique), "UQ");
        assert_eq!(generator.format_attribute(&Attribute::Nullable), "NULL");
        assert_eq!(generator.format_attribute(&Attribute::AutoIncrement), "AI");
    }

    #[test]
    fn test_relationship_types_in_dot() {
        let mut schema = Schema::new();
        schema.tables = vec![
            Table {
                name: "A".to_string(),
                columns: vec![],
                span: None,
            },
            Table {
                name: "B".to_string(),
                columns: vec![],
                span: None,
            },
        ];

        // Test OneToMany
        schema.relationships = vec![
            Relationship {
                from_table: "A".to_string(),
                from_field: "id".to_string(),
                to_table: "B".to_string(),
                to_field: "a_id".to_string(),
                relationship_type: RelationshipType::OneToMany,
                span: None,
            },
        ];
        let generator = SvgGenerator::new(schema.clone());
        let dot = generator.generate_dot();
        assert!(dot.contains("arrowhead=crow"));

        // Test OneToOne
        schema.relationships[0].relationship_type = RelationshipType::OneToOne;
        let generator = SvgGenerator::new(schema.clone());
        let dot = generator.generate_dot();
        assert!(dot.contains("style=dashed"));

        // Test ManyToMany
        schema.relationships[0].relationship_type = RelationshipType::ManyToMany;
        let generator = SvgGenerator::new(schema);
        let dot = generator.generate_dot();
        assert!(dot.contains("arrowtail=crow"));
    }

    #[test]
    fn test_title_in_dot() {
        let mut schema = Schema::new();
        schema.title = Some("My Database".to_string());
        
        let generator = SvgGenerator::new(schema);
        let dot = generator.generate_dot();
        
        assert!(dot.contains("My Database"));
        assert!(dot.contains("labelloc=t"));
    }

    #[test]
    fn test_clone() {
        let schema = create_test_schema();
        let generator = SvgGenerator::new(schema);
        let cloned = generator.clone();
        
        assert_eq!(generator.schema.tables.len(), cloned.schema.tables.len());
        assert_eq!(generator.schema.relationships.len(), cloned.schema.relationships.len());
    }

    #[test]
    fn test_empty_schema() {
        let schema = Schema::new();
        let generator = SvgGenerator::new(schema);
        let dot = generator.generate_dot();
        
        assert!(dot.contains("digraph ERD"));
        assert!(dot.contains("}"));
    }

    #[test]
    fn test_column_formatting() {
        let schema = create_test_schema();
        let generator = SvgGenerator::new(schema);
        
        let column = Column {
            name: "test_col".to_string(),
            datatype: DataType::String,
            attributes: vec![Attribute::PrimaryKey, Attribute::Unique],
            span: None,
        };
        
        let formatted = generator.format_column_html(&column);
        assert!(formatted.contains("test_col"));
        assert!(formatted.contains("string"));
        assert!(formatted.contains("PK"));
        assert!(formatted.contains("UQ"));
    }
}

