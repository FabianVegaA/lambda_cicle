use lambda_cicle::core::ast::Decl;
use lambda_cicle::modules::loader::{inject_prelude, load_prelude};

#[test]
fn test_load_prelude_returns_decls() {
    let result = load_prelude();
    if let Err(e) = &result {
        eprintln!("Prelude parse error: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "load_prelude should succeed: {:?}",
        result.err()
    );

    let decls = result.unwrap();
    assert!(!decls.is_empty(), "Prelude should not be empty");
}

#[test]
fn test_load_prelude_contains_type_decls() {
    let prelude = load_prelude().unwrap();

    let has_bool = prelude
        .iter()
        .any(|d| matches!(d, Decl::TypeDecl { name, .. } if name == "Bool"));
    assert!(has_bool, "Prelude should contain Bool type");

    let has_int = prelude
        .iter()
        .any(|d| matches!(d, Decl::TypeDecl { name, .. } if name == "Int"));
    assert!(has_int, "Prelude should contain Int type");
}

#[test]
fn test_load_prelude_contains_trait_decls() {
    let prelude = load_prelude().unwrap();

    let has_eq = prelude
        .iter()
        .any(|d| matches!(d, Decl::TraitDecl { name, .. } if name == "Eq"));
    assert!(has_eq, "Prelude should contain Eq trait");

    let has_ord = prelude
        .iter()
        .any(|d| matches!(d, Decl::TraitDecl { name, .. } if name == "Ord"));
    assert!(has_ord, "Prelude should contain Ord trait");
}

#[test]
fn test_inject_prelude_adds_decls() {
    let mut decls: Vec<Decl> = Vec::new();

    let result = inject_prelude(&mut decls);
    assert!(result.is_ok(), "inject_prelude should succeed");
    assert!(!decls.is_empty(), "Declarations should be added");
}

#[test]
fn test_prelude_injected_before_user_decls() {
    let prelude = load_prelude().unwrap();
    let prelude_len = prelude.len();

    let mut user_decls: Vec<Decl> = Vec::new();
    inject_prelude(&mut user_decls).unwrap();

    assert_eq!(
        user_decls.len(),
        prelude_len,
        "User decls should only contain prelude (no user decls added)"
    );
}

#[test]
fn test_load_prelude_multiple_times_returns_same() {
    let first = load_prelude().unwrap();
    let second = load_prelude().unwrap();

    assert_eq!(
        first.len(),
        second.len(),
        "Multiple loads should return same number of declarations"
    );
}
