#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    KwLet,
    KwIn,
    KwMatch,
    KwView,
    KwWith,
    KwTrue,
    KwFalse,
    KwForall,
    KwUnit,
    TyInt,
    TyFloat,
    TyBool,
    TyChar,
    MultiplicityZero,
    MultiplicityOne,
    MultiplicityOmega,
    MultiplicityBorrow,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Dot,
    Equals,
    Arrow,
    Pipe,
    Underscore,
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    CharLit(char),
    UnitLit,
    Ident(String),
    EOF,
}

pub struct Lexer {
    input: String,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: impl Into<String>) -> Lexer {
        Lexer {
            input: input.into(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }
            match self.peek() {
                Some('(') => {
                    self.advance();
                    tokens.push(Token::LParen);
                }
                Some(')') => {
                    self.advance();
                    tokens.push(Token::RParen);
                }
                Some('{') => {
                    self.advance();
                    tokens.push(Token::LBrace);
                }
                Some('}') => {
                    self.advance();
                    tokens.push(Token::RBrace);
                }
                Some(':') => {
                    self.advance();
                    tokens.push(Token::Colon);
                }
                Some(',') => {
                    self.advance();
                    tokens.push(Token::Comma);
                }
                Some('.') => {
                    self.advance();
                    tokens.push(Token::Dot);
                }
                Some('=') => {
                    self.advance();
                    tokens.push(Token::Equals);
                }
                Some('|') => {
                    self.advance();
                    tokens.push(Token::Pipe);
                }
                Some('_') => {
                    self.advance();
                    tokens.push(Token::Underscore);
                }
                Some('-') => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token::Arrow);
                    } else {
                        return Err(LexError::UnexpectedChar('-', self.line, self.col));
                    }
                }
                Some('0') => {
                    self.advance();
                    tokens.push(Token::MultiplicityZero);
                }
                Some('1') => {
                    self.advance();
                    tokens.push(Token::MultiplicityOne);
                }
                Some('ω') => {
                    self.advance();
                    tokens.push(Token::MultiplicityOmega);
                }
                Some('&') => {
                    self.advance();
                    tokens.push(Token::MultiplicityBorrow);
                }
                Some('0'..='9') => {
                    let num = self.read_number()?;
                    tokens.push(num);
                }
                Some('a'..='z') | Some('A'..='Z') => {
                    let ident = self.read_identifier()?;
                    let token = self.keyword_or_ident(&ident);
                    tokens.push(token);
                }
                Some('λ') => {
                    self.advance();
                    tokens.push(Token::Ident("λ".to_string()));
                }
                Some(c) => {
                    return Err(LexError::UnexpectedChar(c, self.line, self.col));
                }
                None => break,
            }
        }
        tokens.push(Token::EOF);
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn peek_next(&self) -> Option<char> {
        self.input.chars().nth(self.pos + 1)
    }

    fn advance(&mut self) -> char {
        let c = self.input.chars().nth(self.pos).unwrap();
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '-' && self.peek_next() == Some('-') {
                while let Some(c) = self.peek() {
                    self.advance();
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        let mut has_dot = false;

        while let Some(c) = self.peek() {
            match c {
                '0'..='9' => {
                    self.advance();
                }
                '.' => {
                    if has_dot {
                        return Err(LexError::UnexpectedChar('.', self.line, self.col));
                    }
                    has_dot = true;
                    self.advance();
                }
                _ => break,
            }
        }

        let num_str = &self.input[start..self.pos];
        if has_dot {
            match num_str.parse::<f64>() {
                Ok(n) => Ok(Token::FloatLit(n)),
                Err(_) => Err(LexError::InvalidNumber(num_str.to_string())),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(n) => Ok(Token::IntLit(n)),
                Err(_) => Err(LexError::InvalidNumber(num_str.to_string())),
            }
        }
    }

    fn read_identifier(&mut self) -> Result<String, LexError> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn keyword_or_ident(&self, s: &str) -> Token {
        match s {
            "let" => Token::KwLet,
            "in" => Token::KwIn,
            "match" => Token::KwMatch,
            "view" => Token::KwView,
            "with" => Token::KwWith,
            "true" => Token::BoolLit(true),
            "false" => Token::BoolLit(false),
            "forall" => Token::KwForall,
            "Unit" => Token::KwUnit,
            "Int" => Token::TyInt,
            "Float" => Token::TyFloat,
            "Bool" => Token::TyBool,
            "Char" => Token::TyChar,
            "omega" => Token::MultiplicityOmega,
            "()" => Token::UnitLit,
            _ => Token::Ident(s.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LexError {
    UnexpectedChar(char, usize, usize),
    InvalidNumber(String),
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnexpectedChar(c, line, col) => {
                write!(
                    f,
                    "Unexpected character '{}' at line {}, column {}",
                    c, line, col
                )
            }
            LexError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
        }
    }
}

impl std::error::Error for LexError {}
