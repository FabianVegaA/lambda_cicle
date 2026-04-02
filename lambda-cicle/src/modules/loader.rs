use super::{Exports, Module, ModuleError};
use crate::core::ast::Decl;
use crate::traits::Implementation;
use crate::{parse, parse_program, translate, type_check_with_borrow_check, Term};
use std::collections::{HashMap, HashSet};
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

#[derive(Debug, Clone)]
pub struct ImportGraph {
    edges: HashMap<String, Vec<String>>,
}

impl ImportGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, module: &str) {
        self.edges
            .entry(module.to_string())
            .or_insert_with(Vec::new);
    }

    pub fn add_import(&mut self, from: &str, to: &str) {
        self.edges
            .entry(from.to_string())
            .or_insert_with(Vec::new)
            .push(to.to_string());
    }

    pub fn get_imports(&self, module: &str) -> Vec<String> {
        self.edges.get(module).cloned().unwrap_or_default()
    }

    pub fn get_modules(&self) -> Vec<String> {
        self.edges.keys().cloned().collect()
    }
}

impl Default for ImportGraph {
    fn default() -> Self {
        Self::new()
    }
}

pub fn extract_imports(decls: &[Decl]) -> Vec<(String, String)> {
    let mut imports = Vec::new();
    let mut current_module = String::from("__main__");

    for decl in decls {
        match decl {
            Decl::UseDecl { path, .. } => {
                let target = path.join(".");
                imports.push((current_module.clone(), target));
            }
            Decl::TypeDecl { name, .. } => {
                current_module = name.clone();
            }
            _ => {}
        }
    }

    imports
}

#[derive(Debug, Clone)]
pub struct CycleInfo {
    pub modules: Vec<String>,
}

impl std::fmt::Display for CycleInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cycle detected: ")?;
        for (i, module) in self.modules.iter().enumerate() {
            if i > 0 {
                write!(f, " -> ")?;
            }
            write!(f, "{}", module)?;
        }
        Ok(())
    }
}

pub fn detect_cycles(graph: &ImportGraph) -> Option<CycleInfo> {
    let modules: Vec<String> = graph.get_modules();
    let mut visited: HashSet<String> = HashSet::new();
    let mut in_stack: HashSet<String> = HashSet::new();
    let mut path: Vec<String> = Vec::new();

    fn dfs(
        graph: &ImportGraph,
        node: &str,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<CycleInfo> {
        visited.insert(node.to_string());
        in_stack.insert(node.to_string());
        path.push(node.to_string());

        for neighbor in graph.get_imports(node) {
            if !visited.contains(&neighbor) {
                if let Some(cycle) = dfs(graph, &neighbor, visited, in_stack, path) {
                    return Some(cycle);
                }
            } else if in_stack.contains(&neighbor) {
                if let Some(pos) = path.iter().position(|m| m == &neighbor) {
                    let mut cycle_modules: Vec<String> = path[pos..].iter().cloned().collect();
                    cycle_modules.push(neighbor.clone());
                    return Some(CycleInfo {
                        modules: cycle_modules,
                    });
                }
            }
        }

        path.pop();
        in_stack.remove(node);
        None
    }

    for module in &modules {
        if !visited.contains(module) {
            if let Some(cycle) = dfs(graph, module, &mut visited, &mut in_stack, &mut path) {
                return Some(cycle);
            }
        }
    }

    None
}

pub fn verify_no_cycles(decls: &[Decl]) -> Result<(), ModuleError> {
    let imports = extract_imports(decls);

    let mut graph = ImportGraph::new();
    for (from, to) in &imports {
        graph.add_import(from, to);
    }

    if let Some(cycle) = detect_cycles(&graph) {
        return Err(ModuleError {
            message: format!("{}", cycle),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_imports_simple() {
        let source = r#"
use Std.List
use Std.Map
"#;
        let decls = parse_program(source).unwrap();
        let imports = extract_imports(&decls);
        assert_eq!(imports.len(), 2);
        assert!(imports.contains(&("__main__".to_string(), "Std.List".to_string())));
        assert!(imports.contains(&("__main__".to_string(), "Std.Map".to_string())));
    }

    #[test]
    fn test_detect_simple_cycle() {
        let mut graph = ImportGraph::new();
        graph.add_module("A");
        graph.add_module("B");
        graph.add_import("A", "B");
        graph.add_import("B", "A");

        let cycle = detect_cycles(&graph);
        assert!(cycle.is_some());
        let cycle = cycle.unwrap();
        assert!(cycle.modules.contains(&"A".to_string()));
        assert!(cycle.modules.contains(&"B".to_string()));
    }

    #[test]
    fn test_detect_three_way_cycle() {
        let mut graph = ImportGraph::new();
        graph.add_module("A");
        graph.add_module("B");
        graph.add_module("C");
        graph.add_import("A", "B");
        graph.add_import("B", "C");
        graph.add_import("C", "A");

        let cycle = detect_cycles(&graph);
        assert!(cycle.is_some());
    }

    #[test]
    fn test_diamond_dependency_no_cycle() {
        let mut graph = ImportGraph::new();
        graph.add_module("A");
        graph.add_module("B");
        graph.add_module("C");
        graph.add_module("D");
        graph.add_import("A", "B");
        graph.add_import("A", "C");
        graph.add_import("B", "D");
        graph.add_import("C", "D");

        let cycle = detect_cycles(&graph);
        assert!(cycle.is_none());
    }

    #[test]
    fn test_self_import() {
        let mut graph = ImportGraph::new();
        graph.add_module("A");
        graph.add_import("A", "A");

        let cycle = detect_cycles(&graph);
        assert!(cycle.is_some());
    }
}
