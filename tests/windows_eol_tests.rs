use nelang::lang::{Span, program};

#[test]
fn test_mixed_line_endings() {
    // Test inputs with mixed line endings
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

        // With our unified parser, these should always be parseable
        // but some may have remaining content
        assert!(
            result.is_ok(),
            "Expected successful parse for mixed line endings: {}",
            input
        );
    }
}

#[test]
fn test_windows_eol_handling() {
    // Both types of line endings should work on all platforms
    let expressions_crlf = [
        // Simple expressions with CRLF
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

    for expr in expressions_crlf {
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

    // Now try the same expressions but with LF only
    let expressions_lf = [
        // Simple expressions with LF
        "1\n",
        "1 + 2\n",
        "1 + 2 * 3\n",
        "(1 + 2) * 3\n",
        // Variable assignments
        "x = 10\n",
        "y = x + 5\n",
        // Function definitions
        "double(x) = x * 2\n",
        "add(a, b) = a + b\n",
        // Function calls
        "double(5)\n",
        "add(1, 2)\n",
    ];

    for expr in expressions_lf {
        let span = Span::new(expr);
        let result = program(span);

        assert!(
            result.is_ok(),
            "Expression should parse successfully with \\n: {}",
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

    // Also test with LF only
    let unix_input = "1 + 2\n"; // Contains just LF
    let span = Span::new(unix_input);
    let result = program(span);

    assert!(result.is_ok(), "String with LF should parse successfully");
}

#[test]
fn test_strange_eol_variations() {
    // Test some unusual combinations
    let test_cases = [
        // Extra whitespace after expression but before EOL
        ("1 + 2 \r\n", true),
        ("1 + 2 \n", true),
        ("1 + 2\n\r", true),
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
            assert!(
                result.is_err(),
                "Expected parsing error for input: {:?}",
                input
            );
        }
    }
}
