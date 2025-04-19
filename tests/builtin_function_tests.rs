use nelang::lang::{Context, Func, Span, program, EvaluateExpressionError};
use std::rc::Rc;

#[test]
fn test_builtin_function_failure() {
    let mut ctx = Context::new();

    // Add a failing built-in function
    ctx.funcs.insert(
        "fail_func".to_string(),
        Func::Builtin {
            inner: Rc::new(|_args| Err("This function always fails".to_string())),
            argc: 1,
        },
    );

    // Test calling the failing function
    let input = "fail_func(42)\r\n";
    let span = Span::new(input);
    let result = program(span).unwrap();

    if let (_, nelang::lang::Program::Expression(token)) = result {
        let eval_result = ctx.evaluate_expression(&token);

        // Assert that the evaluation fails
        assert!(eval_result.is_err(), "Expected the function to fail");

        // Assert the specific error message
        if let Err(EvaluateExpressionError::BuiltinFunctionError(_, err_msg)) = eval_result {
            assert_eq!(err_msg, "This function always fails");
        } else {
            panic!("Unexpected error type");
        }
    } else {
        panic!("Expected an expression");
    }
}
