use std::{
    io::{Write, stdin, stdout},
    time::Instant,
};

use parser::expr;

mod parser {
    use std::{fmt::Debug, rc::Rc};

    use nom::{
        IResult, Parser,
        branch::alt,
        character::complete::{char, digit1},
        combinator::value,
        multi::many0,
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

    fn _expr(input: &str) -> IResult<&str, ExprReturn> {
        (
            alt((
                int.map(|int| ExprReturn {
                    exprs: vec![int],
                    opers: vec![],
                }),
                (lpar, _expr, rpar).map(|(_, ret, _)| ret.simplify()),
            )),
            many0((op, _expr)),
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
        match {
            let t1 = Instant::now();
            let r = expr(buffer.trim());
            (r, Instant::now().duration_since(t1))
        } {
            (Ok(("", value)), d) => {
                println!("{} ({}mcs elapsed)", value.execute(), d.as_micros())
            }
            _ => println!("Whoopsie, you're f*cked up"),
        }
        buffer.clear();
    }
}
