#[derive(Debug, Clone)]
pub enum Token {
    KwLet(usize, usize),
    KwIn(usize, usize),
    KwMatch(usize, usize),
    KwView(usize, usize),
    KwWith(usize, usize),
    KwForall(usize, usize),
    KwUnit(usize, usize),
    KwLambda(usize, usize),
    KwPub(usize, usize),
    KwUse(usize, usize),
    KwType(usize, usize),
    KwTrait(usize, usize),
    KwImpl(usize, usize),
    KwNoPrelude(usize, usize),
    KwWhere(usize, usize),
    KwVal(usize, usize),
    KwAs(usize, usize),
    TyInt(usize, usize),
    TyFloat(usize, usize),
    TyChar(usize, usize),
    MultiplicityZero(usize, usize),
    MultiplicityOne(usize, usize),
    MultiplicityOmega(usize, usize),
    MultiplicityBorrow(usize, usize),
    LParen(usize, usize),
    RParen(usize, usize),
    LBrace(usize, usize),
    RBrace(usize, usize),
    Colon(usize, usize),
    Comma(usize, usize),
    Dot(usize, usize),
    DotDot(usize, usize),
    Equals(usize, usize),
    Arrow(usize, usize),
    FatArrow(usize, usize),
    Pipe(usize, usize),
    Underscore(usize, usize),
    IntLit(i64, usize, usize),
    FloatLit(f64, usize, usize),
    CharLit(char, usize, usize),
    StringLit(String, usize, usize),
    UnitLit(usize, usize),
    Ident(String, usize, usize),
    EOF(usize, usize),
}

impl Token {
    pub fn position(&self) -> Option<(usize, usize)> {
        match self {
            Token::KwLet(l, c)
            | Token::KwIn(l, c)
            | Token::KwMatch(l, c)
            | Token::KwView(l, c)
            | Token::KwWith(l, c)
            | Token::KwForall(l, c)
            | Token::KwUnit(l, c)
            | Token::KwLambda(l, c)
            | Token::KwPub(l, c)
            | Token::KwUse(l, c)
            | Token::KwType(l, c)
            | Token::KwTrait(l, c)
            | Token::KwImpl(l, c)
            | Token::KwNoPrelude(l, c)
            | Token::KwWhere(l, c)
            | Token::KwVal(l, c)
            | Token::KwAs(l, c)
            | Token::TyInt(l, c)
            | Token::TyFloat(l, c)
            | Token::TyChar(l, c)
            | Token::MultiplicityZero(l, c)
            | Token::MultiplicityOne(l, c)
            | Token::MultiplicityOmega(l, c)
            | Token::MultiplicityBorrow(l, c)
            | Token::LParen(l, c)
            | Token::RParen(l, c)
            | Token::LBrace(l, c)
            | Token::RBrace(l, c)
            | Token::Colon(l, c)
            | Token::Comma(l, c)
            | Token::Dot(l, c)
            | Token::DotDot(l, c)
            | Token::Equals(l, c)
            | Token::Arrow(l, c)
            | Token::FatArrow(l, c)
            | Token::Pipe(l, c)
            | Token::Underscore(l, c)
            | Token::UnitLit(l, c)
            | Token::EOF(l, c) => Some((*l, *c)),
            Token::IntLit(_, l, c)
            | Token::FloatLit(_, l, c)
            | Token::CharLit(_, l, c)
            | Token::StringLit(_, l, c)
            | Token::Ident(_, l, c) => Some((*l, *c)),
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
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
                    tokens.push(Token::LParen(self.line, self.col));
                }
                Some(')') => {
                    self.advance();
                    tokens.push(Token::RParen(self.line, self.col));
                }
                Some('{') => {
                    self.advance();
                    tokens.push(Token::LBrace(self.line, self.col));
                }
                Some('}') => {
                    self.advance();
                    tokens.push(Token::RBrace(self.line, self.col));
                }
                Some(':') => {
                    self.advance();
                    tokens.push(Token::Colon(self.line, self.col));
                }
                Some(',') => {
                    self.advance();
                    tokens.push(Token::Comma(self.line, self.col));
                }
                Some('.') => {
                    self.advance();
                    if let Some('.') = self.peek() {
                        self.advance();
                        tokens.push(Token::DotDot(self.line, self.col));
                    } else {
                        tokens.push(Token::Dot(self.line, self.col));
                    }
                }
                Some('|') => {
                    self.advance();
                    tokens.push(Token::Pipe(self.line, self.col));
                }
                Some('=') => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token::FatArrow(self.line, self.col));
                    } else {
                        tokens.push(Token::Equals(self.line, self.col));
                    }
                }
                Some('_') => {
                    self.advance();
                    if let Some(c) = self.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            let ident = self.read_identifier()?;
                            let ident = format!("_{}", ident);
                            tokens.push(self.keyword_or_ident(&ident));
                        } else {
                            tokens.push(Token::Underscore(self.line, self.col));
                        }
                    } else {
                        tokens.push(Token::Underscore(self.line, self.col));
                    }
                }
                Some('-') => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token::Arrow(self.line, self.col));
                    } else {
                        return Err(LexError::UnexpectedChar('-', self.line, self.col));
                    }
                }
                Some('0') | Some('1') => {
                    // Always treat as start of number - read the full number
                    let num = self.read_number()?;
                    tokens.push(num);
                }
                Some('ω') => {
                    self.advance();
                    tokens.push(Token::MultiplicityOmega(self.line, self.col));
                }
                Some('&') => {
                    self.advance();
                    tokens.push(Token::MultiplicityBorrow(self.line, self.col));
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
                    tokens.push(Token::KwLambda(self.line, self.col));
                }
                Some('\\') => {
                    self.advance();
                    tokens.push(Token::KwLambda(self.line, self.col));
                }
                Some('"') => {
                    let s = self.read_string()?;
                    tokens.push(Token::StringLit(s, self.line, self.col));
                }
                Some(c) => {
                    return Err(LexError::UnexpectedChar(c, self.line, self.col));
                }
                None => break,
            }
        }
        tokens.push(Token::EOF(self.line, self.col));
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.chars().count() as usize
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn peek_next(&self) -> Option<char> {
        self.input.chars().nth(self.pos + 1 as usize)
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
        let mut result = String::new();
        let mut has_dot = false;
        let line = self.line;
        let col = self.col;

        while let Some(c) = self.peek() {
            match c {
                '0'..='9' => {
                    result.push(c);
                    self.advance();
                }
                '.' => {
                    if has_dot {
                        return Err(LexError::UnexpectedChar('.', self.line, self.col));
                    }
                    has_dot = true;
                    result.push(c);
                    self.advance();
                }
                _ => break,
            }
        }

        if result.is_empty() {
            return Err(LexError::InvalidNumber("empty".to_string()));
        }

        let num_str = result.as_str();
        if has_dot {
            match num_str.parse::<f64>() {
                Ok(n) => Ok(Token::FloatLit(n, line, col)),
                Err(_) => Err(LexError::InvalidNumber(num_str.to_string())),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(n) => Ok(Token::IntLit(n, line, col)),
                Err(_) => Err(LexError::InvalidNumber(num_str.to_string())),
            }
        }
    }

    fn read_string(&mut self) -> Result<String, LexError> {
        self.advance(); // consume opening "
        let mut result = String::new();
        loop {
            match self.peek() {
                None => return Err(LexError::UnexpectedChar('"', self.line, self.col)),
                Some('"') => {
                    self.advance(); // consume closing "
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n') => {
                            self.advance();
                            result.push('\n');
                        }
                        Some('t') => {
                            self.advance();
                            result.push('\t');
                        }
                        Some('\\') => {
                            self.advance();
                            result.push('\\');
                        }
                        Some('"') => {
                            self.advance();
                            result.push('"');
                        }
                        Some(c) => {
                            let c = c;
                            self.advance();
                            result.push('\\');
                            result.push(c);
                        }
                        None => return Err(LexError::UnexpectedChar('\\', self.line, self.col)),
                    }
                }
                Some(c) => {
                    let c = c;
                    self.advance();
                    result.push(c);
                }
            }
        }
        Ok(result)
    }

    fn read_identifier(&mut self) -> Result<String, LexError> {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }
        Ok(result)
    }

    fn keyword_or_ident(&self, s: &str) -> Token {
        match s {
            "let" => Token::KwLet(self.line, self.col),
            "in" => Token::KwIn(self.line, self.col),
            "match" => Token::KwMatch(self.line, self.col),
            "view" => Token::KwView(self.line, self.col),
            "with" => Token::KwWith(self.line, self.col),
            "forall" => Token::KwForall(self.line, self.col),
            "Unit" => Token::KwUnit(self.line, self.col),
            "lambda" => Token::KwLambda(self.line, self.col),
            "λ" => Token::KwLambda(self.line, self.col),
            "pub" => Token::KwPub(self.line, self.col),
            "use" => Token::KwUse(self.line, self.col),
            "type" => Token::KwType(self.line, self.col),
            "trait" => Token::KwTrait(self.line, self.col),
            "impl" => Token::KwImpl(self.line, self.col),
            "no_prelude" => Token::KwNoPrelude(self.line, self.col),
            "where" => Token::KwWhere(self.line, self.col),
            "val" => Token::KwVal(self.line, self.col),
            "as" => Token::KwAs(self.line, self.col),
            "Int" => Token::TyInt(self.line, self.col),
            "Float" => Token::TyFloat(self.line, self.col),
            "Char" => Token::TyChar(self.line, self.col),
            "omega" => Token::MultiplicityOmega(self.line, self.col),
            "()" => Token::UnitLit(self.line, self.col),
            _ => Token::Ident(s.to_string(), self.line, self.col),
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
