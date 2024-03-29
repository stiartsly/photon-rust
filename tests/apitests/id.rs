use boson::id;
use boson::id::Id;

#[test]
fn test_default() {
    let def = Id::default();
    let min = Id::min();
    let max = Id::max();
    assert_eq!(def, min);
    assert_eq!(
        min.into_hex(),
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
    assert_eq!(
        max.into_hex(),
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );
}

#[test]
fn test_from_hex() {
    let hex_str = "71e1b2ecdf528b623192f899d984c53f2b13508e21ccd53de5d7158672820636";
    let id = Id::try_from_hex(hex_str).expect("Invalid hex Id");
    assert_eq!(id.into_hex(), hex_str);
}

#[test]
fn test_from_base58() {
    let base58_str = "HZXXs9LTfNQjrDKvvexRhuMk8TTJhYCfrHwaj3jUzuhZ";
    let id = Id::try_from_base58(base58_str).expect("Invalid base58 Id");
    assert_eq!(id.into_base58(), base58_str);
}

#[test]
fn test_distance() {
    let id1 = Id::try_from_hex("00000000f528d6132c15787ed16f09b08a4e7de7e2c5d3838974711032cb7076")
        .expect("Invalid hex Id");
    let id2 = Id::try_from_hex("00000000f0a8d6132c15787ed16f09b08a4e7de7e2c5d3838974711032cb7076")
        .expect("Invalid hex Id");
    let distance_str = "0000000005800000000000000000000000000000000000000000000000000000";
    assert_ne!(id1, id2);
    assert_eq!(id::distance(&id1, &id2).into_hex(), distance_str);
    assert_eq!(id::distance(&id1, &id2).into_hex(), distance_str);
}

#[test]
fn test_equal() {
    let hex_str = "71e1b2ecdf528b623192f899d984c53f2b13508e21ccd53de5d7158672820636";
    let id1 = Id::try_from_hex(hex_str).expect("Invalid hex Id");
    let id2 = Id::try_from_hex(hex_str).expect("Invalid hex Id");
    assert_eq!(id1, id2);
}
