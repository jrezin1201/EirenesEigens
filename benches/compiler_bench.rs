// Benchmark suite for RavensOne compiler performance
// Measures compilation speed, memory usage, and throughput

use ravensone_compiler::{Compiler, BuildTarget};
use std::time::{Duration, Instant};

/// Sample RavensOne programs for benchmarking
mod samples {
    pub const SMALL_PROGRAM: &str = r#"
        let x = 10;
        let y = 20;
        return x + y;
    "#;

    pub const MEDIUM_PROGRAM: &str = r#"
        component Counter() {
            let count = Signal::new(0);

            let increment = || {
                count.set(count.get() + 1);
            };

            <div>
                <h1>Counter: {count.get()}</h1>
                <button onclick={increment}>Increment</button>
            </div>
        }
    "#;

    pub const LARGE_PROGRAM: &str = r#"
        component TodoApp() {
            let todos = Signal::new([]);
            let input = Signal::new("");
            let filter = Signal::new("all");

            let filtered_todos = Computed::new(|| {
                let all = todos.get();
                match filter.get() {
                    "active" => all.filter(|t| !t.completed),
                    "completed" => all.filter(|t| t.completed),
                    _ => all
                }
            });

            let add_todo = || {
                let text = input.get();
                if text != "" {
                    todos.update(|list| {
                        list.push({ text: text, completed: false });
                        list
                    });
                    input.set("");
                }
            };

            let toggle_todo = |id| {
                todos.update(|list| {
                    list[id].completed = !list[id].completed;
                    list
                });
            };

            let delete_todo = |id| {
                todos.update(|list| {
                    list.splice(id, 1);
                    list
                });
            };

            <div class="todo-app">
                <header>
                    <h1>Todo List</h1>
                    <input
                        value={input.get()}
                        oninput={|e| input.set(e.target.value)}
                        onkeydown={|e| if e.key == "Enter" { add_todo() }}
                        placeholder="What needs to be done?"
                    />
                </header>

                <section class="filters">
                    <button onclick={|| filter.set("all")}>All</button>
                    <button onclick={|| filter.set("active")}>Active</button>
                    <button onclick={|| filter.set("completed")}>Completed</button>
                </section>

                <ul class="todo-list">
                    {filtered_todos.get().map(|todo, idx| {
                        <li class={if todo.completed { "completed" } else { "" }}>
                            <input
                                type="checkbox"
                                checked={todo.completed}
                                onchange={|| toggle_todo(idx)}
                            />
                            <span>{todo.text}</span>
                            <button onclick={|| delete_todo(idx)}>×</button>
                        </li>
                    })}
                </ul>

                <footer>
                    <span>{filtered_todos.get().length} items</span>
                </footer>
            </div>
        }
    "#;

    pub const REACTIVE_HEAVY: &str = r#"
        component Dashboard() {
            // Multiple signals
            let users = Signal::new(0);
            let revenue = Signal::new(0.0);
            let orders = Signal::new(0);
            let conversion = Signal::new(0.0);

            // Computed values with dependencies
            let avg_order_value = Computed::new(|| {
                if orders.get() > 0 {
                    revenue.get() / orders.get() as f64
                } else {
                    0.0
                }
            });

            let revenue_per_user = Computed::new(|| {
                if users.get() > 0 {
                    revenue.get() / users.get() as f64
                } else {
                    0.0
                }
            });

            let projected_revenue = Computed::new(|| {
                revenue.get() * (1.0 + conversion.get() / 100.0)
            });

            // Effects for side effects
            Effect::new(|| {
                console.log("Users changed:", users.get());
            });

            Effect::new(|| {
                console.log("Revenue changed:", revenue.get());
            });

            <div class="dashboard">
                <div class="metric">
                    <h3>Users</h3>
                    <p>{users.get()}</p>
                </div>
                <div class="metric">
                    <h3>Revenue</h3>
                    <p>${revenue.get()}</p>
                </div>
                <div class="metric">
                    <h3>Orders</h3>
                    <p>{orders.get()}</p>
                </div>
                <div class="metric">
                    <h3>Avg Order Value</h3>
                    <p>${avg_order_value.get()}</p>
                </div>
                <div class="metric">
                    <h3>Revenue/User</h3>
                    <p>${revenue_per_user.get()}</p>
                </div>
                <div class="metric">
                    <h3>Projected Revenue</h3>
                    <p>${projected_revenue.get()}</p>
                </div>
            </div>
        }
    "#;
}

/// Benchmark result
#[derive(Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: Duration,
    pub ops_per_sec: f64,
    pub source_size: usize,
    pub wasm_size: usize,
}

impl BenchmarkResult {
    pub fn display(&self) {
        println!("\n{}", "=".repeat(60));
        println!("📊 Benchmark: {}", self.name);
        println!("{}", "-".repeat(60));
        println!("  Duration:       {:?}", self.duration);
        println!("  Ops/sec:        {:.2}", self.ops_per_sec);
        println!("  Source size:    {} bytes", self.source_size);
        println!("  WASM size:      {} bytes", self.wasm_size);
        println!("  Compression:    {:.1}x",
            self.source_size as f64 / self.wasm_size as f64);
        println!("{}", "=".repeat(60));
    }
}

/// Run a single benchmark
pub fn bench_compile(name: &str, source: &str, target: BuildTarget, iterations: usize) -> BenchmarkResult {
    let compiler = Compiler::new();
    let mut durations = Vec::new();
    let mut wasm_size = 0;

    // Warmup
    for _ in 0..3 {
        let _ = compiler.compile_source(source, target);
    }

    // Actual benchmark
    for _ in 0..iterations {
        let start = Instant::now();
        match compiler.compile_source(source, target) {
            Ok(wasm) => {
                wasm_size = wasm.len();
            },
            Err(e) => {
                eprintln!("Compilation error: {:?}", e);
                wasm_size = 0;
            }
        }
        durations.push(start.elapsed());
    }

    let total_duration: Duration = durations.iter().sum();
    let avg_duration = total_duration / iterations as u32;
    let ops_per_sec = 1.0 / avg_duration.as_secs_f64();

    BenchmarkResult {
        name: name.to_string(),
        duration: avg_duration,
        ops_per_sec,
        source_size: source.len(),
        wasm_size,
    }
}

/// Benchmark suite runner
pub fn run_benchmarks() {
    println!("\n🚀 RavensOne Compiler Benchmarks");
    println!("==================================\n");

    let iterations = 100;

    // Small program benchmarks
    println!("Running small program benchmarks (100 iterations)...");
    let small_client = bench_compile(
        "Small Program (Client)",
        samples::SMALL_PROGRAM,
        BuildTarget::Client,
        iterations
    );
    small_client.display();

    let small_server = bench_compile(
        "Small Program (Server)",
        samples::SMALL_PROGRAM,
        BuildTarget::Server,
        iterations
    );
    small_server.display();

    // Medium program benchmarks
    println!("\nRunning medium program benchmarks (100 iterations)...");
    let medium_client = bench_compile(
        "Medium Program (Counter Component)",
        samples::MEDIUM_PROGRAM,
        BuildTarget::Client,
        iterations
    );
    medium_client.display();

    // Large program benchmarks
    println!("\nRunning large program benchmarks (50 iterations)...");
    let large_client = bench_compile(
        "Large Program (Todo App)",
        samples::LARGE_PROGRAM,
        BuildTarget::Client,
        50
    );
    large_client.display();

    // Reactive-heavy benchmarks
    println!("\nRunning reactive-heavy benchmarks (50 iterations)...");
    let reactive_client = bench_compile(
        "Reactive-Heavy (Dashboard)",
        samples::REACTIVE_HEAVY,
        BuildTarget::Client,
        50
    );
    reactive_client.display();

    // Summary
    println!("\n📈 Benchmark Summary");
    println!("====================\n");
    println!("  Small programs:    {:.2} compilations/sec", small_client.ops_per_sec);
    println!("  Medium programs:   {:.2} compilations/sec", medium_client.ops_per_sec);
    println!("  Large programs:    {:.2} compilations/sec", large_client.ops_per_sec);
    println!("  Reactive-heavy:    {:.2} compilations/sec", reactive_client.ops_per_sec);

    println!("\n💾 Code Size Efficiency");
    println!("=======================\n");
    println!("  Small:   {} bytes source → {} bytes WASM ({:.1}x compression)",
        small_client.source_size, small_client.wasm_size,
        small_client.source_size as f64 / small_client.wasm_size.max(1) as f64);
    println!("  Medium:  {} bytes source → {} bytes WASM ({:.1}x compression)",
        medium_client.source_size, medium_client.wasm_size,
        medium_client.source_size as f64 / medium_client.wasm_size.max(1) as f64);
    println!("  Large:   {} bytes source → {} bytes WASM ({:.1}x compression)",
        large_client.source_size, large_client.wasm_size,
        large_client.source_size as f64 / large_client.wasm_size.max(1) as f64);

    println!("\n✅ All benchmarks complete!\n");
}

fn main() {
    run_benchmarks();
}
