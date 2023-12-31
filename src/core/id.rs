use std::fmt;
use std::cmp::Ordering;
use libsodium_sys::randombytes_buf;
use crate::signature::PublicKey;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Id {
    bytes: [u8; Self::BYTES],
}

impl Id {
    pub const BYTES: usize = 32;
    pub const BITS: usize = 256;

    pub fn new() -> Self {
        Id { bytes: [0; Self::BYTES] }
    }

    pub fn zero() -> Self {
        Id::new()
    }

    pub fn random() -> Self {
        let mut bytes = [0u8; Self::BYTES];
        unsafe {
            randombytes_buf(
                bytes.as_mut_ptr() as *mut libc::c_void,
                Self::BYTES
            );
        }
        Id { bytes }
    }

    pub fn from_key(publick_key: &PublicKey) -> Self {
        let mut bytes = [0u8; Self::BYTES];

        bytes.copy_from_slice(publick_key.as_bytes());
        Id { bytes }
    }

    pub fn from_hex(idstr: &str) -> Result<Self, &'static str> {
        let decoded = hex::decode(idstr)
            .map_err(|_| "Decoding failed")?;

        if decoded.len() != Self::BYTES {
            return Err("Invalid hex ID length");
        }

        let bytes: [u8; 32] = decoded.try_into()
            .map_err(|_| "Conversion from Hex to Id failed")?;
        Ok(Id{ bytes })
    }

    pub fn from_base58(idstr: &str) -> Result<Self, &'static str> {
        let mut bytes: [u8; 32] = [0; Self::BYTES];
        let decoded = bs58::decode(idstr)
            .onto(&mut bytes)
            .map_err(|_| "Conversion from base58 to Id failed")?;

        if decoded != Self::BYTES {
            return Err("Invalid base58 Id length");
        }
        Ok(Id { bytes })
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(self.bytes)
            .with_alphabet(bs58::Alphabet::FLICKR)
            .into_string()
    }

    pub fn to_signature_key(&self) -> PublicKey {
        PublicKey::from(self.bytes.as_slice()).unwrap()
    }

    pub fn distance(&self, to: &Id) -> Self {
        let mut data: [u8; 32] = [0; Self::BYTES];
        for i in 0..Self::BYTES {
            data[i] = self.bytes[i] ^ to.bytes[i];
        }
        Id { bytes: data }
    }

    pub fn three_way_compare(&self, id1: &Id, id2: &Id) -> Ordering {
        let mut mmi = i32::MAX;
        for i in 0..Self::BYTES {
            if id1.bytes[i] != id2.bytes[i] {
                mmi = i as i32;
                break;
            }
        }
        if mmi == i32::MAX {
            return Ordering::Equal;
        }

        let a = id1.bytes[mmi as usize] ^ self.bytes[mmi as usize];
        let b = id2.bytes[mmi as usize] ^ self.bytes[mmi as usize];
        a.cmp(&b)
    }

    pub fn bits_equal(id1: &Id, id2: &Id, depth: i32) -> bool {
        if depth < 0 {
            return true;
        }

        let mut mmi = i32::MAX;
        for i in 0..Self::BYTES {
            if id1.bytes[i] != id2.bytes[i] {
                mmi = i as i32;
                break;
            }
        }

        let idx = depth >> 3;
        let diff: u8 = id1.bytes[idx as usize] ^ id2.bytes[idx as usize];
        let mask: u8 = (1 << (depth & 0x07)) - 1;  // Create a bitmask with the lower n bits set
        let is_diff = (diff & !mask) == 0;  // Use the bitmask to check if the lower bits are all zeros

        if mmi == idx {
            is_diff
        } else {
            mmi > idx
        }
    }

    pub fn bits_copy(src: &Id, dest: &mut Id, depth: i32) {
        if depth < 0 {
            return;
        }
        let idx = depth >> 3;
        if idx > 0 {
            dest.bytes[..idx as usize].copy_from_slice(&src.bytes[..idx as usize]);
        }
        let mask: u8 = (1 << (depth & 0x07)) - 1;
        dest.bytes[idx as usize] &= !mask;
        dest.bytes[idx as usize] |= src.bytes[idx as usize] & mask;
    }

    pub fn to_string(&self) -> String {
        self.to_hex()
    }

}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}
