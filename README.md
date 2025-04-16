# NeLang - Simple Expression Interpreter

NeLang is a lightweight expression-based language implemented in Rust, designed for simple arithmetic operations, variable assignments, and function definitions. It offers a clean REPL interface for interactive calculations.

## Features

- Basic arithmetic operations: addition, subtraction, multiplication, division
- Support for unary operations (both + and -)
- Support for negative numbers and floating-point calculations
- Variable assignment and scoping
- User-defined functions with arguments
- Built-in functions (like `abs()`)
- Error handling for common issues (division by zero, undefined variables, etc.)
- Comparison operators: <, <=, ==, !=, >=, >
- Ternary conditional operations (?:)

## Getting Started

### Prerequisites

- Rust and Cargo (2021 edition or newer)

### Installation

Clone this repository:

```bash
git clone https://github.com/decatile/simple-lang-rs.git
cd simple-lang-rs
```

### Run the Interpreter

Start the REPL interpreter:

```bash
cargo run
```

## Language Guide

### Basic Expressions

Expressions can include numbers, variables, function calls, and arithmetic operations:

```
> 1 + 2
3
> 10 - 5
5
> 3 * 4
12
> 10 / 2
5
```

Both unary plus and minus are supported:

```
> -5
-5
> +5
5
> -+5  # Unary operations can be chained
-5
> +-5
-5
```

NeLang follows standard operator precedence rules:

```
> 1 + 2 * 3
7
> (1 + 2) * 3
9
```

### Variables

Assign values to variables with the `=` operator:

```
> x = 10
10
> y = 20
20
> x + y
30
```

Variables can be reassigned:

```
> x = x + 5
15
> x
15
```

### Functions

Define functions using the format `name(param1, param2, ...) = expression`:

```
> double(x) = x * 2
Ok!
> double(5)
10
```

Functions operate within their own scope and can only access their parameters, not global variables:

```
> base = 10
10
> scale(x) = x * 2 
Ok!
> scale(5)
10
> scale(base)
20
```

Multiple arguments are supported:

```
> sum(a, b) = a + b
Ok!
> sum(5, 10)
15
> product(a, b) = a * b
Ok!
> sum(product(2, 3), product(4, 5))
26
```

### Complex Examples

Combine variables, functions, and expressions:

```
> radius = 5
5
> pi = 3.14159
3.14159
> circle_area(r) = pi * r * r
Ok!
> circle_area(radius)
78.53975
```

Nested function calls and operations:

```
> square(x) = x * x
Ok!
> cube(x) = x * square(x)
Ok!
> cube(3)
27
```

Using built-in functions:

```
> abs(-10)
10
> x = -5
-5
> abs(x) + 10
15
```

### Error Handling

NeLang provides informative error messages:

```
> 10 / 0
DivisionByZero(...)

> undefined_var
UndefinedVar(...)

> undefined_func(5)
UndefinedFunction(...)

> double(1, 2)  # after defining double(x) = x * 2
InvalidFunctionArgc(...)
```

### Special REPL Commands

The interpreter responds to these special commands:

```
> help
```
Displays a list of available commands.

```
> help functions
```
Lists all defined functions (both built-in and user-defined) with their parameter counts.

```
> clear
```
Clears the terminal screen.

```
> exit
```
Exits the interpreter.

## License

This project is open source and available under the [MIT License](LICENSE).

## Contributing

Contributions are welcome! Feel free to submit issues and pull requests.