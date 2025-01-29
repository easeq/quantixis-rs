use crate::ast::{ASTNode, FunctionArgValue, FunctionArgs, FunctionResult, Parser};
use std::collections::HashMap;
use std::sync::Arc;

pub type Function = Arc<dyn Fn(&FunctionArgs) -> Result<FunctionResult, String> + Send + Sync>;

pub struct Evaluator {
    pub(crate) functions: HashMap<String, Function>,
}

impl Evaluator {
    /// Creates a new `Evaluator` with a given maximum cache size.
    pub fn new(_max_cache_size: usize) -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Parse an expression string into an AST.
    pub fn parse_expression(&self, expression: &str) -> Result<ASTNode, String> {
        let ast = Parser::parse_expression(expression)?; // Parse the expression using the grammar.
        Ok(ast)
    }

    /// Evaluates a given expression string against a provided context.
    ///
    /// # Arguments
    ///
    /// * `expression` - A string slice that holds the expression to be evaluated.
    /// * `context` - A reference to a `HashMap` that contains variable values for the expression.
    ///
    /// # Returns
    ///
    /// * `Ok(f64)` if the evaluation succeeds.
    /// * `Err(String)` if parsing or evaluation fails.
    pub fn evaluate_expression(
        &mut self,
        expression: &str,
        context: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        // Step 1: Parse the expression into an AST
        let ast = self.parse_expression(expression)?;

        // Step 2: Resolve identifiers in the AST
        let resolved_ast = ast.resolve_identifiers(context)?;

        // Step 3: Evaluate the resolved AST
        self.evaluate_ast(&resolved_ast, context)
    }

    /// Evaluate a single AST node against a single context.
    pub fn evaluate_ast(
        &mut self,
        ast: &ASTNode,
        context: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        let resolved_ast = ast.resolve_identifiers(context)?; // Resolve identifiers per context.
        self.evaluate(&resolved_ast, context) // Evaluate the resolved AST.
    }

    /// Registers a function with the evaluator.
    pub fn register_function<F>(&mut self, name: &str, function: F)
    where
        F: Fn(&FunctionArgs) -> Result<FunctionResult, String> + Send + Sync + 'static,
    {
        self.functions.insert(name.to_string(), Arc::new(function));
    }

    /// Evaluates an `ASTNode` with a given context.
    pub fn evaluate(
        &mut self,
        ast: &ASTNode,
        context: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        // Evaluate the AST node
        let result = match ast {
            ASTNode::Number(n) => Ok(*n),

            ASTNode::Identifier(ident) => context
                .get(ident)
                .copied()
                .ok_or_else(|| format!("Identifier '{}' not found in context", ident)),

            ASTNode::BinaryOperation {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left, context)?;
                // assert_eq!(left_value, 200.0);
                let right_value = self.evaluate(right, context)?;
                // assert_eq!(right_value, 200.0);
                operator.apply(left_value, right_value)
            }

            ASTNode::LogicalOperation {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left, context)?;
                let right_value = self.evaluate(right, context)?;
                operator.apply(left_value, right_value)
            }

            ASTNode::NotOperation(inner) => {
                Ok((self.evaluate(inner, context)? == 0.0) as i32 as f64)
            }

            ASTNode::FunctionCall { name, args } => {
                // Evaluate the function
                let function = self
                    .functions
                    .get(name)
                    .ok_or_else(|| format!("Function {} not registered", name))?;

                // Evaluate the arguments, resolving identifiers to values from the context
                let mut new_args = args.clone();
                for (arg_name, arg_value) in args.args.iter() {
                    let resolved_value: FunctionArgValue = match arg_value {
                        FunctionArgValue::Identifier(ident) => {
                            // Resolve the identifier to a value in the context
                            context
                                .get(ident)
                                .copied()
                                .ok_or_else(|| {
                                    format!("Identifier '{}' not found in context", ident)
                                })?
                                .try_into()
                                .unwrap()
                        }
                        _ => arg_value.clone(),
                    };

                    new_args.insert(&arg_name, resolved_value);
                }

                // Call the function with the resolved arguments
                let result = function(&new_args)?;

                match result {
                    FunctionResult::UnnamedF64(value) => Ok(value),
                    FunctionResult::NamedF64Map(_) => {
                        Err("Expected single value, got multi-value".to_string())
                    }
                }
            }
            ASTNode::PropertyAccess { base, property } => {
                if let ASTNode::FunctionCall { name, args } = &**base {
                    let function = self
                        .functions
                        .get(name)
                        .ok_or_else(|| format!("Function {} not registered", name))?;
                    if let FunctionResult::NamedF64Map(map) = function(args)? {
                        map.get(property)
                            .copied()
                            .ok_or_else(|| format!("Property {} not found in result", property))
                    } else {
                        Err("Expected multi-value, got single value".to_string())
                    }
                } else {
                    Err("Base must be a function call".to_string())
                }
            }
            ASTNode::Group(inner) => self.evaluate(inner, context),
            _ => Err("Unsupported AST node".to_string()),
        }?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{LogicalOperator, Operator};

    // Helper function to register basic functions for testing
    fn setup_evaluator() -> Evaluator {
        let mut evaluator = Evaluator::new(100);

        evaluator.register_function("add", |args| {
            let a = args.get_number("a")?;
            let b = args.get_number("b")?;
            Ok(FunctionResult::UnnamedF64(a + b))
        });

        evaluator.register_function("map_example", |args| {
            let a = args.get_number("a")?;
            let b = args.get_number("b")?;
            let c = args.get_string("c")?;
            let mut result = HashMap::new();
            result.insert("sum".to_string(), a + b);
            result.insert("label".to_string(), c.parse::<f64>().unwrap_or(0.0));
            Ok(FunctionResult::NamedF64Map(result))
        });

        evaluator.register_function("constant", |_args| Ok(FunctionResult::UnnamedF64(42.0)));

        evaluator.register_function("multiply", |args| {
            let a = args.get_number("a")?;
            let b = args.get_number("b")?;
            Ok(FunctionResult::UnnamedF64(a * b))
        });

        evaluator.register_function("complex_map", |args| {
            let x = args.get_number("x")?;
            let y = args.get_number("y")?;
            let mut result = HashMap::new();
            result.insert("sum".to_string(), x + y);
            result.insert("diff".to_string(), x - y);
            Ok(FunctionResult::NamedF64Map(result))
        });

        evaluator
    }

    #[test]
    fn test_simple_binary_expression() {
        let mut evaluator = Evaluator::new(100);
        let context = HashMap::from([("price".to_string(), 100.0), ("volume".to_string(), 50.0)]);

        assert_eq!(
            evaluator
                .evaluate_expression("price + volume", &context)
                .unwrap(),
            150.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("price - volume", &context)
                .unwrap(),
            50.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("price * volume", &context)
                .unwrap(),
            5000.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("price / volume", &context)
                .unwrap(),
            2.0
        );
    }

    #[test]
    fn test_complex_logical_expression() {
        let mut evaluator = Evaluator::new(100);
        let context = HashMap::from([("price".to_string(), 100.0), ("volume".to_string(), 50.0)]);

        assert_eq!(
            evaluator
                .evaluate_expression("price > 90 AND volume < 100", &context)
                .unwrap(),
            1.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("price > 100 OR volume <= 50", &context)
                .unwrap(),
            1.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("NOT (price == 100)", &context)
                .unwrap(),
            0.0
        );
    }

    #[test]
    fn test_logical_expression() {
        let input = "price > 100 AND volume < 5000";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 120.0), ("volume".to_string(), 3000.0)]);
        let result = evaluator.evaluate_expression(input, &context).unwrap();
        assert_eq!(result, 1.0); // Logical true
    }

    #[test]
    fn test_nested_logical_expression() {
        let input = "(price > 100 AND NOT volume < 2000) OR volume >= 3000";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 120.0), ("volume".to_string(), 3000.0)]);
        let result = evaluator.evaluate_expression(input, &context).unwrap();
        assert_eq!(result, 1.0); // Logical true
    }

    #[test]
    fn test_function_call_with_args() {
        let input = "add(a: price, b: 10)";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 50.0)]);
        let result = evaluator.evaluate_expression(input, &context).unwrap();
        assert_eq!(result, 60.0);
    }

    #[test]
    fn test_function_returning_map() {
        let input = "map_example(a: 20, b: 30, c: label).sum";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([]);
        let result = evaluator.evaluate_expression(input, &context).unwrap();
        assert_eq!(result, 50.0); // Sum of a and b
    }

    #[test]
    fn test_invalid_function_call() {
        let input = "add(a: 10)"; // Missing required argument "b"
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([]);
        let result = evaluator.evaluate_expression(input, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_arithmetic() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 100.0), ("volume".to_string(), 20.0)]);

        let result = evaluator.evaluate_expression("price + 20 * volume", &context);
        assert_eq!(result.unwrap(), 500.0); // 100 + 20*20
    }

    #[test]
    fn test_direct_ast_binary_operation() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 50.0)]);
        let ast = ASTNode::BinaryOperation {
            left: Box::new(ASTNode::Identifier("price".to_string())),
            operator: Operator::Add,
            right: Box::new(ASTNode::Number(20.0)),
        };
        let result = evaluator.evaluate_ast(&ast, &context).unwrap();
        assert_eq!(result, 70.0);
    }

    #[test]
    fn test_direct_ast_logical_operation() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 120.0), ("volume".to_string(), 3000.0)]);
        let ast = ASTNode::LogicalOperation {
            left: Box::new(ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("price".to_string())),
                operator: Operator::GreaterThan,
                right: Box::new(ASTNode::Number(100.0)),
            }),
            operator: LogicalOperator::And,
            right: Box::new(ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("volume".to_string())),
                operator: Operator::LessThan,
                right: Box::new(ASTNode::Number(5000.0)),
            }),
        };
        let result = evaluator.evaluate_ast(&ast, &context).unwrap();
        assert_eq!(result, 1.0); // Logical true
    }

    #[test]
    fn test_empty_logical_expression() {
        let input = "price > AND volume < 5000";
        let mut evaluator = setup_evaluator();
        let result = evaluator.evaluate_expression(input, &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_large_expression() {
        let input = (0..50)
            .map(|i| format!("price{} > {}", i, i * 10))
            .collect::<Vec<_>>()
            .join(" AND ");

        let mut evaluator = setup_evaluator();
        let context =
            HashMap::from_iter((0..50).map(|i| (format!("price{}", i), (i * 10) as f64 + 1.0)));
        let result = evaluator.evaluate_expression(&input, &context).unwrap();
        assert_eq!(result, 1.0); // Logical true
    }

    #[test]
    fn test_unsupported_characters() {
        let input = "price > 100 @ volume < 5000"; // Unsupported character '@'
        let mut evaluator = setup_evaluator();
        let result = evaluator.evaluate_expression(input, &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_excess_whitespace() {
        let input = "  price    >    100    AND    volume   <  5000  ";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 120.0), ("volume".to_string(), 3000.0)]);
        let result = evaluator.evaluate_expression(input, &context).unwrap();
        assert_eq!(result, 1.0); // Logical true
    }

    #[test]
    fn test_function_with_invalid_syntax() {
        let input = "add(a: price, 10)"; // Missing 'b:'
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 50.0)]);
        let result = evaluator.evaluate_expression(input, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_grouped_expressions() {
        let input = "(price + 10) * (volume - 5)";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 20.0), ("volume".to_string(), 50.0)]);
        let result = evaluator.evaluate_expression(input, &context).unwrap();
        assert_eq!(result, 1350.0); // (20 + 10) * (50 - 5)
    }

    #[test]
    fn test_nested_expressions() {
        let input = "(price > 100 AND volume < 5000) OR (price < 50 AND volume > 100)";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 120.0), ("volume".to_string(), 200.0)]);
        let result = evaluator.evaluate_expression(input, &context).unwrap();
        assert_eq!(result, 1.0); // True due to first group
    }

    #[test]
    fn test_complex_function_call_with_property_access() {
        let input = "complex_map(x: 100, y: 50).sum > 120";
        let mut evaluator = setup_evaluator();
        let result = evaluator
            .evaluate_expression(input, &HashMap::new())
            .unwrap();
        assert_eq!(result, 1.0); // 100 + 50 > 120
    }

    #[test]
    fn test_large_nested_expression() {
        let input =
            "(price0 > 10 AND price1 < 20) OR ((price2 >= 15 AND price3 <= 30) AND (price4 != 25))";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([
            ("price0".to_string(), 15.0),
            ("price1".to_string(), 10.0),
            ("price2".to_string(), 20.0),
            ("price3".to_string(), 30.0),
            ("price4".to_string(), 30.0),
        ]);
        let result = evaluator.evaluate_expression(input, &context).unwrap();
        assert_eq!(result, 1.0); // Logical true due to second group
    }

    #[test]
    fn test_very_large_mixed_expression() {
        let input = (0..50)
            .map(|i| format!("(price{} > {})", i, i * 10))
            .collect::<Vec<_>>()
            .join(" AND ");

        let mut evaluator = setup_evaluator();

        let context =
            HashMap::from_iter((0..50).map(|i| (format!("price{}", i), (i * 10) as f64 + 1.0)));
        let result = evaluator.evaluate_expression(&input, &context).unwrap();
        assert_eq!(result, 1.0); // Logical true
    }

    #[test]
    fn test_mixed_function_and_logical_expressions() {
        let input = "price > 50 AND multiply(a: price, b: 2) < 150";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 60.0)]);
        let result = evaluator.evaluate_expression(input, &context).unwrap();
        assert_eq!(result, 1.0); // 60 > 50 AND (60 * 2) < 150
    }

    #[test]
    fn test_edge_case_missing_values() {
        let input = "price > 50 AND volume < 500";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 60.0)]);
        let result = evaluator.evaluate_expression(input, &context);
        assert!(result.is_err()); // Missing "volume" in context
    }

    #[test]
    fn test_edge_case_division_by_zero() {
        let input = "price / volume";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 100.0), ("volume".to_string(), 0.0)]);
        let result = evaluator.evaluate_expression(input, &context);
        assert!(result.is_err()); // Division by zero
    }

    #[test]
    fn test_edge_case_invalid_identifier() {
        let input = "invalid_id > 10";
        let mut evaluator = setup_evaluator();
        let result = evaluator.evaluate_expression(input, &HashMap::new());
        assert!(result.is_err()); // "invalid_id" not in context
    }

    #[test]
    fn test_invalid_syntax() {
        let input = "price > AND volume < 500";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 60.0), ("volume".to_string(), 400.0)]);
        let result = evaluator.evaluate_expression(input, &context);
        assert!(result.is_err()); // Invalid syntax
    }

    #[test]
    fn test_excess_whitespace_and_complex_expression() {
        let input = "   (   price   +  10  )   *   (  volume  -  5  )   ";
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 20.0), ("volume".to_string(), 50.0)]);
        let result = evaluator.evaluate_expression(input, &context).unwrap();
        assert_eq!(result, 1350.0); // (20 + 10) * (50 - 5)
    }

    #[test]
    fn test_error_cases() {
        let mut evaluator = setup_evaluator();

        // Undefined identifier
        assert!(evaluator
            .evaluate_expression("undefined_variable", &HashMap::new())
            .is_err());

        // Undefined function
        assert!(evaluator
            .evaluate_expression("undefined_function()", &HashMap::new())
            .is_err());

        // Invalid expression
        assert!(evaluator
            .evaluate_expression("price + ", &HashMap::new())
            .is_err());
    }
}
