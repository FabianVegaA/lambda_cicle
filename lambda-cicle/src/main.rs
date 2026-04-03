use clap::{Parser, Subcommand};
use lambda_cicle::core::ast::Decl;
use lambda_cicle::modules::{
    elaborate_declarations, inject_prelude as lc_inject_prelude, Exports, Module,
};
use lambda_cicle::runtime::evaluator::{Evaluator, SequentialEvaluator};
use lambda_cicle::tools::{net_to_dot, run_benchmark, run_repl_with_debug, TraceDebugger};
use lambda_cicle::{
    build_registry_from_decls, desugar_term, parse, parse_program, translate,
    type_check_with_borrow_check, Term,
};
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
    Repl {
        /// Enable debug mode with specified level (1-3)
        #[arg(short, long)]
        debug: Option<u8>,
    },

    /// Run a source file
    Run {
        /// Source file to run
        file: PathBuf,

        /// Show parsed AST without executing
        #[arg(long)]
        grammar: bool,
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

    /// Compile a .λ file to .λo object file
    Build {
        /// Source file to compile
        file: PathBuf,

        /// Output file (default: <input>.o)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Link .λo files into an executable
    Link {
        /// Object files to link
        files: Vec<PathBuf>,

        /// Output executable file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Remove build artifacts
    Clean {
        /// Directory to clean (default: current directory)
        #[arg(default_value = ".")]
        directory: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Repl { debug } => {
            run_repl_with_debug(debug)?;
        }

        Commands::Run { file, grammar } => {
            let source = std::fs::read_to_string(&file)?;

            // Try parsing as declarations first
            let decls_result = parse_program(&source);

            let term: Term;
            let ty: lambda_cicle::Type;

            if let Ok(mut decls) = decls_result {
                // Inject prelude if not opted out
                if let Err(e) = lc_inject_prelude(&mut decls) {
                    eprintln!("Warning: Could not load prelude: {}", e);
                }

                // Build the trait registry from declarations (includes prelude)
                let registry = build_registry_from_decls(&decls);

                // If --grammar flag, show AST and exit
                if grammar {
                    for decl in &decls {
                        println!("{:#?}", decl);
                    }
                    return Ok(());
                }

                // Elaborate all declarations into a single executable term
                match elaborate_declarations(&decls) {
                    Ok(elaborated_term) => {
                        // Desugar trait method calls to primitive calls first
                        let desugared_term = desugar_term(&elaborated_term, &registry);
                        // Then type check the desugared term
                        let ty = type_check_with_borrow_check(&desugared_term)?;
                        term = desugared_term;
                    }
                    Err(e) => {
                        eprintln!("Warning: Could not elaborate declarations: {}", e);
                        eprintln!("Falling back to expression parsing...");
                        // Fall back to parsing as expression
                        term = parse(&source)?;
                        ty = type_check_with_borrow_check(&term)?;
                    }
                }
            } else {
                // Fall back to parsing as expression
                term = parse(&source)?;
                ty = type_check_with_borrow_check(&term)?;

                if grammar {
                    println!("{:#?}", term);
                    return Ok(());
                }
            }

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

        Commands::Build { file, output } => {
            let source = std::fs::read_to_string(&file)?;

            // Try parsing as declarations first
            let decls_result = parse_program(&source);

            let term: Term;
            let _ty: lambda_cicle::Type;

            if let Ok(mut decls) = decls_result {
                // Inject prelude if not opted out
                if let Err(e) = lc_inject_prelude(&mut decls) {
                    eprintln!("Warning: Could not load prelude: {}", e);
                }

                // Find the main entry point
                let main_decl = decls.iter().find_map(|d| {
                    if let Decl::ValDecl {
                        name,
                        term: main_term,
                        ..
                    } = d
                    {
                        if name == "main" {
                            return Some((**main_term).clone());
                        }
                    }
                    None
                });

                term = main_decl
                    .unwrap_or_else(|| Term::NativeLiteral(lambda_cicle::core::ast::Literal::Unit));
                _ty = type_check_with_borrow_check(&term)?;
            } else {
                // Fall back to parsing as expression
                term = parse(&source)?;
                _ty = type_check_with_borrow_check(&term)?;
            }

            let net = translate(&term);

            let module = Module {
                name: file
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                exports: Exports::from_term(&term, _ty.clone()),
                impls: Vec::new(),
                net,
            };

            let output_path = output.unwrap_or_else(|| {
                let mut p = file.clone();
                p.set_extension("o");
                p
            });

            lambda_cicle::modules::serialize_module(&module)
                .and_then(|data| {
                    std::fs::write(&output_path, data)?;
                    Ok(())
                })
                .map_err(|e| format!("Build failed: {}", e))?;

            println!("Compiled {} -> {}", file.display(), output_path.display());
        }

        Commands::Link { files, output } => {
            let object_files: Vec<PathBuf> = files;

            lambda_cicle::modules::link(&object_files, &output)
                .map_err(|e| format!("Link failed: {}", e))?;

            println!(
                "Linked {} files -> {}",
                object_files.len(),
                output.display()
            );
        }

        Commands::Clean { directory } => {
            let dir_path = &directory;

            for entry in std::fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "o" {
                        std::fs::remove_file(&path)?;
                        println!("Removed {}", path.display());
                    }
                }
            }

            println!("Cleaned directory {}", directory.display());
        }
    }

    Ok(())
}
