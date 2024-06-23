use std::iter::repeat;

use crate::Value;

pub fn unparse(v: &Value) -> String {
    assert!(matches!(v, Value::Object(_)), "top level should be object");
    let v = match v {
        Value::Object(v) => v,
        _ => unreachable!(),
    };
    
    let mut s = String::new();

    for (k, v) in v {
        assert!(matches!(v, Value::Object(_)), "top level children should be objects");
        let v = match v {
            Value::Object(v) => v,
            _ => unreachable!(),
        };
        
        s.push_str(&unparse_object(0, k, v));
    }

    s
}

fn indent(depth: u32) -> impl Iterator<Item = char> {
    repeat('\t').take(depth as usize)
}

// number, float, bool, tuple or null
fn unparse_value(v: &Value) -> Option<String> {
    match v {
        Value::Number(v) => Some(v.to_string()),
        Value::Float(v) => Some(v.to_string()),
        Value::Boolean(v) => Some(if *v { "True"} else {"False"}.to_owned()),
        Value::Tuple(v) => Some(unparse_tuple(v)),
        Value::Null => Some(String::new()),
        Value::String(v) => Some(v.to_owned()),
        _ => None,
    }
}

fn unparse_object(indent_depth: u32, k: &str, v: &[(String, Value)]) -> String {
    let mut s = String::new();

    s.extend(indent(indent_depth));
    s.push_str(k);
    s.push('\n');
    s.extend(indent(indent_depth));
    s.push('{');
    s.push('\n');
    for (k, v) in v.iter() {
        if let Some(v) = unparse_value(v) { 
            s.extend(indent(indent_depth+1));

            s.push_str(k);
            s.push(' ');
            s.push('=');
            s.push(' ');
            s.push_str(&v);
            s.push('\n')
        } else {
            s.push_str(&match v {
                Value::Object(v) => unparse_object(indent_depth+1, k, v),
                Value::Array(v) => unparse_array(indent_depth+1, k, v),
                _ => unreachable!(),
            }
            )
        }
    }
    s.extend(indent(indent_depth));
    s.push('}');
    s.push('\n');

    s
}

fn unparse_array(indent_depth: u32, k: &str, v: &[(String, Value)]) -> String {
    let mut s = String::new();

    s.extend(indent(indent_depth));
    s.push_str(k);
    s.push('\n');
    s.extend(indent(indent_depth));
    s.push('{');
    s.push('\n');
    for (k, v) in v {
        s.push_str(&
        match v {
            Value::Object(v) => unparse_object(indent_depth+1, k, v),
            Value::Array(v) => unparse_array(indent_depth+1, k, v),
            _ => panic!("invalid data"),
        }
        )
    }
    s.extend(indent(indent_depth));
    s.push('}');
    s.push('\n');

    s
}

fn unparse_tuple(v: &[Value]) -> String {
    let mut s = String::new();

    s.push('(');
    for v in v {
        s.push_str(&unparse_value(v).unwrap());
        s.push(',');
    }
    s.push(')');

    s
}
