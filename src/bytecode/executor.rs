use crate::bytecode::{Bytecode, BytecodeCompiler};
// use log::debug;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Number(f64),
    Boolean(bool),
    Str(String),
    ArrayF64(Vec<f64>),
    Map(HashMap<String, Value>),
}

// type Func = fn(&[Value]) -> Result<Value, String>;

pub struct BytecodeExecutor {
    stack: Vec<Value>,
    variables: HashMap<String, Value>, // Named variables for easier access
    functions: HashMap<String, fn(&[Value]) -> Result<Value, String>>, // Function registry
}

impl BytecodeExecutor {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Registers a function that can be called during execution
    pub fn register_function(&mut self, name: &str, func: fn(&[Value]) -> Result<Value, String>) {
        self.functions.insert(name.to_string(), func);
    }

    // Bind a variable to the execution context
    pub fn bind_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn execute(&mut self, bytecode: &[Bytecode]) -> Result<Option<Value>, String> {
        let mut pc = 0; // Program counter

        while pc < bytecode.len() {
            let instruction = &bytecode[pc];
            pc += 1;

            match instruction {
                // Stack Operations
                Bytecode::PushInt(value) => self.stack.push(Value::Int(*value)),
                Bytecode::PushFloat(value) => self.stack.push(Value::Number(*value)),
                Bytecode::PushBool(value) => self.stack.push(Value::Boolean(*value)),
                Bytecode::PushString(value) => self.stack.push(Value::Str(value.clone())),
                Bytecode::PushArrayF64(values) => self.stack.push(Value::ArrayF64(values.clone())),
                Bytecode::PushMap(map) => self.stack.push(Value::Map(map.clone())),

                // Arithmetic Operations
                Bytecode::Add => self.binary_op(|a, b| a + b)?,
                Bytecode::Sub => self.binary_op(|a, b| a - b)?,
                Bytecode::Mul => self.binary_op(|a, b| a * b)?,
                Bytecode::Div => self.binary_op(|a, b| a / b)?,
                Bytecode::Mod => self.binary_op(|a, b| a % b)?,
                Bytecode::Pow => self.binary_op(|a, b| a.powf(b))?,

                // Comparison Operations
                Bytecode::Eq => self.binary_op_bool(|a, b| a == b)?,
                Bytecode::Ne => self.binary_op_bool(|a, b| a != b)?,
                Bytecode::Gt => self.binary_op_bool(|a, b| a > b)?,
                Bytecode::Lt => self.binary_op_bool(|a, b| a < b)?,
                Bytecode::Ge => self.binary_op_bool(|a, b| a >= b)?,
                Bytecode::Le => self.binary_op_bool(|a, b| a <= b)?,

                // Logical Operations
                Bytecode::And => self.binary_op_bool(|a, b| a && b)?,
                Bytecode::Or => self.binary_op_bool(|a, b| a || b)?,
                Bytecode::Not => self.unary_op_bool(|a| !a)?,

                // Function Calls
                Bytecode::Call(fn_name, arg_count) => {
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(self.stack.pop().ok_or("Stack underflow on function call")?);
                    }
                    args.reverse(); // Reverse the order of arguments

                    // Assuming you have a mechanism for looking up functions
                    if let Some(func) = self.functions.get(fn_name) {
                        let result = func(&args);
                        self.stack.push(result?);
                    } else {
                        return Err(format!("Call to undefined function: '{fn_name}'"));
                    }

                    // let func: Func = unsafe { std::mem::transmute(fn_addr) };
                    // let result = func(&args);
                    // self.stack.push(result?);
                }

                // Variable Handling
                Bytecode::LoadVariable(var_name) => {
                    if let Some(value) = self.variables.get(var_name) {
                        self.stack.push(value.clone());
                    } else {
                        return Err(format!("Undefined variable: {}", var_name));
                    }
                }
                Bytecode::StoreVariable(var_name) => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or("Stack underflow when storing variable")?;
                    self.variables.insert(var_name.clone(), value);
                }

                // Property Access
                Bytecode::GetProperty(property_name) => match self.stack.pop() {
                    Some(Value::Map(map)) => {
                        if let Some(value) = map.get(property_name) {
                            self.stack.push(value.clone());
                        } else {
                            return Err(format!("Property '{}' not found in map", property_name));
                        }
                    }
                    _ => return Err("Cannot access property on a non-map value".to_string()),
                },

                // Control Flow
                Bytecode::Jump(target) => {
                    pc = *target;
                    continue;
                }
                Bytecode::JumpIfTrue(target) => {
                    if self.pop_bool()? {
                        pc = *target;
                        continue;
                    }
                }
                Bytecode::JumpIfFalse(target) => {
                    if !self.pop_bool()? {
                        pc = *target;
                        continue;
                    }
                }
                Bytecode::Return => return Ok(self.stack.pop()),
                Bytecode::NoOp => {}
            }
        }

        Ok(self.stack.pop())
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), String>
    where
        F: Fn(f64, f64) -> f64,
    {
        let (b, a) = (self.pop_number()?, self.pop_number()?);
        self.stack.push(Value::Number(op(a, b)));
        Ok(())
    }

    fn binary_op_bool<F>(&mut self, op: F) -> Result<(), String>
    where
        F: Fn(bool, bool) -> bool,
    {
        let (b, a) = (self.pop_bool()?, self.pop_bool()?);
        self.stack.push(Value::Boolean(op(a, b)));
        Ok(())
    }

    fn unary_op_bool<F>(&mut self, op: F) -> Result<(), String>
    where
        F: Fn(bool) -> bool,
    {
        let a = self.pop_bool()?;
        self.stack.push(Value::Boolean(op(a)));
        Ok(())
    }

    fn pop_number(&mut self) -> Result<f64, String> {
        match self.stack.pop() {
            Some(Value::Int(v)) => Ok(v as f64),
            Some(Value::Number(v)) => Ok(v),
            Some(Value::Boolean(v)) => Ok(v as i64 as f64),
            Some(Value::Str(v)) => Ok(v.parse::<f64>().map_err(|e| e.to_string())?),
            _ => Err("Expected a number on stack".to_string()),
        }
    }

    fn pop_bool(&mut self) -> Result<bool, String> {
        match self.stack.pop() {
            Some(Value::Boolean(v)) => Ok(v),
            Some(Value::Int(v)) => Ok(v != 0),
            Some(Value::Number(v)) => Ok(v != 0.0),
            _ => Err("Expected a boolean on stack".to_string()),
        }
    }
}

mod tests {
    use super::*;
    use quantixis_macros::quantinxis_fn;

    #[allow(unused)]
    fn compile_and_execute(expression: &str) -> Value {
        let bytecode = compile(expression).expect("Compilation failed");
        let mut executor = BytecodeExecutor::new();
        executor
            .execute(&bytecode)
            .expect("Execution failed")
            .expect("Execute option failed")
    }

    #[allow(unused)]
    fn compile_and_execute_result(expression: &str) -> Result<Value, String> {
        let bytecode = compile(expression)?;
        let mut executor = BytecodeExecutor::new();
        executor.execute(&bytecode)?.ok_or("None found".to_string())
    }

    fn compile(expression: &str) -> Result<Vec<Bytecode>, String> {
        let mut compiler = BytecodeCompiler::new();
        compiler.compile(expression)
    }

    #[quantinxis_fn]
    fn add(a: f64, b: f64) -> Result<Value, String> {
        Ok(Value::Number(a + b))
    }

    #[quantinxis_fn]
    fn subtract(a: f64, b: f64) -> Result<Value, String> {
        Ok(Value::Number(a - b))
    }

    #[quantinxis_fn]
    fn multiply(a: f64, b: f64) -> Result<Value, String> {
        Ok(Value::Number(a * b))
    }

    #[quantinxis_fn]
    fn multiply_result_obj(a: f64, b: f64) -> Result<Value, String> {
        Ok(Value::Map(HashMap::from([(
            "value".to_string(),
            Value::Number(a * b),
        )])))
    }

    #[quantinxis_fn]
    fn divide(a: f64, b: f64) -> Result<Value, String> {
        Ok(Value::Number(a / b))
    }

    #[quantinxis_fn]
    fn square(a: f64) -> Result<Value, String> {
        Ok(Value::Number(a * a))
    }

    // 1. Arithmetic Expressions
    #[test]
    fn test_simple_arithmetic() {
        assert_eq!(compile_and_execute("2 + 3"), Value::Number(5.0));
        assert_eq!(compile_and_execute("10 - 5"), Value::Number(5.0));
        assert_eq!(compile_and_execute("6 * 7"), Value::Number(42.0));
        assert_eq!(compile_and_execute("9 / 3"), Value::Number(3.0));
        assert_eq!(compile_and_execute("10 % 3"), Value::Number(1.0));
    }

    #[test]
    fn test_complex_arithmetic() {
        assert_eq!(compile_and_execute("2 + 3 * 4"), Value::Number(14.0));
        assert_eq!(compile_and_execute("(10 - 2) / 4"), Value::Number(2.0));
        assert_eq!(
            compile_and_execute("10 + 2 * 3 - 4 / 2"),
            Value::Number(14.0)
        );
    }

    #[test]
    fn test_nested_grouped_arithmetic() {
        assert_eq!(
            compile_and_execute("(2 + 3) * (4 + 5)"),
            Value::Number(45.0)
        );
        assert_eq!(
            compile_and_execute("((10 - 2) * 3) / (4 + 2)"),
            Value::Number(4.0)
        );
    }

    // 2. Logical Expressions
    #[test]
    fn test_simple_logic() {
        assert_eq!(compile_and_execute("true AND false"), Value::Boolean(false));
        assert_eq!(compile_and_execute("true OR false"), Value::Boolean(true));
        assert_eq!(compile_and_execute("NOT true"), Value::Boolean(false));
    }

    #[test]
    fn test_complex_logic() {
        assert_eq!(
            compile_and_execute("true AND false OR true"),
            Value::Boolean(true)
        );
        assert_eq!(
            compile_and_execute("(true OR false) AND NOT false"),
            Value::Boolean(true)
        );
    }

    #[test]
    fn test_nested_grouped_logic() {
        assert_eq!(
            compile_and_execute("((true OR false) AND (false OR true))"),
            Value::Boolean(true)
        );
    }

    // 3. Function Calls
    #[test]
    fn test_function_calls() {
        let mut executor = BytecodeExecutor::new();
        executor.bind_variable("x", Value::Number(4.0));
        executor.bind_variable("y", Value::Number(2.0));
        executor.register_function("square", square);
        executor.register_function("multiply", multiply);
        let bytecode = BytecodeCompiler::new().compile("square(x)").unwrap();
        let result = executor.execute(&bytecode).unwrap();
        assert_eq!(result, Some(Value::Number(16.0)));

        let bytecode = BytecodeCompiler::new().compile("multiply(y, 3)").unwrap();
        let result = executor.execute(&bytecode).unwrap();
        assert_eq!(result, Some(Value::Number(6.0)));
    }

    #[test]
    fn test_nested_function_calls() {
        let mut executor = BytecodeExecutor::new();
        executor.bind_variable("x", Value::Number(3.0));
        executor.bind_variable("y", Value::Number(5.0));
        executor.register_function("square", square);
        executor.register_function("multiply", multiply);
        executor.register_function("add", add);

        let bytecode = BytecodeCompiler::new()
            .compile("add(square(x), multiply(2, y))")
            .unwrap();
        let result = executor.execute(&bytecode).unwrap();
        assert_eq!(result, Some(Value::Number(19.0)));
    }

    // 4. Property Access
    #[test]
    fn test_property_access() {
        let mut executor = BytecodeExecutor::new();
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Value::Number(42.0));
        executor.bind_variable("obj", Value::Map(map));

        let bytecode = BytecodeCompiler::new().compile("obj.key1").unwrap();
        let result = executor.execute(&bytecode).unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    // 5. Mixed Expressions
    #[test]
    fn test_mixed_expressions() {
        let expr = "add(3, 4) * user.score";
        let mut executor = BytecodeExecutor::new();
        let mut map = HashMap::new();
        map.insert("score".to_string(), Value::Number(10.0));
        executor.bind_variable("user", Value::Map(map));
        executor.register_function("add", add);

        let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        let result = executor.execute(&bytecode).unwrap();
        assert_eq!(result, Some(Value::Number(70.0)));
    }

    #[test]
    fn test_large_expression() {
        let expr = "(10 + 2 * 3 - 4 / 2) AND (user.age > 18 OR NOT false)";

        let mut executor = BytecodeExecutor::new();
        let mut map = HashMap::new();
        map.insert("age".to_string(), Value::Number(42.0));
        executor.bind_variable("user", Value::Map(map));
        executor.register_function("add", add);

        let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        let result = executor.execute(&bytecode).unwrap();
        assert_eq!(result, Some(Value::Boolean(true)));
    }

    #[test]
    fn test_complex_nested_grouped_mixed() {
        let expr =
            "add((square(3) + multiply(2, 5)), user.profile.score) AND NOT (false OR obj.flag)";

        let mut executor = BytecodeExecutor::new();
        executor.bind_variable(
            "user",
            Value::Map(HashMap::from([(
                "profile".to_string(),
                Value::Map(HashMap::from([("score".to_string(), Value::Number(2.0))])),
            )])),
        );
        executor.bind_variable(
            "obj",
            Value::Map(HashMap::from([("flag".to_string(), Value::Boolean(true))])),
        );
        executor.register_function("add", add);
        executor.register_function("multiply", multiply);
        executor.register_function("square", square);

        let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        let result = executor.execute(&bytecode).unwrap();
        assert_eq!(result, Some(Value::Boolean(false)));
    }

    // 6. Edge Cases
    // #[test]
    // fn test_division_by_zero() {
    //     let result = compile_and_execute_result("10 / 0");
    //     assert!(result.is_err(), "Expected division by zero error");
    // }
    //
    // #[test]
    // fn test_modulo_by_zero() {
    //     let result = compile("10 % 0");
    //     assert!(result.is_err(), "Expected modulo by zero error");
    // }
    //
    // #[test]
    // fn test_infinity_propagation() {
    //     let expr = "1 / 0 + 5";
    //     assert!(
    //         compile_and_execute_result(expr).is_err(),
    //         "Expected error due to infinity propagation"
    //     );
    // }
    //
    // #[test]
    // fn test_invalid_arithmetic_nan() {
    //     let expr = "0 / 0";
    //     assert!(
    //         compile_and_execute_result(expr).is_err(),
    //         "Expected NaN result error"
    //     );
    // }
    //
    // #[test]
    // fn test_nan_propagation() {
    //     let expr = "(0 / 0) + 5";
    //     assert!(
    //         compile_and_execute_result(expr).is_err(),
    //         "Expected NaN propagation error"
    //     );
    // }

    #[test]
    fn test_undefined_variable() {
        let result = compile_and_execute_result("x + 2");
        assert_eq!(result, Err("Undefined variable: x".to_string()));
    }

    #[test]
    fn test_undefined_function_call() {
        let result = compile_and_execute_result("undefined_func(4)");
        assert_eq!(
            result,
            Err("Call to undefined function: 'undefined_func'".to_string())
        );
    }

    // 7. Type Mismatch
    #[test]
    fn test_type_mismatch_addition() {
        let expr = "true + 3";
        assert_eq!(compile_and_execute(expr), Value::Number(4.0));
    }

    #[test]
    fn test_type_mismatch_comparison() {
        let expr = "10 > true";
        assert_eq!(compile_and_execute(expr), Value::Boolean(false));
    }

    #[test]
    fn test_string_number_addition() {
        let expr = "\"10\" + 5";
        assert!(compile(expr).is_err(), "Expected error for string + number");
    }

    #[test]
    fn test_boolean_number_multiplication() {
        let expr = "true * 5";
        assert_eq!(compile_and_execute(expr), Value::Number(5.0));
    }

    // 8. Nested Function Calls with Invalid Inputs
    #[test]
    fn test_invalid_function_argument_type() {
        let expr = "add(2, true)";

        let mut executor = BytecodeExecutor::new();
        executor.register_function("add", add);

        let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        let result = executor.execute(&bytecode).unwrap();
        assert_eq!(result, Some(Value::Number(3.0)));
    }

    #[test]
    fn test_invalid_function_argument_count() {
        let expr = "multiply(2)";

        let mut executor = BytecodeExecutor::new();
        executor.register_function("multiply", multiply);

        let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        let result = executor.execute(&bytecode);
        assert_eq!(result, Err("Expected 2 arguments, but got 1".to_string()));
    }

    // 9. Nested Property Access with Invalid Object
    #[test]
    fn test_invalid_property_access_number() {
        let result = compile_and_execute_result("10.x");
        assert!(result.is_err(), "Undefined variable: 10");
    }

    #[test]
    fn test_nonexistent_property_access() {
        let mut executor = BytecodeExecutor::new();
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Value::Number(42.0));
        executor.bind_variable("obj", Value::Map(map));

        let bytecode = BytecodeCompiler::new().compile("obj.unknown").unwrap();
        let result = executor.execute(&bytecode);
        assert!(result.is_err());
    }

    // 10. Large Expressions with Deep Nesting
    #[test]
    fn test_deeply_nested_expression() {
        let expr = "((((1 + 2) * 3) / 4) - 5) AND (true OR false)";
        assert_eq!(compile_and_execute(expr), Value::Boolean(true));
    }

    #[test]
    fn test_long_expression_chain() {
        let expr = "add(1, multiply(2, subtract(5, divide(10, 2))))";

        let mut executor = BytecodeExecutor::new();
        executor.register_function("add", add);
        executor.register_function("subtract", subtract);
        executor.register_function("multiply", multiply);
        executor.register_function("divide", divide);

        let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        let result = executor.execute(&bytecode).unwrap();
        assert_eq!(result, Some(Value::Number(1.0)))
    }

    // 11. Syntax Errors
    #[test]
    fn test_unclosed_parentheses() {
        let result = compile("(5 + 3");
        assert!(result.is_err(), "Expected error for unclosed parentheses");
    }

    #[test]
    fn test_extra_closing_parentheses() {
        let result = compile("5 + 3)");
        assert!(
            result.is_err(),
            "Expected error for extra closing parentheses"
        );
    }

    #[test]
    fn test_missing_operator() {
        let result = compile("5 3");
        assert!(
            result.is_err(),
            "Expected syntax error for missing operator"
        );
    }

    // 12. Arithmetic Edge Cases
    #[test]
    fn test_multiplication_overflow() {
        let result = compile("1e308 * 1e308");
        assert!(result.is_err(), "Expected overflow error");
    }

    // #[test]
    // fn test_division_underflow() {
    //     let expr = "1e-308 / 1e308";
    //     assert_eq!(compile_and_execute(expr), Value::Number(0.0));
    // }

    #[test]
    fn test_negative_exponents() {
        let expr = "2 ^ -3";
        assert_eq!(compile_and_execute(expr), Value::Number(0.125));
    }

    // 13. Boolean Logic Edge Cases
    #[test]
    fn test_redundant_logical_expressions() {
        let expr = "true AND true AND true AND true";
        assert_eq!(compile_and_execute(expr), Value::Boolean(true));
    }

    #[test]
    fn test_always_false_logical_expression() {
        let expr = "false OR false OR false";
        assert_eq!(compile_and_execute(expr), Value::Boolean(false));
    }

    // 14. Function Call Edge Cases
    #[test]
    fn test_function_call_with_extra_args() {
        let expr = "add(2, 3, 4)";

        let mut executor = BytecodeExecutor::new();
        executor.register_function("add", add);

        let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        let result = executor.execute(&bytecode);
        assert!(result.is_err(), "Expected 2 arguments, but got 3");
    }

    // 15. Property Access Edge Cases
    #[test]
    fn test_access_function_call_as_object() {
        let expr = "multiply(2, 3).value";

        let mut executor = BytecodeExecutor::new();
        executor.register_function("multiply", multiply_result_obj);

        let bytecode = BytecodeCompiler::new().compile(expr).unwrap();
        let result = executor.execute(&bytecode).unwrap();
        assert_eq!(result, Some(Value::Number(6.0)));
    }

    #[test]
    fn test_chained_property_access_undefined() {
        let result = compile_and_execute_result("undefined_obj.prop1.prop2");
        assert!(result.is_err(), "Undefined variable: undefined_obj");
    }

    // 18. Invalid Operators & Syntax Edge Cases//
    #[test]
    fn test_unsupported_operator() {
        let result = compile("5 ** 3");
        assert!(
            result.is_err(),
            "Expected error for unsupported exponentiation operator"
        );
    }

    #[test]
    fn test_incomplete_expression() {
        let result = compile("5 +");
        assert!(result.is_err(), "Expected error for incomplete expression");
    }

    #[test]
    fn test_empty_input() {
        let result = compile("");
        assert!(result.is_err(), "Expected error for empty expression");
    }

    // 19.  Recursive Function Calls
    #[test]
    fn test_simple_recursion() {
        let result = compile("factorial(5)");
        assert!(result.is_ok(), "Expected recursion to execute correctly");
    }

    #[test]
    fn test_exceed_recursion_limit() {
        let expr = "infinite_recursion()";
        let result = compile_and_execute_result(expr);
        assert!(
            result.is_err(),
            "Expected stack overflow error due to excessive recursion"
        );
    }
}
