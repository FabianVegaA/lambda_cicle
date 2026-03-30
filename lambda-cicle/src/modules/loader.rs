use super::{Exports, Module, ModuleError};
use crate::core::ast::Decl;
use crate::traits::Implementation;
use crate::{parse, parse_program, translate, type_check_with_borrow_check, Term};
use std::path::Path;
use std::sync::OnceLock;

static PRELUDE_DECLS: OnceLock<Vec<Decl>> = OnceLock::new();

fn find_stdlib_path() -> Option<std::path::PathBuf> {
    // Try relative to current working directory first
    let relative = std::path::Path::new("stdlib/Prelude.λ");
    if relative.exists() {
        return Some(relative.to_path_buf());
    }

    // Try relative to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let relative_to_exe = exe_dir.join("stdlib/Prelude.λ");
            if relative_to_exe.exists() {
                return Some(relative_to_exe);
            }
        }
    }

    // Try stdlib in current directory
    let stdlib_path = std::path::Path::new("stdlib");
    if stdlib_path.join("Prelude.λ").exists() {
        return Some(stdlib_path.join("Prelude.λ"));
    }

    None
}

pub fn load_prelude() -> Result<&'static Vec<Decl>, ModuleError> {
    if let Some(decls) = PRELUDE_DECLS.get() {
        return Ok(decls);
    }

    let prelude_path = find_stdlib_path().ok_or_else(|| ModuleError {
        message: "Could not find stdlib/Prelude.λ".to_string(),
    })?;

    let decls = parse_module_decls(&prelude_path)?;

    PRELUDE_DECLS.set(decls).ok();

    Ok(PRELUDE_DECLS.get().unwrap())
}

pub fn inject_prelude(decls: &mut Vec<Decl>) -> Result<(), ModuleError> {
    // Check if user opted out of prelude
    if decls.iter().any(|d| matches!(d, Decl::NoPrelude)) {
        return Ok(());
    }

    // Load prelude
    let prelude = load_prelude()?;

    // Prepend prelude declarations
    let mut combined = prelude.clone();
    combined.append(decls);
    *decls = combined;

    Ok(())
}

pub fn parse_module_file(path: &Path) -> Result<Term, ModuleError> {
    let source = std::fs::read_to_string(path).map_err(|e| ModuleError {
        message: format!("Failed to read file: {}", e),
    })?;

    let term = parse(&source)?;
    Ok(term)
}

pub fn parse_module_decls(path: &Path) -> Result<Vec<Decl>, ModuleError> {
    let source = std::fs::read_to_string(path).map_err(|e| ModuleError {
        message: format!("Failed to read file: {}", e),
    })?;

    let decls = parse_program(&source)?;
    Ok(decls)
}

pub fn module_name_from_path(path: &Path) -> String {
    let mut components = Vec::new();

    if let Some(parent) = path.parent() {
        for part in parent.iter() {
            if let Some(s) = part.to_str() {
                if s != "src" && s != "." {
                    components.push(s);
                }
            }
        }
    }

    if let Some(stem) = path.file_stem() {
        if let Some(s) = stem.to_str() {
            components.push(s);
        }
    }

    if components.is_empty() {
        "Main".to_string()
    } else {
        components.join(".")
    }
}

pub fn validate_module_name(path: &Path, expected_name: &str) -> Result<(), ModuleError> {
    let actual_name = module_name_from_path(path);

    if actual_name != expected_name {
        return Err(ModuleError {
            message: format!(
                "Module name mismatch: file '{}' defines module '{}' but expected '{}'",
                path.display(),
                actual_name,
                expected_name
            ),
        });
    }

    Ok(())
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

pub fn compile_module_with_decls(
    path: &Path,
    name: String,
    decls: Vec<Decl>,
) -> Result<Module, ModuleError> {
    validate_module_name(path, &name)?;

    let exports = Exports::from_decl(&decls);

    Ok(Module {
        name,
        exports,
        impls: Vec::new(),
        net: crate::runtime::net::Net::new(),
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
