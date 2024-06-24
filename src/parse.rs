/*
vts_parsing: rust parser for the VTS and VTM (including others) files generated by VTOL VR.
Copyright (C) 2024 BlockListed

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use indexmap::IndexMap;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::bytes::complete::take_until1;
use nom::bytes::complete::take_while1;
use nom::character::complete::multispace0;
use nom::character::complete::multispace1;
use nom::character::complete::newline;
use nom::character::complete::space0;
use nom::multi::many0;
use nom::multi::many1;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::separated_pair;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::AsChar;
use nom::IResult;
use nom::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    /// This also includes the original formatting of the float, so that we change the file as
    /// little as possible. You don't need to worry about that though.
    Float(Float),
    Boolean(bool),
    Tuple(Vec<Value>),
    String(String),
    Object(Object),
    Null,
}

impl Value {
    /// Any value which can be put after the equals sign in an object.
    pub fn is_scalar(&self) -> bool {
        use Value::*;
        matches!(self, Number(_) | Float(_) | Boolean(_) | Tuple(_) | String(_) | Null)
    }

    /// An array is an object, whose children are only objects, which all have the same key / name.
    pub fn is_array(&self) -> bool {
        match self {
            Value::Object(Object(o)) => {
                o.iter().fold((true, None), |(acc, name), (k, v)| match v {
                    Value::Object(_) => match name {
                        Some(n) if n == k => (acc, name),
                        None => (acc, Some(k)),
                        _ => (false, None),
                    },
                    _ => (false, None),
                }).0
            },
            _ => false,
        } 
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object(pub Vec<(String, Value)>);

#[derive(Debug, Clone)]
/// This also includes the original formatting.
/// If modified, this will automatically serialize the new value, instead of the saved original.
/// Basically just treat this as a float.
pub struct Float(pub f64, pub(crate) String);

impl Float {
    pub fn new(v: f64) -> Float {
        Float(v, String::new())
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

pub fn parse(vts: &str) -> Value {
    let (title, object) = parse_object(vts).unwrap().1;

    Value::Object(Object(vec![(title.to_string(), Value::Object(object))]))
}

fn parse_object(vts: &str) -> IResult<&str, (&str, Object)> {
    let (vts, title) =
        terminated(take_while1(|c: char| c.is_alpha() || c == '_'), multispace1)(vts)?;

    let fields_parser = many0(
        delimited(
            space0,
            parse_object_field.map(|(t, v)| (t.to_owned(), v)),
            newline,
        ),
    );

    let (vts, fields) = delimited(
        terminated(tag("{"), multispace0),
        fields_parser,
        preceded(multispace0, tag("}")),
    )(vts)?;

    Ok((vts, (title, Object(fields))))
}

fn parse_value(vts: &str) -> IResult<&str, Value> {
    // number first
    Ok(vts
        .parse::<i64>()
        .ok()
        .map(Value::Number)
        // float
        .or_else(|| {
            vts.parse::<f64>()
                .map(|v| Value::Float(Float(v, vts.to_owned())))
                .ok()
        })
        // boolean
        .or(match vts {
            "True" => Some(Value::Boolean(true)),
            "False" => Some(Value::Boolean(false)),
            _ => None,
        })
        .map(|v| ("", v))
        // tuple
        .or_else(|| parse_tuple(vts).ok())
        .or_else(|| {
            if vts.is_empty() {
                return Some(("", Value::Null));
            }

            None
        })
        // string
        .unwrap_or_else(|| ("", Value::String(vts.to_owned()))))
}

fn parse_tuple(vts: &str) -> IResult<&str, Value> {
    let tuple_elems = many1(terminated(
        take_until1(",").and_then(parse_value),
        tuple((tag(","), space0)),
    ))
    .and(take_until1(")").and_then(parse_value));

    delimited(terminated(tag("("), space0), tuple_elems, tag(")"))(vts).map(
        |(vts, (mut before, rest))| {
            before.push(rest);
            (vts, Value::Tuple(before))
        },
    )
}

fn parse_object_field(vts: &str) -> IResult<&str, (&str, Value)> {
    let parse_field = separated_pair::<_, _, _, _, nom::error::Error<&str>, _, _, _>(
        take_while1(|c: char| c.is_alpha() || c == '_'),
        tuple((space0, tag("="), space0)),
        take_until("\n").and_then(parse_value),
    );

    parse_field.or(parse_object.map(|(k, v)| (k, Value::Object(v)))).parse(vts)
}

#[cfg(test)]
mod testing {
    use super::parse;
    use super::parse_tuple;
    use super::Float;
    use super::Value;

    const TEST_STR: &str = include_str!("../amogus testing.vts");

    #[test]
    fn test_parse() {
        eprintln!("{:#?}", parse(TEST_STR))
    }

    #[test]
    fn test_tuple() {
        assert_eq!(
            parse_tuple("(-234.3, 5, 403.3)"),
            Ok((
                "",
                Value::Tuple(vec![
                    Value::Float(Float(-234.3, "-234.3".into())),
                    Value::Number(5),
                    Value::Float(Float(403.3, "403.3".into()))
                ])
            ))
        )
    }
}
