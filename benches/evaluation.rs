use cranelift::prelude::{types, AbiParam};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use evalexpr::*;
use pyo3::ffi::c_str;
use pyo3::prelude::*;
use quantixis_rs::ast::{Compiler, Executor, Value as ASTValue}; // Assuming the crate name

// use meval::eval_str;
use quantixis_macros::quantinxis_fn;
use quantixis_rs::bytecode::{BytecodeCompiler, BytecodeExecutor, Value};
use quantixis_rs::jit::{execute, JITCompiler, JITCompilerBuilder};
use std::collections::HashMap;

#[quantinxis_fn]
fn sin(x: f64) -> Result<Value, String> {
    Ok(Value::Number(f64::sin(x)))
}

#[quantinxis_fn]
fn cos(x: f64) -> Result<Value, String> {
    Ok(Value::Number(f64::cos(x)))
}

#[quantinxis_fn]
fn tan(x: f64) -> Result<Value, String> {
    Ok(Value::Number(f64::tan(x)))
}

#[quantinxis_fn]
fn sqrt(x: f64) -> Result<Value, String> {
    Ok(Value::Number(f64::sqrt(x)))
}

#[quantinxis_fn]
fn log(x: f64) -> Result<Value, String> {
    Ok(Value::Number(f64::log(x, 2.0)))
}

#[quantinxis_fn]
fn f(x: f64) -> Result<Value, String> {
    Ok(Value::Number(x + 1.0))
}

#[quantinxis_fn]
fn g(x: f64) -> Result<Value, String> {
    Ok(Value::Number(x - 2.0))
}

#[quantinxis_fn]
fn h(x: f64, y: f64) -> Result<Value, String> {
    Ok(Value::Number(x * y))
}

// Sample Rust functions
fn _add(a: f64, b: f64) -> f64 {
    a + b
}

fn _multiply(a: f64, b: f64) -> f64 {
    a * b
}

fn _square(a: f64) -> f64 {
    a * a
}

fn _sum_array(a: Vec<f64>) -> f64 {
    a.into_iter().reduce(|acc, e| acc + e).unwrap()
}

fn jit_compiler() -> JITCompiler {
    let builder = JITCompilerBuilder::new()
        .add_function(
            "add".to_string(),
            _add as *const u8,
            vec![AbiParam::new(types::F64), AbiParam::new(types::F64)],
            vec![AbiParam::new(types::F64)],
        )
        .add_function(
            "multiply".to_string(),
            _multiply as *const u8,
            vec![AbiParam::new(types::F64), AbiParam::new(types::F64)],
            vec![AbiParam::new(types::F64)],
        )
        .add_function(
            "square".to_string(),
            _square as *const u8,
            vec![AbiParam::new(types::F64)],
            vec![AbiParam::new(types::F64)],
        )
        .add_function(
            "sum_array".to_string(),
            _square as *const u8,
            vec![AbiParam::new(types::F64)],
            vec![AbiParam::new(types::F64)],
        );

    builder.build().unwrap()
}

fn benchmark_expressions(c: &mut Criterion) {
    let expressions = vec![
        ("add(a, b)", "First Function"),
        ("add(a, b) > multiply(a, b)", "Function results comparison"),
        // ("1 + 2", "Simple Arithmetic"),
        // ("(3 + 5) * 2", "Grouped Arithmetic"),
        // ("(1 + 2) > (3 - 4)", "Simple Logic"),
        // ("((1 + 2) > 0) && ((3 - 4) < 0)", "Grouped Logic"),
        // ("sin(0.5) + cos(1.2) - tan(0.3)", "Function Calls"),
        // ("f(g(h((3 + 2), 2)))", "Deeply Nested Functions"),
        // (
        //     "(a * b) + sin(c) / sqrt(d) - log(e)",
        //     "Mixed Functions & Arithmetic",
        // ),
        // // ("map.key1 + map.key2 * 2", "Property Access in Map"),
        // ("f(x) + g(y) + h(z)", "Multiple Function Calls"),
        // (
        //     "(obj.prop1 + obj.prop2) * f(obj.prop3)",
        //     "Property Access + Function Call",
        // ),
        // (
        //     "((1 + 2) * (f(3) + g(h(4, 3)))) - obj.value",
        //     "Complex Grouped & Nested",
        // ),
    ];

    let mut group = c.benchmark_group("Expression Execution");

    for (expr, label) in expressions {
        let _a: f64 = rand::random_range(0.0..20.0);
        let _b: f64 = rand::random_range(0.0..20.0);
        group.bench_function(format!("JIT Compile and Execute - {label}"), |b| {
            let bytecode = BytecodeCompiler::new().compile(expr).unwrap();

            b.iter(|| {
                let mut jit_compiler = jit_compiler();
                let (func_id, mut env) = jit_compiler.compile(&bytecode).unwrap();
                env.set_f64("a", _a);
                env.set_f64("b", _b);
                env.init();
                black_box(execute(func_id, env.as_ptr()).unwrap());
            })
        });

        group.bench_function(format!("JIT Pre-Compiled Execution - {label}"), |b| {
            let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
            let mut jit_compiler = jit_compiler();
            let (func_id, mut env) = jit_compiler.compile(&bytecode).unwrap();

            env.set_f64("a", _a);
            env.set_f64("b", _b);
            env.init();
            b.iter(|| black_box(execute(func_id, env.as_ptr()).unwrap()))
        });

        group.bench_function(format!("Native Rust - {label}"), |b| {
            b.iter(|| _add(black_box(_a), black_box(_b)))
        });

        // group.bench_function(format!("meval - {label}"), |b| {
        //     b.iter(|| black_box(eval_str(expr).unwrap()))
        // });

        // group.bench_function(format!("evalexpr - {label}"), |b| {
        //     b.iter(|| black_box(eval(expr).unwrap()))
        // });
        //
        // group.bench_function(format!("IR Executor - {label}"), |b| {
        //     b.iter(|| black_box(bytecode_exec.execute_interpreter(&bytecode).unwrap()))
        // });
        //
        // group.bench_function(format!("Bytecode Compile and Execute - {label}"), |b| {
        //     let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        //     b.iter(|| black_box(bytecode_exec.execute(&bytecode).unwrap()))
        // });
        //
        // group.bench_function(format!("Bytecode Interpreter - {label}"), |b| {
        //     b.iter(|| black_box(bytecode_exec.execute(&bytecode).unwrap()))
        // });
    }

    group.finish();
}

// // Sample Rust functions
// fn _add(a: f64, b: f64) -> f64 {
//     a + b
// }
//
// /// Benchmark simple arithmetic expressions
// fn benchmark_simple_arithmetic(c: &mut Criterion) {
//     let mut group = c.benchmark_group("Simple arithmetic Expression Evaluation");
//
//     let mut executor = Executor::new();
//
//     let expr = "a + b";
//     let compiled = Compiler::compile_expression(expr).unwrap();
//     let precompiled_evalexpr = build_operator_tree::<DefaultNumericTypes>(expr).unwrap();
//
//     let mut bytecode_exec = BytecodeExecutor::new();
//     bytecode_exec.bind_variable("a", Value::Number(4.0));
//     bytecode_exec.bind_variable("b", Value::Number(10.0));
//     let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
//
//     group.bench_function(format!("JIT Compile and Addition"), |b| {
//         b.iter(|| {
//             let mut bytecode_exec = BytecodeExecutor::new();
//             let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
//             let mut jit = JITCompiler::from(
//                 [],
//                 [
//                     ("a".to_string(), &4.0 as *const _ as *const u8),
//                     ("b".to_string(), &10.0 as *const _ as *const u8),
//                 ],
//             );
//             let jit_func = jit.compile(&bytecode).unwrap(); // Compile once
//             black_box(jit.execute(jit_func));
//         })
//     });
//
//     group.bench_function(format!("JIT Pre-Compile and Addition"), |b| {
//         let mut jit = JITCompiler::from(
//             [],
//             [
//                 ("a".to_string(), &4.0 as *const _ as *const u8),
//                 ("b".to_string(), &10.0 as *const _ as *const u8),
//             ],
//         );
//         let jit_func = jit.compile(&bytecode).unwrap(); // Compile once
//         b.iter(|| {
//             black_box(jit.execute(jit_func));
//         })
//     });
//
//     group.bench_function("native_rust_arithmetic", |b| {
//         let a: f64 = 4.0;
//         let _b: f64 = 10.0;
//         b.iter(|| _add(black_box(a), black_box(_b)))
//     });
//
//     group.bench_function(format!("Bytecode Compile and Execute"), |b| {
//         b.iter(|| {
//             let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
//             black_box(bytecode_exec.execute(&bytecode).unwrap())
//         })
//     });
//
//     group.bench_function(format!("Bytecode Pre-Compile and Execute"), |b| {
//         b.iter(|| bytecode_exec.execute(black_box(&bytecode)).unwrap())
//     });
//
//     group.bench_function("compiled_arithmetic", |b| {
//         b.iter(|| executor.execute_expression(black_box(expr), &black_box(Default::default())))
//     });
//
//     group.bench_function("precompiled_arithmetic", |b| {
//         b.iter(|| executor.execute(black_box(&compiled), &black_box(Default::default())))
//     });
//
//     group.bench_function("meval_arithmetic", |b| {
//         b.iter(|| meval::eval_str(black_box(expr)).unwrap())
//     });
//
//     group.bench_function("evalexpr_arithmetic", |b| {
//         b.iter(|| evalexpr::eval(black_box(expr)).unwrap())
//     });
//
//     group.bench_function("precompiled_evalexpr_arithmetic", |b| {
//         b.iter(|| precompiled_evalexpr.eval().unwrap())
//     });
//
//     Python::with_gil(|py| {
//         group.bench_function("native_python_arithmetic", |b| {
//             b.iter(|| {
//                 let code = c_str!("2 + 3 * 4");
//                 let result: f64 = py.eval(code, None, None).unwrap().extract().unwrap();
//                 black_box(result)
//             })
//         });
//     });
// }
//
// /// Benchmark complex arithmetic expressions
// fn benchmark_complex_arithmetic(c: &mut Criterion) {
//     let mut group = c.benchmark_group("Complex arithmetic Expression Evaluation");
//
//     let mut executor = Executor::new();
//
//     let expr = "(10 + 20) * 3 / (4 - 1) + 5";
//     let compiled = Compiler::compile_expression(expr).unwrap();
//     let precompiled_evalexpr = build_operator_tree::<DefaultNumericTypes>(expr).unwrap();
//
//     let mut bytecode_exec = BytecodeExecutor::new();
//     let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
//
//     group.bench_function(format!("Bytecode Compile and Execute"), |b| {
//         b.iter(|| {
//             let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
//             black_box(bytecode_exec.execute(&bytecode).unwrap())
//         })
//     });
//
//     group.bench_function(format!("Bytecode Pre-Compile and Execute"), |b| {
//         b.iter(|| black_box(bytecode_exec.execute(&bytecode).unwrap()))
//     });
//
//     group.bench_function("compiled_complex_arithmetic", |b| {
//         b.iter(|| executor.execute_expression(black_box(expr), &black_box(Default::default())))
//     });
//
//     group.bench_function("precompiled_complex_arithmetic", |b| {
//         b.iter(|| executor.execute(black_box(&compiled), &black_box(Default::default())))
//     });
//
//     group.bench_function("native_rust_complex_arithmetic", |b| {
//         b.iter(|| black_box((10.0 + 20.0) * 3.0 / (4.0 - 1.0) + 5.0))
//     });
//
//     group.bench_function("meval_arithmetic", |b| {
//         b.iter(|| meval::eval_str(black_box(expr)).unwrap())
//     });
//
//     group.bench_function("evalexpr_arithmetic", |b| {
//         b.iter(|| evalexpr::eval(black_box(expr)).unwrap())
//     });
//
//     group.bench_function("precompiled_evalexpr_arithmetic", |b| {
//         b.iter(|| precompiled_evalexpr.eval().unwrap())
//     });
//
//     Python::with_gil(|py| {
//         group.bench_function("native_python_complex_arithmetic", |b| {
//             b.iter(|| {
//                 let code = c_str!("(10 + 20) * 3 / (4 - 1) + 5");
//                 let result: f64 = py.eval(code, None, None).unwrap().extract().unwrap();
//                 black_box(result)
//             })
//         });
//     });
// }
// //
// // /// Benchmark logical expressions
// // fn benchmark_logic_expressions(c: &mut Criterion) {
// //     let mut group = c.benchmark_group("Logic Expression Evaluation");
// //     let mut executor = Executor::new();
// //
// //     let expr = "true && false || true";
// //     let compiled = Compiler::compile_expression(expr).unwrap();
// //     let precompiled_evalexpr = build_operator_tree::<DefaultNumericTypes>(expr).unwrap();
// //
// //     group.bench_function("compiled_logic_expression", |b| {
// //         b.iter(|| executor.execute_expression(black_box(expr), &black_box(Default::default())))
// //     });
// //
// //     group.bench_function("precompiled_logic_expression", |b| {
// //         b.iter(|| executor.execute(black_box(&compiled), &black_box(Default::default())))
// //     });
// //
// //     group.bench_function("native_rust_logic_expression", |b| {
// //         b.iter(|| black_box(true && false || true))
// //     });
// //
// //     group.bench_function("evalexpr_arithmetic", |b| {
// //         b.iter(|| evalexpr::eval(black_box(expr)).unwrap())
// //     });
// //
// //     group.bench_function("precompiled_evalexpr_arithmetic", |b| {
// //         b.iter(|| precompiled_evalexpr.eval().unwrap())
// //     });
// //
// //     Python::with_gil(|py| {
// //         group.bench_function("native_python_logic_expression", |b| {
// //             b.iter(|| {
// //                 let code = c_str!("True and False or True");
// //                 let result: bool = py.eval(code, None, None).unwrap().extract().unwrap();
// //                 black_box(result)
// //             })
// //         });
// //     });
// // }
// //
// // /// Benchmark property access
// // fn benchmark_property_access(c: &mut Criterion) {
// //     let mut group = c.benchmark_group("Property Access Evaluation");
// //     let mut executor = Executor::new();
// //
// //     executor.register_function("get_data", |_args| {
// //         let mut map = std::collections::HashMap::new();
// //         map.insert("value".to_string(), Value::Number(42.0));
// //         Ok(Value::Identifier("data".to_string()))
// //     });
// //
// //     let expr = "get_data().value";
// //     let compiled = Compiler::compile_expression(expr).unwrap();
// //
// //     group.bench_function("compiled_property_access", |b| {
// //         b.iter(|| executor.execute_expression(black_box(expr), &black_box(Default::default())))
// //     });
// //
// //     group.bench_function("precompiled_property_access", |b| {
// //         b.iter(|| executor.execute(black_box(&compiled), &black_box(Default::default())))
// //     });
// //
// //     group.bench_function("native_rust_property_access", |b| {
// //         b.iter(|| {
// //             let data = 42.0;
// //             black_box(data)
// //         })
// //     });
// //
// //     Python::with_gil(|py| {
// //         group.bench_function("native_python_property_access", |b| {
// //             b.iter(|| {
// //                 let code = c_str!("{'value': 42}['value']");
// //                 let result: f64 = py.eval(code, None, None).unwrap().extract().unwrap();
// //                 black_box(result)
// //             })
// //         });
// //     });
// // }
//
// #[quantinxis_fn]
// fn square(a: f64) -> Result<Value, String> {
//     Ok(Value::Number(a * a))
// }
//
// /// Benchmark function calls
// fn benchmark_function_calls(c: &mut Criterion) {
//     let expr = "square(4)";
//
//     let mut group = c.benchmark_group("Function Call Evaluation");
//     let mut executor = Executor::new();
//
//     executor.register_function("square", |args| {
//         if let [ASTValue::Number(x)] = args {
//             Ok(ASTValue::Number(x * x))
//         } else {
//             Err("Invalid arguments".into())
//         }
//     });
//
//     let mut bytecode_exec = BytecodeExecutor::new();
//     bytecode_exec.register_function("square", square);
//
//     let mut compiler = BytecodeCompiler::new();
//     let bytecode = compiler.compile(expr).unwrap();
//
//     let compiled = Compiler::compile_expression("square(a: 4)").unwrap();
//
//     group.bench_function(format!("Bytecode Compile and Execute"), |b| {
//         b.iter(|| {
//             let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
//             black_box(bytecode_exec.execute(&bytecode).unwrap())
//         })
//     });
//
//     group.bench_function(format!("Bytecode Pre-Compile and Execute"), |b| {
//         b.iter(|| black_box(bytecode_exec.execute(&bytecode).unwrap()))
//     });
//
//     group.bench_function("compiled_function_call", |b| {
//         let expr = "square(a: 4)";
//         b.iter(|| executor.execute_expression(black_box(expr), &black_box(Default::default())))
//     });
//
//     group.bench_function("precompiled_function_call", |b| {
//         b.iter(|| executor.execute(black_box(&compiled), &black_box(Default::default())))
//     });
//
//     group.bench_function("native_rust_function_call", |b| {
//         b.iter(|| black_box(4.0 * 4.0))
//     });
//
//     Python::with_gil(|py| {
//         group.bench_function("native_python_function_call", |b| {
//             b.iter(|| {
//                 let code = c_str!("4 ** 2");
//                 let result: f64 = py.eval(code, None, None).unwrap().extract().unwrap();
//                 black_box(result)
//             })
//         });
//     });
// }

// Grouping benchmarks
criterion_group!(
    benches,
    benchmark_expressions,
    // benchmark_simple_arithmetic,
    // benchmark_function_calls,
    // benchmark_complex_arithmetic,
    // benchmark_logic_expressions,
    // benchmark_property_access,
);
criterion_main!(benches);
