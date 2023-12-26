//use boson::Id;

/*
fn main() {
    let id = Id::zero();
    println!("{}", id);
}
*/

/*
use hex;

fn main() {
    let input = "090A0B0C";

    let mut decoded = [0; 4];
    hex::decode_to_slice(input, &mut decoded).expect("Decoding failed");

    println!("{:?}", decoded);
}
*/
/*
use hex::FromHex;

fn main() {
    //let input = "090A0B0C";
    let input = "4833af415161cbd0a3ef83aa59a55fbadc9bd520a885a8ca214a3d09b6676cb8";

    let decoded = <[u8; 32]>::from_hex(input).expect("Decoding failed");

    println!("{:?}", decoded);
}*/

#[cfg(test)]
mod apitests {
    use super::*;

    #[test]
    fn test_id() {
        assert_eq!(3, 3);
    }
}
