# **Quantixis-rs**

**Quantixis-rs** is a powerful and extensible Rust library designed for parsing, analyzing, and evaluating custom expressions. Whether you're building dynamic rule engines, lightweight scripting systems, or complex evaluators, **Quantixis-rs** provides the flexibility and performance you need.

---

## **Features**

- **Expression Parsing**  
  Parse logical and arithmetic expressions with support for grouping and nesting.

- **Custom Function Support**  
  Register your own functions to extend the library's capabilities.

- **Property Access**  
  Access properties from multi-valued function results seamlessly.

- **Rich Operator Support**  
  Includes support for arithmetic (`+`, `-`, `*`, `/`, `%`), comparison (`>`, `<`, `>=`, `<=`, `==`, `!=`), and logical operators (`AND`, `OR`, `NOT`).

- **Dynamic Context Handling**  
  Evaluate expressions with dynamically resolved variables provided in a runtime context.

- **Error Handling**  
  Get detailed error messages for undefined variables, unregistered functions, or malformed expressions.

---

## **Installation**

Add the following to your `Cargo.toml` to include **Quantixis-rs** in your project:

```toml
[dependencies]
quantixis = "0.1.0"
```

Then, run:

```sh
cargo build
```

## **Getting Started**

### Example Usage

Below is an example showcasing expression evaluation using **Quantixis-rs**:

```rust
use quantixis::{Evaluator, FunctionArgs, FunctionResult};
use std::collections::HashMap;

fn main() {
    // Create an evaluator
    let mut evaluator = Evaluator::new(100);

    // Register a custom function
    evaluator.register_function("add", |args| {
        let a = args.get_number("a")?;
        let b = args.get_number("b")?;
        Ok(FunctionResult::UnnamedF64(a + b))
    });

    // Define the variable context
    let context = HashMap::from([
        ("price".to_string(), 100.0),
        ("volume".to_string(), 50.0),
    ]);

    // Define an expression
    let expression = "add(a: price, b: volume) * 2";

    // Evaluate the expression
    let result = evaluator.evaluate_expression(expression, &context).unwrap();
    println!("Result: {}", result); // Output: 300
}
```

## Key Concepts

### Expression Syntax

- Logical Operators: AND, OR, NOT
- Arithmetic Operators: +, -, *, /, %
- Comparison Operators: >, <, >=, <=, ==, !=
- Parentheses: Use () to group expressions.

#### Examples:

- (price > 100 AND volume < 5000) OR volume >= 3000
- add(a: price, b: volume) + 10

### Evaluator API

#### Create an Evaluator

```rust
let mut evaluator = Evaluator::new(100);
```

The `new` method initializes an evaluator instance.

#### Register Custom Functions

You can define and register your own functions. Each function receives named arguments (`FunctionArgs`) and returns a `FunctionResult`.

```rust
evaluator.register_function("multiply", |args| {
    let a = args.get_number("a")?;
    let b = args.get_number("b")?;
    Ok(FunctionResult::UnnamedF64(a * b))
});
```

#### Evaluate an Expression

To evaluate an expression, provide the expression string and a variable context.

```rust
let context = HashMap::from([
    ("price".to_string(), 150.0),
    ("volume".to_string(), 80.0),
]);

let result = evaluator.evaluate_expression("price * volume", &context).unwrap();
println!("Result: {}", result); // Output: 12000
```

## Advanced Usage

### Property Access

Access properties of multi-valued results returned by a function:

```rust
evaluator.register_function("stats", |args| {
    let mean = 100.0;
    let median = 95.0;
    Ok(FunctionResult::NamedF64Map(HashMap::from([
        ("mean".to_string(), mean),
        ("median".to_string(), median),
    ])))
});

let expression = "stats().mean";
let result = evaluator.evaluate_expression(expression, &HashMap::new()).unwrap();
println!("Mean: {}", result); // Output: 100
```

## Tests

The library is extensively tested to ensure correctness for:

- Simple arithmetic and logical expressions.
- Complex nested and grouped expressions.
- Custom function evaluations with arguments.
- Property access for multi-valued function results.

Run tests using:

```sh
cargo test
```

## Contributing

Contributions are welcome!

- Report bugs or request features by opening an issue.
- Submit pull requests to enhance functionality or documentation.

## License

This project is licensed under the MIT License.
