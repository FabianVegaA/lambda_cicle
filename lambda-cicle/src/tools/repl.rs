use crate::core::ast::Decl;
use crate::core::parser::ParseError;
use crate::core::typecheck::TypeError;
use crate::runtime::evaluator::{Evaluator, SequentialEvaluator};
use crate::{
    build_registry_from_decls, desugar_term, elaborate_declarations, inject_prelude, parse,
    parse_program, translate, type_check_with_borrow_check,
};

pub struct Repl {
    history: Vec<String>,
    prelude_decls: Vec<crate::core::ast::Decl>,
    registry: crate::traits::registry::Registry,
    debug_level: Option<u8>,
}

impl Repl {
    pub fn new() -> Self {
        Self::with_debug_level(None)
    }

    pub fn with_debug_level(debug_level: Option<u8>) -> Self {
        let prelude_source = include_str!("../../stdlib/Prelude.λ");
        let prelude_decls = parse_program(prelude_source).unwrap_or_default();
        let registry = build_registry_from_decls(&prelude_decls);

        Repl {
            history: Vec::new(),
            prelude_decls,
            registry,
            debug_level,
        }
    }

    pub fn run(&mut self) -> Result<(), ReplError> {
        println!("λ◦ (lambda-circle) v0.1.0");
        println!("Type :help for commands, :quit to exit\n");

        loop {
            print!("λ> ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            let mut line = String::new();
            if std::io::stdin().read_line(&mut line).unwrap() == 0 {
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            self.history.push(line.to_string());

            match self.eval_line(line) {
                Ok(result) => {
                    println!("{}", result);
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }

        Ok(())
    }

    fn eval_line(&mut self, line: &str) -> Result<String, ReplError> {
        if line == ":quit" || line == ":q" {
            return Ok("Goodbye!".to_string());
        }

        if line == ":help" || line == ":h" {
            return Ok(self.help());
        }

        if line == ":debug" {
            return Ok(self.toggle_debug(None));
        }

        if line.starts_with(":debug ") {
            let arg = &line[7..].trim();
            return Ok(self.toggle_debug(Some(arg)));
        }

        if line == ":type" {
            return Err(ReplError::Usage("Usage: :type <expression>".to_string()));
        }

        if line.starts_with(":type ") {
            let expr = &line[6..];
            return self.typecheck_expr(expr);
        }

        if line == ":clear" {
            print!("\x1B[2J\x1B[1J");
            return Ok("".to_string()); // TODO: Implement clear
        }

        self.eval_expr(line)
    }

    fn toggle_debug(&mut self, arg: Option<&str>) -> String {
        match arg {
            Some("off") | Some("0") => {
                self.debug_level = None;
                "Debug mode: off".to_string()
            }
            Some("1") | Some("2") | Some("3") => {
                let level = arg.unwrap().parse().unwrap();
                self.debug_level = Some(level);
                format!("Debug mode: level {}", level)
            }
            None => {
                if self.debug_level.is_some() {
                    self.debug_level = None;
                    "Debug mode: off".to_string()
                } else {
                    self.debug_level = Some(1);
                    "Debug mode: on (level 1)".to_string()
                }
            }
            _ => "Usage: :debug [1|2|3|off]".to_string(),
        }
    }

    fn help(&self) -> String {
        r#"
Commands:
  :type <expr>   Show type of expression
  :debug [1|2|3] Toggle or set debug level (1=steps, 2=+nodes, 3=+net)
  :load <file>   Load and run file (not implemented)
  :clear         Clear screen
  :help, :h     Show this help
  :quit, :q     Exit REPL

Examples:
  42                           -- evaluate integer
  \x : Int. x + 1            -- lambda expression
  let x = 5 in x * 2        -- let binding
"#
        .to_string()
    }

    fn eval_expr(&self, expr: &str) -> Result<String, ReplError> {
        // Try parsing as program (declarations)
        let decls_result = parse_program(expr);

        // Try to get declarations even if it's just an expression (for use statements)
        let mut all_decls = decls_result.unwrap_or_default();

        // Try parsing as expression for non-program input
        let parse_expr = parse(expr);

        let (term, _) = if !all_decls.is_empty() {
            // We have declarations (possibly including use statements)
            if let Err(e) = inject_prelude(&mut all_decls) {
                eprintln!("Warning: Could not inject prelude: {}", e);
            }

            eprintln!("DEBUG: Total declarations: {}", all_decls.len());

            // Count impl declarations
            let impl_count = all_decls
                .iter()
                .filter(|d| matches!(d, Decl::ImplDecl { .. }))
                .count();
            eprintln!("DEBUG: Impl declarations: {}", impl_count);

            let registry = build_registry_from_decls(&all_decls);

            // Debug: list eq implementations
            eprintln!("DEBUG: eq implementations in registry:");
            for (trait_name, for_type, _) in registry.iter() {
                if trait_name.0 == "Eq" {
                    eprintln!("DEBUG:   - {:?}", for_type);
                }
            }

            let elaborated = elaborate_declarations(&all_decls)
                .map_err(|e| ReplError::Parse(ParseError::SyntaxError(e.to_string())))?;

            let desugared = desugar_term(&elaborated, &registry);
            let ty = type_check_with_borrow_check(&desugared).map_err(ReplError::Type)?;

            (desugared, ty)
        } else if let Ok(term) = parse_expr {
            // Pure expression - still need prelude for trait methods
            let mut decls = Vec::new();
            if let Err(e) = inject_prelude(&mut decls) {
                eprintln!("Error injecting prelude: {}", e.message);
            }
            let registry = build_registry_from_decls(&decls);

            let desugared = desugar_term(&term, &registry);
            let ty = type_check_with_borrow_check(&desugared).map_err(ReplError::Type)?;
            (desugared, ty)
        } else {
            return Err(ReplError::Parse(ParseError::UnexpectedToken(
                "Could not parse".to_string(),
            )));
        };

        let mut net = translate(&term);
        let evaluator = SequentialEvaluator::new();

        if let Some(level) = self.debug_level {
            if level > 0 {
                eprintln!("[debug] Evaluating expression with debug level {}", level);
            }
            return evaluator
                .evaluate_with_debug(&mut net, level)
                .map_err(ReplError::Eval)
                .map(|r| format!("{}", r));
        }
        return evaluator
            .evaluate(&mut net)
            .map_err(ReplError::Eval)
            .map(|r| format!("{}", r));
    }

    fn typecheck_expr(&self, expr: &str) -> Result<String, ReplError> {
        let term = parse(expr).map_err(ReplError::Parse)?;
        let ty = type_check_with_borrow_check(&term).map_err(ReplError::Type)?;

        Ok(format!("{}", ty))
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run_repl() -> Result<(), ReplError> {
    run_repl_with_debug(None)
}

pub fn run_repl_with_debug(debug_level: Option<u8>) -> Result<(), ReplError> {
    let mut repl = Repl::with_debug_level(debug_level);
    repl.run()
}

#[derive(Debug)]
pub enum ReplError {
    Parse(ParseError),
    Type(TypeError),
    Eval(crate::runtime::evaluator::EvalError),
    Usage(String),
}

impl std::fmt::Display for ReplError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplError::Parse(e) => write!(f, "Parse error: {}", e),
            ReplError::Type(e) => write!(f, "Type error: {}", e),
            ReplError::Eval(e) => write!(f, "Evaluation error: {:?}", e),
            ReplError::Usage(s) => write!(f, "{}", s),
        }
    }
}

impl From<ParseError> for ReplError {
    fn from(e: ParseError) -> Self {
        ReplError::Parse(e)
    }
}

impl From<TypeError> for ReplError {
    fn from(e: TypeError) -> Self {
        ReplError::Type(e)
    }
}

impl std::error::Error for ReplError {}
