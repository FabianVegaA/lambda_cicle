use super::{Exports, Module, ModuleError};
use crate::traits::Implementation;
use crate::{parse, translate, type_check_with_borrow_check, Term};
use std::path::Path;

pub fn parse_module_file(path: &Path) -> Result<Term, ModuleError> {
    let source = std::fs::read_to_string(path).map_err(|e| ModuleError {
        message: format!("Failed to read file: {}", e),
    })?;

    let term = parse(&source)?;
    Ok(term)
}

pub fn compile_module(name: String, term: Term) -> Result<Module, ModuleError> {
    let ty = type_check_with_borrow_check(&term)?;

    let net = translate(&term);

    let exports = Exports::from_term(&term, ty);

    let impls = extract_impls(&term);

    Ok(Module {
        name,
        exports,
        impls,
        net,
    })
}

fn extract_impls(term: &crate::core::ast::Term) -> Vec<Implementation> {
    let mut impls = Vec::new();
    collect_impls(term, &mut impls);
    impls
}

fn collect_impls(term: &crate::core::ast::Term, impls: &mut Vec<Implementation>) {
    match term {
        crate::core::ast::Term::Abs { body, .. } => {
            collect_impls(body, impls);
        }
        crate::core::ast::Term::App { fun, arg } => {
            collect_impls(fun, impls);
            collect_impls(arg, impls);
        }
        crate::core::ast::Term::Let { body, value, .. } => {
            collect_impls(value, impls);
            collect_impls(body, impls);
        }
        crate::core::ast::Term::Match { scrutinee, arms } => {
            collect_impls(scrutinee, impls);
            for arm in arms {
                collect_impls(&arm.body, impls);
            }
        }
        crate::core::ast::Term::View { scrutinee, arms } => {
            collect_impls(scrutinee, impls);
            for arm in arms {
                collect_impls(&arm.body, impls);
            }
        }
        crate::core::ast::Term::BinaryOp { left, right, .. } => {
            collect_impls(left, impls);
            collect_impls(right, impls);
        }
        crate::core::ast::Term::UnaryOp { arg, .. } => {
            collect_impls(arg, impls);
        }
        _ => {}
    }
}

pub fn load_module(path: &Path) -> Result<Module, ModuleError> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main")
        .to_string();

    let term = parse_module_file(path)?;
    compile_module(name, term)
}
