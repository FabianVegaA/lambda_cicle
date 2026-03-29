use crate::tools::{compile_source, run_source};
use std::time::{Duration, Instant};

pub struct Benchmark {
    name: String,
    source: String,
}

impl Benchmark {
    pub fn new(name: &str, source: &str) -> Self {
        Benchmark {
            name: name.to_string(),
            source: source.to_string(),
        }
    }

    pub fn run(&self, iterations: usize) -> BenchmarkResult {
        let mut total_time = Duration::ZERO;
        let mut compile_time = Duration::ZERO;
        let mut eval_time = Duration::ZERO;
        let mut errors = 0;

        for _ in 0..iterations {
            let start = Instant::now();

            match compile_source(&self.source) {
                Ok((_, _, _)) => {
                    compile_time += start.elapsed();
                }
                Err(_) => {
                    errors += 1;
                    continue;
                }
            }

            let eval_start = Instant::now();

            match run_source(&self.source) {
                Ok(_) => {
                    eval_time += eval_start.elapsed();
                }
                Err(_) => {
                    errors += 1;
                }
            }

            total_time += start.elapsed();
        }

        BenchmarkResult {
            name: self.name.clone(),
            iterations,
            total_time,
            compile_time,
            eval_time,
            errors,
            avg_time: total_time / iterations as u32,
        }
    }
}

pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub compile_time: Duration,
    pub eval_time: Duration,
    pub errors: usize,
    pub avg_time: Duration,
}

impl std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} iterations in {:?} (avg: {:?}, compile: {:?}, eval: {:?}, errors: {})",
            self.name,
            self.iterations,
            self.total_time,
            self.avg_time,
            self.compile_time,
            self.eval_time,
            self.errors
        )
    }
}

pub fn run_benchmark(name: &str, source: &str, iterations: usize) -> BenchmarkResult {
    let bench = Benchmark::new(name, source);
    bench.run(iterations)
}

pub fn run_default_benchmarks() {
    println!("Running default benchmarks...\n");

    let benchmarks = vec![
        Benchmark::new("simple_arith", "1 + 2 * 3"),
        Benchmark::new("lambda_apply", "(\\x : Int. x + 1) 5"),
        Benchmark::new("let_binding", "let x = 10 in x + x"),
        Benchmark::new("nested_app", "(\\x. \\y. x + y) ((\\z. z) 1) 2"),
    ];

    for bench in benchmarks {
        let result = bench.run(100);
        println!("{}", result);
    }
}
