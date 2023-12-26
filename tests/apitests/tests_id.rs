use boson::id::Id;

#[cfg(test)]
mod apitests {
    use super::*;

    #[test]
    fn test_zero() {
        let a = Id::zero();
        let b = Id::zero();
        assert_eq!(a.to_hex(), "0000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(a.to_hex(), b.to_hex());
    }

    #[test]
    fn test_id() {
        assert_eq!(3, 3);
    }
}
