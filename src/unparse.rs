use std::fmt::Write;

use crate::{parse::{Float, Object}, Value};

pub fn unparse(v: &Value) -> String {
    assert!(matches!(v, Value::Object(_)), "top level should be object");
    let v = match v {
        Value::Object(v) => v,
        _ => unreachable!(),
    };

    let mut s = String::new();

    assert_eq!(v.0.len(), 1, "only one top level object allowed");

    match &v.0[0] {
        (k, Value::Object(o)) => unparse_object(0, &k, &o, &mut s),
        _ => panic!("invalid data"),
    };

    s
}

// number, float, bool, tuple or null
fn unparse_value(v: &Value, output: &mut String) -> Option<()> {
    match v {
        Value::Number(v) => {
            output.write_fmt(format_args!("{}", v)).unwrap();

            Some(())
        }
        Value::Float(v) => {
            unparse_float(v, output);

            Some(())
        }
        Value::Boolean(v) => {
            if *v {
                output.write_str("True").unwrap();
            } else {
                output.write_str("False").unwrap();
            }

            Some(())
        }
        Value::Tuple(v) => {
            unparse_tuple(v, output);

            Some(())
        }
        Value::Null => Some(()),
        Value::String(v) => {
            output.write_str(v).unwrap();

            Some(())
        }
        _ => None,
    }
}

fn unparse_float(v: &Float, output: &mut String) {
    let Float(v, original) = v;
    if original
        .parse::<f64>()
        .ok()
        .is_some_and(|parsed| parsed.eq(v))
    {
        output.write_str(original).unwrap();
    } else {
        output.write_fmt(format_args!("{}", v)).unwrap();
    }
}

fn indent(depth: u32, output: &mut String) {
    (0..depth).for_each(|_| {
        output.write_char('\t').unwrap();
    });
}

fn unparse_object(indent_depth: u32, k: &str, v: &Object, output: &mut String) {
    indent(indent_depth, output);
    output.write_str(k).unwrap();
    output.write_char('\n').unwrap();
    indent(indent_depth, output);
    output.write_char('{').unwrap();
    output.write_char('\n').unwrap();
    for (k, v) in v.0.iter() {
        match v {
            v if v.is_scalar() => {
                indent(indent_depth+1, output);

                output.write_str(&k).unwrap();
                output.write_str(" = ").unwrap();

                unparse_value(v, output);
                output.write_char('\n').unwrap();
            },
            Value::Object(o) => unparse_object(indent_depth + 1, k, o, output),
            // first case should match everything that isn't an object
            _ => unreachable!(),
        }
    }
    indent(indent_depth, output);
    output.write_str("}\n").unwrap();
}

fn unparse_tuple(v: &[Value], output: &mut String) {
    output.write_char('(').unwrap();
    match v.split_last() {
        Some((last, rest)) => {
            for v in rest {
                unparse_value(v, output);
                output.write_str(", ").unwrap();
            }
            unparse_value(last, output);
        }
        None => (),
    }

    output.write_char(')').unwrap();
}
