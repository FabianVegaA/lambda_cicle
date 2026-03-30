use super::lexer::Token;
use crate::core::ast::{
    Arm, Decl, Literal, MethodDef, MethodName, MethodSig, Multiplicity, Pattern, Term, TraitName,
    Type, UseMode, Visibility,
};

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

    pub fn parse_program(&mut self) -> Result<Vec<Decl>, ParseError> {
        let mut decls = Vec::new();
        while !self.is_at_end() {
            match self.peek() {
                Some(&Token::EOF) => break,
                _ => {
                    let decl = self.decl()?;
                    decls.push(decl);
                }
            }
        }
        if !self.is_at_end() {
            self.consume(&Token::EOF)?;
        }
        Ok(decls)
    }

    fn decl(&mut self) -> Result<Decl, ParseError> {
        match self.peek() {
            Some(&Token::KwNoPrelude) => {
                self.advance();
                Ok(Decl::NoPrelude)
            }
            Some(&Token::KwType) => self.type_decl(None),
            Some(&Token::KwVal) => self.val_decl(None),
            Some(&Token::KwTrait) => self.trait_decl(None),
            Some(&Token::KwImpl) => self.impl_decl(),
            Some(&Token::KwUse) => self.use_decl(),
            Some(&Token::KwPub) => {
                self.advance();
                match self.peek() {
                    Some(&Token::KwType) => self.type_decl(Some(Visibility::Public)),
                    Some(&Token::KwVal) => self.val_decl(Some(Visibility::Public)),
                    Some(&Token::KwTrait) => self.trait_decl(Some(Visibility::Public)),
                    _ => Err(ParseError::UnexpectedToken(format!(
                        "expected declaration after pub, found {:?}",
                        self.peek()
                    ))),
                }
            }
            _ => {
                let term = self.term()?;
                Ok(Decl::ValDecl {
                    visibility: Visibility::Private,
                    name: "main".to_string(),
                    ty: term.get_type().unwrap_or(Type::unit()),
                    term: Box::new(term),
                })
            }
        }
    }

    fn type_decl(&mut self, visibility_override: Option<Visibility>) -> Result<Decl, ParseError> {
        let visibility = visibility_override
            .unwrap_or_else(|| self.parse_visibility().unwrap_or(Visibility::Private));
        self.consume(&Token::KwType)?;
        let name = self.expect_ident()?;
        let params = self.parse_type_params()?;

        let (ty, transparent) = match self.peek() {
            Some(&Token::LParen) => {
                self.advance();
                match self.peek() {
                    Some(&Token::DotDot) => {
                        self.advance();
                        self.consume(&Token::RParen)?;
                        (Type::unit(), true)
                    }
                    _ => return Err(ParseError::UnexpectedToken("expected ..".to_string())),
                }
            }
            _ => {
                self.consume(&Token::Equals)?;
                let ty = self.ty()?;
                (ty, false)
            }
        };

        Ok(Decl::TypeDecl {
            visibility,
            name,
            params,
            ty,
            transparent,
        })
    }

    fn val_decl(&mut self, visibility_override: Option<Visibility>) -> Result<Decl, ParseError> {
        let visibility = visibility_override
            .unwrap_or_else(|| self.parse_visibility().unwrap_or(Visibility::Private));
        self.consume(&Token::KwVal)?;
        let name = self.expect_ident()?;
        self.consume(&Token::Colon)?;
        let ty = self.ty()?;
        self.consume(&Token::Equals)?;
        let term = Box::new(self.term()?);
        Ok(Decl::ValDecl {
            visibility,
            name,
            ty,
            term,
        })
    }

    fn trait_decl(&mut self, visibility_override: Option<Visibility>) -> Result<Decl, ParseError> {
        let visibility = visibility_override
            .unwrap_or_else(|| self.parse_visibility().unwrap_or(Visibility::Private));
        self.consume(&Token::KwTrait)?;
        let name = self.expect_ident()?;
        let params = self.parse_type_params()?;

        // Handle optional where clause with supertrait
        // Syntax: where TraitName type_param* [where {...}]
        // We may have multiple where clauses - keep consuming until we hit {
        loop {
            if matches!(self.peek(), Some(&Token::LBrace)) {
                break;
            }
            if matches!(self.peek(), Some(&Token::KwWhere)) {
                self.advance();
                // Skip supertrait parsing - consume the identifier and type params
                if matches!(self.peek(), Some(&Token::Ident(_))) {
                    self.advance();
                    // Consume type params (type application)
                    while matches!(self.peek(), Some(&Token::Ident(_)))
                        || matches!(self.peek(), Some(&Token::TyInt))
                        || matches!(self.peek(), Some(&Token::TyFloat))
                        || matches!(self.peek(), Some(&Token::TyBool))
                        || matches!(self.peek(), Some(&Token::TyChar))
                        || matches!(self.peek(), Some(&Token::KwUnit))
                    {
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }

        // Handle optional trait body - may be absent for marker traits
        let mut methods = Vec::new();
        if matches!(self.peek(), Some(&Token::LBrace)) {
            self.consume(&Token::LBrace)?;
            while !matches!(self.peek(), Some(&Token::RBrace)) {
                // Parse optional "val" keyword
                if matches!(self.peek(), Some(&Token::KwVal)) {
                    self.advance();
                }

                let method_name_str = self.expect_ident()?;
                self.consume(&Token::Colon)?;
                let method_ty = self.ty()?;
                methods.push(MethodSig {
                    name: MethodName(method_name_str),
                    ty: method_ty,
                });
                if matches!(self.peek(), Some(Token::RBrace)) {
                    break;
                }
            }
            self.consume(&Token::RBrace)?;
        }

        Ok(Decl::TraitDecl {
            visibility,
            name,
            params,
            methods,
        })
    }

    fn impl_decl(&mut self) -> Result<Decl, ParseError> {
        self.consume(&Token::KwImpl)?;
        let ty = self.ty()?;
        self.consume(&Token::Colon)?;
        let trait_name_str = self.expect_ident()?;
        let trait_name = TraitName(trait_name_str);
        self.consume(&Token::KwWhere)?;
        self.consume(&Token::LBrace)?;
        let mut methods = Vec::new();
        while !matches!(self.peek(), Some(&Token::RBrace)) {
            let method_name_str = self.expect_ident()?;
            self.consume(&Token::Equals)?;
            let term = Box::new(self.term()?);
            methods.push(MethodDef {
                name: MethodName(method_name_str),
                term,
            });
            if matches!(self.peek(), Some(Token::RBrace)) {
                break;
            }
        }
        self.consume(&Token::RBrace)?;
        Ok(Decl::ImplDecl {
            ty,
            trait_name,
            methods,
        })
    }

    fn use_decl(&mut self) -> Result<Decl, ParseError> {
        self.consume(&Token::KwUse)?;
        let path = self.parse_module_path()?;
        let mode = match self.peek() {
            Some(&Token::LParen) => {
                self.advance();
                match self.peek() {
                    Some(&Token::DotDot) => {
                        self.advance();
                        self.consume(&Token::RParen)?;
                        UseMode::Unqualified
                    }
                    _ => {
                        let mut items = Vec::new();
                        items.push(self.expect_ident()?);
                        while matches!(self.peek(), Some(Token::Comma)) {
                            self.advance();
                            items.push(self.expect_ident()?);
                        }
                        self.consume(&Token::RParen)?;
                        UseMode::Selective(items)
                    }
                }
            }
            Some(&Token::KwAs) => {
                self.advance();
                let alias = self.expect_ident()?;
                UseMode::Aliased(alias)
            }
            _ => UseMode::Qualified,
        };
        Ok(Decl::UseDecl { path, mode })
    }

    fn parse_visibility(&mut self) -> Result<Visibility, ParseError> {
        if matches!(self.peek(), Some(&Token::KwPub)) {
            self.advance();
            Ok(Visibility::Public)
        } else {
            Ok(Visibility::Private)
        }
    }

    fn parse_type_params(&mut self) -> Result<Vec<String>, ParseError> {
        let mut params = Vec::new();
        while matches!(self.peek(), Some(&Token::Ident(_))) {
            params.push(self.expect_ident()?);
        }
        Ok(params)
    }

    fn parse_module_path(&mut self) -> Result<Vec<String>, ParseError> {
        let mut path = Vec::new();
        path.push(self.expect_ident()?);
        while matches!(self.peek(), Some(Token::Dot)) {
            self.advance();
            path.push(self.expect_ident()?);
        }
        Ok(path)
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
            Some(Token::KwUnit) => {
                self.advance();
                Ok(Term::literal(Literal::Unit))
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Ok(Term::var(name))
            }
            Some(Token::KwLambda) => self.lambda_expr(),
            Some(Token::KwLet) => self.let_expr(),
            Some(Token::KwMatch) => self.match_expr(),
            Some(Token::KwView) => self.view_expr(),
            Some(Token::LParen) => self.parens_expr(),
            _ => Err(ParseError::UnexpectedToken(format!("{:?}", self.peek()))),
        }
    }

    fn lambda_expr(&mut self) -> Result<Term, ParseError> {
        self.consume(&Token::KwLambda)?;

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
        self.consume(&Token::Dot)?;
        let body = self.term()?;

        Ok(Term::abs(var, mult, annot, body))
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
        self.ty_arrow()
    }

    fn ty_arrow(&mut self) -> Result<Type, ParseError> {
        let mut lhs = self.ty_app()?;

        if matches!(self.peek(), Some(&Token::Arrow)) {
            self.advance();
            let rhs = self.ty_arrow()?;
            lhs = Type::arrow(lhs, Multiplicity::One, rhs);
        }

        Ok(lhs)
    }

    fn ty_app(&mut self) -> Result<Type, ParseError> {
        let mut ty = self.ty_atom()?;

        while matches!(self.peek(), Some(&Token::Ident(_)))
            || matches!(self.peek(), Some(&Token::TyInt))
            || matches!(self.peek(), Some(&Token::TyFloat))
            || matches!(self.peek(), Some(&Token::TyBool))
            || matches!(self.peek(), Some(&Token::TyChar))
            || matches!(self.peek(), Some(&Token::KwUnit))
            || matches!(self.peek(), Some(&Token::LParen))
            || matches!(self.peek(), Some(&Token::MultiplicityBorrow))
        {
            let arg = self.ty_atom()?;
            ty = match &ty {
                Type::Inductive(name, args) => {
                    let mut new_args = args.clone();
                    new_args.push(arg);
                    Type::inductive(name.0.clone(), new_args)
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        "expected type constructor".to_string(),
                    ))
                }
            };
        }

        Ok(ty)
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
            Some(Token::MultiplicityBorrow) => {
                self.advance();
                let inner = self.ty_atom()?;
                Ok(Type::borrow(inner))
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                if name
                    .chars()
                    .next()
                    .map(|c| c.is_lowercase())
                    .unwrap_or(false)
                {
                    Ok(Type::Var(name))
                } else {
                    Ok(Type::inductive(name, vec![]))
                }
            }
            Some(Token::LParen) => {
                self.consume(&Token::LParen)?;
                let ty = self.ty()?;
                match self.peek() {
                    Some(Token::Comma) => {
                        self.advance();
                        let ty2 = self.ty()?;
                        self.consume(&Token::RParen)?;
                        Ok(Type::product(ty, ty2))
                    }
                    _ => {
                        self.consume(&Token::RParen)?;
                        Ok(ty)
                    }
                }
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

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        self.pos += 1;
        tok
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            Some(Token::TyInt) => {
                self.advance();
                Ok("Int".to_string())
            }
            Some(Token::TyFloat) => {
                self.advance();
                Ok("Float".to_string())
            }
            Some(Token::TyBool) => {
                self.advance();
                Ok("Bool".to_string())
            }
            Some(Token::TyChar) => {
                self.advance();
                Ok("Char".to_string())
            }
            Some(Token::KwUnit) => {
                self.advance();
                Ok("Unit".to_string())
            }
            Some(token) => Err(ParseError::ExpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", token),
            }),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
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
