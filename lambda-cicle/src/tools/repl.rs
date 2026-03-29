use crate::core::parser::ParseError;
use crate::core::typecheck::TypeError;
use crate::runtime::evaluator::{Evaluator, SequentialEvaluator};
use crate::{parse, translate, type_check_with_borrow_check};

pub struct Repl {
    history: Vec<String>,
}

impl Repl {
    pub fn new() -> Self {
        Repl {
            history: Vec::new(),
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

    fn help(&self) -> String {
        r#"
Commands:
  :type <expr>   Show type of expression
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
        let term = parse(expr).map_err(ReplError::Parse)?;
        let ty = type_check_with_borrow_check(&term).map_err(ReplError::Type)?;

        let mut net = translate(&term);
        let evaluator = SequentialEvaluator::new();
        let result = evaluator.evaluate(&mut net).map_err(ReplError::Eval)?;

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
    let mut repl = Repl::new();
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
