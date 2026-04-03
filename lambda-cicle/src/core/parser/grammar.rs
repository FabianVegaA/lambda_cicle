use super::lexer::Token;
use crate::core::ast::terms::Constructor;
use crate::core::ast::{
    Arm, Constraint, Decl, Literal, MethodDef, MethodName, MethodSig, Multiplicity, Pattern, Term,
    TraitName, Type, UseMode, Visibility,
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
        let _is_explicit_visibility = visibility_override.is_some();
        let visibility = visibility_override
            .unwrap_or_else(|| self.parse_visibility().unwrap_or(Visibility::Private));

        // Consume 'type' keyword - caller guarantees we're positioned at it
        self.consume(&Token::KwType)?;

        let name = self.expect_ident()?;
        let params = self.parse_type_params()?;

        // Check for (..) - abstract type marker
        let is_abstract = if matches!(self.peek(), Some(&Token::LParen)) {
            self.advance();
            match self.peek() {
                Some(&Token::DotDot) => {
                    self.advance();
                    self.consume(&Token::RParen)?;
                    true
                }
                _ => return Err(ParseError::UnexpectedToken("expected ..".to_string())),
            }
        } else {
            false
        };

        let (ty, transparent, constructors) = if matches!(self.peek(), Some(&Token::Equals)) {
            self.advance();
            self.parse_type_body()?
        } else if is_abstract {
            (Type::unit(), true, Vec::new())
        } else {
            return Err(ParseError::UnexpectedToken(
                "expected = or (..)".to_string(),
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
            Some(Token::Ident(name)) => {
                let known_ctors = ["None", "Some", "Ok", "Err", "LT", "EQ", "GT"];
                name.chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                    || known_ctors.contains(&name.as_str())
            }
            Some(Token::KwTrue) | Some(Token::KwFalse) => true,
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

        while matches!(self.peek(), Some(&Token::Pipe)) {
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

        while self.is_type_atom_start() || matches!(self.peek(), Some(Token::MultiplicityBorrow)) {
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
        let _is_explicit_visibility = visibility_override.is_some();
        let visibility = visibility_override
            .unwrap_or_else(|| self.parse_visibility().unwrap_or(Visibility::Private));

        // Consume 'trait' keyword - caller guarantees we're positioned at it
        self.consume(&Token::KwTrait)?;
        let name = self.expect_ident()?;
        let params = self.parse_type_params()?;

        let supertrait = if matches!(self.peek(), Some(&Token::KwWhere)) {
            self.advance();
            // Check if next token is a supertrait name (uppercase) or val (method)
            if matches!(self.peek(), Some(&Token::Ident(ref name)) if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
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
        while matches!(self.peek(), Some(&Token::KwVal)) {
            self.advance();
            let method_name_str = self.expect_ident()?;
            self.consume(&Token::Colon)?;
            let method_ty = self.ty()?;
            // Skip optional default implementation
            if matches!(self.peek(), Some(&Token::Equals)) {
                self.advance();
                let _default_term = self.term()?;
            }
            methods.push(MethodSig {
                name: MethodName(method_name_str),
                ty: method_ty,
            });
        }

        if matches!(self.peek(), Some(&Token::LBrace)) {
            self.consume(&Token::LBrace)?;
            while !matches!(self.peek(), Some(&Token::RBrace)) {
                if matches!(self.peek(), Some(&Token::KwVal)) {
                    self.advance();
                }

                let method_name_str = self.expect_ident()?;
                self.consume(&Token::Colon)?;
                let method_ty = self.ty()?;
                // Skip optional default implementation
                if matches!(self.peek(), Some(&Token::Equals)) {
                    self.advance();
                    let _default_term = self.term()?;
                }
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
            supertrait,
            methods,
        })
    }

    fn impl_decl(&mut self) -> Result<Decl, ParseError> {
        self.consume(&Token::KwImpl)?;

        // Support three syntax forms:
        //   impl Trait for Type [where ...] [with ...]
        //   impl Trait Type [where ...] [with ...]          (no 'for')
        //   impl (C1, C2) => Trait Type [where ...] [with ...]  (prefix constraints)

        // Check for prefix constraint syntax: impl (Eq a, Eq b) => ...
        let prefix_constraints: Vec<Constraint> = if matches!(self.peek(), Some(&Token::LParen)) {
            // Peek ahead to decide: is this a constraint list or a parenthesised type?
            // Heuristic: if the content of the parens looks like "TraitName typevar",
            // treat it as constraints. We consume tentatively.
            self.advance(); // consume '('
            if matches!(self.peek(), Some(&Token::RParen)) {
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
                    if matches!(self.peek(), Some(&Token::Comma)) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.consume(&Token::RParen)?;
                // Must be followed by '=>'
                if matches!(self.peek(), Some(&Token::FatArrow)) {
                    self.advance(); // consume '=>'
                    cs
                } else {
                    return Err(ParseError::UnexpectedToken(
                        "expected '=>' after impl constraints".to_string(),
                    ));
                }
            }
        } else {
            Vec::new()
        };

        let trait_name_str = self.expect_ident()?;
        let trait_name = TraitName(trait_name_str);

        // Optional 'for' keyword
        if matches!(self.peek(), Some(Token::Ident(name)) if name == "for") {
            self.advance();
        }

        let ty = self.ty()?;

        let mut constraints = if matches!(self.peek(), Some(&Token::KwWhere)) {
            self.advance();
            self.parse_constraints()?
        } else {
            Vec::new()
        };
        constraints.extend(prefix_constraints);

        let mut methods = Vec::new();

        // Check for 'with' keyword for inline methods
        if matches!(self.peek(), Some(&Token::KwWith)) {
            self.advance();
            loop {
                // Break if we hit a new declaration
                match self.peek() {
                    Some(&Token::KwImpl)
                    | Some(&Token::KwType)
                    | Some(&Token::KwVal)
                    | Some(&Token::KwTrait)
                    | Some(&Token::EOF)
                    | None => break,
                    Some(&Token::RBrace) => {
                        self.advance();
                        break;
                    }
                    _ => {}
                }

                // Skip optional 'val' keyword
                let method_name = if matches!(self.peek(), Some(&Token::KwVal)) {
                    self.advance();
                    self.expect_ident()?
                } else if matches!(self.peek(), Some(&Token::Ident(_))) {
                    self.expect_ident()?
                } else {
                    break;
                };

                let method_ty = if matches!(self.peek(), Some(&Token::Colon)) {
                    self.advance();
                    self.ty()?
                } else {
                    Type::unit()
                };

                if matches!(self.peek(), Some(&Token::Equals)) {
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
                if matches!(self.peek(), Some(&Token::Comma)) {
                    self.advance();
                    continue;
                }
                // Continue if next token looks like a method
                match self.peek() {
                    Some(&Token::Ident(_)) | Some(&Token::KwVal) => continue,
                    _ => break,
                }
            }
        }

        // Check for '{' for block methods
        if matches!(self.peek(), Some(&Token::LBrace)) {
            self.consume(&Token::LBrace)?;
            while !matches!(self.peek(), Some(&Token::RBrace)) {
                self.consume(&Token::KwVal)?;

                let method_name_str = self.expect_ident()?;
                self.consume(&Token::Colon)?;
                let method_ty = self.ty()?;
                self.consume(&Token::Equals)?;
                let term = Box::new(self.term()?);

                methods.push(MethodDef {
                    name: MethodName(method_name_str),
                    ty: method_ty,
                    term,
                });

                if matches!(self.peek(), Some(&Token::Comma)) {
                    self.advance();
                }
            }
            self.consume(&Token::RBrace)?;
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

            if matches!(self.peek(), Some(Token::Comma)) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(constraints)
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
            let name = self.expect_ident()?;
            if !name
                .chars()
                .next()
                .map(|c| c.is_lowercase() || c == '_')
                .unwrap_or(false)
            {
                return Err(ParseError::UnexpectedToken(format!(
                    "type parameter must be lowercase, found '{}'",
                    name
                )));
            }
            params.push(name);
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
                | Some(Token::StringLit(_))
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
            Some(Token::StringLit(s)) => {
                let val = s.clone();
                self.advance();
                Ok(Term::literal(Literal::Str(val)))
            }
            Some(Token::UnitLit) => {
                self.advance();
                Ok(Term::literal(Literal::Unit))
            }
            Some(Token::KwUnit) => {
                self.advance();
                Ok(Term::literal(Literal::Unit))
            }
            Some(Token::KwTrue) => {
                self.advance();
                Ok(Term::Constructor("True".to_string(), Vec::new()))
            }
            Some(Token::KwFalse) => {
                self.advance();
                Ok(Term::Constructor("False".to_string(), Vec::new()))
            }
            Some(Token::Ident(name)) => {
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

        // Type annotation is optional
        let has_colon = matches!(self.peek(), Some(&Token::Colon));

        let (mult, annot) = if has_colon {
            self.advance(); // consume first colon

            // Peek to see what's next
            let tok = self.peek().cloned();
            let mult = match tok {
                Some(Token::MultiplicityZero) => {
                    self.advance();
                    self.consume(&Token::Colon)?;
                    Multiplicity::Zero
                }
                Some(Token::MultiplicityOne) => {
                    self.advance();
                    self.consume(&Token::Colon)?;
                    Multiplicity::One
                }
                Some(Token::MultiplicityOmega) => {
                    self.advance();
                    self.consume(&Token::Colon)?;
                    Multiplicity::Omega
                }
                Some(Token::MultiplicityBorrow) => {
                    // & is a multiplicity annotation only when followed by ':'
                    // e.g. \x : & : Bool  (multiplicity borrow)
                    // vs   \x : &Bool     (borrow type, & is part of the type)
                    if matches!(self.tokens.get(self.pos + 1), Some(&Token::Colon)) {
                        self.advance(); // consume &
                        self.consume(&Token::Colon)?; // consume :
                        Multiplicity::Borrow
                    } else {
                        // & starts a borrow type, not a multiplicity
                        Multiplicity::One
                    }
                }
                Some(Token::IntLit(n)) => {
                    self.advance();
                    self.consume(&Token::Colon)?;
                    match n {
                        0 => Multiplicity::Zero,
                        1 => Multiplicity::One,
                        _ => {
                            return Err(ParseError::UnexpectedToken(format!(
                                "expected multiplicity (0, 1), found {}",
                                n
                            )))
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

        // Type annotation is optional
        let (mult, annot) = if matches!(self.peek(), Some(&Token::Colon)) {
            self.advance();

            // Check if next token is a multiplicity
            let tok = self.peek().cloned();
            let mult = match tok {
                Some(Token::MultiplicityZero) => {
                    self.advance();
                    self.consume(&Token::Colon)?;
                    Multiplicity::Zero
                }
                Some(Token::MultiplicityOne) => {
                    self.advance();
                    self.consume(&Token::Colon)?;
                    Multiplicity::One
                }
                Some(Token::MultiplicityOmega) => {
                    self.advance();
                    self.consume(&Token::Colon)?;
                    Multiplicity::Omega
                }
                Some(Token::MultiplicityBorrow) => {
                    // & is a multiplicity annotation only when followed by ':'
                    if matches!(self.tokens.get(self.pos + 1), Some(&Token::Colon)) {
                        self.advance(); // consume &
                        self.consume(&Token::Colon)?; // consume :
                        Multiplicity::Borrow
                    } else {
                        // & starts a borrow type, not a multiplicity
                        Multiplicity::One
                    }
                }
                Some(Token::IntLit(n)) => {
                    // Handle number as multiplicity (0 or 1)
                    self.advance();
                    self.consume(&Token::Colon)?;
                    match n {
                        0 => Multiplicity::Zero,
                        1 => Multiplicity::One,
                        _ => {
                            return Err(ParseError::UnexpectedToken(format!(
                                "expected multiplicity (0, 1), found {}",
                                n
                            )))
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
            self.consume(&Token::FatArrow)?;
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
            self.consume(&Token::FatArrow)?;
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

    #[allow(dead_code)]
    fn multiplicity(&mut self) -> Result<Multiplicity, ParseError> {
        let tok = self.peek().cloned();
        match tok {
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
            Some(Token::IntLit(n)) => {
                // Handle number as multiplicity (0 or 1)
                self.advance();
                match n {
                    0 => Ok(Multiplicity::Zero),
                    1 => Ok(Multiplicity::One),
                    _ => Err(ParseError::UnexpectedToken(format!(
                        "expected multiplicity (0, 1, ω, &), found {}",
                        n
                    ))),
                }
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
        let lhs = self.ty_app()?;

        if matches!(self.peek(), Some(&Token::Arrow)) {
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
                    ))
                }
            };
        }

        Ok(ty)
    }

    fn is_type_atom_start(&self) -> bool {
        matches!(
            self.peek(),
            Some(&Token::TyInt)
                | Some(&Token::TyFloat)
                | Some(&Token::TyChar)
                | Some(&Token::KwUnit)
                | Some(&Token::LParen)
                | Some(&Token::KwTrue)  // Bool constructors
                | Some(&Token::KwFalse)
                | Some(&Token::Ident(_)) // All identifiers (both uppercase constructors and lowercase type variables)
        )
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
            Some(Token::KwTrue) => {
                self.advance();
                Ok(Type::inductive("True".to_string(), vec![]))
            }
            Some(Token::KwFalse) => {
                self.advance();
                Ok(Type::inductive("False".to_string(), vec![]))
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
            Some(Token::Underscore) => {
                self.advance();
                Ok(Pattern::wildcard())
            }
            Some(Token::Ident(name)) => {
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
            Some(Token::KwTrue) => {
                self.advance();
                Ok(Pattern::Constructor("True".to_string(), Vec::new()))
            }
            Some(Token::KwFalse) => {
                self.advance();
                Ok(Pattern::Constructor("False".to_string(), Vec::new()))
            }
            Some(Token::LParen) => self.pattern_parens(),
            _ => Err(ParseError::UnexpectedToken("expected pattern".to_string())),
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
            Some(Token::Underscore) | Some(Token::Ident(_)) | Some(Token::LParen)
        )
    }

    fn pattern_parens(&mut self) -> Result<Pattern, ParseError> {
        self.consume(&Token::LParen)?;
        let p = self.pattern()?;
        self.consume(&Token::RParen)?;
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
            Some(Token::TyChar) => {
                self.advance();
                Ok("Char".to_string())
            }
            Some(Token::KwUnit) => {
                self.advance();
                Ok("Unit".to_string())
            }
            Some(Token::KwTrue) => {
                self.advance();
                Ok("True".to_string())
            }
            Some(Token::KwFalse) => {
                self.advance();
                Ok("False".to_string())
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
