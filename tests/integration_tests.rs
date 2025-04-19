use nelang::lang::{Context, Program, Span, program};

#[test]
fn test_program_execution_sequence() {
    let mut ctx = Context::new();

    // Define some variables
    let inputs = vec![
        // Define variables
        "x = 10\r\n",
        "y = 20\r\n",
        // Use variables in expressions
        "x + y\r\n",
        "x * y\r\n",
        // Define a function
        "add(a, b) = a + b\r\n",
        // Call the function
        "add(x, y)\r\n",
        // Define a more complex function
        "calculate(a, b) = a * b + a\r\n",
        // Call it with expressions
        "calculate(x, y)\r\n",
        // Reassign a variable
        "x = x + 5\r\n",
        // Use the updated variable
        "x\r\n",
        "add(x, y)\r\n",
    ];

    let expected_results = vec![
        "10",  // x = 10
        "20",  // y = 20
        "30",  // x + y
        "200", // x * y
        "Ok!", // fn add(a, b) = a + b
        "30",  // add(x, y)
        "Ok!", // fn calculate(a, b) = a * b + a
        "210", // calculate(x, y) = 10 * 20 + 10
        "15",  // x = x + 5
        "15",  // x
        "35",  // add(x, y) = 15 + 20
    ];

    for (i, (input, expected)) in inputs.iter().zip(expected_results.iter()).enumerate() {
        let span = Span::new(input);
        let result = program(span);

        assert!(
            result.is_ok(),
            "Failed to parse input at step {}: {}",
            i,
            input
        );

        let (_, program) = result.unwrap();

        match program {
            Program::Expression(token) => {
                let eval_result = ctx.evaluate_expression(&token);
                assert!(
                    eval_result.is_ok(),
                    "Failed to evaluate expression at step {}: {}",
                    i,
                    input
                );
                let value = eval_result.unwrap();
                assert_eq!(
                    value.to_string(),
                    *expected,
                    "Unexpected result at step {}: expected {}, got {}",
                    i,
                    expected,
                    value
                );
            }
            Program::Func(token) => {
                ctx.funcs.insert(
                    token.data.ident.data.0.clone(),
                    nelang::lang::Func::Custom(token),
                );
                assert_eq!(
                    "Ok!", *expected,
                    "Function definition should return Ok! at step {}",
                    i
                );
            }
            Program::Var(token) => {
                let eval_result = ctx.evaluate_expression(match &token.data.expr {
                    nelang::lang::VarAssignExpr::Expression(token) => token,
                    nelang::lang::VarAssignExpr::UserInput(_) => unreachable!(),
                });
                assert!(
                    eval_result.is_ok(),
                    "Failed to evaluate variable assignment at step {}: {}",
                    i,
                    input
                );
                let value = eval_result.unwrap();
                ctx.vars.insert(token.data.ident.data.0.clone(), value);
                assert_eq!(
                    value.to_string(),
                    *expected,
                    "Unexpected variable value at step {}: expected {}, got {}",
                    i,
                    expected,
                    value
                );
            }
        }
    }
}

#[test]
fn test_error_propagation() {
    let mut ctx = Context::new();

    // Start with valid operations
    let setup_inputs = vec!["x = 10\r\n", "y = 0\r\n", "test_func(a) = a * 2\r\n"];

    // Apply setup
    for input in setup_inputs {
        let span = Span::new(input);
        let (_, program) = program(span).unwrap();

        match program {
            Program::Var(token) => {
                let value = ctx.evaluate_expression(match &token.data.expr {
                    nelang::lang::VarAssignExpr::Expression(token) => token,
                    nelang::lang::VarAssignExpr::UserInput(_) => unreachable!(),
                }).unwrap();
                ctx.vars.insert(token.data.ident.data.0.clone(), value);
            }
            Program::Func(token) => {
                ctx.funcs.insert(
                    token.data.ident.data.0.clone(),
                    nelang::lang::Func::Custom(token),
                );
            }
            _ => {}
        }
    }

    // Test various error cases
    let error_tests = vec![
        // Division by zero
        ("x / y\r\n", true),
        // Undefined variable
        ("undefined_var\r\n", true),
        // Undefined function
        ("undefined_func(10)\r\n", true),
        // Wrong number of arguments
        ("test_func(x, y)\r\n", true),
        // These should be valid and not produce errors
        ("x + y\r\n", false),
        ("test_func(x)\r\n", false),
    ];

    for (input, should_error) in error_tests {
        let span = Span::new(input);
        let result = program(span).unwrap();

        if let (_, Program::Expression(token)) = result {
            let eval_result = ctx.evaluate_expression(&token);

            if should_error {
                assert!(eval_result.is_err(), "Expected error for input: {}", input);
            } else {
                assert!(eval_result.is_ok(), "Unexpected error for input: {}", input);
            }
        }
    }
}

#[test]
fn test_nested_expressions() {
    let mut ctx = Context::new();

    // Set up some variables and functions
    let setup_inputs = vec![
        "a = 5\r\n",
        "b = 10\r\n",
        "c = 2\r\n",
        "square(x) = x * x\r\n",
        "cube(x) = x * x * x\r\n",
    ];

    // Apply setup
    for input in setup_inputs {
        let span = Span::new(input);
        let (_, program) = program(span).unwrap();

        match program {
            Program::Var(token) => {
                let value = ctx.evaluate_expression(match &token.data.expr {
                    nelang::lang::VarAssignExpr::Expression(token) => token,
                    nelang::lang::VarAssignExpr::UserInput(_) => unreachable!(),
                }).unwrap();
                ctx.vars.insert(token.data.ident.data.0.clone(), value);
            }
            Program::Func(token) => {
                ctx.funcs.insert(
                    token.data.ident.data.0.clone(),
                    nelang::lang::Func::Custom(token),
                );
            }
            _ => {}
        }
    }

    // Test complex nested expressions
    let test_expressions = vec![
        ("(a + b) * c\r\n", "30"),            // (5 + 10) * 2 = 30
        ("square(a + b)\r\n", "225"),         // square(15) = 225
        ("cube(a)\r\n", "125"),               // cube(5) = 125
        ("square(a) + square(b)\r\n", "125"), // 25 + 100 = 125
        ("square(a + c)\r\n", "49"),          // square(7) = 49
        ("cube(a) / square(c)\r\n", "31.25"), // 125 / 4 = 31.25
    ];

    for (input, expected) in test_expressions {
        let span = Span::new(input);
        let (_, program) = program(span).unwrap();

        if let Program::Expression(token) = program {
            let result = ctx.evaluate_expression(&token).unwrap();
            assert_eq!(
                result.to_string(),
                expected,
                "Unexpected result for expression {}: got {}, expected {}",
                input,
                result,
                expected
            );
        }
    }
}
