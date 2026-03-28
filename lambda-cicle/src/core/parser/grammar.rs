use super::lexer::Token;
use crate::core::ast::{Arm, Literal, Multiplicity, Pattern, Term, Type};

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Parser<'a> {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Term, ParseError> {
        let result = self.term()?;
        self.consume(&Token::EOF)?;
        Ok(result)
    }

    fn term(&mut self) -> Result<Term, ParseError> {
        self.app_expr()
    }

    fn app_expr(&mut self) -> Result<Term, ParseError> {
        let head = self.atom_expr()?;
        let mut args = Vec::new();
        while self.is_app_continuation() {
            let arg = self.atom_expr()?;
            args.push(arg);
        }
        Ok(args.into_iter().fold(head, Term::app))
    }

    fn is_app_continuation(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::Ident(_))
                | Some(Token::LParen)
                | Some(Token::KwLet)
                | Some(Token::KwMatch)
                | Some(Token::KwView)
                | Some(Token::IntLit(_))
                | Some(Token::FloatLit(_))
                | Some(Token::BoolLit(_))
                | Some(Token::CharLit(_))
                | Some(Token::UnitLit)
                | Some(Token::KwTrue)
                | Some(Token::KwFalse)
        )
    }

    fn atom_expr(&mut self) -> Result<Term, ParseError> {
        match self.peek() {
            Some(Token::IntLit(n)) => {
                let val = *n;
                self.advance();
                Ok(Term::literal(Literal::Int(val)))
            }
            Some(Token::FloatLit(n)) => {
                let val = *n;
                self.advance();
                Ok(Term::literal(Literal::Float(val)))
            }
            Some(Token::BoolLit(b)) => {
                let val = *b;
                self.advance();
                Ok(Term::literal(Literal::Bool(val)))
            }
            Some(Token::CharLit(c)) => {
                let val = *c;
                self.advance();
                Ok(Term::literal(Literal::Char(val)))
            }
            Some(Token::UnitLit) => {
                self.advance();
                Ok(Term::literal(Literal::Unit))
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Ok(Term::var(name))
            }
            Some(Token::KwLet) => self.let_expr(),
            Some(Token::KwMatch) => self.match_expr(),
            Some(Token::KwView) => self.view_expr(),
            Some(Token::LParen) => self.parens_expr(),
            _ => Err(ParseError::UnexpectedToken(format!("{:?}", self.peek()))),
        }
    }

    fn let_expr(&mut self) -> Result<Term, ParseError> {
        self.consume(&Token::KwLet)?;

        let var = match self.peek() {
            Some(Token::Ident(name)) => {
                let n = name.clone();
                self.advance();
                n
            }
            _ => return Err(ParseError::UnexpectedToken("expected variable".to_string())),
        };

        self.consume(&Token::Colon)?;
        let mult = self.multiplicity()?;
        self.consume(&Token::Colon)?;
        let annot = self.ty()?;
        self.consume(&Token::Equals)?;
        let value = self.term()?;
        self.consume(&Token::KwIn)?;
        let body = self.term()?;

        Ok(Term::let_in(var, mult, annot, value, body))
    }

    fn match_expr(&mut self) -> Result<Term, ParseError> {
        self.consume(&Token::KwMatch)?;
        let scrutinee = self.term()?;
        self.consume(&Token::KwWith)?;
        self.consume(&Token::LBrace)?;

        let mut arms = Vec::new();
        loop {
            let pattern = self.pattern()?;
            self.consume(&Token::Arrow)?;
            let body = self.term()?;
            arms.push(Arm::new(pattern, body));

            match self.peek() {
                Some(Token::Pipe) => {
                    self.advance();
                    continue;
                }
                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }
                _ => return Err(ParseError::UnexpectedToken("expected | or }".to_string())),
            }
        }

        Ok(Term::match_on(scrutinee, arms))
    }

    fn view_expr(&mut self) -> Result<Term, ParseError> {
        self.consume(&Token::KwView)?;
        let scrutinee = self.term()?;
        self.consume(&Token::KwWith)?;
        self.consume(&Token::LBrace)?;

        let mut arms = Vec::new();
        loop {
            let pattern = self.pattern()?;
            self.consume(&Token::Arrow)?;
            let body = self.term()?;
            arms.push(Arm::new(pattern, body));

            match self.peek() {
                Some(Token::Pipe) => {
                    self.advance();
                    continue;
                }
                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }
                _ => return Err(ParseError::UnexpectedToken("expected | or }".to_string())),
            }
        }

        Ok(Term::view_on(scrutinee, arms))
    }

    fn parens_expr(&mut self) -> Result<Term, ParseError> {
        self.consume(&Token::LParen)?;
        let expr = self.term()?;
        self.consume(&Token::RParen)?;
        Ok(expr)
    }

    fn multiplicity(&mut self) -> Result<Multiplicity, ParseError> {
        match self.peek() {
            Some(Token::MultiplicityZero) => {
                self.advance();
                Ok(Multiplicity::Zero)
            }
            Some(Token::MultiplicityOne) => {
                self.advance();
                Ok(Multiplicity::One)
            }
            Some(Token::MultiplicityOmega) => {
                self.advance();
                Ok(Multiplicity::Omega)
            }
            Some(Token::MultiplicityBorrow) => {
                self.advance();
                Ok(Multiplicity::Borrow)
            }
            _ => Err(ParseError::UnexpectedToken(
                "expected multiplicity".to_string(),
            )),
        }
    }

    fn ty(&mut self) -> Result<Type, ParseError> {
        self.ty_atom()
    }

    fn ty_atom(&mut self) -> Result<Type, ParseError> {
        match self.peek() {
            Some(Token::KwUnit) => {
                self.advance();
                Ok(Type::unit())
            }
            Some(Token::TyInt) => {
                self.advance();
                Ok(Type::int())
            }
            Some(Token::TyFloat) => {
                self.advance();
                Ok(Type::float())
            }
            Some(Token::TyBool) => {
                self.advance();
                Ok(Type::bool())
            }
            Some(Token::TyChar) => {
                self.advance();
                Ok(Type::char())
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Ok(Type::inductive(name, vec![]))
            }
            Some(Token::LParen) => {
                self.consume(&Token::LParen)?;
                let ty = self.ty()?;
                self.consume(&Token::RParen)?;
                Ok(ty)
            }
            _ => Err(ParseError::UnexpectedToken("expected type".to_string())),
        }
    }

    fn pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.peek() {
            Some(Token::Underscore) => {
                self.advance();
                Ok(Pattern::wildcard())
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Ok(Pattern::var(name))
            }
            _ => Err(ParseError::UnexpectedToken("expected pattern".to_string())),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        self.pos += 1;
        tok
    }

    fn consume(&mut self, expected: &Token) -> Result<(), ParseError> {
        match self.peek() {
            Some(token) if token == expected => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(ParseError::ExpectedToken {
                expected: format!("{:?}", expected),
                found: format!("{:?}", token),
            }),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    ExpectedToken { expected: String, found: String },
    UnexpectedEndOfInput,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(s) => write!(f, "Unexpected token: {}", s),
            ParseError::ExpectedToken { expected, found } => {
                write!(f, "Expected {}, found {}", expected, found)
            }
            ParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
        }
    }
}

impl std::error::Error for ParseError {}
