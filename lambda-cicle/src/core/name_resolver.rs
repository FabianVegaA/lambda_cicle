use crate::core::ast::{Decl, UseMode, Visibility};
use crate::modules::Exports;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct Import {
    pub path: Vec<String>,
    pub mode: UseMode,
    pub alias: Option<String>,
}

impl Import {
    pub fn module_name(&self) -> String {
        self.path.join(".")
    }

    pub fn qualifier(&self) -> String {
        if let Some(ref alias) = self.alias {
            return alias.clone();
        }
        self.path
            .last()
            .cloned()
            .unwrap_or_else(|| self.module_name())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ImportGraph {
    nodes: HashSet<String>,
    edges: HashMap<String, Vec<String>>,
}

impl ImportGraph {
    pub fn new() -> Self {
        ImportGraph {
            nodes: HashSet::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, name: &str) {
        self.nodes.insert(name.to_string());
        self.edges.entry(name.to_string()).or_default();
    }

    pub fn add_dependency(&mut self, from: &str, to: &str) {
        self.nodes.insert(from.to_string());
        self.nodes.insert(to.to_string());
        self.edges
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
    }

    pub fn get_dependencies(&self, module: &str) -> Vec<String> {
        self.edges.get(module).cloned().unwrap_or_default()
    }

    pub fn modules(&self) -> impl Iterator<Item = &String> {
        self.nodes.iter()
    }

    pub fn detect_cycle(&self) -> Option<Vec<String>> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();
        let mut path: Vec<String> = Vec::new();

        for module in &self.nodes {
            if !visited.contains(module) {
                if let Some(cycle) =
                    self.detect_cycle_from(module, &mut visited, &mut rec_stack, &mut path)
                {
                    return Some(cycle);
                }
            }
        }

        None
    }

    fn detect_cycle_from(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(deps) = self.edges.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    if let Some(cycle) = self.detect_cycle_from(dep, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dep) {
                    let mut result = Vec::new();
                    let mut found = false;
                    for item in path.iter() {
                        if item == dep {
                            found = true;
                        }
                        if found {
                            result.push(item.clone());
                        }
                    }
                    result.push(dep.clone());
                    return Some(result);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
        None
    }

    pub fn topological_sort(&self) -> Result<Vec<String>, CycleError> {
        let mut out_degree: HashMap<String, usize> = HashMap::new();
        for node in &self.nodes {
            out_degree.insert(node.clone(), 0);
        }
        for (node, deps) in &self.edges {
            out_degree.insert(node.clone(), deps.len());
        }

        let mut queue: VecDeque<String> = out_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            for (other, deps) in &self.edges {
                if deps.contains(&node) {
                    if let Some(deg) = out_degree.get_mut(other) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(other.clone());
                        }
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            if let Some(cycle) = self.detect_cycle() {
                return Err(CycleError { modules: cycle });
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct CycleError {
    pub modules: Vec<String>,
}

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cycle detected: {}", self.modules.join(" -> "))
    }
}

impl std::error::Error for CycleError {}

#[derive(Debug, Clone)]
pub enum NameResolutionError {
    ModuleNotFound(String),
    NameNotFound { module: String, name: String },
    CycleDetected(Vec<String>),
}

impl std::fmt::Display for NameResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameResolutionError::ModuleNotFound(name) => {
                write!(f, "Module not found: {}", name)
            }
            NameResolutionError::NameNotFound { module, name } => {
                write!(f, "Name '{}' not found in module '{}'", name, module)
            }
            NameResolutionError::CycleDetected(modules) => {
                write!(f, "Cyclic dependency: {}", modules.join(" -> "))
            }
        }
    }
}

impl std::error::Error for NameResolutionError {}

impl From<CycleError> for NameResolutionError {
    fn from(e: CycleError) -> Self {
        NameResolutionError::CycleDetected(e.modules)
    }
}

pub fn build_import_graph(module_name: &str, decls: &[Decl]) -> (ImportGraph, Vec<Import>) {
    let mut graph = ImportGraph::new();
    graph.add_module(module_name);

    let mut imports = Vec::new();

    for decl in decls {
        if let Decl::UseDecl { path, mode } = decl {
            let import = Import {
                path: path.clone(),
                mode: mode.clone(),
                alias: extract_alias(mode),
            };
            let target_module = import.module_name();
            graph.add_dependency(module_name, &target_module);
            imports.push(import);
        }
    }

    (graph, imports)
}

fn extract_alias(mode: &UseMode) -> Option<String> {
    match mode {
        UseMode::Aliased(name) => Some(name.clone()),
        _ => None,
    }
}

pub fn has_no_prelude(decls: &[Decl]) -> bool {
    decls.iter().any(|d| matches!(d, Decl::NoPrelude))
}

pub fn check_cycles(graph: &ImportGraph) -> Result<(), NameResolutionError> {
    if let Some(cycle) = graph.detect_cycle() {
        return Err(NameResolutionError::CycleDetected(cycle));
    }
    Ok(())
}

pub fn resolve_qualified_name(
    module: &str,
    name: &str,
    exports: &HashMap<String, Exports>,
) -> Result<Option<crate::core::ast::Type>, NameResolutionError> {
    let module_exports = exports
        .get(module)
        .ok_or_else(|| NameResolutionError::ModuleNotFound(module.to_string()))?;

    if let Some(entry) = module_exports.get_value(name) {
        return Ok(Some(entry.ty.clone()));
    }

    if let Some(entry) = module_exports.get_type(name) {
        return Ok(Some(entry.ty.clone()));
    }

    Err(NameResolutionError::NameNotFound {
        module: module.to_string(),
        name: name.to_string(),
    })
}

pub fn resolve_selective_import(
    module: &str,
    items: &[String],
    exports: &HashMap<String, Exports>,
) -> Result<HashMap<String, crate::core::ast::Type>, NameResolutionError> {
    let module_exports = exports
        .get(module)
        .ok_or_else(|| NameResolutionError::ModuleNotFound(module.to_string()))?;

    let mut resolved = HashMap::new();

    for item in items {
        if let Some(entry) = module_exports.get_value(item) {
            resolved.insert(item.clone(), entry.ty.clone());
        } else if let Some(entry) = module_exports.get_type(item) {
            resolved.insert(item.clone(), entry.ty.clone());
        } else {
            return Err(NameResolutionError::NameNotFound {
                module: module.to_string(),
                name: item.clone(),
            });
        }
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_graph_add() {
        let mut graph = ImportGraph::new();
        graph.add_module("Main");
        graph.add_dependency("Main", "Std");

        assert!(graph.nodes.contains("Main"));
        assert!(graph.nodes.contains("Std"));
        assert_eq!(graph.get_dependencies("Main"), vec!["Std"]);
    }

    #[test]
    fn test_cycle_detection_simple() {
        let mut graph = ImportGraph::new();
        graph.add_module("A");
        graph.add_module("B");
        graph.add_dependency("A", "B");
        graph.add_dependency("B", "A");

        let cycle = graph.detect_cycle();
        assert!(cycle.is_some());
    }

    #[test]
    fn test_no_cycle() {
        let mut graph = ImportGraph::new();
        graph.add_module("Main");
        graph.add_module("Std");
        graph.add_module("List");
        graph.add_dependency("Main", "Std");
        graph.add_dependency("Main", "List");
        graph.add_dependency("List", "Std");

        assert!(graph.detect_cycle().is_none());
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = ImportGraph::new();
        graph.add_module("Main");
        graph.add_module("Std");
        graph.add_module("List");
        graph.add_dependency("Main", "Std");
        graph.add_dependency("Main", "List");
        graph.add_dependency("List", "Std");

        let sorted = graph.topological_sort();
        assert!(sorted.is_ok());
        let order = sorted.unwrap();
        let std_idx = order.iter().position(|m| m == "Std").unwrap();
        let list_idx = order.iter().position(|m| m == "List").unwrap();
        let main_idx = order.iter().position(|m| m == "Main").unwrap();

        assert!(std_idx < list_idx);
        assert!(list_idx < main_idx);
    }

    #[test]
    fn test_prelude_detection() {
        let decls_no_prelude = vec![Decl::NoPrelude];
        assert!(has_no_prelude(&decls_no_prelude));

        let decls_with_prelude = vec![Decl::ValDecl {
            visibility: Visibility::Private,
            name: "x".to_string(),
            ty: crate::core::ast::Type::int(),
            term: Box::new(crate::core::ast::Term::NativeLiteral(
                crate::core::ast::Literal::Int(1),
            )),
        }];
        assert!(!has_no_prelude(&decls_with_prelude));
    }
}
