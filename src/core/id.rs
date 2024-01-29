use std::fmt;
use std::cmp::Ordering;
use hex::FromHexError;
use bs58::decode;

use crate::signature;
use crate::cryptobox;
use crate::error::Error;

pub const ID_BYTES: usize = 32;
pub const ID_BITS: usize = 256;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Id {
    bytes: [u8; ID_BYTES],
}

impl Id {
    pub fn default() -> Self {
        Id { bytes: [0; ID_BYTES] }
    }

    pub fn random() -> Self {
        let mut bytes = [0u8; ID_BYTES];
        unsafe {
            libsodium_sys::randombytes_buf(
                bytes.as_mut_ptr() as *mut libc::c_void,
                ID_BYTES
            );
        }
        Id { bytes }
    }

    pub fn from_signature_key(publick_key: &signature::PublicKey) -> Self {
        let mut bytes = [0u8; ID_BYTES];

        bytes.copy_from_slice(publick_key.as_bytes());
        Id { bytes }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(
            bytes.len(),
            ID_BYTES,
            "Incorrect bytes length {} for Id, should be {}",
            bytes.len(),
            ID_BYTES
        );

        let mut _bytes = [0u8; ID_BYTES];
        _bytes.copy_from_slice(bytes);
        Id { bytes: _bytes }
    }

    pub fn try_from_hex(idstr: &str) -> Result<Self, Error> {
        let mut bytes = [0u8; ID_BYTES];
        let _ = hex::decode_to_slice(idstr, &mut bytes[..]).map_err(|err| match err {
            FromHexError::InvalidHexCharacter { c, index } => {
                Error::Argument(format!("Invalid hex character '{}' at index {}", c, index))
            }
            FromHexError::OddLength => {
                Error::Argument(format!("Odd length hex string"))
            },
            FromHexError::InvalidStringLength => {
                Error::Argument(format!("Invalid hex string length {}", idstr.len()))
            }
        });
        Ok(Id{ bytes })
    }

    pub fn try_from_base58(idstr: &str) -> Result<Self, Error> {
        let mut bytes = [0u8; ID_BYTES];
        let _ = bs58::decode(idstr)
            .with_alphabet(bs58::Alphabet::DEFAULT)
            .onto(&mut bytes)
            .map_err(|err| match err {
                decode::Error::BufferTooSmall => {
                    Error::Argument(format!("Invalid base58 string length {}", idstr.len()))
                }
                decode::Error::InvalidCharacter {character, index} => {
                    Error::Argument(format!("Invalid base58 character '{}' at index {}", character, index))
                }
                _ => {
                    Error::Argument(format!("Invalid base58 string with unknown error"))
                }
        });
        Ok(Id { bytes })
    }

    pub fn min() -> Self {
        Id { bytes: [0x0; ID_BYTES]}
    }

    pub fn max() -> Self {
        Id { bytes: [0xFF; ID_BYTES] }
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(self.bytes)
            .with_alphabet(bs58::Alphabet::DEFAULT)
            .into_string()
    }

    pub fn to_signature_key(&self) -> signature::PublicKey {
        signature::PublicKey::from(self.bytes.as_slice())
    }

    pub fn to_encryption_key(&self) -> cryptobox::PublicKey {
        unimplemented!()
    }

    pub fn size(&self) -> usize {
        ID_BYTES
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn three_way_compare(&self, id1: &Self, id2: &Self) -> Ordering {
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
}

#[allow(dead_code)]
pub fn distance(id1: &Id, id2: &Id) -> Id {
    let mut bytes: [u8; ID_BYTES] = [0; ID_BYTES];
    for i in 0..ID_BYTES {
        bytes[i] = id1.bytes[i] ^ id2.bytes[i];
    }
    Id { bytes }
}

#[allow(dead_code)]
pub(crate) fn bits_equal(id1: &Id, id2: &Id, depth: i32) -> bool {
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

    match mmi == idx {
        true => is_diff,
        false => mmi > idx
    }
}

#[allow(dead_code)]
pub(crate) fn bits_copy(src: &Id, dest: &mut Id, depth: i32) {
    if depth < 0 {
        return;
    }

    let idx = (depth >> 3) as usize;
    if idx > 0 {
        dest.bytes[..idx].copy_from_slice(&src.bytes[..idx]);
    }

    let mask: u8 = (1 << (depth & 0x07)) - 1;
    dest.bytes[idx] &= !mask;
    dest.bytes[idx] |= src.bytes[idx] & mask;
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", self.to_hex())?;
        Ok(())
    }
}
