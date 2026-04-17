use crate::error::{CompileError, CompileResult};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Class,
    Interface,
    Enum,
    Abstract,
    Function,
    Const,
    Static,
    Final,
    Public,
    Private,
    Protected,
    Return,
    If,
    Else,
    While,
    For,
    Foreach,
    Break,
    Continue,
    New,
    Use,
    True,
    False,
    Null,
    SelfRef,
    Parent,
    Void,
    Extends,
    Implements,
    Case,

    // Types
    Type(String),
    TypeInt8,
    TypeInt16,
    TypeInt32,
    TypeInt64,
    TypeFloat32,
    TypeFloat64,
    TypeBoolean,
    TypeByte,
    TypeInt,
    TypeFloat,

    // Identifiers & Variables
    Identifier(String),
    Variable(String),

    // Literals
    StringLiteral(String),
    IntLiteral(i64),
    FloatLiteral(f64),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    Dot,
    Comma,
    Colon,
    Semicolon,
    Question,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Ampersand,
    Pipe,
    Not,
    Tilde,
    Arrow,

    // Special
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    Eof,
}

pub struct Lexer {
    input: String,
    pos: usize,
    ch: char,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer {
            input,
            pos: 0,
            ch: '\0',
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        self.ch = self.input.chars().nth(self.pos).unwrap_or('\0');
        self.pos += 1;
    }

    fn peek_char(&self) -> char {
        self.input.chars().nth(self.pos).unwrap_or('\0')
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() {
            self.read_char();
        }
    }

    fn skip_php_tag(&mut self) {
        // Skip <?php at the beginning of the file
        if self.ch == '<' {
            let start_pos = self.pos;
            self.read_char();
            if self.ch == '?' {
                self.read_char();
                if self.ch == 'p' || self.ch == 'P' {
                    // Check for "php" or "PHP"
                    let mut tag = String::from("?");
                    while self.ch.is_alphabetic() && tag.len() < 4 {
                        tag.push(self.ch);
                        self.read_char();
                    }
                    if tag.to_lowercase() == "?php" {
                        // Skip whitespace after <?php
                        while self.ch.is_whitespace() {
                            self.read_char();
                        }
                        return;
                    }
                }
            }
            // Not a PHP tag, restore position
            self.pos = start_pos;
            self.ch = '<';
        }
    }

    fn skip_comment(&mut self) {
        if self.ch == '/' {
            self.read_char();
            match self.ch {
                '/' => {
                    while self.ch != '\n' && self.ch != '\0' {
                        self.read_char();
                    }
                }
                '*' => {
                    self.read_char();
                    loop {
                        if self.ch == '*' {
                            self.read_char();
                            if self.ch == '/' {
                                self.read_char();
                                break;
                            }
                        } else if self.ch == '\0' {
                            return;
                        } else {
                            self.read_char();
                        }
                    }
                }
                _ => {
                    self.pos -= 1;
                    self.ch = '/';
                }
            }
        }
    }

    pub fn next_token(&mut self) -> CompileResult<Token> {
        loop {
            self.skip_whitespace();
            self.skip_php_tag();
            self.skip_comment();
            if self.ch.is_whitespace() {
                continue;
            }
            break;
        }

        if self.ch == '\0' {
            return Ok(Token::Eof);
        }

        if self.ch == '$' {
            return self.read_variable();
        }

        if self.ch.is_alphabetic() || self.ch == '_' {
            return self.read_identifier();
        }

        if self.ch.is_ascii_digit() {
            return self.read_number();
        }

        if self.ch == '"' || self.ch == '\'' {
            return self.read_string();
        }

        let token = match self.ch {
            '+' => Token::Plus,
            '-' => {
                // Check for -> (arrow)
                let next_ch = self.peek_char();
                if next_ch == '>' {
                    self.read_char();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '=' => {
                // Check for == (Eq)
                let next_ch = self.peek_char();
                if next_ch == '=' {
                    self.read_char();
                    Token::Eq
                } else {
                    Token::Equal
                }
            }
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            ':' => Token::Colon,
            '.' => Token::Dot,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '<' => {
                // Check for <= (Le)
                let next_ch = self.peek_char();
                if next_ch == '=' {
                    self.read_char();
                    Token::Le
                } else {
                    Token::Lt
                }
            }
            '>' => {
                // Check for >= (Ge)
                let next_ch = self.peek_char();
                if next_ch == '=' {
                    self.read_char();
                    Token::Ge
                } else {
                    Token::Gt
                }
            }
            '!' => {
                // Check for != (Ne)
                let next_ch = self.peek_char();
                if next_ch == '=' {
                    self.read_char();
                    Token::Ne
                } else {
                    Token::Not
                }
            }
            '?' => Token::Question,
            '&' => {
                // Check for && (And)
                let next_ch = self.peek_char();
                if next_ch == '&' {
                    self.read_char();
                    Token::And
                } else {
                    Token::Ampersand
                }
            }
            '|' => {
                // Check for || (Or)
                let next_ch = self.peek_char();
                if next_ch == '|' {
                    self.read_char();
                    Token::Or
                } else {
                    Token::Pipe
                }
            }
            '~' => Token::Tilde,
            _ => {
                return Err(CompileError::LexerError(format!(
                    "Unknown character: {}",
                    self.ch
                )))
            }
        };

        self.read_char();
        Ok(token)
    }

    fn read_variable(&mut self) -> CompileResult<Token> {
        self.read_char();
        let mut name = String::new();
        while self.ch.is_alphanumeric() || self.ch == '_' {
            name.push(self.ch);
            self.read_char();
        }
        Ok(Token::Variable(name))
    }

    fn read_identifier(&mut self) -> CompileResult<Token> {
        let mut name = String::new();
        while self.ch.is_alphanumeric() || self.ch == '_' {
            name.push(self.ch);
            self.read_char();
        }

        let token = match name.as_str() {
            "class" => Token::Class,
            "interface" => Token::Interface,
            "enum" => Token::Enum,
            "abstract" => Token::Abstract,
            "function" | "fn" => Token::Function,
            "const" => Token::Const,
            "static" => Token::Static,
            "final" => Token::Final,
            "public" => Token::Public,
            "private" => Token::Private,
            "protected" => Token::Protected,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "foreach" => Token::Foreach,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "new" => Token::New,
            "use" => Token::Use,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "self" => Token::SelfRef,
            "parent" => Token::Parent,
            "void" => Token::Void,
            "extends" => Token::Extends,
            "implements" => Token::Implements,
            "case" => Token::Case,
            "string" | "String" => Token::Type(String::from("string")),
            "boolean" | "bool" => Token::TypeBoolean,
            "int8" => Token::TypeInt8,
            "int16" => Token::TypeInt16,
            "int32" => Token::TypeInt32,
            "int64" => Token::TypeInt64,
            "float32" => Token::TypeFloat32,
            "float64" => Token::TypeFloat64,
            "byte" => Token::TypeByte,
            "int" => Token::TypeInt,
            "float" => Token::TypeFloat,
            _ => Token::Identifier(name),
        };

        Ok(token)
    }

    fn read_number(&mut self) -> CompileResult<Token> {
        let mut num_str = String::new();
        let mut has_dot = false;

        while self.ch.is_ascii_digit() || self.ch == '.' {
            if self.ch == '.' {
                if has_dot {
                    break;
                }
                has_dot = true;
            }
            num_str.push(self.ch);
            self.read_char();
        }

        if has_dot {
            match num_str.parse::<f64>() {
                Ok(n) => Ok(Token::FloatLiteral(n)),
                Err(_) => Err(CompileError::LexerError(format!(
                    "Invalid float: {}",
                    num_str
                ))),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(n) => Ok(Token::IntLiteral(n)),
                Err(_) => Err(CompileError::LexerError(format!(
                    "Invalid int: {}",
                    num_str
                ))),
            }
        }
    }

    fn read_string(&mut self) -> CompileResult<Token> {
        let quote = self.ch;
        self.read_char();

        let mut value = String::new();
        while self.ch != quote && self.ch != '\0' {
            if self.ch == '\\' {
                self.read_char();
                match self.ch {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '\'' => value.push('\''),
                    '"' => value.push('"'),
                    _ => value.push(self.ch),
                }
            } else {
                value.push(self.ch);
            }
            self.read_char();
        }

        if self.ch != quote {
            return Err(CompileError::LexerError("Unterminated string".to_string()));
        }

        self.read_char();
        Ok(Token::StringLiteral(value))
    }
}
