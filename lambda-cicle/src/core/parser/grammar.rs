use super::lexer::Token;
use crate::core::ast::terms::Constructor;
use crate::core::ast::{
    Arm, Constraint, Decl, Literal, MethodDef, MethodName, MethodSig, Multiplicity, Pattern, Term,
    TraitName, Type, UseMode, Visibility,
};

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Parser<'a> {
        Parser {
            tokens,
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn update_pos(&mut self) {
        if let Some(token) = self.peek() {
            if let Some((l, c)) = token.position() {
                self.line = l;
                self.col = c;
            }
        }
    }

    fn token_matches(token: &Token, expected: &Token) -> bool {
        std::mem::discriminant(token) == std::mem::discriminant(expected)
    }

    fn consume(&mut self, expected: &Token) -> Result<(), ParseError> {
        match self.peek() {
            Some(token) if Self::token_matches(token, expected) => {
                let pos = token.position();
                if let Some((l, c)) = pos {
                    self.line = l;
                    self.col = c;
                }
                self.advance();
                Ok(())
            }
            Some(token) => Err(ParseError::ExpectedToken {
                expected: format!("{:?}", expected),
                found: format!("{:?}", token),
                line: self.line,
                col: self.col,
            }),
            None => Err(ParseError::UnexpectedEndOfInput(self.line, self.col)),
        }
    }

    pub fn parse(&mut self) -> Result<Term, ParseError> {
        self.update_pos();
        let result = self.term()?;
        self.consume(&Token::EOF(self.line, self.col))?;
        Ok(result)
    }

    pub fn parse_program(&mut self) -> Result<Vec<Decl>, ParseError> {
        let mut decls = Vec::new();
        while !self.is_at_end() {
            self.update_pos();
            match self.peek() {
                Some(token) if Self::token_matches(token, &Token::EOF(0, 0)) => break,
                _ => {
                    let decl = self.decl()?;
                    decls.push(decl);
                }
            }
        }
        if !self.is_at_end() {
            self.consume(&Token::EOF(self.line, self.col))?;
        }
        Ok(decls)
    }

    fn decl(&mut self) -> Result<Decl, ParseError> {
        match self.peek() {
            Some(&Token::KwNoPrelude(_, _)) => {
                self.advance();
                Ok(Decl::NoPrelude)
            }
            Some(&Token::KwType(_, _)) => self.type_decl(None),
            Some(&Token::KwVal(_, _)) => self.val_decl(None),
            Some(&Token::KwTrait(_, _)) => self.trait_decl(None),
            Some(&Token::KwImpl(_, _)) => self.impl_decl(),
            Some(&Token::KwUse(_, _)) => self.use_decl(),
            Some(&Token::KwPub(_, _)) => {
                self.advance();
                match self.peek() {
                    Some(&Token::KwType(_, _)) => self.type_decl(Some(Visibility::Public)),
                    Some(&Token::KwVal(_, _)) => self.val_decl(Some(Visibility::Public)),
                    Some(&Token::KwTrait(_, _)) => self.trait_decl(Some(Visibility::Public)),
                    _ => Err(ParseError::UnexpectedToken(
                        format!("expected declaration after pub, found {:?}", self.peek()),
                        self.line,
                        self.col,
                    )),
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
        let _is_explicit_visibility = visibility_override.is_some();
        let visibility = visibility_override
            .unwrap_or_else(|| self.parse_visibility().unwrap_or(Visibility::Private));

        // Consume 'type' keyword - caller guarantees we're positioned at it
        self.consume(&Token::KwType(self.line, self.col))?;

        let name = self.expect_ident()?;
        let params = self.parse_type_params()?;

        // Check for (..) - abstract type marker
        let is_abstract = if matches!(self.peek(), Some(&Token::LParen(_, _))) {
            self.advance();
            match self.peek() {
                Some(&Token::DotDot(_, _)) => {
                    self.advance();
                    self.consume(&Token::RParen(self.line, self.col))?;
                    true
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        "expected ..".to_string(),
                        self.line,
                        self.col,
                    ))
                }
            }
        } else {
            false
        };

        let (ty, transparent, constructors) = if matches!(self.peek(), Some(&Token::Equals(_, _))) {
            self.advance();
            self.parse_type_body()?
        } else if is_abstract {
            (Type::unit(), true, Vec::new())
        } else {
            return Err(ParseError::UnexpectedToken(
                "expected = or (..)".to_string(),
                self.line,
                self.col,
            ));
        };

        Ok(Decl::TypeDecl {
            visibility,
            name,
            params,
            ty,
            transparent,
            constructors,
        })
    }

    fn parse_type_body(&mut self) -> Result<(Type, bool, Vec<Constructor>), ParseError> {
        // Decide whether this is a sum-type (constructors) or a type alias (plain type expr).
        // A sum type starts with:
        // - An uppercase identifier (e.g. True, Cons, Nil)
        // - A keyword that represents a constructor (True, False)
        // - Known lowercase constructors: None, Some, Ok, Err, LT, EQ, GT
        let is_sum_type = match self.peek() {
            Some(Token::Ident(name, _, _)) => {
                let known_ctors = ["None", "Some", "Ok", "Err", "LT", "EQ", "GT"];
                name.chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                    || known_ctors.contains(&name.as_str())
            }
            Some(Token::KwTrue(_, _)) | Some(Token::KwFalse(_, _)) => true,
            _ => false,
        };

        if !is_sum_type {
            // Type alias: parse as a full type expression (supports arrows, tuples, type apps)
            let ty = self.ty()?;
            return Ok((ty, false, Vec::new()));
        }

        // Sum type: parse one or more | -separated constructors
        let mut constructors = Vec::new();

        let first_constructor = self.parse_constructor()?;
        constructors.push(first_constructor);

        while matches!(self.peek(), Some(&Token::Pipe(_, _))) {
            self.advance();
            constructors.push(self.parse_constructor()?);
        }

        if constructors.len() == 1 {
            // Single-constructor sum: treat as a newtype wrapper; body = first arg or Unit
            let ctor = constructors.remove(0);
            let ty = ctor.args.first().cloned().unwrap_or(Type::unit());
            Ok((ty, false, Vec::new()))
        } else {
            let ty = self.build_sum_type(&constructors);
            Ok((ty, false, constructors))
        }
    }

    fn parse_constructor(&mut self) -> Result<Constructor, ParseError> {
        let name = self.expect_ident()?;
        let mut args = Vec::new();

        while self.is_type_atom_start()
            || matches!(self.peek(), Some(Token::MultiplicityBorrow(_, _)))
        {
            let arg_ty = self.ty_atom()?;
            args.push(arg_ty);
        }

        Ok(Constructor { name, args })
    }

    fn build_sum_type(&self, constructors: &[Constructor]) -> Type {
        if constructors.is_empty() {
            return Type::unit();
        }

        let mut result = Type::unit();
        for ctor in constructors.iter().rev() {
            let ctor_type = if ctor.args.is_empty() {
                Type::inductive(&ctor.name, vec![])
            } else {
                Type::inductive(&ctor.name, ctor.args.clone())
            };
            result = Type::sum(ctor_type, result);
        }
        result
    }

    fn val_decl(&mut self, visibility_override: Option<Visibility>) -> Result<Decl, ParseError> {
        let _is_explicit_visibility = visibility_override.is_some();
        let visibility = visibility_override
            .unwrap_or_else(|| self.parse_visibility().unwrap_or(Visibility::Private));

        // Consume 'val' keyword - caller guarantees we're positioned at it
        self.consume(&Token::KwVal(self.line, self.col))?;

        let name = self.expect_ident()?;
        self.consume(&Token::Colon(self.line, self.col))?;
        let ty = self.ty()?;
        self.consume(&Token::Equals(self.line, self.col))?;
        let term = Box::new(self.term()?);
        Ok(Decl::ValDecl {
            visibility,
            name,
            ty,
            term,
        })
    }

    fn trait_decl(&mut self, visibility_override: Option<Visibility>) -> Result<Decl, ParseError> {
        let _is_explicit_visibility = visibility_override.is_some();
        let visibility = visibility_override
            .unwrap_or_else(|| self.parse_visibility().unwrap_or(Visibility::Private));

        // Consume 'trait' keyword - caller guarantees we're positioned at it
        self.consume(&Token::KwTrait(self.line, self.col))?;
        let name = self.expect_ident()?;
        let params = self.parse_type_params()?;

        let supertrait = if matches!(self.peek(), Some(&Token::KwWhere(_, _))) {
            self.advance();
            // Check if next token is a supertrait name (uppercase) or val (method)
            if matches!(self.peek(), Some(&Token::Ident(ref name, _, _)) if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
            {
                let supertrait_name_str = self.expect_ident()?;
                let supertrait_name = TraitName(supertrait_name_str);
                let supertrait_params = self.parse_type_params()?;
                Some((supertrait_name, supertrait_params))
            } else {
                None
            }
        } else {
            None
        };

        let mut methods = Vec::new();

        // Parse method signatures - either inline after supertrait or in body {}
        while matches!(self.peek(), Some(&Token::KwVal(_, _))) {
            self.advance();
            let method_name_str = self.expect_ident()?;
            self.consume(&Token::Colon(self.line, self.col))?;
            let method_ty = self.ty()?;
            // Skip optional default implementation
            if matches!(self.peek(), Some(&Token::Equals(_, _))) {
                self.advance();
                let _default_term = self.term()?;
            }
            methods.push(MethodSig {
                name: MethodName(method_name_str),
                ty: method_ty,
            });
        }

        if matches!(self.peek(), Some(&Token::LBrace(_, _))) {
            self.consume(&Token::LBrace(self.line, self.col))?;
            while !matches!(self.peek(), Some(&Token::RBrace(_, _))) {
                if matches!(self.peek(), Some(&Token::KwVal(_, _))) {
                    self.advance();
                }

                let method_name_str = self.expect_ident()?;
                self.consume(&Token::Colon(self.line, self.col))?;
                let method_ty = self.ty()?;
                // Skip optional default implementation
                if matches!(self.peek(), Some(&Token::Equals(_, _))) {
                    self.advance();
                    let _default_term = self.term()?;
                }
                methods.push(MethodSig {
                    name: MethodName(method_name_str),
                    ty: method_ty,
                });
                if matches!(self.peek(), Some(Token::RBrace(_, _))) {
                    break;
                }
            }
            self.consume(&Token::RBrace(self.line, self.col))?;
        }

        Ok(Decl::TraitDecl {
            visibility,
            name,
            params,
            supertrait,
            methods,
        })
    }

    fn impl_decl(&mut self) -> Result<Decl, ParseError> {
        self.consume(&Token::KwImpl(self.line, self.col))?;

        // Support three syntax forms:
        //   impl Trait for Type [where ...] [with ...]
        //   impl Trait Type [where ...] [with ...]          (no 'for')
        //   impl (C1, C2) => Trait Type [where ...] [with ...]  (prefix constraints)

        // Check for prefix constraint syntax: impl (Eq a, Eq b) => ...
        let prefix_constraints: Vec<Constraint> =
            if matches!(self.peek(), Some(&Token::LParen(_, _))) {
                // Peek ahead to decide: is this a constraint list or a parenthesised type?
                // Heuristic: if the content of the parens looks like "TraitName typevar",
                // treat it as constraints. We consume tentatively.
                self.advance(); // consume '('
                if matches!(self.peek(), Some(&Token::RParen(_, _))) {
                    // empty parens — unusual, treat as no constraints
                    self.advance();
                    Vec::new()
                } else {
                    let mut cs = Vec::new();
                    loop {
                        let trait_str = self.expect_ident()?;
                        let cty = self.ty_atom()?;
                        cs.push(Constraint {
                            trait_name: TraitName(trait_str),
                            ty: cty,
                        });
                        if matches!(self.peek(), Some(&Token::Comma(_, _))) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.consume(&Token::RParen(self.line, self.col))?;
                    // Must be followed by '=>'
                    if matches!(self.peek(), Some(&Token::FatArrow(_, _))) {
                        self.advance(); // consume '=>'
                        cs
                    } else {
                        return Err(ParseError::UnexpectedToken(
                            "expected '=>' after impl constraints".to_string(),
                            self.line,
                            self.col,
                        ));
                    }
                }
            } else {
                Vec::new()
            };

        let trait_name_str = self.expect_ident()?;
        let trait_name = TraitName(trait_name_str);

        // Optional 'for' keyword
        if matches!(self.peek(), Some(Token::Ident(name, _, _)) if name == "for") {
            self.advance();
        }

        let ty = self.ty()?;

        let mut constraints = if matches!(self.peek(), Some(&Token::KwWhere(_, _))) {
            self.advance();
            self.parse_constraints()?
        } else {
            Vec::new()
        };
        constraints.extend(prefix_constraints);

        let mut methods = Vec::new();

        // Check for 'with' keyword for inline methods
        if matches!(self.peek(), Some(&Token::KwWith(_, _))) {
            self.advance();
            loop {
                // Break if we hit a new declaration
                match self.peek() {
                    Some(&Token::KwImpl(_, _))
                    | Some(&Token::KwType(_, _))
                    | Some(&Token::KwVal(_, _))
                    | Some(&Token::KwTrait(_, _))
                    | Some(&Token::EOF(_, _))
                    | None => break,
                    Some(&Token::RBrace(_, _)) => {
                        self.advance();
                        break;
                    }
                    _ => {}
                }

                // Skip optional 'val' keyword
                let method_name = if matches!(self.peek(), Some(&Token::KwVal(_, _))) {
                    self.advance();
                    self.expect_ident()?
                } else if matches!(self.peek(), Some(&Token::Ident(_, _, _))) {
                    self.expect_ident()?
                } else {
                    break;
                };

                let method_ty = if matches!(self.peek(), Some(&Token::Colon(_, _))) {
                    self.advance();
                    self.ty()?
                } else {
                    Type::unit()
                };

                if matches!(self.peek(), Some(&Token::Equals(_, _))) {
                    self.advance();
                    let term = Box::new(self.term()?);
                    methods.push(MethodDef {
                        name: MethodName(method_name),
                        ty: method_ty,
                        term,
                    });
                } else {
                    methods.push(MethodDef {
                        name: MethodName(method_name),
                        ty: method_ty,
                        term: Box::new(Term::NativeLiteral(crate::core::ast::Literal::Unit)),
                    });
                }

                // Check for comma
                if matches!(self.peek(), Some(&Token::Comma(_, _))) {
                    self.advance();
                    continue;
                }
                // Continue if next token looks like a method
                match self.peek() {
                    Some(&Token::Ident(_, _, _)) | Some(&Token::KwVal(_, _)) => continue,
                    _ => break,
                }
            }
        }

        // Check for '{' for block methods
        if matches!(self.peek(), Some(&Token::LBrace(_, _))) {
            self.consume(&Token::LBrace(self.line, self.col))?;
            while !matches!(self.peek(), Some(&Token::RBrace(_, _))) {
                self.consume(&Token::KwVal(self.line, self.col))?;

                let method_name_str = self.expect_ident()?;
                self.consume(&Token::Colon(self.line, self.col))?;
                let method_ty = self.ty()?;
                self.consume(&Token::Equals(self.line, self.col))?;
                let term = Box::new(self.term()?);

                methods.push(MethodDef {
                    name: MethodName(method_name_str),
                    ty: method_ty,
                    term,
                });

                if matches!(self.peek(), Some(&Token::Comma(_, _))) {
                    self.advance();
                }
            }
            self.consume(&Token::RBrace(self.line, self.col))?;
        }

        Ok(Decl::ImplDecl {
            ty,
            trait_name,
            constraints,
            methods,
        })
    }

    fn parse_constraints(&mut self) -> Result<Vec<Constraint>, ParseError> {
        let mut constraints = Vec::new();
        loop {
            let trait_name_str = self.expect_ident()?;
            let trait_name = TraitName(trait_name_str);
            let ty = self.ty()?;
            constraints.push(Constraint { trait_name, ty });

            if matches!(self.peek(), Some(Token::Comma(_, _))) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(constraints)
    }

    fn use_decl(&mut self) -> Result<Decl, ParseError> {
        self.consume(&Token::KwUse(self.line, self.col))?;
        let path = self.parse_module_path()?;
        let mode = match self.peek() {
            Some(&Token::LParen(_, _)) => {
                self.advance();
                match self.peek() {
                    Some(&Token::DotDot(_, _)) => {
                        self.advance();
                        self.consume(&Token::RParen(self.line, self.col))?;
                        UseMode::Unqualified
                    }
                    _ => {
                        let mut items = Vec::new();
                        items.push(self.expect_ident()?);
                        while matches!(self.peek(), Some(Token::Comma(_, _))) {
                            self.advance();
                            items.push(self.expect_ident()?);
                        }
                        self.consume(&Token::RParen(self.line, self.col))?;
                        UseMode::Selective(items)
                    }
                }
            }
            Some(&Token::KwAs(_, _)) => {
                self.advance();
                let alias = self.expect_ident()?;
                UseMode::Aliased(alias)
            }
            _ => UseMode::Qualified,
        };
        Ok(Decl::UseDecl { path, mode })
    }

    fn parse_visibility(&mut self) -> Result<Visibility, ParseError> {
        if matches!(self.peek(), Some(&Token::KwPub(_, _))) {
            self.advance();
            Ok(Visibility::Public)
        } else {
            Ok(Visibility::Private)
        }
    }

    fn parse_type_params(&mut self) -> Result<Vec<String>, ParseError> {
        let mut params = Vec::new();
        while matches!(self.peek(), Some(&Token::Ident(_, _, _))) {
            let name = self.expect_ident()?;
            if !name
                .chars()
                .next()
                .map(|c| c.is_lowercase() || c == '_')
                .unwrap_or(false)
            {
                return Err(ParseError::UnexpectedToken(
                    format!("type parameter must be lowercase, found '{}'", name),
                    self.line,
                    self.col,
                ));
            }
            params.push(name);
        }
        Ok(params)
    }

    fn parse_module_path(&mut self) -> Result<Vec<String>, ParseError> {
        let mut path = Vec::new();
        path.push(self.expect_ident()?);
        while matches!(self.peek(), Some(Token::Dot(_, _))) {
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
            Some(Token::Ident(_, _, _))
                | Some(Token::LParen(_, _))
                | Some(Token::KwLet(_, _))
                | Some(Token::KwMatch(_, _))
                | Some(Token::KwView(_, _))
                | Some(Token::IntLit(_, _, _))
                | Some(Token::FloatLit(_, _, _))
                | Some(Token::BoolLit(_, _, _))
                | Some(Token::CharLit(_, _, _))
                | Some(Token::StringLit(_, _, _))
                | Some(Token::UnitLit(_, _))
                | Some(Token::KwTrue(_, _))
                | Some(Token::KwFalse(_, _))
        )
    }

    fn atom_expr(&mut self) -> Result<Term, ParseError> {
        match self.peek() {
            Some(Token::IntLit(n, _, _)) => {
                let val = *n;
                self.advance();
                Ok(Term::literal(Literal::Int(val)))
            }
            Some(Token::FloatLit(n, _, _)) => {
                let val = *n;
                self.advance();
                Ok(Term::literal(Literal::Float(val)))
            }
            Some(Token::BoolLit(b, _, _)) => {
                let val = *b;
                self.advance();
                Ok(Term::literal(Literal::Bool(val)))
            }
            Some(Token::CharLit(c, _, _)) => {
                let val = *c;
                self.advance();
                Ok(Term::literal(Literal::Char(val)))
            }
            Some(Token::StringLit(s, _, _)) => {
                let val = s.clone();
                self.advance();
                Ok(Term::literal(Literal::Str(val)))
            }
            Some(Token::UnitLit(_, _)) => {
                self.advance();
                Ok(Term::literal(Literal::Unit))
            }
            Some(Token::KwUnit(_, _)) => {
                self.advance();
                Ok(Term::literal(Literal::Unit))
            }
            Some(Token::KwTrue(_, _)) => {
                self.advance();
                Ok(Term::Constructor("True".to_string(), Vec::new()))
            }
            Some(Token::KwFalse(_, _)) => {
                self.advance();
                Ok(Term::Constructor("False".to_string(), Vec::new()))
            }
            Some(Token::Ident(name, _, _)) => {
                let name = name.clone();
                self.advance();
                if name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    Ok(Term::Constructor(name, Vec::new()))
                } else {
                    Ok(Term::var(name))
                }
            }
            Some(Token::KwLambda(_, _)) => self.lambda_expr(),
            Some(Token::KwLet(_, _)) => self.let_expr(),
            Some(Token::KwMatch(_, _)) => self.match_expr(),
            Some(Token::KwView(_, _)) => self.view_expr(),
            Some(Token::LParen(_, _)) => self.parens_expr(),
            _ => Err(ParseError::UnexpectedToken(
                format!("{:?}", self.peek()),
                self.line,
                self.col,
            )),
        }
    }

    fn let_expr(&mut self) -> Result<Term, ParseError> {
        self.consume(&Token::KwLet(self.line, self.col))?;

        let var = match self.peek() {
            Some(Token::Ident(name, _, _)) => {
                let n = name.clone();
                self.advance();
                n
            }
            _ => {
                return Err(ParseError::UnexpectedToken(
                    "expected variable".to_string(),
                    self.line,
                    self.col,
                ))
            }
        };

        // Type annotation is optional
        let (mult, annot) = if matches!(self.peek(), Some(&Token::Colon(_, _))) {
            self.advance();

            // Check if next token is a multiplicity
            let tok = self.peek().cloned();
            let mult = match tok {
                Some(Token::MultiplicityZero(_, _)) => {
                    self.advance();
                    self.consume(&Token::Colon(self.line, self.col))?;
                    Multiplicity::Zero
                }
                Some(Token::MultiplicityOne(_, _)) => {
                    self.advance();
                    self.consume(&Token::Colon(self.line, self.col))?;
                    Multiplicity::One
                }
                Some(Token::MultiplicityOmega(_, _)) => {
                    self.advance();
                    self.consume(&Token::Colon(self.line, self.col))?;
                    Multiplicity::Omega
                }
                Some(Token::MultiplicityBorrow(_, _)) => {
                    // & is a multiplicity annotation only when followed by ':'
                    if matches!(self.tokens.get(self.pos + 1), Some(&Token::Colon(_, _))) {
                        self.advance(); // consume &
                        self.consume(&Token::Colon(self.line, self.col))?; // consume :
                        Multiplicity::Borrow
                    } else {
                        // & starts a borrow type, not a multiplicity
                        Multiplicity::One
                    }
                }
                Some(Token::IntLit(n, _, _)) => {
                    // Handle number as multiplicity (0 or 1)
                    self.advance();
                    self.consume(&Token::Colon(self.line, self.col))?;
                    match n {
                        0 => Multiplicity::Zero,
                        1 => Multiplicity::One,
                        _ => {
                            return Err(ParseError::UnexpectedToken(
                                format!("expected multiplicity (0, 1), found {}", n),
                                self.line,
                                self.col,
                            ))
                        }
                    }
                }
                _ => {
                    // No multiplicity - type follows directly
                    Multiplicity::One
                }
            };

            let annot = self.ty()?;
            (mult, annot)
        } else {
            // No type annotation - use default
            (Multiplicity::One, Type::unit())
        };

        self.consume(&Token::Equals(self.line, self.col))?;
        let value = self.term()?;
        self.consume(&Token::KwIn(self.line, self.col))?;
        let body = self.term()?;

        Ok(Term::let_in(var, mult, annot, value, body))
    }

    fn lambda_expr(&mut self) -> Result<Term, ParseError> {
        self.consume(&Token::KwLambda(self.line, self.col))?;

        let var = match self.peek() {
            Some(Token::Ident(name, _, _)) => {
                let n = name.clone();
                self.advance();
                n
            }
            _ => {
                return Err(ParseError::UnexpectedToken(
                    "expected variable".to_string(),
                    self.line,
                    self.col,
                ))
            }
        };

        // Type annotation is optional
        let (mult, annot) = if matches!(self.peek(), Some(&Token::Colon(_, _))) {
            self.advance();

            // Check if next token is a multiplicity
            let tok = self.peek().cloned();
            let mult = match tok {
                Some(Token::MultiplicityZero(_, _)) => {
                    self.advance();
                    self.consume(&Token::Colon(self.line, self.col))?;
                    Multiplicity::Zero
                }
                Some(Token::MultiplicityOne(_, _)) => {
                    self.advance();
                    self.consume(&Token::Colon(self.line, self.col))?;
                    Multiplicity::One
                }
                Some(Token::MultiplicityOmega(_, _)) => {
                    self.advance();
                    self.consume(&Token::Colon(self.line, self.col))?;
                    Multiplicity::Omega
                }
                Some(Token::MultiplicityBorrow(_, _)) => {
                    // & is a multiplicity annotation only when followed by ':'
                    if matches!(self.tokens.get(self.pos + 1), Some(&Token::Colon(_, _))) {
                        self.advance(); // consume &
                        self.consume(&Token::Colon(self.line, self.col))?; // consume :
                        Multiplicity::Borrow
                    } else {
                        // & starts a borrow type, not a multiplicity
                        Multiplicity::One
                    }
                }
                Some(Token::IntLit(n, _, _)) => {
                    // Handle number as multiplicity (0 or 1)
                    self.advance();
                    self.consume(&Token::Colon(self.line, self.col))?;
                    match n {
                        0 => Multiplicity::Zero,
                        1 => Multiplicity::One,
                        _ => {
                            return Err(ParseError::UnexpectedToken(
                                format!("expected multiplicity (0, 1), found {}", n),
                                self.line,
                                self.col,
                            ))
                        }
                    }
                }
                _ => {
                    // No multiplicity - type follows directly
                    Multiplicity::One
                }
            };

            let annot = self.ty()?;
            (mult, annot)
        } else {
            // No type annotation - require explicit annotations for now
            (Multiplicity::One, Type::unit())
        };

        self.consume(&Token::Equals(self.line, self.col))?;
        let value = self.term()?;
        self.consume(&Token::KwIn(self.line, self.col))?;
        let body = self.term()?;

        Ok(Term::let_in(var, mult, annot, value, body))
    }

    fn match_expr(&mut self) -> Result<Term, ParseError> {
        self.consume(&Token::KwMatch(self.line, self.col))?;
        let scrutinee = self.term()?;
        self.consume(&Token::KwWith(self.line, self.col))?;
        self.consume(&Token::LBrace(self.line, self.col))?;

        let mut arms = Vec::new();
        loop {
            let pattern = self.pattern()?;
            self.consume(&Token::FatArrow(self.line, self.col))?;
            let body = self.term()?;
            arms.push(Arm::new(pattern, body));

            match self.peek() {
                Some(Token::Pipe(_, _)) => {
                    self.advance();
                    continue;
                }
                Some(Token::RBrace(_, _)) => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        "expected | or }".to_string(),
                        self.line,
                        self.col,
                    ))
                }
            }
        }

        Ok(Term::match_on(scrutinee, arms))
    }

    fn view_expr(&mut self) -> Result<Term, ParseError> {
        self.consume(&Token::KwView(self.line, self.col))?;
        let scrutinee = self.term()?;
        self.consume(&Token::KwWith(self.line, self.col))?;
        self.consume(&Token::LBrace(self.line, self.col))?;

        let mut arms = Vec::new();
        loop {
            let pattern = self.pattern()?;
            self.consume(&Token::FatArrow(self.line, self.col))?;
            let body = self.term()?;
            arms.push(Arm::new(pattern, body));

            match self.peek() {
                Some(Token::Pipe(_, _)) => {
                    self.advance();
                    continue;
                }
                Some(Token::RBrace(_, _)) => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        "expected | or }".to_string(),
                        self.line,
                        self.col,
                    ))
                }
            }
        }

        Ok(Term::view_on(scrutinee, arms))
    }

    fn parens_expr(&mut self) -> Result<Term, ParseError> {
        self.consume(&Token::LParen(self.line, self.col))?;
        let expr = self.term()?;
        self.consume(&Token::RParen(self.line, self.col))?;
        Ok(expr)
    }

    #[allow(dead_code)]
    fn multiplicity(&mut self) -> Result<Multiplicity, ParseError> {
        let tok = self.peek().cloned();
        match tok {
            Some(Token::MultiplicityZero(_, _)) => {
                self.advance();
                Ok(Multiplicity::Zero)
            }
            Some(Token::MultiplicityOne(_, _)) => {
                self.advance();
                Ok(Multiplicity::One)
            }
            Some(Token::MultiplicityOmega(_, _)) => {
                self.advance();
                Ok(Multiplicity::Omega)
            }
            Some(Token::MultiplicityBorrow(_, _)) => {
                self.advance();
                Ok(Multiplicity::Borrow)
            }
            Some(Token::IntLit(n, _, _)) => {
                // Handle number as multiplicity (0 or 1)
                self.advance();
                match n {
                    0 => Ok(Multiplicity::Zero),
                    1 => Ok(Multiplicity::One),
                    _ => Err(ParseError::UnexpectedToken(
                        format!("expected multiplicity (0, 1, ω, &), found {}", n),
                        self.line,
                        self.col,
                    )),
                }
            }
            _ => Err(ParseError::UnexpectedToken(
                "expected multiplicity".to_string(),
                self.line,
                self.col,
            )),
        }
    }

    fn ty(&mut self) -> Result<Type, ParseError> {
        self.ty_arrow()
    }

    fn ty_arrow(&mut self) -> Result<Type, ParseError> {
        let lhs = self.ty_app()?;

        if matches!(self.peek(), Some(&Token::Arrow(_, _))) {
            self.advance();
            let rhs = self.ty_arrow()?;
            Ok(Type::arrow(lhs, Multiplicity::One, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn ty_app(&mut self) -> Result<Type, ParseError> {
        let mut ty = self.ty_atom()?;

        while self.is_type_atom_start() {
            let arg = self.ty_atom()?;
            ty = match &ty {
                Type::Inductive(name, args) => {
                    let mut new_args = args.clone();
                    new_args.push(arg);
                    Type::inductive(name.0.clone(), new_args)
                }
                Type::Var(name) => Type::app(Type::Var(name.clone()), vec![arg]),
                Type::App(base, args) => {
                    let mut new_args = args.clone();
                    new_args.push(arg);
                    Type::app(*base.clone(), new_args)
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        "expected type constructor or type variable".to_string(),
                        self.line,
                        self.col,
                    ))
                }
            };
        }

        Ok(ty)
    }

    fn is_type_atom_start(&self) -> bool {
        matches!(
            self.peek(),
            Some(&Token::TyInt(_, _))
                | Some(&Token::TyFloat(_, _))
                | Some(&Token::TyChar(_, _))
                | Some(&Token::KwUnit(_, _))
                | Some(&Token::LParen(_, _))
                | Some(&Token::KwTrue(_, _))  // Bool constructors
                | Some(&Token::KwFalse(_, _))
                | Some(&Token::Ident(_, _, _)) // All identifiers (both uppercase constructors and lowercase type variables)
        )
    }

    fn ty_atom(&mut self) -> Result<Type, ParseError> {
        match self.peek() {
            Some(Token::KwUnit(_, _)) => {
                self.advance();
                Ok(Type::unit())
            }
            Some(Token::TyInt(_, _)) => {
                self.advance();
                Ok(Type::int())
            }
            Some(Token::TyFloat(_, _)) => {
                self.advance();
                Ok(Type::float())
            }
            Some(Token::TyChar(_, _)) => {
                self.advance();
                Ok(Type::char())
            }
            Some(Token::MultiplicityBorrow(_, _)) => {
                self.advance();
                let inner = self.ty_atom()?;
                Ok(Type::borrow(inner))
            }
            Some(Token::Ident(name, _, _)) => {
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
            Some(Token::KwTrue(_, _)) => {
                self.advance();
                Ok(Type::inductive("True".to_string(), vec![]))
            }
            Some(Token::KwFalse(_, _)) => {
                self.advance();
                Ok(Type::inductive("False".to_string(), vec![]))
            }
            Some(Token::LParen(_, _)) => {
                self.consume(&Token::LParen(self.line, self.col))?;
                let ty = self.ty()?;
                match self.peek() {
                    Some(Token::Comma(_, _)) => {
                        self.advance();
                        let ty2 = self.ty()?;
                        self.consume(&Token::RParen(self.line, self.col))?;
                        Ok(Type::product(ty, ty2))
                    }
                    _ => {
                        self.consume(&Token::RParen(self.line, self.col))?;
                        Ok(ty)
                    }
                }
            }
            _ => Err(ParseError::UnexpectedToken(
                "expected type".to_string(),
                self.line,
                self.col,
            )),
        }
    }

    fn pattern(&mut self) -> Result<Pattern, ParseError> {
        let p = self.pattern_atom()?;
        let mut args = Vec::new();
        while self.is_pattern_atom_start() {
            args.push(self.pattern_atom()?);
        }
        if args.is_empty() {
            Ok(p)
        } else {
            match p {
                Pattern::Constructor(name, mut ctor_args) => {
                    ctor_args.extend(args);
                    Ok(Pattern::Constructor(name, ctor_args))
                }
                _ => Ok(p),
            }
        }
    }

    fn pattern_atom(&mut self) -> Result<Pattern, ParseError> {
        match self.peek() {
            Some(Token::Underscore(_, _)) => {
                self.advance();
                Ok(Pattern::wildcard())
            }
            Some(Token::Ident(name, _, _)) => {
                let name = name.clone();
                self.advance();
                if name == "true" {
                    Ok(Pattern::Constructor("True".to_string(), Vec::new()))
                } else if name == "false" {
                    Ok(Pattern::Constructor("False".to_string(), Vec::new()))
                } else if name
                    .chars()
                    .next()
                    .map(|c| c.is_lowercase())
                    .unwrap_or(false)
                {
                    Ok(Pattern::Var(name))
                } else {
                    let args = self.pattern_args()?;
                    Ok(Pattern::Constructor(name, args))
                }
            }
            Some(Token::KwTrue(_, _)) => {
                self.advance();
                Ok(Pattern::Constructor("True".to_string(), Vec::new()))
            }
            Some(Token::KwFalse(_, _)) => {
                self.advance();
                Ok(Pattern::Constructor("False".to_string(), Vec::new()))
            }
            Some(Token::LParen(_, _)) => self.pattern_parens(),
            _ => Err(ParseError::UnexpectedToken(
                "expected pattern".to_string(),
                self.line,
                self.col,
            )),
        }
    }

    fn pattern_args(&mut self) -> Result<Vec<Pattern>, ParseError> {
        let mut args = Vec::new();
        while self.is_pattern_atom_start() {
            args.push(self.pattern_atom()?);
        }
        Ok(args)
    }

    fn is_pattern_atom_start(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::Underscore(_, _)) | Some(Token::Ident(_, _, _)) | Some(Token::LParen(_, _))
        )
    }

    fn pattern_parens(&mut self) -> Result<Pattern, ParseError> {
        self.consume(&Token::LParen(self.line, self.col))?;
        let p = self.pattern()?;
        self.consume(&Token::RParen(self.line, self.col))?;
        Ok(p)
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
            Some(Token::Ident(name, _, _)) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            Some(Token::TyInt(_, _)) => {
                self.advance();
                Ok("Int".to_string())
            }
            Some(Token::TyFloat(_, _)) => {
                self.advance();
                Ok("Float".to_string())
            }
            Some(Token::TyChar(_, _)) => {
                self.advance();
                Ok("Char".to_string())
            }
            Some(Token::KwUnit(_, _)) => {
                self.advance();
                Ok("Unit".to_string())
            }
            Some(Token::KwTrue(_, _)) => {
                self.advance();
                Ok("True".to_string())
            }
            Some(Token::KwFalse(_, _)) => {
                self.advance();
                Ok("False".to_string())
            }
            Some(token) => Err(ParseError::ExpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", token),
                line: self.line,
                col: self.col,
            }),
            None => Err(ParseError::UnexpectedEndOfInput(self.line, self.col)),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String, usize, usize),
    ExpectedToken {
        expected: String,
        found: String,
        line: usize,
        col: usize,
    },
    UnexpectedEndOfInput(usize, usize),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(s, line, col) => write!(
                f,
                "Parse error at line {}, column {}: Unexpected token: {}",
                line, col, s
            ),
            ParseError::ExpectedToken {
                expected,
                found,
                line,
                col,
            } => {
                write!(
                    f,
                    "Parse error at line {}, column {}: Expected {}, found {}",
                    line, col, expected, found
                )
            }
            ParseError::UnexpectedEndOfInput(line, col) => write!(
                f,
                "Parse error at line {}, column {}: Unexpected end of input",
                line, col
            ),
        }
    }
}

impl std::error::Error for ParseError {}
