use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Span {
    pub fn new(line: usize, column: usize, length: usize) -> Self {
        Span { line, column, length }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Spanned { value, span }
    }
}

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
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        Lexer {
            input: chars,
            position: 0,
            current_char,
            line: 1,
            column: 1,
        }
    }
    
    fn advance(&mut self) {
        if let Some('\n') = self.current_char {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }
    
    fn current_span(&self, length: usize) -> Span {
        Span::new(self.line, self.column, length)
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
    
    pub fn tokenize(&mut self) -> Vec<Spanned<Token>> {
        let mut tokens = Vec::new();
        let input_copy = self.input.clone(); // For extracting source text
        
        while let Some(ch) = self.current_char {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.skip_whitespace();
                }
                '\n' => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::Newline, span));
                    self.advance();
                }
                '"' | '\'' => {
                    let start_line = self.line;
                    let start_col = self.column;
                    let string = self.read_string(ch);
                    let length = self.column - start_col;
                    let span = Span::new(start_line, start_col, length);
                    tokens.push(Spanned::new(Token::String(string), span));
                }
                ':' => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::Colon, span));
                    self.advance();
                }
                ',' => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::Comma, span));
                    self.advance();
                }
                '.' => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::Dot, span));
                    self.advance();
                }
                '{' => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::LeftBrace, span));
                    self.advance();
                }
                '}' => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::RightBrace, span));
                    self.advance();
                }
                '[' => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::LeftBracket, span));
                    self.advance();
                }
                ']' => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::RightBracket, span));
                    self.advance();
                }
                '=' => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::Equals, span));
                    self.advance();
                }
                '/' if self.peek(1) == Some('/') => {
                    let start_line = self.line;
                    let start_col = self.column;
                    let comment = self.read_comment();
                    let length = self.column - start_col;
                    let span = Span::new(start_line, start_col, length);
                    tokens.push(Spanned::new(Token::Comment(comment), span));
                }
                '<' if self.peek(1) == Some('>') => {
                    let span = self.current_span(2);
                    tokens.push(Spanned::new(Token::ManyToMany, span));
                    self.advance();
                    self.advance();
                }
                '<' => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::ManyToOne, span));
                    self.advance();
                }
                '>' => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::OneToMany, span));
                    self.advance();
                }
                '-' if !self.peek(1).map(|c| c.is_numeric()).unwrap_or(false) => {
                    let span = self.current_span(1);
                    tokens.push(Spanned::new(Token::OneToOne, span));
                    self.advance();
                }
                _ if ch.is_numeric() || (ch == '-' && self.peek(1).map(|c| c.is_numeric()).unwrap_or(false)) => {
                    let start_line = self.line;
                    let start_col = self.column;
                    let num = self.read_number();
                    let length = self.column - start_col;
                    let span = Span::new(start_line, start_col, length);
                    tokens.push(Spanned::new(Token::Number(num), span));
                }
                _ if ch.is_alphabetic() || ch == '_' => {
                    let start_line = self.line;
                    let start_col = self.column;
                    let ident = self.read_identifier();
                    let length = self.column - start_col;
                    let span = Span::new(start_line, start_col, length);
                    let token = match ident.to_lowercase().as_str() {
                        "title" => Token::Title,
                        "table" => Token::Table,
                        _ => Token::Identifier(ident),
                    };
                    tokens.push(Spanned::new(token, span));
                }
                _ => {
                    // Unknown character, skip it
                    self.advance();
                }
            }
        }
        
        let span = self.current_span(0);
        tokens.push(Spanned::new(Token::Eof, span));
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
        
        assert_eq!(tokens[0].value, Token::Title);
        assert_eq!(tokens[1].value, Token::String("My ERD".to_string()));
    }
    
    #[test]
    fn test_table_tokenization() {
        let input = r#"table Products { id: int }"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].value, Token::Table);
        assert_eq!(tokens[1].value, Token::Identifier("Products".to_string()));
        assert_eq!(tokens[2].value, Token::LeftBrace);
    }
}
