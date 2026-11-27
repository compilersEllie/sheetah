#[cfg(test)]
mod tests {
    use crate::optic::{OpticId, OpticKind};
    use crate::optic_store::OpticStore;

    #[test]
    fn optic_store_indexes_registered_optics() {
        let store = OpticStore::new();

        assert_eq!(store.get_by_name("My Optics").len(), 2);
        assert_eq!(store.get_by_id(OpticId::new(0)).unwrap().name, "i32_to_i64");
        assert_eq!(store.optic_by_id(OpticId::new(1)).name, "i64_to_f64");
        assert!(store.get_by_id(OpticId::new(99)).is_none());

        assert_eq!(store.get_by_in_type::<i32>().len(), 1);
        assert_eq!(store.get_by_out_type::<f64>().len(), 1);
        assert!(store.get_by_in_type::<String>().is_empty());

        let first = store.get_by_id(OpticId::new(0)).unwrap();
        assert_eq!(store.find_composable_after(&first).len(), 1);
        assert!(store.find_composable_before(&first).is_empty());

        let second = store.get_by_id(OpticId::new(1)).unwrap();
        assert!(store.find_composable_after(&second).is_empty());
        assert_eq!(store.find_composable_before(&second).len(), 1);
    }

    #[test]
    fn optic_store_adds_explicit_kinds() {
        fn create_optics() -> Vec<crate::optic::ErasedOptic> {
            vec![crate::optic::ErasedOptic::from(crate::optic::Iso::new(
                "explicit",
                |n: i32| n as i64,
                |n: i64| n as i32,
            ))]
        }

        let mut store = OpticStore::default();
        store.add_optics(&OpticKind {
            name: "Explicit",
            create_optics,
        });

        assert_eq!(store.get_by_name("Explicit").len(), 1);
        assert_eq!(store.get_by_name("Missing").len(), 0);
    }
}
