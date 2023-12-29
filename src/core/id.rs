use std::fmt;
use std::cmp::Ordering;
use libsodium_sys::randombytes_buf;
use crate::signature::PublicKey;

pub const ID_BITS: usize = 256;
pub const ID_BYTES: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Id {
    bytes: [u8; ID_BYTES],
}

impl Id {
    pub fn new() -> Self {
        Id { bytes: [0; ID_BYTES] }
    }

    pub fn zero() -> Self {
        Id { bytes: [0; ID_BYTES] }
    }

    pub fn random() -> Self {
        let mut bytes = [0u8; ID_BYTES];
        unsafe {
            randombytes_buf(
                bytes.as_mut_ptr() as *mut libc::c_void,
                ID_BYTES
            );
        }
        Id { bytes }
    }

    pub fn of_public_key(publick_key: &PublicKey) -> Self {
        Id { bytes: *publick_key.bytes() }
    }

    pub fn of_hex(idstr: &str) -> Result<Self, &'static str> {
        let decoded = hex::decode(idstr).map_err(|_| "Decoding failed")?;
        if decoded.len() != ID_BYTES {
            return Err("Invalid hex ID length");
        }

        let bytes: Result<[u8; 32], _> = decoded.try_into();
        match bytes {
            Ok(array) => {
                Ok(Id { bytes: array })
            },
            Err(_) => {
                Err("Conversion from Hex to Id failed")
            }
        }
    }

    pub fn of_base58(idstr: &str) -> Result<Self, &'static str> {
        let mut data: [u8; 32] = [0; ID_BYTES];
        let decoded = bs58::decode(idstr).onto(&mut data);
        match decoded {
            Ok(len)=> {
                if len != ID_BYTES {
                    return Err("Invalid base58 Id length");
                }
                Ok(Id { bytes: data })
            },
            Err(_) => {
                Err("Conversion from base58 to Id failed")
            }
        }
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    pub fn to_signature_key(&self) -> PublicKey {
        PublicKey::from(self.bytes.as_slice()).unwrap()
    }

    pub fn distance(&self, to: &Id) -> Self {
        let mut data: [u8; 32] = [0; ID_BYTES];
        for i in 0..ID_BYTES {
            data[i] = self.bytes[i] ^ to.bytes[i];
        }
        Id { bytes: data }
    }

    pub fn three_way_compare(&self, id1: &Id, id2: &Id) -> Ordering {
        let mut mmi = i32::MAX;
        for i in 0..ID_BYTES {
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
        for i in 0..ID_BYTES {
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

}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}
