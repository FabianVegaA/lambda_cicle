use clap::{Parser, Subcommand};
use lambda_cicle::runtime::evaluator::{Evaluator, SequentialEvaluator};
use lambda_cicle::tools::{net_to_dot, run_benchmark, run_repl, TraceDebugger};
use lambda_cicle::{parse, translate, type_check_with_borrow_check};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lambda-cicle")]
#[command(about = "λ◦ - A functional language with linear types and interaction nets", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive REPL
    Repl,

    /// Run a source file
    Run {
        /// Source file to run
        file: PathBuf,
    },

    /// Type check a file without running
    Check {
        /// Source file to check
        file: PathBuf,
    },

    /// Export net to DOT format for Graphviz
    Dot {
        /// Source file
        file: PathBuf,

        /// Output DOT file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show reduction trace
    Trace {
        /// Source file
        file: PathBuf,

        /// Maximum steps
        #[arg(short, long, default_value = "1000")]
        max_steps: usize,
    },

    /// Run benchmark
    Bench {
        /// Source file (optional, runs default benchmarks if not provided)
        file: Option<PathBuf>,

        /// Number of iterations
        #[arg(short, long, default_value = "100")]
        iterations: usize,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Repl => {
            run_repl()?;
        }

        Commands::Run { file } => {
            let source = std::fs::read_to_string(&file)?;
            let term = parse(&source)?;
            let _ty = type_check_with_borrow_check(&term)?;
            let mut net = translate(&term);

            let evaluator = SequentialEvaluator::new();
            match evaluator.evaluate(&mut net) {
                Ok(result) => {
                    if let Some(term) = result {
                        println!("{}", term);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Check { file } => {
            let source = std::fs::read_to_string(&file)?;
            let term = parse(&source)?;
            let ty = type_check_with_borrow_check(&term)?;
            println!("{} : {}", file.display(), ty);
        }

        Commands::Dot { file, output } => {
            let source = std::fs::read_to_string(&file)?;
            let term = parse(&source)?;
            let net = translate(&term);

            let dot = net_to_dot(&net);

            if let Some(output_path) = output {
                std::fs::write(&output_path, &dot)?;
                println!("DOT written to {}", output_path.display());
            } else {
                println!("{}", dot);
            }
        }

        Commands::Trace { file, max_steps } => {
            let source = std::fs::read_to_string(&file)?;
            let term = parse(&source)?;
            let net = translate(&term);

            let mut net_mut = net;
            let debugger = TraceDebugger::new().with_max_steps(max_steps);
            let trace = debugger.trace(&mut net_mut);
            debugger.print_trace(&trace);
        }

        Commands::Bench { file, iterations } => {
            if let Some(file) = file {
                let source = std::fs::read_to_string(&file)?;
                let result = run_benchmark(&file.display().to_string(), &source, iterations);
                println!("{}", result);
            } else {
                // Run default benchmarks
                lambda_cicle::tools::bench::run_default_benchmarks();
            }
        }
    }

    Ok(())
}
