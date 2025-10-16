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
                Token::Identifier(_) => {
                    let relationship = self.parse_relationship()?;
                    schema.relationships.push(relationship);
                }
                Token::Comment(_) | Token::Newline => {
                    self.advance();
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "title, table, or relationship".to_string(),
                        found: self.current_token().clone(),
                        span: self.current_span(),
                    });
                }
            }
            
            self.skip_comments_and_newlines();
        }
        
        Ok(schema)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_title() {
        let input = r#"title "My ERD Diagram""#;
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
}
