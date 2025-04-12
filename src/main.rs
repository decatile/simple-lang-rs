use std::{
    io::{Write, stdin, stdout},
    iter::repeat_n,
    time::Instant,
};

use nom::Err;
use parser::expr;

mod parser {
    use std::{fmt::Debug, rc::Rc};

    use nom::{
        IResult, Parser,
        branch::alt,
        character::complete::{char, digit1, one_of},
        combinator::value,
        multi::many0,
        sequence::delimited,
    };

    #[derive(Clone, Debug)]
    pub enum Operation {
        Add,
        Sub,
        Mul,
        Div,
    }

    #[derive(Clone, Debug)]
    pub enum Expression {
        Integer(i64),
        Binary(Rc<Expression>, Operation, Rc<Expression>),
    }

    impl Expression {
        pub fn execute(&self) -> i64 {
            match self {
                Expression::Integer(value) => *value,
                Expression::Binary(lhs, op, rhs) => match op {
                    Operation::Add => lhs.execute() + rhs.execute(),
                    Operation::Sub => lhs.execute() - rhs.execute(),
                    Operation::Mul => lhs.execute() * rhs.execute(),
                    Operation::Div => lhs.execute() / rhs.execute(),
                },
            }
        }
    }

    fn int(input: &str) -> IResult<&str, Rc<Expression>> {
        digit1
            .map_res(|int: &str| int.parse())
            .map(|int| Rc::new(Expression::Integer(int)))
            .parse(input)
    }

    fn op(input: &str) -> IResult<&str, Operation> {
        alt((
            value(Operation::Add, char('+')),
            value(Operation::Sub, char('-')),
            value(Operation::Mul, char('*')),
            value(Operation::Div, char('/')),
        ))
        .parse(input)
    }

    fn lpar(input: &str) -> IResult<&str, char> {
        char('(').parse(input)
    }

    fn rpar(input: &str) -> IResult<&str, char> {
        char(')').parse(input)
    }

    struct ExprReturn {
        exprs: Vec<Rc<Expression>>,
        opers: Vec<Operation>,
    }

    impl ExprReturn {
        fn simplify(mut self) -> ExprReturn {
            while let Some(pos) = self
                .opers
                .iter()
                .position(|op| matches!(op, Operation::Mul | Operation::Div))
            {
                let op = self.opers.remove(pos);
                let rhs = self.exprs.remove(pos + 1);
                let lhs = self.exprs[pos].clone();
                self.exprs[pos] = Rc::new(Expression::Binary(lhs, op, rhs));
            }
            while !self.opers.is_empty() {
                let op = self.opers.remove(0);
                let rhs = self.exprs.remove(1);
                let lhs = self.exprs[0].clone();
                self.exprs[0] = Rc::new(Expression::Binary(lhs, op, rhs));
            }
            self
        }

        fn extend(&mut self, mut other: Self) {
            self.exprs.append(&mut other.exprs);
            self.opers.append(&mut other.opers);
        }
    }

    fn _consume_ws(input: &str) -> IResult<&str, ()> {
        many0(one_of(" \t")).map(|_| ()).parse(input)
    }

    fn _delim_space<'a, P: Parser<&'a str, Error = nom::error::Error<&'a str>>>(
        parser: P,
    ) -> impl Parser<&'a str, Output = <P as Parser<&'a str>>::Output, Error = nom::error::Error<&'a str>>
    {
        delimited(_consume_ws, parser, _consume_ws)
    }

    fn _expr(input: &str) -> IResult<&str, ExprReturn> {
        (
            alt((
                _delim_space(int).map(|int| ExprReturn {
                    exprs: vec![int],
                    opers: vec![],
                }),
                (_delim_space(lpar), _expr, _delim_space(rpar)).map(|(_, ret, _)| ret.simplify()),
            )),
            many0((_delim_space(op), _expr)),
        )
            .map(|(mut ret, rets)| {
                for (op, r) in rets {
                    ret.opers.push(op);
                    ret.extend(r);
                }
                ret
            })
            .parse(input)
    }

    pub fn expr(input: &str) -> IResult<&str, Rc<Expression>> {
        _expr
            .map(|ret| ret.simplify().exprs[0].clone())
            .parse(input)
    }
}

fn main() {
    let mut buffer = String::new();
    loop {
        print!("ready> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut buffer).unwrap();
        let trimmed = buffer.trim();
        let t1 = Instant::now();
        match expr(trimmed) {
            Ok(("", value)) => {
                let d = Instant::now().duration_since(t1);
                println!("{} ({}us elapsed)", value.execute(), d.as_micros())
            }
            Ok((rem, _)) => {
                println!(
                    "{}^ Expression ends here",
                    repeat_n(' ', trimmed.len() - rem.len() + "ready> ".len()).collect::<String>()
                )
            }
            Err(Err::Error(err) | Err::Failure(err)) => {
                println!(
                    "{}^ {:?}",
                    repeat_n(' ', trimmed.len() - err.input.len() + "ready> ".len())
                        .collect::<String>(),
                    err.code
                )
            }
            _ => println!("Whoopsie, you're f*cked up"),
        }
        buffer.clear();
    }
}
