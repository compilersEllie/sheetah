// A library of optics to use in spreadsheets

// TODO(feat): Add all the basics and then also things like templating. #2
// Use: https://github.com/sunng87/handlebars-rust

#[cfg(test)]
mod tests {
    use crate::optic::{Compose, ErasedOptic, Iso, OpticKind};

    #[test]
    fn add_two_iso() {
        let a = Iso::new("add2", |n| n + 2, |n| n - 2);
        for x in 0..4 {
            assert_eq!(a.view(x), x + 2);
            assert_eq!(a.review(a.view(x)), x);
        }
    }

    #[test]
    fn compose_iso() {
        let a = Iso::new("add2", |n| n + 2, |n| n - 2);
        let b = Iso::new("times2", |n| n * 2, |n| n / 2);
        let c = a.and_then(b);
        for x in 0..4 {
            assert_eq!(c.view(x), (x + 2) * 2);
            assert_eq!(c.review(c.view(x)), x);
        }
    }

    #[test]
    fn modify_retains_update() {
        let a = Iso::new("add2", |n| n + 2, |n| n - 2);
        let b = Iso::new("times2", |n| n * 2, |n| n / 2);
        let c = a.and_then(b);
        for x in 0..4 {
            let mut v = x;
            let res = c.view(v) + 2;
            v = c.review(res);
            assert_eq!(c.view(v), res);
        }
    }

    pub fn create_my_optics() -> Vec<ErasedOptic> {
        vec![
            ErasedOptic::from(Iso::new(
                "i32_to_i64".to_string(),
                |n: i32| n as i64,
                |n: i64| n as i32,
            )),
            ErasedOptic::from(Iso::new(
                "i64_to_f64".to_string(),
                |n: i64| n as f64,
                |n: f64| n as i64,
            )),
        ]
    }

    inventory::submit!(OpticKind {
        name: "My Optics",
        create_optics: create_my_optics,
    });
}
