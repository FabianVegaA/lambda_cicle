use lambda_cicle::core::parser::parse_program;
use lambda_cicle::modules::loader::{
    detect_cycles, extract_imports, verify_no_cycles, ImportGraph,
};

#[test]
fn test_parse_use_statements() {
    let source = r#"
use Std.Prelude
use Std.List
"#;
    let decls = parse_program(source).unwrap();
    let imports = extract_imports(&decls);

    assert_eq!(imports.len(), 2);
    assert!(imports.contains(&("__main__".to_string(), "Std.Prelude".to_string())));
    assert!(imports.contains(&("__main__".to_string(), "Std.List".to_string())));
}

#[test]
fn test_verify_no_cycles_with_valid_imports() {
    let source = r#"
use Std.Prelude
use Std.List
"#;
    let decls = parse_program(source).unwrap();
    let result = verify_no_cycles(&decls);
    assert!(result.is_ok());
}

#[test]
fn test_verify_detects_cycle() {
    let source = r#"
use A
"#;
    let decls = parse_program(source).unwrap();
    let imports = extract_imports(&decls);

    let mut graph = ImportGraph::new();
    for (from, to) in imports {
        graph.add_import(&from, &to);
    }
    graph.add_module("A");
    graph.add_import("A", "A");

    let cycle = detect_cycles(&graph);
    assert!(cycle.is_some());
}

#[test]
fn test_import_graph_operations() {
    let mut graph = ImportGraph::new();
    graph.add_module("A");
    graph.add_module("B");
    graph.add_import("A", "B");

    let mut modules = graph.get_modules();
    modules.sort();
    assert_eq!(modules, vec!["A", "B"]);
    assert_eq!(graph.get_imports("A"), vec!["B"]);
    assert_eq!(graph.get_imports("B"), Vec::<String>::new());
}

#[test]
fn test_diamond_dependency_is_valid() {
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
fn test_simple_chain_is_valid() {
    let mut graph = ImportGraph::new();
    graph.add_module("A");
    graph.add_module("B");
    graph.add_module("C");
    graph.add_import("A", "B");
    graph.add_import("B", "C");

    let cycle = detect_cycles(&graph);
    assert!(cycle.is_none());
}

#[test]
fn test_three_way_cycle_detected() {
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
fn test_cycle_info_display() {
    use lambda_cicle::modules::loader::CycleInfo;

    let cycle = CycleInfo {
        modules: vec!["A".to_string(), "B".to_string(), "C".to_string()],
    };
    let display = format!("{}", cycle);
    assert!(display.contains("Cycle detected"));
    assert!(display.contains("A -> B -> C"));
}
