use criterion::{black_box, criterion_group, criterion_main, Criterion};
use evalexpr::*;
use pyo3::ffi::c_str;
use pyo3::prelude::*;
use quantixis_rs::ast::{Compiler, Executor, Value}; // Assuming the crate name

/// Benchmark simple arithmetic expressions
fn benchmark_simple_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("Simple arithmetic Expression Evaluation");

    let mut executor = Executor::new();

    let expr = "2 + 3";
    let compiled = Compiler::compile_expression(expr).unwrap();
    let precompiled_evalexpr = build_operator_tree::<DefaultNumericTypes>(expr).unwrap();

    group.bench_function("compiled_arithmetic", |b| {
        b.iter(|| executor.execute_expression(black_box(expr), &black_box(Default::default())))
    });

    group.bench_function("precompiled_arithmetic", |b| {
        b.iter(|| executor.execute(black_box(&compiled), &black_box(Default::default())))
    });

    group.bench_function("native_rust_arithmetic", |b| {
        b.iter(|| black_box(2.0 + 3.0 * 4.0))
    });

    group.bench_function("meval_arithmetic", |b| {
        b.iter(|| meval::eval_str(black_box(expr)).unwrap())
    });

    group.bench_function("evalexpr_arithmetic", |b| {
        b.iter(|| evalexpr::eval(black_box(expr)).unwrap())
    });

    group.bench_function("precompiled_evalexpr_arithmetic", |b| {
        b.iter(|| precompiled_evalexpr.eval().unwrap())
    });

    Python::with_gil(|py| {
        group.bench_function("native_python_arithmetic", |b| {
            b.iter(|| {
                let code = c_str!("2 + 3 * 4");
                let result: f64 = py.eval(code, None, None).unwrap().extract().unwrap();
                black_box(result)
            })
        });
    });
}

/// Benchmark complex arithmetic expressions
fn benchmark_complex_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("Complex arithmetic Expression Evaluation");

    let mut executor = Executor::new();

    let expr = "(10 + 20) * 3 / (4 - 1) + 5";
    let compiled = Compiler::compile_expression(expr).unwrap();
    let precompiled_evalexpr = build_operator_tree::<DefaultNumericTypes>(expr).unwrap();

    group.bench_function("compiled_complex_arithmetic", |b| {
        b.iter(|| executor.execute_expression(black_box(expr), &black_box(Default::default())))
    });

    group.bench_function("precompiled_complex_arithmetic", |b| {
        b.iter(|| executor.execute(black_box(&compiled), &black_box(Default::default())))
    });

    group.bench_function("native_rust_complex_arithmetic", |b| {
        b.iter(|| black_box((10.0 + 20.0) * 3.0 / (4.0 - 1.0) + 5.0))
    });

    group.bench_function("meval_arithmetic", |b| {
        b.iter(|| meval::eval_str(black_box(expr)).unwrap())
    });

    group.bench_function("evalexpr_arithmetic", |b| {
        b.iter(|| evalexpr::eval(black_box(expr)).unwrap())
    });

    group.bench_function("precompiled_evalexpr_arithmetic", |b| {
        b.iter(|| precompiled_evalexpr.eval().unwrap())
    });

    Python::with_gil(|py| {
        group.bench_function("native_python_complex_arithmetic", |b| {
            b.iter(|| {
                let code = c_str!("(10 + 20) * 3 / (4 - 1) + 5");
                let result: f64 = py.eval(code, None, None).unwrap().extract().unwrap();
                black_box(result)
            })
        });
    });
}

/// Benchmark logical expressions
fn benchmark_logic_expressions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Logic Expression Evaluation");
    let mut executor = Executor::new();

    let expr = "true && false || true";
    let compiled = Compiler::compile_expression(expr).unwrap();
    let precompiled_evalexpr = build_operator_tree::<DefaultNumericTypes>(expr).unwrap();

    group.bench_function("compiled_logic_expression", |b| {
        b.iter(|| executor.execute_expression(black_box(expr), &black_box(Default::default())))
    });

    group.bench_function("precompiled_logic_expression", |b| {
        b.iter(|| executor.execute(black_box(&compiled), &black_box(Default::default())))
    });

    group.bench_function("native_rust_logic_expression", |b| {
        b.iter(|| black_box(true && false || true))
    });

    group.bench_function("evalexpr_arithmetic", |b| {
        b.iter(|| evalexpr::eval(black_box(expr)).unwrap())
    });

    group.bench_function("precompiled_evalexpr_arithmetic", |b| {
        b.iter(|| precompiled_evalexpr.eval().unwrap())
    });

    Python::with_gil(|py| {
        group.bench_function("native_python_logic_expression", |b| {
            b.iter(|| {
                let code = c_str!("True and False or True");
                let result: bool = py.eval(code, None, None).unwrap().extract().unwrap();
                black_box(result)
            })
        });
    });
}

/// Benchmark property access
fn benchmark_property_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("Property Access Evaluation");
    let mut executor = Executor::new();

    executor.register_function("get_data", |_args| {
        let mut map = std::collections::HashMap::new();
        map.insert("value".to_string(), Value::Number(42.0));
        Ok(Value::Identifier("data".to_string()))
    });

    let expr = "get_data().value";
    let compiled = Compiler::compile_expression(expr).unwrap();

    group.bench_function("compiled_property_access", |b| {
        b.iter(|| executor.execute_expression(black_box(expr), &black_box(Default::default())))
    });

    group.bench_function("precompiled_property_access", |b| {
        b.iter(|| executor.execute(black_box(&compiled), &black_box(Default::default())))
    });

    group.bench_function("native_rust_property_access", |b| {
        b.iter(|| {
            let data = 42.0;
            black_box(data)
        })
    });

    Python::with_gil(|py| {
        group.bench_function("native_python_property_access", |b| {
            b.iter(|| {
                let code = c_str!("{'value': 42}['value']");
                let result: f64 = py.eval(code, None, None).unwrap().extract().unwrap();
                black_box(result)
            })
        });
    });
}

/// Benchmark function calls
fn benchmark_function_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("Function Call Evaluation");
    let mut executor = Executor::new();

    executor.register_function("square", |args| {
        if let [Value::Number(x)] = args {
            Ok(Value::Number(x * x))
        } else {
            Err("Invalid arguments".into())
        }
    });

    let expr = "square(4)";
    let compiled = Compiler::compile_expression(expr).unwrap();

    group.bench_function("compiled_function_call", |b| {
        b.iter(|| executor.execute_expression(black_box(expr), &black_box(Default::default())))
    });

    group.bench_function("precompiled_function_call", |b| {
        b.iter(|| executor.execute(black_box(&compiled), &black_box(Default::default())))
    });

    group.bench_function("native_rust_function_call", |b| {
        b.iter(|| black_box(4.0 * 4.0))
    });

    Python::with_gil(|py| {
        group.bench_function("native_python_function_call", |b| {
            b.iter(|| {
                let code = c_str!("4 ** 2");
                let result: f64 = py.eval(code, None, None).unwrap().extract().unwrap();
                black_box(result)
            })
        });
    });
}

/// Grouping benchmarks
criterion_group!(
    benches,
    benchmark_simple_arithmetic,
    benchmark_complex_arithmetic,
    benchmark_logic_expressions,
    benchmark_property_access,
    benchmark_function_calls,
);
criterion_main!(benches);
