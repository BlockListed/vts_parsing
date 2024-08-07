use std::char;

use indexmap::IndexMap;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{char, i64, line_ending, multispace0, not_line_ending, space0};
use nom::combinator::{opt, value};
use nom::error::{context, convert_error, ContextError, ParseError};
use nom::multi::{many0, many1, many_m_n};
use nom::number::complete::double;
use nom::sequence::{delimited, pair, separated_pair, terminated, tuple};
use nom::{IResult, Parser};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    Float(f64),
    Boolean(bool),
    Vector([f64; 3]),
    VectorGroup(Vec<[f64; 3]>),
    String(String),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: String,
    pub values: IndexMap<String, Value>,
    pub nodes: Vec<Node>,
}

impl Node {
    pub fn new(name: String, values: IndexMap<String, Value>, nodes: Vec<Node>) -> Node {
        Node {
            name,
            values,
            nodes,
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
    }

    pub fn nodes_mut(&mut self) -> impl Iterator<Item = &mut Node> {
        self.nodes.iter_mut()
    }

    pub fn get_node(&self, k: &str) -> Option<&Node> {
        self.nodes().find(|n| n.name == k)
    }

    pub fn get_node_mut(&mut self, k: &str) -> Option<&mut Node> {
        self.nodes_mut().find(|n| n.name == k)
    }
}

pub fn parse(vts: &str) -> Node {
    let res = parse_node::<nom::error::VerboseError<&str>>(vts).map_err(|e| match e {
        nom::Err::Error(e) | nom::Err::Failure(e) => convert_error(vts, e),
        _ => e.to_string(),
    });

    match res {
        Ok((_, n)) => n,
        Err(e) => {
            eprintln!("{e}");
            Err::<(), _>(e).unwrap();
            unreachable!();
        }
    }
}

fn parse_bool<'a, E: ParseError<&'a str>>(vts: &'a str) -> IResult<&'a str, bool, E> {
    let true_parser = value(true, tag("True"));
    let false_parser = value(false, tag("False"));

    alt((true_parser, false_parser))(vts)
}

fn parse_number<'a, E: ParseError<&'a str>>(vts: &'a str) -> IResult<&'a str, i64, E> {
    i64(vts)
}

fn parse_float<'a, E: ParseError<&'a str>>(vts: &'a str) -> IResult<&'a str, f64, E> {
    double(vts)
}

fn parse_vector<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    vts: &'a str,
) -> IResult<&'a str, [f64; 3], E> {
    let (vts, components) = context(
        "vector",
        delimited(
            char('('),
            many_m_n(3, 3, terminated(parse_float, opt(pair(char(','), space0)))),
            char(')'),
        ),
    )(vts)?;

    assert_eq!(components.len(), 3, "many_m_n should guarantee length");

    Ok((
        vts,
        components
            .try_into()
            .expect("this should never error we are guaranteed to be the correct length"),
    ))
}

fn parse_vectorgroup<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    vts: &'a str,
) -> IResult<&'a str, Vec<[f64; 3]>, E> {
    context(
        "vector_group",
        many1(terminated(parse_vector, opt(pair(char(';'), space0)))),
    )(vts)
}

fn parse_null<'a, E: ParseError<&'a str>>(vts: &'a str) -> IResult<&'a str, (), E> {
    space0.map(|_| ()).parse(vts)
}

fn parse_string<'a, E: ParseError<&'a str>>(vts: &'a str) -> IResult<&'a str, &'a str, E> {
    // a string is literally anything that's not a line ending
    not_line_ending(vts)
}

fn parse_value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    vts: &'a str,
) -> IResult<&'a str, Value, E> {
    context(
        "value",
        alt((
            terminated(parse_null, line_ending).map(|()| Value::Null),
            terminated(parse_number, line_ending).map(|n| Value::Number(n)),
            terminated(parse_float, line_ending).map(|f| Value::Float(f)),
            terminated(parse_bool, line_ending).map(|b| Value::Boolean(b)),
            terminated(parse_vector, line_ending).map(|v| Value::Vector(v)),
            terminated(parse_vectorgroup, line_ending).map(|vg| Value::VectorGroup(vg)),
            terminated(parse_string, line_ending).map(|s: &str| Value::String(s.into())),
        )),
    )(vts)
}

fn parse_name<'a, E: ParseError<&'a str>>(vts: &'a str) -> IResult<&'a str, &'a str, E> {
    let allowed_special = &[
        '_',
        '-',
        // these two are here because of briefings
        '{',
        '}',
    ];

    take_while1(|c: char| c.is_alphanumeric() || allowed_special.contains(&c))(vts)
}

fn parse_kv<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    vts: &'a str,
) -> IResult<&'a str, (&'a str, Value), E> {
    context(
        "kv",
        separated_pair(parse_name, tuple((space0, char('='), space0)), parse_value),
    )(vts)
}

fn parse_node<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    vts: &'a str,
) -> IResult<&'a str, Node, E> {
    let values = many0(terminated(parse_kv, multispace0));
    let nodes = many0(terminated(parse_node, multispace0));

    let (vts, (name, (values, nodes))) = context(
        "node",
        pair(
            terminated(parse_name, multispace0),
            delimited(
                pair(char('{'), multispace0),
                tuple((values, nodes)),
                char('}'),
            ),
        ),
    )(vts)?;

    Ok((
        vts,
        Node {
            name: name.to_string(),
            values: values
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            nodes,
        },
    ))
}

#[cfg(test)]
mod testing {
    use super::parse;
    use super::parse_vector;

    const TEST_STR: &str = include_str!("../amogus testing.vts");

    const NAVAL_TEST_LAKE_PARSE: &str = include_str!("../StrikeOnNavalTestLake.vts");

    // here because the kv-pair keys contain weird characters
    // which used to cause issues.
    const BREAD_BRIEFING: &str = include_str!("../Bread_Line_Briefing.vts");

    #[test]
    fn test_parse() {
        eprintln!("{:#?}", parse(TEST_STR));
    }

    #[test]
    fn test_naval_lake_parse() {
        eprintln!("{:#?}", parse(NAVAL_TEST_LAKE_PARSE));
    }

    #[test]
    fn test_bread_briefing() {
        eprintln!("{:#?}", parse(BREAD_BRIEFING));
    }

    #[test]
    fn test_tuple() {
        assert_eq!(
            parse_vector::<nom::error::Error<&str>>("(-234.3, 5, 403.3)"),
            Ok(("", [-234.3, 5.0, 403.3,]))
        );
    }

}
