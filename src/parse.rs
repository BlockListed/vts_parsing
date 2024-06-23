use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
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
use nom::IResult;
use nom::AsChar;
use nom::Parser;

#[derive(Debug)]
pub enum Value {
    Number(i64),
    Float(f64),
    Boolean(bool),
    Tuple(Vec<Value>),
    String(String),
    Array(Vec<(String, Value)>),
    // can't be a map, because object fields can appear twice for some ungodly reason
    Object(Vec<(String, Value)>),
    Null,
}

pub fn parse(vts: &str) -> Value {
    let (title, object) = parse_object(vts).unwrap().1;
    
    Value::Object(vec![(title.to_string(), object)])
}

fn parse_object(vts: &str) -> IResult<&str, (&str, Value)> {
    let (vts, title) = terminated(take_while1(|c: char| c.is_alpha() || c == '_'), multispace1)(vts)?;

    let fields_parser = many1(delimited(space0, parse_object_field.map(|(t, v)| (t.to_owned(), v)), newline));

    let (vts, fields) = delimited(terminated(tag("{"), multispace0), fields_parser, preceded(multispace0, tag("}")))(vts)?;

    Ok((vts, (title, Value::Object(fields))))
}

fn parse_array(vts: &str) -> nom::IResult<&str, (&str, Value)> {
    let (vts, title) = terminated(take_while1(|c: char| c.is_alpha() || c == '_'), multispace1)(vts)?;

    let element_parser = many0(terminated(parse_object.map(|(t, v)| (t.to_owned(), v)), multispace1));

    let (vts, elements) = delimited(terminated(tag("{"), multispace0), element_parser, preceded(multispace0, tag("}")))(vts)?;

    Ok((vts, (title, Value::Array(elements))))
}

fn parse_value(vts: &str) -> IResult<&str, Value> {
    // number first
    Ok(vts.parse::<i64>()
        .ok()
        .map(|v| Value::Number(v))
        // float
        .or_else(|| vts.parse::<f64>().map(|v| Value::Float(v)).ok())
        // boolean
        .or_else(|| match vts {
            "True" => Some(Value::Boolean(true)),
            "False" => Some(Value::Boolean(false)),
            _ => None,
        })
        .map(|v| ("", v))
        // tuple
        .or_else(|| {
            delimited(terminated(tag("("), space0), many1(terminated(parse_value, tuple((tag(","), space0)))), tag(")"))(vts).ok().map(|(vts, tuple)| (vts, Value::Tuple(tuple)))
        })
        .or_else(|| {
            if vts == "" {
                return Some(("", Value::Null));
            }

            None
        })
        // string
        .unwrap_or_else(|| ("", Value::String(vts.to_owned())))
    )
}

fn parse_object_field(vts: &str) -> IResult<&str, (&str, Value)> {
    let parse_field = separated_pair::<_, _, _, _, nom::error::Error<&str>, _, _, _>(take_while1(|c: char| c.is_alpha() || c == '_'), tuple((space0, tag("="), space0)), take_until("\n").and_then(parse_value));

    parse_field.or(parse_array).or(parse_object).parse(vts)
}

#[cfg(test)]
mod testing {
    const TEST_STR: &str = include_str!("../amogus testing.vts");

    #[test]
    fn test_parse() {
        eprintln!("{:#?}", super::parse(TEST_STR))
    }
}
