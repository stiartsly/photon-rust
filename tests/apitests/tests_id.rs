use boson::id::Id;

#[cfg(test)]
mod apitests {
    use super::*;

    #[test]
    fn test_zero() {
        let a = Id::zero();
        assert_eq!(a.to_hex(), "0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_ofhex() {
        let hex_str = "71e1b2ecdf528b623192f899d984c53f2b13508e21ccd53de5d7158672820636";
        let hex_id = Id::of_hex(hex_str).expect("Invalid hex Id");
        assert_eq!(hex_id.to_hex(), hex_str);
    }

    #[test]
    fn test_distance() {
        let a = Id::of_hex("00000000f528d6132c15787ed16f09b08a4e7de7e2c5d3838974711032cb7076").expect("Invalid hex Id");
        let b = Id::of_hex("00000000f0a8d6132c15787ed16f09b08a4e7de7e2c5d3838974711032cb7076").expect("Invalid hex Id");
        let distance_str = "0000000005800000000000000000000000000000000000000000000000000000";
        assert_eq!(a.distance(&b).to_hex(), distance_str);
        assert_eq!(Id::distance(&a, &b).to_hex(), distance_str);
    }

    #[test]
    fn test_id() {
        assert_eq!(3, 3);
    }
}
