use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Title,
    Table,
    
    // Identifiers and Literals
    Identifier(String),
    String(String),
    Number(i64),
    
    // Relationship Operators
    OneToMany,      // >
    ManyToOne,      // <
    ManyToMany,     // <>
    OneToOne,       // -
    
    // Punctuation
    Colon,          // :
    Comma,          // ,
    Dot,            // .
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Equals,         // =
    
    // Special
    Comment(String),
    Newline,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Title => write!(f, "TITLE"),
            Token::Table => write!(f, "TABLE"),
            Token::Identifier(s) => write!(f, "IDENTIFIER({})", s),
            Token::String(s) => write!(f, "STRING(\"{}\")", s),
            Token::Number(n) => write!(f, "NUMBER({})", n),
            Token::OneToMany => write!(f, ">"),
            Token::ManyToOne => write!(f, "<"),
            Token::ManyToMany => write!(f, "<>"),
            Token::OneToOne => write!(f, "-"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Equals => write!(f, "="),
            Token::Comment(s) => write!(f, "COMMENT({})", s),
            Token::Newline => write!(f, "NEWLINE"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        Lexer {
            input: chars,
            position: 0,
            current_char,
        }
    }
    
    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }
    
    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn read_string(&mut self, quote: char) -> String {
        let mut result = String::new();
        self.advance(); // Skip opening quote
        
        while let Some(ch) = self.current_char {
            if ch == quote {
                self.advance(); // Skip closing quote
                break;
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char {
                    result.push(escaped);
                    self.advance();
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }
        
        result
    }
    
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        result
    }
    
    fn read_number(&mut self) -> i64 {
        let mut result = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_numeric() {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        result.parse().unwrap_or(0)
    }
    
    fn read_comment(&mut self) -> String {
        let mut result = String::new();
        self.advance(); // Skip first /
        self.advance(); // Skip second /
        
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            result.push(ch);
            self.advance();
        }
        
        result.trim().to_string()
    }
    
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while let Some(ch) = self.current_char {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.skip_whitespace();
                }
                '\n' => {
                    tokens.push(Token::Newline);
                    self.advance();
                }
                '"' | '\'' => {
                    let string = self.read_string(ch);
                    tokens.push(Token::String(string));
                }
                ':' => {
                    tokens.push(Token::Colon);
                    self.advance();
                }
                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                }
                '.' => {
                    tokens.push(Token::Dot);
                    self.advance();
                }
                '{' => {
                    tokens.push(Token::LeftBrace);
                    self.advance();
                }
                '}' => {
                    tokens.push(Token::RightBrace);
                    self.advance();
                }
                '[' => {
                    tokens.push(Token::LeftBracket);
                    self.advance();
                }
                ']' => {
                    tokens.push(Token::RightBracket);
                    self.advance();
                }
                '=' => {
                    tokens.push(Token::Equals);
                    self.advance();
                }
                '/' if self.peek(1) == Some('/') => {
                    let comment = self.read_comment();
                    tokens.push(Token::Comment(comment));
                }
                '<' if self.peek(1) == Some('>') => {
                    tokens.push(Token::ManyToMany);
                    self.advance();
                    self.advance();
                }
                '<' => {
                    tokens.push(Token::ManyToOne);
                    self.advance();
                }
                '>' => {
                    tokens.push(Token::OneToMany);
                    self.advance();
                }
                '-' if !self.peek(1).map(|c| c.is_numeric()).unwrap_or(false) => {
                    tokens.push(Token::OneToOne);
                    self.advance();
                }
                _ if ch.is_numeric() || (ch == '-' && self.peek(1).map(|c| c.is_numeric()).unwrap_or(false)) => {
                    let num = self.read_number();
                    tokens.push(Token::Number(num));
                }
                _ if ch.is_alphabetic() || ch == '_' => {
                    let ident = self.read_identifier();
                    let token = match ident.to_lowercase().as_str() {
                        "title" => Token::Title,
                        "table" => Token::Table,
                        _ => Token::Identifier(ident),
                    };
                    tokens.push(token);
                }
                _ => {
                    // Unknown character, skip it
                    self.advance();
                }
            }
        }
        
        tokens.push(Token::Eof);
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokenization() {
        let input = r#"title "My ERD""#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0], Token::Title);
        assert_eq!(tokens[1], Token::String("My ERD".to_string()));
    }
    
    #[test]
    fn test_table_tokenization() {
        let input = r#"table Products { id: int }"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0], Token::Table);
        assert_eq!(tokens[1], Token::Identifier("Products".to_string()));
        assert_eq!(tokens[2], Token::LeftBrace);
    }
}
