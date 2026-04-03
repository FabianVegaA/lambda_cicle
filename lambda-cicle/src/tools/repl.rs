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
                    if let Some(s) = result {
                        println!("{}", s);
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }

        Ok(())
    }

    fn eval_line(&mut self, line: &str) -> Result<Option<String>, ReplError> {
        if line == ":quit" || line == ":q" {
            return Ok(Some("Goodbye!".to_string()));
        }

        if line == ":help" || line == ":h" {
            return Ok(Some(self.help()));
        }

        if line == ":debug" {
            return Ok(Some(self.toggle_debug(None)));
        }

        if line.starts_with(":debug ") {
            let arg = &line[7..].trim();
            return Ok(Some(self.toggle_debug(Some(arg))));
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
            return Ok(None);
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

    fn eval_expr(&self, expr: &str) -> Result<Option<String>, ReplError> {
        let decls_result = parse_program(expr);

        let (term, ty) = if let Ok(mut decls) = decls_result {
            if let Err(e) = inject_prelude(&mut decls) {
                eprintln!("Warning: Could not inject prelude: {}", e);
            }

            let registry = build_registry_from_decls(&decls);

            let elaborated = elaborate_declarations(&decls)
                .map_err(|e| ReplError::Parse(ParseError::SyntaxError(e.to_string())))?;

            let desugared = desugar_term(&elaborated, &registry);
            let ty = type_check_with_borrow_check(&desugared).map_err(ReplError::Type)?;

            (desugared, ty)
        } else {
            let term = parse(expr).map_err(ReplError::Parse)?;
            let desugared = desugar_term(&term, &self.registry);
            let ty = type_check_with_borrow_check(&desugared).map_err(ReplError::Type)?;
            (desugared, ty)
        };

        let mut net = translate(&term);
        let evaluator = SequentialEvaluator::new();
        let result = if let Some(level) = self.debug_level {
            if level > 0 {
                eprintln!("[debug] Evaluating expression with debug level {}", level);
            }
            evaluator.evaluate_with_debug(&mut net, level)
        } else {
            evaluator.evaluate(&mut net)
        }
        .map_err(ReplError::Eval)?;

        Ok(result.map(|r| format!("{} : {}", r, ty)))
    }

    fn typecheck_expr(&self, expr: &str) -> Result<Option<String>, ReplError> {
        let term = parse(expr).map_err(ReplError::Parse)?;
        let ty = type_check_with_borrow_check(&term).map_err(ReplError::Type)?;

        Ok(Some(format!("{}", ty)))
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
