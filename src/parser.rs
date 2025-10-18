use crate::ast::*;
use crate::lexer::{Lexer, Token, Spanned, Span};

pub struct Parser {
    tokens: Vec<Spanned<Token>>,
    position: usize,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { expected: String, found: Token, span: Span },
    InvalidAttribute { name: String, span: Span },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found, .. } => {
                write!(f, "Expected {}, but found {}", expected, found)
            }
            ParseError::InvalidAttribute { name, .. } => write!(f, "Invalid attribute: {}", name),
        }
    }
}

impl ParseError {
    pub fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => *span,
            ParseError::InvalidAttribute { span, .. } => *span,
        }
    }
    
    /// Format error with Rust-like error messages including line numbers and source context
    pub fn format_with_source(&self, source: &str, filename: &str) -> String {
        let mut output = String::new();
        
        // Error header
        output.push_str(&format!("\x1b[1;31merror\x1b[0m: {}\n", self));
        
        let span = self.span();
        // Location info
        output.push_str(&format!("  \x1b[1;34m-->\x1b[0m {}:{}:{}\n", filename, span.line, span.column));
        output.push_str("   \x1b[1;34m|\x1b[0m\n");
        
        // Get the source line
        if let Some(line_text) = source.lines().nth(span.line - 1) {
            // Line number and source
            output.push_str(&format!(" \x1b[1;34m{:>3} |\x1b[0m {}\n", span.line, line_text));
            
            // Error indicator (^^^)
            output.push_str("   \x1b[1;34m|\x1b[0m ");
            output.push_str(&" ".repeat(span.column - 1));
            output.push_str(&format!("\x1b[1;31m{}\x1b[0m", "^".repeat(span.length.max(1))));
            output.push_str("\n");
        }
        
        output
    }
}

impl std::error::Error for ParseError {}

// Helper enum to distinguish between relationships and shorthand edges
enum IdentifierStatement {
    Relationship(Relationship),
    ShorthandEdge(Edge),
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        Parser {
            tokens,
            position: 0,
        }
    }
    
    fn current_token(&self) -> &Token {
        self.tokens.get(self.position).map(|t| &t.value).unwrap_or(&Token::Eof)
    }
    
    fn current_span(&self) -> Span {
        self.tokens.get(self.position).map(|t| t.span).unwrap_or(Span::new(0, 0, 0))
    }
    
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
    
    fn skip_newlines(&mut self) {
        while matches!(self.current_token(), Token::Newline) {
            self.advance();
        }
    }
    
    fn skip_comments_and_newlines(&mut self) {
        loop {
            match self.current_token() {
                Token::Newline | Token::Comment(_) => self.advance(),
                _ => break,
            }
        }
    }
    
    fn expect_token(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current_token() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{}", expected),
                found: self.current_token().clone(),
                span: self.current_span(),
            })
        }
    }
    
    pub fn parse(&mut self) -> Result<Schema, ParseError> {
        let mut schema = Schema::new();
        
        self.skip_comments_and_newlines();
        
        while !matches!(self.current_token(), Token::Eof) {
            match self.current_token() {
                Token::Title => {
                    schema.title = Some(self.parse_title()?);
                }
                Token::Table => {
                    let table = self.parse_table()?;
                    schema.tables.push(table);
                }
                Token::Node => {
                    let node = self.parse_node()?;
                    schema.nodes.push(node);
                }
                Token::Edge => {
                    let edge = self.parse_complex_edge()?;
                    schema.edges.push(edge);
                }
                Token::Identifier(_) => {
                    // Could be a relationship or shorthand edge
                    let edge_or_rel = self.parse_identifier_statement()?;
                    match edge_or_rel {
                        IdentifierStatement::Relationship(rel) => schema.relationships.push(rel),
                        IdentifierStatement::ShorthandEdge(edge) => schema.edges.push(edge),
                    }
                }
                Token::Comment(_) | Token::Newline => {
                    self.advance();
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "title, table, node, edge, or relationship".to_string(),
                        found: self.current_token().clone(),
                        span: self.current_span(),
                    });
                }
            }
            
            self.skip_comments_and_newlines();
        }
        
        Ok(schema)
    }
    
    fn parse_identifier_statement(&mut self) -> Result<IdentifierStatement, ParseError> {
        // Peek ahead to determine if this is a shorthand edge or relationship
        // Shorthand edge: NodeName -[EDGE]-> NodeName
        // Relationship: Table.field > Table.field
        
        let start_pos = self.position;
        let _first_ident = if let Token::Identifier(name) = self.current_token() {
            name.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        self.advance();
        
        // Check what comes next
        match self.current_token() {
            Token::OutgoingEdge | Token::IncomingEdge | Token::BidirectionalEdge => {
                // This is a shorthand edge with old syntax (not implemented), reset and parse it
                self.position = start_pos;
                Ok(IdentifierStatement::ShorthandEdge(self.parse_shorthand_edge()?))
            }
            Token::OneToOne | Token::ManyToOne => {
                // This is a shorthand edge: Node -[EDGE]-> Node or Node <-[EDGE]- Node
                self.position = start_pos;
                Ok(IdentifierStatement::ShorthandEdge(self.parse_shorthand_edge()?))
            }
            Token::Dot => {
                // This is a relationship, reset and parse it
                self.position = start_pos;
                Ok(IdentifierStatement::Relationship(self.parse_relationship()?))
            }
            _ => {
                Err(ParseError::UnexpectedToken {
                    expected: "'.', '-[]->', '<-[]-', or '<-[]->'".to_string(),
                    found: self.current_token().clone(),
                    span: self.current_span(),
                })
            }
        }
    }
    
    fn parse_title(&mut self) -> Result<String, ParseError> {
        self.expect_token(Token::Title)?;
        self.skip_newlines();
        
        if let Token::String(title) = self.current_token() {
            let title = title.clone();
            self.advance();
            Ok(title)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "string".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            })
        }
    }
    
    fn parse_table(&mut self) -> Result<Table, ParseError> {
        self.expect_token(Token::Table)?;
        self.skip_newlines();
        
        let table_name_span = self.current_span();
        let table_name = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "table name".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.skip_newlines();
        self.expect_token(Token::LeftBrace)?;
        self.skip_comments_and_newlines();
        
        let mut table = Table::with_span(table_name, table_name_span);
        
        while !matches!(self.current_token(), Token::RightBrace | Token::Eof) {
            let column = self.parse_column()?;
            table.columns.push(column);
            
            self.skip_comments_and_newlines();
            
            if matches!(self.current_token(), Token::Comma) {
                self.advance();
                self.skip_comments_and_newlines();
            }
        }
        
        self.expect_token(Token::RightBrace)?;
        
        Ok(table)
    }
    
    fn parse_node(&mut self) -> Result<Node, ParseError> {
        self.expect_token(Token::Node)?;
        self.skip_newlines();
        
        let node_name_span = self.current_span();
        let node_name = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            
            // Validate PascalCase
            if !Self::is_pascal_case(&name) {
                return Err(ParseError::InvalidAttribute {
                    name: format!("Node name '{}' must be PascalCase", name),
                    span: node_name_span,
                });
            }
            
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "node name (PascalCase)".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.skip_newlines();
        self.expect_token(Token::LeftBrace)?;
        self.skip_comments_and_newlines();
        
        let mut node = Node::with_span(node_name, node_name_span);
        
        while !matches!(self.current_token(), Token::RightBrace | Token::Eof) {
            let field = self.parse_node_field()?;
            node.fields.push(field);
            
            self.skip_comments_and_newlines();
            
            if matches!(self.current_token(), Token::Comma) {
                self.advance();
                self.skip_comments_and_newlines();
            }
        }
        
        self.expect_token(Token::RightBrace)?;
        
        Ok(node)
    }
    
    fn parse_node_field(&mut self) -> Result<NodeField, ParseError> {
        let field_start_span = self.current_span();
        let field_name = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            
            // Validate lowercase or snake_case
            if !Self::is_snake_case(&name) {
                return Err(ParseError::InvalidAttribute {
                    name: format!("Field name '{}' must be lowercase or snake_case", name),
                    span: field_start_span,
                });
            }
            
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "field name (lowercase or snake_case)".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.expect_token(Token::Colon)?;
        
        let datatype_name = if let Token::Identifier(dt) = self.current_token() {
            let dt = dt.clone();
            self.advance();
            dt
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "datatype".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        let datatype = DataType::from_str(&datatype_name);
        let mut field = NodeField::with_span(field_name, datatype, field_start_span);
        
        // Parse optional attributes
        if matches!(self.current_token(), Token::LeftBracket) {
            field.attributes = self.parse_attributes()?;
        }
        
        Ok(field)
    }
    
    // Validation helpers
    fn is_pascal_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        let first_char = s.chars().next().unwrap();
        first_char.is_uppercase() && s.chars().all(|c| c.is_alphanumeric())
    }
    
    fn is_snake_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        s.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
    }
    
    fn is_upper_snake_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        s.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_')
    }
    
    fn parse_column(&mut self) -> Result<Column, ParseError> {
        let column_start_span = self.current_span();
        let column_name = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "column name".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.expect_token(Token::Colon)?;
        self.skip_newlines();
        
        let datatype = if let Token::Identifier(type_name) = self.current_token() {
            let datatype = DataType::from_str(type_name);
            self.advance();
            datatype
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "data type".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        let mut column = Column::with_span(column_name, datatype, column_start_span);
        
        self.skip_newlines();
        
        // Parse attributes if present
        if matches!(self.current_token(), Token::LeftBracket) {
            column.attributes = self.parse_attributes()?;
        }
        
        Ok(column)
    }
    
    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, ParseError> {
        self.expect_token(Token::LeftBracket)?;
        self.skip_newlines();
        
        let mut attributes = Vec::new();
        
        while !matches!(self.current_token(), Token::RightBracket | Token::Eof) {
            let attribute = self.parse_attribute()?;
            attributes.push(attribute);
            
            self.skip_newlines();
            
            if matches!(self.current_token(), Token::Comma) {
                self.advance();
                self.skip_newlines();
            }
        }
        
        self.expect_token(Token::RightBracket)?;
        
        Ok(attributes)
    }
    
    fn parse_attribute(&mut self) -> Result<Attribute, ParseError> {
        if let Token::Identifier(attr_name) = self.current_token() {
            let attr = match attr_name.to_lowercase().as_str() {
                "pk" => {
                    self.advance();
                    Attribute::PrimaryKey
                }
                "fk" => {
                    self.advance();
                    Attribute::ForeignKey
                }
                "unique" => {
                    self.advance();
                    Attribute::Unique
                }
                "nullable" => {
                    self.advance();
                    Attribute::Nullable
                }
                "indexed" => {
                    self.advance();
                    Attribute::Indexed
                }
                "autoincrement" => {
                    self.advance();
                    Attribute::AutoIncrement
                }
                "default" => {
                    self.advance();
                    self.expect_token(Token::Equals)?;
                    let default_value = self.parse_default_value()?;
                    Attribute::Default(default_value)
                }
                _ => {
                    return Err(ParseError::InvalidAttribute {
                        name: attr_name.clone(),
                        span: self.current_span(),
                    });
                }
            };
            Ok(attr)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "attribute".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            })
        }
    }
    
    fn parse_default_value(&mut self) -> Result<DefaultValue, ParseError> {
        match self.current_token() {
            Token::Identifier(val) => {
                let value = match val.to_uppercase().as_str() {
                    "NOW" => DefaultValue::Now,
                    "TRUE" => DefaultValue::True,
                    "FALSE" => DefaultValue::False,
                    "NULL" => DefaultValue::Null,
                    _ => DefaultValue::String(val.clone()),
                };
                self.advance();
                Ok(value)
            }
            Token::String(s) => {
                let value = DefaultValue::String(s.clone());
                self.advance();
                Ok(value)
            }
            Token::Number(n) => {
                let value = DefaultValue::Number(*n);
                self.advance();
                Ok(value)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "default value".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            }),
        }
    }
    
    fn parse_relationship(&mut self) -> Result<Relationship, ParseError> {
        // Parse: Table1.field1 <operator> Table2.field2
        let from_table = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "table name".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.expect_token(Token::Dot)?;
        
        let from_field = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "field name".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.skip_newlines();
        
        let relationship_type = match self.current_token() {
            Token::OneToMany => {
                self.advance();
                RelationshipType::OneToMany
            }
            Token::ManyToOne => {
                self.advance();
                RelationshipType::ManyToOne
            }
            Token::ManyToMany => {
                self.advance();
                RelationshipType::ManyToMany
            }
            Token::OneToOne => {
                self.advance();
                RelationshipType::OneToOne
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "relationship operator (<, >, <>, -)".to_string(),
                    found: self.current_token().clone(),
                    span: self.current_span(),
                });
            }
        };
        
        self.skip_newlines();
        
        let to_table = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "table name".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.expect_token(Token::Dot)?;
        
        let to_field = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "field name".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        Ok(Relationship {
            from_table,
            from_field,
            to_table,
            to_field,
            relationship_type,
            span: Some(self.current_span()),
        })
    }
    
    fn parse_complex_edge(&mut self) -> Result<Edge, ParseError> {
        // Parse: edge EDGE_NAME (from: NodeA, to: NodeB) [attributes] { properties }
        self.expect_token(Token::Edge)?;
        self.skip_newlines();
        
        let edge_name_span = self.current_span();
        let edge_name = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            
            // Validate UPPER_SNAKE_CASE
            if !Self::is_upper_snake_case(&name) {
                return Err(ParseError::InvalidAttribute {
                    name: format!("Edge name '{}' must be UPPER_SNAKE_CASE", name),
                    span: edge_name_span,
                });
            }
            
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "edge name (UPPER_SNAKE_CASE)".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.skip_newlines();
        self.expect_token(Token::LeftParen)?;
        self.skip_newlines();
        
        // Parse "from: NodeName"
        if let Token::Identifier(keyword) = self.current_token() {
            if keyword != "from" {
                return Err(ParseError::UnexpectedToken {
                    expected: "'from'".to_string(),
                    found: self.current_token().clone(),
                    span: self.current_span(),
                });
            }
            self.advance();
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "'from'".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        }
        
        self.expect_token(Token::Colon)?;
        self.skip_newlines();
        
        let from_node = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "node name".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.skip_newlines();
        self.expect_token(Token::Comma)?;
        self.skip_newlines();
        
        // Parse "to: NodeName"
        if let Token::Identifier(keyword) = self.current_token() {
            if keyword != "to" {
                return Err(ParseError::UnexpectedToken {
                    expected: "'to'".to_string(),
                    found: self.current_token().clone(),
                    span: self.current_span(),
                });
            }
            self.advance();
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "'to'".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        }
        
        self.expect_token(Token::Colon)?;
        self.skip_newlines();
        
        let to_node = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "node name".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.skip_newlines();
        self.expect_token(Token::RightParen)?;
        self.skip_newlines();
        
        // Parse optional attributes
        let mut edge = Edge::with_span(edge_name, from_node, to_node, EdgeType::Bidirectional, edge_name_span);
        
        if matches!(self.current_token(), Token::LeftBracket) {
            edge.attributes = self.parse_attributes()?;
            self.skip_newlines();
        }
        
        // Parse properties block
        self.expect_token(Token::LeftBrace)?;
        self.skip_comments_and_newlines();
        
        while !matches!(self.current_token(), Token::RightBrace | Token::Eof) {
            let property = self.parse_edge_property()?;
            edge.properties.push(property);
            
            self.skip_comments_and_newlines();
            
            if matches!(self.current_token(), Token::Comma) {
                self.advance();
                self.skip_comments_and_newlines();
            }
        }
        
        self.expect_token(Token::RightBrace)?;
        
        Ok(edge)
    }
    
    fn parse_edge_property(&mut self) -> Result<EdgeProperty, ParseError> {
        let property_start_span = self.current_span();
        let property_name = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "property name".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.expect_token(Token::Colon)?;
        
        let datatype_name = if let Token::Identifier(dt) = self.current_token() {
            let dt = dt.clone();
            self.advance();
            dt
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "datatype".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        let datatype = DataType::from_str(&datatype_name);
        let mut property = EdgeProperty::with_span(property_name, datatype, property_start_span);
        
        // Parse optional attributes
        if matches!(self.current_token(), Token::LeftBracket) {
            property.attributes = self.parse_attributes()?;
        }
        
        Ok(property)
    }
    
    fn parse_shorthand_edge(&mut self) -> Result<Edge, ParseError> {
        // Parse: NodeA -[EDGE_NAME]-> NodeB
        // or: NodeA <-[EDGE_NAME]- NodeB  
        // or: NodeA <-[EDGE_NAME]-> NodeB
        
        let from_node = if let Token::Identifier(name) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "node name".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
        
        self.skip_newlines();
        
        // Determine edge direction by looking at the pattern
        // Pattern 1: -[NAME]->  (Outgoing)
        // Pattern 2: <-[NAME]-  (Incoming)
        // Pattern 3: <-[NAME]-> (Bidirectional)
        
        let _edge_type = if matches!(self.current_token(), Token::OneToOne) {
            // Could be -[NAME]->
            self.advance();
            self.skip_newlines();
            
            if !matches!(self.current_token(), Token::LeftBracket) {
                return Err(ParseError::UnexpectedToken {
                    expected: "'[' for edge name".to_string(),
                    found: self.current_token().clone(),
                    span: self.current_span(),
                });
            }
            self.advance(); // consume [
            self.skip_newlines();
            
            let edge_name_span = self.current_span();
            let edge_name = if let Token::Identifier(name) = self.current_token() {
                let name = name.clone();
                // Validate UPPER_SNAKE_CASE
                if !Self::is_upper_snake_case(&name) {
                    return Err(ParseError::InvalidAttribute {
                        name: format!("Edge name '{}' must be UPPER_SNAKE_CASE", name),
                        span: edge_name_span,
                    });
                }
                self.advance();
                name
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "edge name".to_string(),
                    found: self.current_token().clone(),
                    span: self.current_span(),
                });
            };
            
            self.skip_newlines();
            self.expect_token(Token::RightBracket)?;
            self.skip_newlines();
            self.expect_token(Token::OneToOne)?;
            self.skip_newlines();
            self.expect_token(Token::OneToMany)?;
            self.skip_newlines();
            
            let to_node = if let Token::Identifier(name) = self.current_token() {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "node name".to_string(),
                    found: self.current_token().clone(),
                    span: self.current_span(),
                });
            };
            
            return Ok(Edge::new(edge_name, from_node, to_node, EdgeType::Outgoing));
            
        } else if matches!(self.current_token(), Token::ManyToOne) {
            // Could be <-[NAME]- or <-[NAME]->
            self.advance();
            self.skip_newlines();
            self.expect_token(Token::OneToOne)?;
            self.skip_newlines();
            self.expect_token(Token::LeftBracket)?;
            self.skip_newlines();
            
            let edge_name_span = self.current_span();
            let edge_name = if let Token::Identifier(name) = self.current_token() {
                let name = name.clone();
                // Validate UPPER_SNAKE_CASE
                if !Self::is_upper_snake_case(&name) {
                    return Err(ParseError::InvalidAttribute {
                        name: format!("Edge name '{}' must be UPPER_SNAKE_CASE", name),
                        span: edge_name_span,
                    });
                }
                self.advance();
                name
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "edge name".to_string(),
                    found: self.current_token().clone(),
                    span: self.current_span(),
                });
            };
            
            self.skip_newlines();
            self.expect_token(Token::RightBracket)?;
            self.skip_newlines();
            self.expect_token(Token::OneToOne)?;
            self.skip_newlines();
            
            // Check if it's bidirectional (<-[]->) or incoming (<-[]-)
            let edge_type = if matches!(self.current_token(), Token::OneToMany) {
                self.advance();
                EdgeType::Bidirectional
            } else {
                EdgeType::Incoming
            };
            
            self.skip_newlines();
            
            let to_node = if let Token::Identifier(name) = self.current_token() {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "node name".to_string(),
                    found: self.current_token().clone(),
                    span: self.current_span(),
                });
            };
            
            return Ok(Edge::new(edge_name, from_node, to_node, edge_type));
            
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "edge operator (-, <-)".to_string(),
                found: self.current_token().clone(),
                span: self.current_span(),
            });
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_title() {
        let input = r#"#title "My ERD Diagram""#;
        let mut parser = Parser::new(input);
        let schema = parser.parse().unwrap();
        
        assert_eq!(schema.title, Some("My ERD Diagram".to_string()));
    }
    
    #[test]
    fn test_parse_simple_table() {
        let input = r#"
        table Products {
            id: int [pk],
            name: str
        }
        "#;
        let mut parser = Parser::new(input);
        let schema = parser.parse().unwrap();
        
        assert_eq!(schema.tables.len(), 1);
        assert_eq!(schema.tables[0].name, "Products");
        assert_eq!(schema.tables[0].columns.len(), 2);
    }
    
    #[test]
    fn test_parse_relationship() {
        let input = r#"
        table User { id: int [pk] }
        table Post { user_id: int [fk] }
        User.id > Post.user_id
        "#;
        let mut parser = Parser::new(input);
        let schema = parser.parse().unwrap();
        
        assert_eq!(schema.relationships.len(), 1);
        assert_eq!(schema.relationships[0].from_table, "User");
        assert_eq!(schema.relationships[0].to_table, "Post");
    }
    
    #[test]
    fn test_parse_node() {
        let input = r#"
        node Person {
            id: int [pk],
            name: string,
            age: int [nullable]
        }
        "#;
        let mut parser = Parser::new(input);
        let schema = parser.parse().unwrap();
        
        assert_eq!(schema.nodes.len(), 1);
        assert_eq!(schema.nodes[0].name, "Person");
        assert_eq!(schema.nodes[0].fields.len(), 3);
        assert_eq!(schema.nodes[0].fields[0].name, "id");
        assert_eq!(schema.nodes[0].fields[1].name, "name");
    }
    
    #[test]
    fn test_parse_complex_edge() {
        let input = r#"
        node Person { id: int }
        node Company { id: int }
        edge WORKS_AT (from: Person, to: Company) {
            position: string,
            start_date: date
        }
        "#;
        let mut parser = Parser::new(input);
        let schema = parser.parse().unwrap();
        
        assert_eq!(schema.edges.len(), 1);
        assert_eq!(schema.edges[0].name, "WORKS_AT");
        assert_eq!(schema.edges[0].from_node, "Person");
        assert_eq!(schema.edges[0].to_node, "Company");
        assert_eq!(schema.edges[0].properties.len(), 2);
    }
    
    #[test]
    fn test_node_pascal_case_validation() {
        let input = r#"node invalid_name { id: int }"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_edge_upper_snake_case_validation() {
        let input = r#"
        node Person { id: int }
        node Company { id: int }
        edge WorksAt (from: Person, to: Company) { }
        "#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_shorthand_edge_outgoing() {
        let input = r#"
        node User { id: int }
        node Post { id: int }
        User -[LIKES]-> Post
        "#;
        let mut parser = Parser::new(input);
        let schema = parser.parse().unwrap();
        
        assert_eq!(schema.edges.len(), 1);
        assert_eq!(schema.edges[0].name, "LIKES");
        assert_eq!(schema.edges[0].from_node, "User");
        assert_eq!(schema.edges[0].to_node, "Post");
        assert_eq!(schema.edges[0].edge_type, EdgeType::Outgoing);
        assert_eq!(schema.edges[0].properties.len(), 0);
    }
    
    #[test]
    fn test_parse_shorthand_edge_incoming() {
        let input = r#"
        node Post { id: int }
        node Tag { id: int }
        Post <-[TAGGED_IN]- Tag
        "#;
        let mut parser = Parser::new(input);
        let schema = parser.parse().unwrap();
        
        assert_eq!(schema.edges.len(), 1);
        assert_eq!(schema.edges[0].name, "TAGGED_IN");
        assert_eq!(schema.edges[0].from_node, "Post");
        assert_eq!(schema.edges[0].to_node, "Tag");
        assert_eq!(schema.edges[0].edge_type, EdgeType::Incoming);
    }
    
    #[test]
    fn test_parse_shorthand_edge_bidirectional() {
        let input = r#"
        node User { id: int }
        User <-[MENTIONS]-> User
        "#;
        let mut parser = Parser::new(input);
        let schema = parser.parse().unwrap();
        
        assert_eq!(schema.edges.len(), 1);
        assert_eq!(schema.edges[0].name, "MENTIONS");
        assert_eq!(schema.edges[0].from_node, "User");
        assert_eq!(schema.edges[0].to_node, "User");
        assert_eq!(schema.edges[0].edge_type, EdgeType::Bidirectional);
    }
}
