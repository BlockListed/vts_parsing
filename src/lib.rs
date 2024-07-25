pub mod parse;
pub mod unparse;

pub use parse::{parse, Value, Node};
pub use unparse::unparse;

#[cfg(test)]
mod testing {
    const TEST_STR: &str = include_str!("../amogus testing.vts");

    #[test]
    fn full_test() {
        let left = super::parse(TEST_STR);
        let right = super::parse(&super::unparse(&left));
        eprintln!("{left:?}");
        assert_eq!(left, right);
    }
}
