use crate::core::ast::types::Multiplicity;
use crate::core::ast::{Pattern, Term};
use crate::core::typecheck::TypeError;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BorrowChecker {
    scope_stack: Vec<Vec<String>>,
}

#[allow(dead_code)]
impl BorrowChecker {
    pub fn new() -> BorrowChecker {
        BorrowChecker {
            scope_stack: Vec::new(),
        }
    }

    pub fn check(&self, term: &Term) -> Result<(), TypeError> {
        let mut checker = BorrowChecker::new();
        checker.check_term(term)
    }

    fn check_term(&mut self, term: &Term) -> Result<(), TypeError> {
        match term {
            Term::Var(_name) => Ok(()),
            Term::Abs {
                var,
                multiplicity,
                annot: _,
                body,
            } => {
                self.enter_scope();
                if *multiplicity == Multiplicity::Borrow {
                    self.add_borrow(var.clone());
                }
                let result = self.check_term(body);
                let _ = self.exit_scope();
                result
            }
            Term::App { fun, arg } => {
                self.check_term(fun)?;
                self.check_term(arg)?;
                Ok(())
            }
            Term::Let {
                var,
                multiplicity,
                annot: _,
                value,
                body,
            } => {
                self.check_term(value)?;
                self.enter_scope();
                if *multiplicity == Multiplicity::Borrow {
                    self.add_borrow(var.clone());
                }
                let result = self.check_term(body);
                let _ = self.exit_scope();
                result
            }
            Term::Match { scrutinee, arms } => {
                self.check_term(scrutinee)?;
                for arm in arms {
                    self.enter_scope();
                    self.add_pattern_binds(&arm.pattern, false)?;
                    let result = self.check_term(&arm.body);
                    self.exit_scope()?;
                    result?;
                }
                Ok(())
            }
            Term::View { scrutinee, arms } => {
                self.check_term(scrutinee)?;
                for arm in arms {
                    self.enter_scope();
                    self.add_pattern_binds(&arm.pattern, true)?;
                    let result = self.check_term(&arm.body);
                    self.exit_scope()?;
                    result?;
                }
                Ok(())
            }
            Term::TraitMethod {
                trait_name: _,
                method: _,
                arg,
            } => self.check_term(arg),
            Term::Constructor(_, args) => {
                for arg in args {
                    self.check_term(arg)?;
                }
                Ok(())
            }
            Term::NativeLiteral(_) => Ok(()),
            Term::BinaryOp { op: _, left, right } => {
                self.check_term(left)?;
                self.check_term(right)?;
                Ok(())
            }
            Term::UnaryOp { op: _, arg } => self.check_term(arg),
        }
    }

    fn enter_scope(&mut self) {
        self.scope_stack.push(Vec::new());
    }

    fn exit_scope(&mut self) -> Result<(), TypeError> {
        match self.scope_stack.pop() {
            Some(_) => Ok(()),
            None => Ok(()),
        }
    }

    fn add_borrow(&mut self, var: String) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.push(var);
        }
    }

    fn is_borrow_in_scope(&self, var: &str) -> bool {
        for scope in &self.scope_stack {
            if scope.contains(&var.to_string()) {
                return true;
            }
        }
        false
    }

    fn add_pattern_binds(&mut self, pattern: &Pattern, as_borrow: bool) -> Result<(), TypeError> {
        match pattern {
            Pattern::Wildcard => Ok(()),
            Pattern::Var(name) => {
                if as_borrow {
                    self.add_borrow(name.clone());
                }
                Ok(())
            }
            Pattern::Constructor(_, args) => {
                for arg in args {
                    self.add_pattern_binds(arg, as_borrow)?;
                }
                Ok(())
            }
        }
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}
