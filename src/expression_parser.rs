// Parses the following BNF grammar:
// <expr> ::= <not> | <and> | <or> | <regex>
// <or> ::= "OR(" <expr> ("," <expr>)+ ")"
// <and> ::= "AND(" <expr> ("," <expr>)+ ")"
// <not> ::= "NOT(" <expr> ")"

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{char},
    combinator::{map},
    multi::separated_list0,
    sequence::{delimited, terminated},
    IResult,
    Finish
};
use regex::Regex;
use color_eyre::eyre::Result;
use nom::character::complete::multispace0;
use nom::combinator::opt;


#[derive(Debug, Clone)]
pub enum Expr {
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),
    Regex(Regex),
}

pub fn parse(input: &str) -> Result<Expr> {
    let result = parse_expr(input).finish();
    if let Ok((_, expr)) = result {
        Ok(expr)
    } else {
        Err(color_eyre::eyre::eyre!("Failed to parse expression {}", input))
    }
}

// Parses a single expression, which can be a regular expression, a NOT expression, an
// AND expression or an OR expression
fn parse_expr(input: &str) -> IResult<&str, Expr> {
    // An expression can be a regular expression, a NOT expression, an AND expression or an OR expression
    alt((
        parse_not,
        parse_and,
        parse_or,
        parse_regex,
    ))(input)
}

// Parses a NOT expression
fn parse_not(input: &str) -> IResult<&str, Expr> {
    map(
        delimited(
            tag("NOT("),
            parse_expr,
            char(')'),
        ),
        |expr| Expr::Not(Box::new(expr)),
    )(input)
}

// Parses an AND expression
fn parse_and(input: &str) -> IResult<&str, Expr> {
    map(
        delimited(
            tag("AND("),
            separated_list0(
                char(','),
                parse_expr,
            ),
            char(')'),
        ),
        Expr::And,
    )(input)
}

// Parses an OR expression
fn parse_or(input: &str) -> IResult<&str, Expr> {
    map(
        delimited(
            tag("OR("),
            separated_list0(
                char(','),
                parse_expr,
            ),
            char(')'),
        ),
        Expr::Or,
    )(input)
}

// Parses a string that is not a keyword
fn parse_regex(input: &str) -> IResult<&str, Expr> {
    map(
        terminated(
            take_while(|c: char| c != ')' && c != ','),
            opt(multispace0),
        ),
        |s: &str| Expr::Regex(Regex::new(s).expect("Invalid regex")),
    )(input)
}
