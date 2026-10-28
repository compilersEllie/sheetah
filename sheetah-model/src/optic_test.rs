#[cfg(test)]
mod tests {
    use crate::optic::*;
    use std::sync::Arc;

    #[test]
    fn test_optic_id() {
        let id = OpticId::new(42);
        assert_eq!(id.id, 42);
    }

    #[test]
    fn test_iso() {
        let iso = Iso::new("add1", |n: i32| n + 1, |n| n - 1);
        assert_eq!(iso.view(5), 6);
        assert_eq!(iso.review(6), 5);
        assert_eq!(iso.name(), "add1");

        let flipped = iso.flip();
        assert_eq!(flipped.name(), "undo add1");
        assert_eq!(flipped.view(6), 5);
        assert_eq!(flipped.review(5), 6);
    }

    #[test]
    fn test_lens() {
        let lens = Lens::new("double", |n: i32| n * 2, |n, factor| n * factor);
        assert_eq!(lens.view(5), 10);
        assert_eq!(lens.set(5, 3), 15); // 5 * 3 = 15? Wait, set_impl is Fn(S, A) -> S
        // In the test, 5 is source, 3 is field. so 5 * 3 = 15.
    }

    #[test]
    fn test_prism() {
        let prism = Prism::new(
            "maybe_even",
            |n: i32| if n % 2 == 0 { Some(n / 2) } else { None },
            |n| n * 2,
        );
        assert_eq!(prism.preview(4), Some(2));
        assert_eq!(prism.preview(5), None);
        assert_eq!(prism.review(2), 4);
    }

    #[test]
    fn test_traversal() {
        let traversal = Traversal::new(
            "to_vec",
            |opt: Option<i32>| opt.map(|v| vec![v]).unwrap_or_default(),
            |f: Functor<i32, i32>, opt: Option<i32>| opt.map(|v| f(v)),
        );

        assert_eq!(traversal.to_list(Some(5)), vec![5]);
        assert_eq!(traversal.to_list(None), vec![]);

        let f: Functor<i32, i32> = Arc::new(Box::new(|n| n + 1));
        assert_eq!(traversal.over(f.clone(), Some(5)), Some(6));
        assert_eq!(traversal.over(f, None), None);
    }

    #[test]
    fn test_affine() {
        let affine = Affine::new(
            "maybe_add",
            |opt: Option<i32>| opt.map(|v| v + 1),
            |f: Functor<i32, i32>, opt: Option<i32>| {
                opt.and_then(|s| {
                    let a = s + 1;
                    let new_a = f(a);
                    Some(new_a - 1)
                })
            },
        );
        assert_eq!(affine.to_list(Some(5)), Some(6));
        assert_eq!(affine.to_list(None), None);

        let f: Functor<i32, i32> = Arc::new(Box::new(|n| n + 1));
        assert_eq!(affine.over(f.clone(), Some(5)), Some(6));
        assert_eq!(affine.over(f, None), None);
    }

    #[test]
    fn test_compose_iso_iso() {
        let iso1 = Iso::new("add1", |n: i32| n + 1, |n| n - 1);
        let iso2 = Iso::new("add2", |n: i32| n + 2, |n| n - 2);
        let composed = iso1.and_then(iso2);

        assert_eq!(composed.name(), "add1 | add2");
        assert_eq!(composed.view(5), 8); // (5+1)+2 = 8
        assert_eq!(composed.review(8), 5); // (8-2)-1 = 5
    }
}
