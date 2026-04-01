use lambda_cicle::core::parser::parse_program;
use lambda_cicle::modules::loader::inject_prelude;
use std::path::PathBuf;

fn get_example_path(name: &str) -> PathBuf {
    PathBuf::from("examples").join(name)
}

fn parse_example_with_prelude(name: &str) -> Result<(), String> {
    let path = get_example_path(name);
    let source = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let mut decls =
        parse_program(&source).map_err(|e| format!("Failed to parse {}: {}", name, e))?;

    inject_prelude(&mut decls)
        .map_err(|e| format!("Failed to inject prelude for {}: {}", name, e))?;

    Ok(())
}

#[test]
fn test_parse_list_examples() {
    let result = parse_example_with_prelude("list_examples.λ");
    assert!(
        result.is_ok(),
        "list_examples.λ should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_map_examples() {
    let result = parse_example_with_prelude("map_examples.λ");
    assert!(
        result.is_ok(),
        "map_examples.λ should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_constructor_patterns() {
    let result = parse_example_with_prelude("constructor_patterns.λ");
    assert!(
        result.is_ok(),
        "constructor_patterns.λ should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_traits_examples() {
    let result = parse_example_with_prelude("traits_examples.λ");
    assert!(
        result.is_ok(),
        "traits_examples.λ should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_recursive() {
    let result = parse_example_with_prelude("recursive.λ");
    assert!(
        result.is_ok(),
        "recursive.λ should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_io_examples() {
    let result = parse_example_with_prelude("io_examples.λ");
    assert!(
        result.is_ok(),
        "io_examples.λ should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_examples_have_main_expression() {
    let examples = [
        "list_examples.λ",
        "map_examples.λ",
        "constructor_patterns.λ",
        "traits_examples.λ",
        "recursive.λ",
        "io_examples.λ",
    ];

    for example in examples {
        let path = get_example_path(example);
        let source = std::fs::read_to_string(&path).expect(&format!("Failed to read {}", example));

        // Check that the main expression (42) is present
        assert!(
            source
                .lines()
                .last()
                .map(|l| l.trim())
                .unwrap_or("")
                .ends_with("42"),
            "{} should have 42 as main expression",
            example
        );
    }
}

#[test]
fn test_example_files_exist() {
    let examples = [
        "list_examples.λ",
        "map_examples.λ",
        "constructor_patterns.λ",
        "traits_examples.λ",
        "recursive.λ",
        "io_examples.λ",
        "hello.λ",
        "fib.λ",
        "lambda.λ",
        "let.λ",
        "nested.λ",
    ];

    for example in examples {
        let path = get_example_path(example);
        assert!(path.exists(), "{} should exist", path.display());
    }
}
