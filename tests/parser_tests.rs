use nelang::lang::{Context, Program, Span, program};

fn parse_and_evaluate(input: &str) -> Result<String, String> {
    let span = Span::new(input);
    let result = program(span);

    match result {
        Ok((_, program)) => {
            let mut ctx = Context::new();
            match program {
                Program::Expression(token) => match ctx.evaluate_expression(&token) {
                    Ok(result) => Ok(result.to_string()),
                    Err(err) => Err(format!("{:?}", err)),
                },
                Program::Func(token) => {
                    ctx.funcs.insert(
                        token.data.ident.data.0.clone(),
                        nelang::lang::Func::Custom(token),
                    );
                    Ok("Ok!".to_string())
                }
                Program::Var(token) => match ctx.evaluate_expression(&token.data.expr) {
                    Ok(result) => {
                        ctx.vars.insert(token.data.ident.data.0.clone(), result);
                        Ok(result.to_string())
                    }
                    Err(err) => Err(format!("{:?}", err)),
                },
            }
        }
        Err(err) => Err(format!("{:?}", err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        // All inputs end with \r\n as required
        assert_eq!(parse_and_evaluate("1 + 2\r\n"), Ok("3".to_string()));
        assert_eq!(parse_and_evaluate("10 - 5\r\n"), Ok("5".to_string()));
        assert_eq!(parse_and_evaluate("2 * 3\r\n"), Ok("6".to_string()));
        assert_eq!(parse_and_evaluate("10 / 2\r\n"), Ok("5".to_string()));
    }

    #[test]
    fn test_complex_expressions() {
        assert_eq!(parse_and_evaluate("1 + 2 * 3\r\n"), Ok("7".to_string()));
        assert_eq!(parse_and_evaluate("(1 + 2) * 3\r\n"), Ok("9".to_string()));
        assert_eq!(parse_and_evaluate("10 - 5 / 5\r\n"), Ok("9".to_string()));
        assert_eq!(parse_and_evaluate("-5 + 10\r\n"), Ok("5".to_string()));
        assert_eq!(parse_and_evaluate("+5 - 2\r\n"), Ok("3".to_string()));
        assert_eq!(parse_and_evaluate("+-5\r\n"), Ok("-5".to_string()));
        assert_eq!(parse_and_evaluate("-+5\r\n"), Ok("-5".to_string()));
    }

    #[test]
    fn test_variable_assignment() {
        let mut ctx = Context::new();

        // Test variable assignment
        let input = "x = 10\r\n";
        let span = Span::new(input);
        let result = program(span).unwrap();

        if let (_, Program::Var(token)) = result {
            let result = ctx.evaluate_expression(&token.data.expr).unwrap();
            ctx.vars.insert(token.data.ident.data.0.clone(), result);
            assert_eq!(result, 10.0);
            assert_eq!(ctx.vars.get("x"), Some(&10.0));
        } else {
            panic!("Expected variable assignment");
        }

        // Test using the variable
        let input = "x + 5\r\n";
        let span = Span::new(input);
        let result = program(span).unwrap();

        if let (_, Program::Expression(token)) = result {
            let result = ctx.evaluate_expression(&token).unwrap();
            assert_eq!(result, 15.0);
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_function_definition_and_call() {
        let mut ctx = Context::new();

        // Define function - corrected without "fn" prefix
        let input = "double(x) = x * 2\r\n";
        let span = Span::new(input);
        let result = program(span).unwrap();

        if let (_, Program::Func(token)) = result {
            ctx.funcs.insert(
                token.data.ident.data.0.clone(),
                nelang::lang::Func::Custom(token),
            );
        } else {
            panic!("Expected function definition");
        }

        // Call function
        let input = "double(5)\r\n";
        let span = Span::new(input);
        let result = program(span).unwrap();

        if let (_, Program::Expression(token)) = result {
            let result = ctx.evaluate_expression(&token).unwrap();
            assert_eq!(result, 10.0);
        } else {
            panic!("Expected function call");
        }
    }

    #[test]
    fn test_built_in_function() {
        // Test the abs built-in function
        assert_eq!(parse_and_evaluate("abs(-5)\r\n"), Ok("5".to_string()));
        assert_eq!(parse_and_evaluate("abs(5)\r\n"), Ok("5".to_string()));
    }

    #[test]
    fn test_error_handling() {
        // Division by zero
        let result = parse_and_evaluate("10 / 0\r\n");
        assert!(result.is_err());

        // Undefined variable
        let result = parse_and_evaluate("undefined_var + 5\r\n");
        assert!(result.is_err());

        // Undefined function
        let result = parse_and_evaluate("undefined_func(5)\r\n");
        assert!(result.is_err());

        // Invalid function argument count
        let mut ctx = Context::new();
        let input = "double(x) = x * 2\r\n";
        let span = Span::new(input);
        let result = program(span).unwrap();

        if let (_, Program::Func(token)) = result {
            ctx.funcs.insert(
                token.data.ident.data.0.clone(),
                nelang::lang::Func::Custom(token),
            );
        }

        let input = "double(5, 10)\r\n";
        let span = Span::new(input);
        let result = program(span).unwrap();

        if let (_, Program::Expression(token)) = result {
            let result = ctx.evaluate_expression(&token);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_trailing_cr_lf() {
        // Test that inputs can end with either \r\n or \n
        let input_without_eol = "1 + 2";
        let span = Span::new(input_without_eol);
        let result = program(span);

        // This should fail on all platforms as there's no EOL
        assert!(result.is_err());

        // Test with CRLF
        let input_with_crlf = "1 + 2\r\n";
        let span = Span::new(input_with_crlf);
        let result = program(span);

        assert!(result.is_ok(), "Input with CRLF should be valid");

        // Test with LF
        let input_with_lf = "1 + 2\n";
        let span = Span::new(input_with_lf);
        let result = program(span);

        assert!(result.is_ok(), "Input with LF should be valid");
    }

    #[test]
    fn test_ternary_operations() {
        // Basic ternary operations
        assert_eq!(parse_and_evaluate("1 ? 2 : 3\r\n"), Ok("2".to_string()));
        assert_eq!(parse_and_evaluate("0 ? 2 : 3\r\n"), Ok("3".to_string()));
        
        // Ternary with expressions
        assert_eq!(parse_and_evaluate("(1 + 1) ? 5 : 10\r\n"), Ok("5".to_string()));
        assert_eq!(parse_and_evaluate("(1 - 1) ? 5 : 10\r\n"), Ok("10".to_string()));
        
        // Nested ternary operations
        assert_eq!(parse_and_evaluate("1 ? (0 ? 2 : 3) : 4\r\n"), Ok("3".to_string()));
        assert_eq!(parse_and_evaluate("0 ? 2 : (1 ? 3 : 4)\r\n"), Ok("3".to_string()));
        
        // Ternary with function calls
        assert_eq!(parse_and_evaluate("abs(1) ? 2 : 3\r\n"), Ok("2".to_string()));
        assert_eq!(parse_and_evaluate("abs(-1) ? 2 : 3\r\n"), Ok("2".to_string()));
    }
}
