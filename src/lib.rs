pub mod parse;
pub mod unparse;

pub use parse::{parse, Value};
pub use unparse::unparse;

#[cfg(test)]
mod testing {
    const TEST_STR: &str = include_str!("../amogus testing.vts");

    #[test]
    fn full_test() {
        let left = TEST_STR;
        let right = &super::unparse(&super::parse(TEST_STR));
        eprintln!("{right}");
        assert_eq!(left, right);
    }
}
