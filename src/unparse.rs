use std::fmt::{Display, Formatter, Write};

use crate::{parse::Node, Value};

pub fn unparse(v: &Node) -> String {
    let mut s = String::new();
    
    unparse_node(v, 0, &mut s);

    s
}

struct UnparseValue<'a>(&'a Value);

impl<'a> Display for UnparseValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Value::Number(v) => {
                f.write_fmt(format_args!("{}", v))?;
            }
            Value::Float(ref v) => {
                f.write_fmt(format_args!("{}", v))?;
            }
            Value::Boolean(v) => {
                if *v {
                    f.write_str("True")?;
                } else {
                    f.write_str("False")?;
                }
            }
            Value::Vector(ref v) => {
                unparse_vector(v, f)?;
            }
            Value::VectorGroup(ref v) => {
                unparse_vectorgroup(v, f)?;
            }
            Value::Null => (),
            Value::String(v) => {
                f.write_str(v)?;
            }
        };

        Ok(())
    }
}

fn unparse_vector(v: &[f64; 3], f: &mut Formatter) -> std::fmt::Result {
    f.write_char('(')?;
    match v.split_last() {
        Some((last, rest)) => {
            for v in rest {
                f.write_fmt(format_args!("{}", v))?;
                f.write_str(", ")?;
            }
            f.write_fmt(format_args!("{}", last))?;
        }
        None => (),
    }

    f.write_char(')')?;

    Ok(())
}

fn unparse_vectorgroup(v: &[[f64; 3]], f: &mut Formatter) -> std::fmt::Result {
    for vector in v {
        unparse_vector(vector, f)?;
        f.write_char(';')?;
    }

    Ok(())
}

fn indent(depth: u32, output: &mut String) {
    (0..depth).for_each(|_| {
        output.write_char('\t').unwrap();
    });
}

fn unparse_node(node: &Node, indent_depth: u32, output: &mut String) {
    indent(indent_depth, output);
    output.write_str(&node.name).unwrap();
    output.write_str("\r\n").unwrap();
    indent(indent_depth, output);
    output.write_str("{\r\n").unwrap();
    for (k, v) in node.values.iter() {
        indent(indent_depth+1, output);
        output.write_fmt(format_args!("{} = {}\r\n", k, UnparseValue(v))).unwrap();
    }
    for n in node.nodes() {
        unparse_node(n, indent_depth+1, output);
    }
    indent(indent_depth, output);
    output.write_str("}\r\n").unwrap();
}
