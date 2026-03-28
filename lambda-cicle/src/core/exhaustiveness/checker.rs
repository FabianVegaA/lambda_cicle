use crate::core::ast::{Pattern, Type};
use crate::core::typecheck::TypeError;

#[derive(Debug, Clone)]
pub struct ExhaustivenessChecker;

impl Default for ExhaustivenessChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl ExhaustivenessChecker {
    pub fn new() -> ExhaustivenessChecker {
        ExhaustivenessChecker
    }

    pub fn check(&self, _ty: &Type, patterns: &[Pattern]) -> Result<bool, TypeError> {
        if patterns.is_empty() {
            return Ok(false);
        }

        for pattern in patterns {
            if self.is_wildcard_pattern(pattern) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn check_match(&self, scrutinee_ty: &Type, arms: &[Pattern]) -> Result<(), TypeError> {
        let is_exhaustive = self.check(scrutinee_ty, arms)?;

        if !is_exhaustive {
            return Err(TypeError::NonExhaustivePattern);
        }

        Ok(())
    }

    fn is_wildcard_pattern(&self, pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Wildcard => true,
            Pattern::Var(_) => false,
            Pattern::Constructor(_, args) => args.iter().all(|p| self.is_wildcard_pattern(p)),
        }
    }
}
