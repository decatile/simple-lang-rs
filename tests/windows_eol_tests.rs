use nelang::lang::{Span, program};

#[test]
fn test_mixed_line_endings() {
    // Test inputs with mixed line endings - each should fail on Windows
    let test_cases = [
        // A mix of Unix and Windows line endings
        "x = 10\ny = 20\r\n",
        "x = 10\r\ny = 20\n",
        // Multiple line endings
        "x = 10\r\n\r\n",
        "x = 10\n\r\n",
        // Extra text after the line ending
        "x = 10\r\ntext",
    ];

    for input in test_cases {
        let span = Span::new(input);
        let result = program(span);

        // These should all have various issues on Windows
        #[cfg(target_os = "windows")]
        assert!(
            result.is_err() || result.unwrap().0.fragment().len() > 0,
            "Expected parsing error or incomplete parse for mixed line endings: {}",
            input
        );
    }
}

#[test]
fn test_windows_eol_handling() {
    // Every complete expression must end with \r\n on Windows
    let expressions = [
        // Simple expressions that should work with \r\n
        "1\r\n",
        "1 + 2\r\n",
        "1 + 2 * 3\r\n",
        "(1 + 2) * 3\r\n",
        // Variable assignments
        "x = 10\r\n",
        "y = x + 5\r\n",
        // Function definitions
        "double(x) = x * 2\r\n",
        "add(a, b) = a + b\r\n",
        // Function calls
        "double(5)\r\n",
        "add(1, 2)\r\n",
    ];

    for expr in expressions {
        let span = Span::new(expr);
        let result = program(span);

        assert!(
            result.is_ok(),
            "Expression should parse successfully with \\r\\n: {}",
            expr
        );

        // Ensure the entire input was consumed
        let (rest, _) = result.unwrap();
        assert_eq!(
            rest.fragment().len(),
            0,
            "Entire input should be consumed: {}",
            expr
        );
    }

    // Now try the same expressions but replace \r\n with just \n
    for expr in expressions {
        let unix_expr = expr.replace("\r\n", "\n");
        let span = Span::new(&unix_expr);
        let result = program(span);

        #[cfg(target_os = "windows")]
        assert!(
            result.is_err(),
            "Expression should not parse with \\n on Windows: {}",
            unix_expr
        );
    }
}

#[test]
fn test_escape_sequence_handling() {
    // Test that \r\n is treated as CR+LF characters, not as escape sequences
    let raw_input = r"1 + 2\r\n"; // This is a raw string with actual backslashes
    let span = Span::new(raw_input);
    let result = program(span);

    // This should fail on all platforms because it contains literal \r\n text, not CR+LF chars
    assert!(
        result.is_err(),
        "Raw string with escape sequences should not parse"
    );

    // Now create a string with actual CR+LF characters
    let proper_input = "1 + 2\r\n"; // This contains actual CR+LF, not escape sequences
    let span = Span::new(proper_input);
    let result = program(span);

    assert!(
        result.is_ok(),
        "String with proper CR+LF should parse successfully"
    );
}

#[test]
fn test_strange_eol_variations() {
    // Test some unusual combinations
    let test_cases = [
        // Extra whitespace after expression but before CRLF
        ("1 + 2 \r\n", true),
        ("1 + 2\t\r\n", true),
        // Multiple CRLFs
        ("1 + 2\r\n\r\n", true), // Should parse successfully but have remainder
        // Reverse order of CR and LF (which is invalid)
        ("1 + 2\n\r", false),
    ];

    for (input, should_parse_ok) in test_cases {
        let span = Span::new(input);
        let result = program(span);

        if should_parse_ok {
            assert!(
                result.is_ok(),
                "Expected successful parsing for input: {:?}",
                input
            );
        } else {
            #[cfg(target_os = "windows")]
            assert!(
                result.is_err(),
                "Expected parsing error for input: {:?}",
                input
            );
        }
    }
}
