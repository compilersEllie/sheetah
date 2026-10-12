pub mod optics;
pub mod shape;
pub mod table;
pub mod table_store;
pub mod value;

#[cfg(test)]
mod tests {
    #[test]
    fn dummy_test() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
