extern crate nom;
use crate::ast::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric0, digit1, multispace0},
    combinator::map,
    multi::many0,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

pub fn parser(input: &str) -> IResult<&str, Vec<Expr>> {
    many0(parse_scope)(input)
}

fn parse_scope(input: &str) -> IResult<&str, Expr> {
    delimited(
        multispace0,
        alt((
            parse_return,
            parse_let,
            parse_if_else,
            parse_if,
            parse_while,
            parse_var_expr,
            parse_fn,
        )),
        multispace0,
    )(input)
}

fn parse_int(input: &str) -> IResult<&str, Expr> {
    let (substring, digit) = delimited(multispace0, digit1, multispace0)(input)?;

    Ok((substring, Expr::Int(digit.parse::<i32>().unwrap())))
}

fn parse_bool(input: &str) -> IResult<&str, Expr> {
    delimited(
        multispace0,
        alt((
            map(tag("true"), |_| Expr::Bool(true)),
            map(tag("false"), |_| Expr::Bool(false)),
        )),
        multispace0,
    )(input)
}

fn parse_op(input: &str) -> IResult<&str, Op> {
    delimited(
        multispace0,
        alt((parse_rel_op, parse_log_op, parse_ass_op, parse_ari_op)),
        multispace0,
    )(input)
}

fn parse_ari_op(input: &str) -> IResult<&str, Op> {
    delimited(
        multispace0,
        alt((
            map(tag("+"), |_| Op::AriOp(AriOp::Add)),
            map(tag("-"), |_| Op::AriOp(AriOp::Sub)),
            map(tag("*"), |_| Op::AriOp(AriOp::Mul)),
            map(tag("/"), |_| Op::AriOp(AriOp::Div)),
        )),
        multispace0,
    )(input)
}

fn parse_ass_op(input: &str) -> IResult<&str, Op> {
    delimited(
        multispace0,
        alt((
            map(tag("="), |_| Op::AssOp(AssOp::Eq)),
            map(tag("+="), |_| Op::AssOp(AssOp::AddEq)),
            map(tag("-="), |_| Op::AssOp(AssOp::SubEq)),
            map(tag("/="), |_| Op::AssOp(AssOp::DivEq)),
            map(tag("*="), |_| Op::AssOp(AssOp::MulEq)),
        )),
        multispace0,
    )(input)
}

fn parse_log_op(input: &str) -> IResult<&str, Op> {
    delimited(
        multispace0,
        alt((
            map(tag("&&"), |_| Op::LogOp(LogOp::And)),
            map(tag("||"), |_| Op::LogOp(LogOp::Or)),
            // map(tag("!"), |_| Expr::LogicOp(LogOp::Not)),
        )),
        multispace0,
    )(input)
}

fn parse_rel_op(input: &str) -> IResult<&str, Op> {
    delimited(
        multispace0,
        alt((
            map(tag("=="), |_| Op::RelOp(RelOp::Eq)),
            map(tag("!="), |_| Op::RelOp(RelOp::Neq)),
            map(tag("<="), |_| Op::RelOp(RelOp::Leq)),
            map(tag(">="), |_| Op::RelOp(RelOp::Geq)),
            map(tag("<"), |_| Op::RelOp(RelOp::Les)),
            map(tag(">"), |_| Op::RelOp(RelOp::Gre)),
        )),
        multispace0,
    )(input)
}

fn parse_bin_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            tuple((
                alt((parse_bool, parse_int, parse_paren, parse_fn_call, parse_var)),
                parse_op,
                parse_bin_expr,
            )),
            |(left, op, right)| Expr::BinExpr(Box::new(left), op, Box::new(right)),
        ),
        parse_bool,
        parse_int,
        parse_paren,
        parse_fn_call,
    ))(input)
}

fn parse_return(input: &str) -> IResult<&str, Expr> {
    let (substring, val) = delimited(
        multispace0,
        preceded(
            tag("return"),
            alt((parse_paren, parse_bin_expr, parse_var_expr, parse_var)),
        ),
        multispace0,
    )(input)?;

    Ok((substring, Expr::Return(Box::new(val))))
}

fn parse_paren(input: &str) -> IResult<&str, Expr> {
    delimited(
        multispace0,
        delimited(
            tag("("),
            alt((parse_bin_expr, parse_var_expr, parse_var)),
            tag(")"),
        ),
        multispace0,
    )(input)
}

fn parse_fn_call(input: &str) -> IResult<&str, Expr> {
    let (substring, (fn_name, args)) = tuple((parse_var, parse_args))(input)?;

    Ok((substring, Expr::FnCall(Box::new(fn_name), args)))
}

fn parse_var(input: &str) -> IResult<&str, Expr> {
    delimited(
        multispace0,
        map(alphanumeric0, |var: &str| Expr::Var(var.to_string())),
        multispace0,
    )(input)
}

fn parse_arg(input: &str) -> IResult<&str, Expr> {
    let (substring, val) = terminated(parse_bin_expr, multispace0)(input)?;

    Ok((substring, val))
}

fn parse_args(input: &str) -> IResult<&str, Vec<Expr>> {
    let (substring, vec) = delimited(
        multispace0,
        delimited(
            tag("("),
            many0(alt((parse_arg, preceded(tag(","), parse_arg)))),
            tag(")"),
        ),
        multispace0,
    )(input)?;

    Ok((substring, vec))
}

fn parse_var_expr(input: &str) -> IResult<&str, Expr> {
    let (substring, (var, op, expr)) = tuple((
        alt((parse_int, parse_bool, parse_var)),
        parse_op,
        alt((parse_bin_expr, parse_var)),
    ))(input)?;

    Ok((substring, Expr::VarExpr(Box::new(var), op, Box::new(expr))))
}

fn parse_block(input: &str) -> IResult<&str, Vec<Expr>> {
    alt((
        delimited(
            terminated(multispace0, tag("{")),
            many0(alt((terminated(parse_scope, tag(";")), parse_return))),
            terminated(multispace0, tag("}")),
        ),
        delimited(
            terminated(multispace0, tag("{")),
            parse_block,
            terminated(multispace0, tag("}")),
        ),
    ))(input)
}

pub fn parse_let(input: &str) -> IResult<&str, Expr> {
    let (substring, (var, var_type, expr)) = tuple((
        terminated(
            preceded(delimited(multispace0, tag("let"), multispace0), parse_var),
            tag(":"),
        ),
        parse_type,
        alt((
            parse_bin_expr,
            preceded(
                delimited(multispace0, tag("="), multispace0),
                alt((parse_var_expr, parse_var)),
            ),
        )),
    ))(input)?;

    Ok((
        substring,
        Expr::Let(Box::new(var), var_type, Box::new(expr)),
    ))
}

fn parse_type(input: &str) -> IResult<&str, Type> {
    delimited(
        multispace0,
        alt((
            map(tag("i32"), |_| Type::Int),
            map(tag("bool"), |_| Type::Bool),
            // map(tag("str"), |_| Type::Str),
            map(tag("()"), |_| Type::Void),
        )),
        multispace0,
    )(input)
}

fn parse_if(input: &str) -> IResult<&str, Expr> {
    let (substring, (cond, block)) = tuple((
        preceded(
            delimited(multispace0, tag("if"), multispace0),
            alt((parse_var_expr, parse_bool, parse_var)),
        ),
        parse_block,
    ))(input)?;

    Ok((substring, Expr::If(Box::new(cond), block)))
}

fn parse_if_else(input: &str) -> IResult<&str, Expr> {
    let (substring, (cond, block1, block2)) = tuple((
        preceded(
            delimited(multispace0, tag("if"), multispace0),
            alt((parse_var_expr, parse_bool, parse_var)),
        ),
        parse_block,
        preceded(
            delimited(multispace0, tag("else"), multispace0),
            parse_block,
        ),
    ))(input)?;

    Ok((substring, Expr::IfElse(Box::new(cond), block1, block2)))
}

fn parse_while(input: &str) -> IResult<&str, Expr> {
    let (substring, (cond, block)) = tuple((
        preceded(
            delimited(multispace0, tag("while"), multispace0),
            alt((parse_bool, parse_var_expr)),
        ),
        parse_block,
    ))(input)?;

    Ok((substring, Expr::While(Box::new(cond), block)))
}

fn parse_param(input: &str) -> IResult<&str, (Expr, Type)> {
    let (substring, (var, var_type)) = tuple((terminated(parse_var, tag(":")), parse_type))(input)?;

    Ok((substring, (var, var_type)))
}

fn parse_params(input: &str) -> IResult<&str, Vec<(Expr, Type)>> {
    let (substring, val) = delimited(
        multispace0,
        delimited(
            tag("("),
            many0(alt((parse_param, preceded(tag(","), parse_param)))),
            tag(")"),
        ),
        multispace0,
    )(input)?;

    Ok((substring, val))
}
fn parse_fn(input: &str) -> IResult<&str, Expr> {
    let (substring, (var, params, return_type, block)) = tuple((
        preceded(delimited(multispace0, tag("fn"), multispace0), parse_var),
        parse_params,
        preceded(delimited(multispace0, tag("->"), multispace0), parse_type),
        parse_block,
    ))(input)?;

    Ok((
        substring,
        Expr::Fn(Box::new(var), params, return_type, block),
    ))
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn test_parse_int() {
        assert_eq!(parse_int("1"), Ok(("", Expr::Int(1))));
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_bool("false"), Ok(("", Expr::Bool(false))));
        assert_eq!(parse_bool("true"), Ok(("", Expr::Bool(true))));
    }

    #[test]
    fn test_parse_type() {
        assert_eq!(parse_type("i32"), Ok(("", Type::Int)));
        assert_eq!(parse_type("bool"), Ok(("", Type::Bool)));
        assert_eq!(parse_type("()"), Ok(("", Type::Void)));
    }

    #[test]
    fn test_parse_op() {
        assert_eq!(parse_op("+"), Ok(("", Op::AriOp(AriOp::Add))));
        assert_eq!(parse_op("="), Ok(("", Op::AssOp(AssOp::Eq))));
        assert_eq!(parse_op("&&"), Ok(("", Op::LogOp(LogOp::And))));
        assert_eq!(parse_op(">"), Ok(("", Op::RelOp(RelOp::Gre))));
    }

    #[test]
    fn test_parse_ari_op() {
        assert_eq!(parse_ari_op("+"), Ok(("", Op::AriOp(AriOp::Add))));
        assert_eq!(parse_ari_op("-"), Ok(("", Op::AriOp(AriOp::Sub))));
        assert_eq!(parse_ari_op("*"), Ok(("", Op::AriOp(AriOp::Mul))));
        assert_eq!(parse_ari_op("/"), Ok(("", Op::AriOp(AriOp::Div))));
    }

    #[test]
    fn test_parse_ass_op() {
        assert_eq!(parse_ass_op("="), Ok(("", Op::AssOp(AssOp::Eq))));
        assert_eq!(parse_ass_op("+="), Ok(("", Op::AssOp(AssOp::AddEq))));
        assert_eq!(parse_ass_op("-="), Ok(("", Op::AssOp(AssOp::SubEq))));
        assert_eq!(parse_ass_op("/="), Ok(("", Op::AssOp(AssOp::DivEq))));
        assert_eq!(parse_ass_op("*="), Ok(("", Op::AssOp(AssOp::MulEq))));
    }

    #[test]
    fn test_parse_log_op() {
        assert_eq!(parse_log_op("&&"), Ok(("", Op::LogOp(LogOp::And))));
        assert_eq!(parse_log_op("||"), Ok(("", Op::LogOp(LogOp::Or))));
    }

    #[test]
    fn test_parse_rel_op() {
        assert_eq!(parse_rel_op("=="), Ok(("", Op::RelOp(RelOp::Eq))));
        assert_eq!(parse_rel_op("!="), Ok(("", Op::RelOp(RelOp::Neq))));
        assert_eq!(parse_rel_op("<"), Ok(("", Op::RelOp(RelOp::Les))));
        assert_eq!(parse_rel_op(">"), Ok(("", Op::RelOp(RelOp::Gre))));
        assert_eq!(parse_rel_op("<="), Ok(("", Op::RelOp(RelOp::Leq))));
        assert_eq!(parse_rel_op(">="), Ok(("", Op::RelOp(RelOp::Geq))));
    }

    #[test]
    fn test_parse_bin_expr() {
        assert_eq!(parse_bin_expr("false"), Ok(("", Expr::Bool(false))));
        assert_eq!(parse_bin_expr("1"), Ok(("", Expr::Int(1))));
        assert_eq!(parse_bin_expr("(1)"), Ok(("", Expr::Int(1))));
        assert_eq!(
            parse_bin_expr("1 + 2"),
            Ok((
                "",
                Expr::BinExpr(
                    Box::new(Expr::Int(1)),
                    Op::AriOp(AriOp::Add),
                    Box::new(Expr::Int(2)),
                )
            ))
        );
    }

    #[test]
    fn test_parse_return() {
        assert_eq!(
            parse_return("return true"),
            Ok(("", Expr::Return(Box::new(Expr::Bool(true)))))
        );
        assert_eq!(
            parse_return("return false"),
            Ok(("", Expr::Return(Box::new(Expr::Bool(false)))))
        );
        assert_eq!(
            parse_return("return 1"),
            Ok(("", Expr::Return(Box::new(Expr::Int(1)))))
        );
        assert_eq!(
            parse_return("return a"),
            Ok(("", Expr::Return(Box::new(Expr::Var("a".to_string())))))
        );
        assert_eq!(
            parse_return("return a + b"),
            Ok((
                "",
                Expr::Return(Box::new(Expr::VarExpr(
                    Box::new(Expr::Var("a".to_string())),
                    Op::AriOp(AriOp::Add),
                    Box::new(Expr::Var("b".to_string())),
                )))
            ))
        );
        assert_eq!(
            parse_return("return a + 1"),
            Ok((
                "",
                Expr::Return(Box::new(Expr::BinExpr(
                    Box::new(Expr::Var("a".to_string())),
                    Op::AriOp(AriOp::Add),
                    Box::new(Expr::Int(1)),
                )))
            ))
        );
        assert_eq!(
            parse_return("return testfn(1,false,3)"),
            Ok((
                "",
                Expr::Return(Box::new(Expr::FnCall(
                    Box::new(Expr::Var("testfn".to_string())),
                    vec![Expr::Int(1), Expr::Bool(false), Expr::Int(3)]
                )))
            ))
        );
    }

    #[test]
    fn test_parse_paren() {
        assert_eq!(parse_paren("(1)"), Ok(("", Expr::Int(1))));
        assert_eq!(parse_paren("((1))"), Ok(("", Expr::Int(1))));
    }

    #[test]
    fn test_parse_var() {
        assert_eq!(parse_var("a"), Ok(("", Expr::Var("a".to_string()))));
    }

    #[test]
    fn test_parse_arg() {
        assert_eq!(parse_arg("1"), Ok(("", Expr::Int(1))));
    }

    #[test]
    fn test_parse_args() {
        assert_eq!(
            parse_args("(1, 2, 3)"),
            Ok(("", vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)]))
        );
        assert_eq!(
            parse_args("(1, true, 3)"),
            Ok(("", vec![Expr::Int(1), Expr::Bool(true), Expr::Int(3)]))
        );
    }

    #[test]
    fn test_parse_param() {
        assert_eq!(
            parse_param("a:i32"),
            Ok(("", (Expr::Var("a".to_string()), Type::Int)))
        );
        assert_eq!(
            parse_param("a:bool"),
            Ok(("", (Expr::Var("a".to_string()), Type::Bool)))
        );
    }

    #[test]
    fn test_parse_params() {
        assert_eq!(
            parse_params("(a: i32, b: bool)"),
            Ok((
                "",
                vec![
                    (Expr::Var("a".to_string()), Type::Int),
                    (Expr::Var("b".to_string()), Type::Bool)
                ]
            ))
        );
    }

    #[test]
    fn test_parse_fn_call() {
        assert_eq!(
            parse_fn_call("testfn(1,2,3)"),
            Ok((
                "",
                Expr::FnCall(
                    Box::new(Expr::Var("testfn".to_string())),
                    vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)]
                )
            ))
        );
        assert_eq!(
            parse_fn_call("testfn(1,false,3)"),
            Ok((
                "",
                Expr::FnCall(
                    Box::new(Expr::Var("testfn".to_string())),
                    vec![Expr::Int(1), Expr::Bool(false), Expr::Int(3)]
                )
            ))
        );
    }
    #[test]
    fn test_parse_var_expr() {
        assert_eq!(
            parse_var_expr("a = 1"),
            Ok((
                "",
                Expr::VarExpr(
                    Box::new(Expr::Var("a".to_string())),
                    Op::AssOp(AssOp::Eq),
                    Box::new(Expr::Int(1)),
                )
            ))
        );
        assert_eq!(
            parse_var_expr("a && b"),
            Ok((
                "",
                Expr::VarExpr(
                    Box::new(Expr::Var("a".to_string())),
                    Op::LogOp(LogOp::And),
                    Box::new(Expr::Var("b".to_string())),
                )
            ))
        );
        assert_eq!(
            parse_var_expr("a || b"),
            Ok((
                "",
                Expr::VarExpr(
                    Box::new(Expr::Var("a".to_string())),
                    Op::LogOp(LogOp::Or),
                    Box::new(Expr::Var("b".to_string())),
                )
            ))
        );
        assert_eq!(
            parse_var_expr("a == 1"),
            Ok((
                "",
                Expr::VarExpr(
                    Box::new(Expr::Var("a".to_string())),
                    Op::RelOp(RelOp::Eq),
                    Box::new(Expr::Int(1)),
                )
            ))
        );
        assert_eq!(
            parse_var_expr("a != a"),
            Ok((
                "",
                Expr::VarExpr(
                    Box::new(Expr::Var("a".to_string())),
                    Op::RelOp(RelOp::Neq),
                    Box::new(Expr::Var("a".to_string())),
                )
            ))
        );
    }

    #[test]
    fn test_parse_let() {
        assert_eq!(
            parse_let("let a: i32 = 1"),
            Ok((
                "",
                Expr::Let(
                    Box::new(Expr::Var("a".to_string())),
                    Type::Int,
                    Box::new(Expr::BinExpr(
                        Box::new(Expr::Var("".to_string())),
                        Op::AssOp(AssOp::Eq),
                        Box::new(Expr::Int(1))
                    ))
                ),
            ))
        );
        assert_eq!(
            parse_let("let a: i32 = b"),
            Ok((
                "",
                Expr::Let(
                    Box::new(Expr::Var("a".to_string())),
                    Type::Int,
                    Box::new(Expr::Var("b".to_string()))
                ),
            ))
        );
        assert_eq!(
            parse_let("let a: bool = b && c"),
            Ok((
                "",
                Expr::Let(
                    Box::new(Expr::Var("a".to_string())),
                    Type::Bool,
                    Box::new(Expr::VarExpr(
                        Box::new(Expr::Var("b".to_string())),
                        Op::LogOp(LogOp::And),
                        Box::new(Expr::Var("c".to_string())),
                    ))
                ),
            ))
        );
        assert_eq!(
            parse_let("let a: i32 = b + c"),
            Ok((
                "",
                Expr::Let(
                    Box::new(Expr::Var("a".to_string())),
                    Type::Int,
                    Box::new(Expr::VarExpr(
                        Box::new(Expr::Var("b".to_string())),
                        Op::AriOp(AriOp::Add),
                        Box::new(Expr::Var("c".to_string())),
                    ))
                ),
            ))
        );
        assert_eq!(
            parse_let("let a: bool = true"),
            Ok((
                "",
                Expr::Let(
                    Box::new(Expr::Var("a".to_string())),
                    Type::Bool,
                    Box::new(Expr::BinExpr(
                        Box::new(Expr::Var("".to_string())),
                        Op::AssOp(AssOp::Eq),
                        Box::new(Expr::Bool(true))
                    ))
                ),
            ))
        );
    }
    #[test]
    fn test_parse_block() {
        assert_eq!(
            parse_block("{return 1}"),
            Ok(("", vec![Expr::Return(Box::new(Expr::Int(1)))]))
        );
        assert_eq!(
            parse_block("{{return 1}}"),
            Ok(("", vec![Expr::Return(Box::new(Expr::Int(1)))]))
        );
        assert_eq!(
            parse_block("{let a: i32 = 1; return 1}"),
            Ok((
                "",
                vec![
                    Expr::Let(
                        Box::new(Expr::Var("a".to_string())),
                        Type::Int,
                        Box::new(Expr::BinExpr(
                            Box::new(Expr::Var("".to_string())),
                            Op::AssOp(AssOp::Eq),
                            Box::new(Expr::Int(1))
                        ))
                    ),
                    Expr::Return(Box::new(Expr::Int(1)))
                ]
            ))
        );
        assert_eq!(
            parse_block("{let a: bool = true; return a}"),
            Ok((
                "",
                vec![
                    Expr::Let(
                        Box::new(Expr::Var("a".to_string())),
                        Type::Bool,
                        Box::new(Expr::BinExpr(
                            Box::new(Expr::Var("".to_string())),
                            Op::AssOp(AssOp::Eq),
                            Box::new(Expr::Bool(true))
                        ))
                    ),
                    Expr::Return(Box::new(Expr::Var("a".to_string())))
                ]
            ))
        );
    }
    #[test]
    fn test_parse_if() {
        assert_eq!(
            parse_if("if true {return 1}"),
            Ok((
                "",
                Expr::If(
                    Box::new(Expr::Bool(true)),
                    vec![Expr::Return(Box::new(Expr::Int(1)))]
                )
            ))
        );
        assert_eq!(
            parse_if("if a {return 1}"),
            Ok((
                "",
                Expr::If(
                    Box::new(Expr::Var("a".to_string())),
                    vec![Expr::Return(Box::new(Expr::Int(1)))]
                )
            ))
        );
        assert_eq!(
            parse_if("if a == b {return 1}"),
            Ok((
                "",
                Expr::If(
                    Box::new(Expr::VarExpr(
                        Box::new(Expr::Var("a".to_string())),
                        Op::RelOp(RelOp::Eq),
                        Box::new(Expr::Var("b".to_string()))
                    )),
                    vec![Expr::Return(Box::new(Expr::Int(1)))]
                ),
            ))
        );
    }
    #[test]
    fn test_parse_if_else() {
        assert_eq!(
            parse_if_else("if true {return 1} else {return 1}"),
            Ok((
                "",
                Expr::IfElse(
                    Box::new(Expr::Bool(true)),
                    vec![Expr::Return(Box::new(Expr::Int(1)))],
                    vec![Expr::Return(Box::new(Expr::Int(1)))],
                )
            ))
        );
        assert_eq!(
            parse_if_else("if a {return 1} else {return 1}"),
            Ok((
                "",
                Expr::IfElse(
                    Box::new(Expr::Var("a".to_string())),
                    vec![Expr::Return(Box::new(Expr::Int(1)))],
                    vec![Expr::Return(Box::new(Expr::Int(1)))],
                )
            ))
        );

        assert_eq!(
            parse_if_else("if a == b {return 1} else {return 1}"),
            Ok((
                "",
                Expr::IfElse(
                    Box::new(Expr::VarExpr(
                        Box::new(Expr::Var("a".to_string())),
                        Op::RelOp(RelOp::Eq),
                        Box::new(Expr::Var("b".to_string()))
                    )),
                    vec![Expr::Return(Box::new(Expr::Int(1)))],
                    vec![Expr::Return(Box::new(Expr::Int(1)))],
                )
            ))
        );
    }
    #[test]
    fn test_parse_while() {
        assert_eq!(
            parse_while("while false {return true}"),
            Ok((
                "",
                Expr::While(
                    Box::new(Expr::Bool(false)),
                    vec![Expr::Return(Box::new(Expr::Bool(true)))]
                )
            ))
        );
        assert_eq!(
            parse_while("while a && b {return 1}"),
            Ok((
                "",
                Expr::While(
                    Box::new(Expr::VarExpr(
                        Box::new(Expr::Var("a".to_string())),
                        Op::LogOp(LogOp::And),
                        Box::new(Expr::Var("b".to_string()))
                    )),
                    vec![Expr::Return(Box::new(Expr::Int(1)))]
                ),
            ))
        );
    }
    #[test]
    fn test_parse_fn() {
        assert_eq!(
            parse_fn("fn testfn(a: i32) -> () { return 1 }"),
            Ok((
                "",
                Expr::Fn(
                    Box::new(Expr::Var("testfn".to_string())),
                    vec![(Expr::Var("a".to_string()), Type::Int)],
                    Type::Void,
                    vec![Expr::Return(Box::new(Expr::Int(1)))]
                ),
            ))
        );
        assert_eq!(
            parse_fn("fn testfn(a: bool) -> i32 { if a { let b: i32 = 1; return b};}"),
            Ok((
                "",
                Expr::Fn(
                    Box::new(Expr::Var("testfn".to_string())),
                    vec![(Expr::Var("a".to_string()), Type::Bool)],
                    Type::Int,
                    vec![Expr::If(
                        Box::new(Expr::Var("a".to_string())),
                        vec![
                            Expr::Let(
                                Box::new(Expr::Var("b".to_string())),
                                Type::Int,
                                Box::new(Expr::BinExpr(
                                    Box::new(Expr::Var("".to_string())),
                                    Op::AssOp(AssOp::Eq),
                                    Box::new(Expr::Int(1))
                                ))
                            ),
                            Expr::Return(Box::new(Expr::Var("b".to_string())))
                        ]
                    )]
                ),
            ))
        );
    }

    #[test]
    fn test_parser() {
        assert_eq!(
            parser(
                "        
                fn testfn1(a: bool) -> i32 {
                    let c: i32 = (((2)));
                    if a {
                        let b: i32 = 1;
                        return b
                    } else {
                        return (c)
                    };
                }

                fn testfn2() -> i32 {
                    {{{ return testfn1(true); }}}
                }

                fn testfn3(b: bool, c: bool) -> i32 {
                    let d: bool = b && c;
                    let n: i32 = 0;
                    while d == true {
                        n += 1;
                        d = false;
                    };
                    return n;    
                }

                fn main() -> i32 {
                    let a: i32 = testfn2(); 
                    let b: i32 = testfn3(true, true);
                    return a + b
                }
                "
            ),
            Ok((
                "",
                vec![
                    Expr::Fn(
                        Box::new(Expr::Var("testfn1".to_string())),
                        vec![(Expr::Var("a".to_string()), Type::Bool)],
                        Type::Int,
                        vec![
                            Expr::Let(
                                Box::new(Expr::Var("c".to_string())),
                                Type::Int,
                                Box::new(Expr::BinExpr(
                                    Box::new(Expr::Var("".to_string())),
                                    Op::AssOp(AssOp::Eq),
                                    Box::new(Expr::Int(2)),
                                )),
                            ),
                            Expr::IfElse(
                                Box::new(Expr::Var("a".to_string())),
                                vec![
                                    Expr::Let(
                                        Box::new(Expr::Var("b".to_string())),
                                        Type::Int,
                                        Box::new(Expr::BinExpr(
                                            Box::new(Expr::Var("".to_string())),
                                            Op::AssOp(AssOp::Eq),
                                            Box::new(Expr::Int(1)),
                                        )),
                                    ),
                                    Expr::Return(Box::new(Expr::Var("b".to_string()))),
                                ],
                                vec![Expr::Return(Box::new(Expr::Var("c".to_string())))],
                            ),
                        ],
                    ),
                    Expr::Fn(
                        Box::new(Expr::Var("testfn2".to_string())),
                        vec![],
                        Type::Int,
                        vec![Expr::Return(Box::new(Expr::FnCall(
                            Box::new(Expr::Var("testfn1".to_string())),
                            vec![Expr::Bool(true)],
                        )))],
                    ),
                    Expr::Fn(
                        Box::new(Expr::Var("testfn3".to_string())),
                        vec![
                            (Expr::Var("b".to_string()), Type::Bool),
                            (Expr::Var("c".to_string()), Type::Bool),
                        ],
                        Type::Int,
                        vec![
                            Expr::Let(
                                Box::new(Expr::Var("d".to_string())),
                                Type::Bool,
                                Box::new(Expr::VarExpr(
                                    Box::new(Expr::Var("b".to_string())),
                                    Op::LogOp(LogOp::And),
                                    Box::new(Expr::Var("c".to_string())),
                                )),
                            ),
                            Expr::Let(
                                Box::new(Expr::Var("n".to_string())),
                                Type::Int,
                                Box::new(Expr::BinExpr(
                                    Box::new(Expr::Var("".to_string())),
                                    Op::AssOp(AssOp::Eq),
                                    Box::new(Expr::Int(0)),
                                )),
                            ),
                            Expr::While(
                                Box::new(Expr::VarExpr(
                                    Box::new(Expr::Var("d".to_string())),
                                    Op::RelOp(RelOp::Eq),
                                    Box::new(Expr::Bool(true)),
                                )),
                                vec![
                                    Expr::VarExpr(
                                        Box::new(Expr::Var("n".to_string())),
                                        Op::AssOp(AssOp::AddEq),
                                        Box::new(Expr::Int(1)),
                                    ),
                                    Expr::VarExpr(
                                        Box::new(Expr::Var("d".to_string())),
                                        Op::AssOp(AssOp::Eq),
                                        Box::new(Expr::Bool(false)),
                                    ),
                                ],
                            ),
                            Expr::Return(Box::new(Expr::Var("n".to_string()))),
                        ],
                    ),
                    Expr::Fn(
                        Box::new(Expr::Var("main".to_string())),
                        vec![],
                        Type::Int,
                        vec![
                            Expr::Let(
                                Box::new(Expr::Var("a".to_string())),
                                Type::Int,
                                Box::new(Expr::BinExpr(
                                    Box::new(Expr::Var("".to_string())),
                                    Op::AssOp(AssOp::Eq),
                                    Box::new(Expr::FnCall(
                                        Box::new(Expr::Var("testfn2".to_string())),
                                        vec![],
                                    )),
                                )),
                            ),
                            Expr::Let(
                                Box::new(Expr::Var("b".to_string())),
                                Type::Int,
                                Box::new(Expr::BinExpr(
                                    Box::new(Expr::Var("".to_string())),
                                    Op::AssOp(AssOp::Eq),
                                    Box::new(Expr::FnCall(
                                        Box::new(Expr::Var("testfn3".to_string())),
                                        vec![Expr::Bool(true), Expr::Bool(true)],
                                    )),
                                )),
                            ),
                            Expr::Return(Box::new(Expr::VarExpr(
                                Box::new(Expr::Var("a".to_string())),
                                Op::AriOp(AriOp::Add),
                                Box::new(Expr::Var("b".to_string())),
                            ))),
                        ],
                    ),
                ]
            ))
        );
    }
}
