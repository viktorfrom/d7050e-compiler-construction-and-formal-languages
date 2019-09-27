extern crate nom;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::map,
    sequence::{delimited, tuple},
    IResult,
};

#[derive(Debug)]
pub enum Expr {
    Node(Box<Expr>, Box<Expr>, Box<Expr>),
    Num(i32),
    Bool(Bool),
    LogOp(LogOp),
    AriOp(AriOp),
    // Id(String),
    // UnOp(Op, Box<Tree>),
    // Application(Id, vec<Tree>)
}

#[derive(Debug)]
pub enum AriOp {
    Add,
    Sub,
    Mult,
    Div,
}

#[derive(Debug)]
pub enum Bool {
    True,
    False,
}

#[derive(Debug)]
pub enum LogOp {
    And,
    Or,
    Les,
    Gre,
}

fn parse_binop(input: &str) -> IResult<&str, Expr> {
    delimited(
        multispace0,
        alt((
            map(tag("&&"), |_| Expr::LogOp(LogOp::And)),
            map(tag("||"), |_| Expr::LogOp(LogOp::Or)),
            map(tag("<<"), |_| Expr::LogOp(LogOp::Les)),
            map(tag(">>"), |_| Expr::LogOp(LogOp::Gre)),
        )),
        multispace0,
    )(input)
}

fn parse_bool(input: &str) -> IResult<&str, Expr> {
    delimited(
        multispace0,
        alt((
            map(tag("true"), |_| Expr::Bool(Bool::True)),
            map(tag("false"), |_| Expr::Bool(Bool::False)),
        )),
        multispace0,
    )(input)
}

fn parse_op(input: &str) -> IResult<&str, Expr> {
    delimited(
        multispace0,
        alt((
            map(tag("+"), |_| Expr::AriOp(AriOp::Add)),
            map(tag("-"), |_| Expr::AriOp(AriOp::Sub)),
            map(tag("*"), |_| Expr::AriOp(AriOp::Mult)),
            map(tag("/"), |_| Expr::AriOp(AriOp::Div)),
        )),
        multispace0,
    )(input)
}

fn parse_i32(input: &str) -> IResult<&str, Expr> {
    let (substring, digit) = delimited(multispace0, digit1, multispace0)(input)?;

    Ok((substring, Expr::Num(digit.parse::<i32>().unwrap())))
}

fn parse_expr(input: &str) -> IResult<&str, Expr> {
    delimited(
        multispace0,
        alt((
            map(
                tuple((
                    alt((parse_paren, parse_i32, parse_bool)),
                    alt((parse_op, parse_binop)),
                    parse_expr,
                )),
                |(left, operator, right)| {
                    Expr::Node(Box::new(left), Box::new(operator), Box::new(right))
                },
            ),
            parse_bool,
            parse_i32,
            parse_paren,
        )),
        multispace0,
    )(input)
}

fn parse_paren(input: &str) -> IResult<&str, Expr> {
    delimited(
        multispace0,
        delimited(tag("("), parse_expr, tag(")")),
        multispace0,
    )(input)
}

#[derive(Debug)]
pub enum Content {
    Num(i32),
    Add,
}

fn interpreter(input: Expr) -> Content {
    match input {
        Expr::AriOp(Add) => Content::Add,
        Expr::Num(int) => Content::Num(int),
        Expr::Node(left, operator, right) => eval_expr(
            interpreter(*left),
            interpreter(*operator),
            interpreter(*right),
        ),
        _ => (panic!("Invalid input!")),
    }
}

fn eval_expr(left: Content, operator: Content, right: Content) -> Content {
    let l: i32;
    let r: i32;

    match left {
        Content::Num(num) => l = num,
        _ => panic!("Invalid input!"),
    }

    match right {
        Content::Num(num) => r = num,
        _ => panic!("Invalid input!"),
    }

    match operator {
        Content::Add => Content::Num(l + r),
        _ => panic!("Invalid input!"),
    }
}

fn main() {
    // let string = "        11 + 2 -1 / (5     *      3)                 ;";
    // let string = "            true && false >>           true       ;";
    // let string = "((1 + 2) - (1 + 3))";
    // let string = "(1 + (2 - (3)))";
    // let string = "(((1) - 2) + 3)";
    let string = "1 + 2";
    // cat(tag(")")) for att parsa "("                  ")"

    let tree = interpreter(parse_expr(string).unwrap().1);
    println!("{:#?}", tree);
}
