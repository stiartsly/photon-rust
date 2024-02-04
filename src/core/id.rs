use std::fmt;
use std::cmp::Ordering;
use hex::FromHexError;
use bs58::decode;

use crate::signature;
use crate::cryptobox;
use crate::error::Error;

pub const ID_BYTES: usize = 32;
pub const ID_BITS: usize = 256;

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]

pub struct Id([u8; ID_BYTES]);

impl Id {
    pub fn random() -> Self {
        let mut bytes = [0u8; ID_BYTES];
        unsafe {
            libsodium_sys::randombytes_buf(
                bytes.as_mut_ptr() as *mut libc::c_void,
                ID_BYTES
            );
        }
        Id(bytes)
    }

    pub fn from_signature_key(publick_key: &signature::PublicKey) -> Self {
        let mut bytes = [0u8; ID_BYTES];

        bytes.copy_from_slice(publick_key.as_bytes());
        Id(bytes)
    }

    pub fn from_bytes(input: &[u8]) -> Self {
        assert_eq!(
            input.len(),
            ID_BYTES,
            "Incorrect bytes length {} for Id, should be {}",
            input.len(),
            ID_BYTES
        );

        let mut bytes = [0u8; ID_BYTES];
        bytes.copy_from_slice(input);
        Id(bytes)
    }

    pub fn try_from_hex(input: &str) -> Result<Self, Error> {
        let mut bytes = [0u8; ID_BYTES];
        let _ = hex::decode_to_slice(input, &mut bytes[..])
            .map_err(|e| match e {
                FromHexError::InvalidHexCharacter { c, index } => {
                    Error::Argument(
                        format!("Invalid hex character '{}' at index {}", c, index)
                    )
                }
                FromHexError::OddLength => {
                    Error::Argument(
                        format!("Odd length hex string")
                    )
                },
                FromHexError::InvalidStringLength => {
                    Error::Argument(
                        format!("Invalid hex string length {}", input.len())
                    )
                }
        });
        Ok(Id(bytes))
    }

    pub fn try_from_base58(input: &str) -> Result<Self, Error> {
        let mut bytes = [0u8; ID_BYTES];
        let _ = bs58::decode(input)
            .with_alphabet(bs58::Alphabet::DEFAULT)
            .onto(&mut bytes)
            .map_err(|e| match e {
                decode::Error::BufferTooSmall => {
                    Error::Argument(
                        format!("Invalid base58 string length {}", input.len())
                    )
                }
                decode::Error::InvalidCharacter {character, index} => {
                    Error::Argument(
                        format!("Invalid base58 character '{}' at index {}", character, index)
                    )
                }
                _ => {
                    Error::Argument(
                        format!("Invalid base58 string with unknown error")
                    )
                }
        });
        Ok(Id(bytes))
    }

    pub const fn min() -> Self {
        Id([0x0; ID_BYTES])
    }

    pub const fn max() -> Self {
        Id([0xFF; ID_BYTES])
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(self.0)
            .with_alphabet(bs58::Alphabet::DEFAULT)
            .into_string()
    }

    pub fn to_signature_key(&self) -> signature::PublicKey {
        signature::PublicKey::from(self.as_bytes())
    }

    pub fn to_encryption_key(&self) -> cryptobox::PublicKey {
        cryptobox::PublicKey::from_signature_key(
            &self.to_signature_key()
        ).unwrap()
    }

    pub fn distance(&self, other: &Id) -> Id {
        let mut bytes = [0u8; ID_BYTES];
        for i in 0..ID_BYTES {
            bytes[i] = self.0[i] ^ other.0[i];
        }
        Id(bytes)
    }

    pub const fn size(&self) -> usize {
        ID_BYTES
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub(crate) fn as_mut_bytes(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }

    pub fn three_way_compare(&self, id1: &Self, id2: &Self) -> Ordering {
        let mut mmi = usize::MAX;
        for i in 0..ID_BYTES {
            if id1.0[i] != id2.0[i] {
                mmi = i;
                break;
            }
        }
        if mmi == usize::MAX {
            return Ordering::Equal;
        }

        let a = id1.0[mmi] ^ self.0[mmi];
        let b = id2.0[mmi] ^ self.0[mmi];
        a.cmp(&b)
    }
}

pub fn distance(a: &Id, b: &Id) -> Id {
    a.distance(b)
}

pub(crate) fn bits_equal(a: &Id, b: &Id, depth: i32) -> bool {
    if depth < 0 {
        return true;
    }

    let mut mmi = usize::MAX;
    for i in 0..ID_BYTES {
        if a.0[i] != b.0[i] {
            mmi = i;
            break;
        }
    }

    let idx = (depth >> 3) as usize;
    let diff: u8 = a.0[idx] ^ b.0[idx];
    // Create a bitmask with the lower n bits set
    let mask: u8 = (1 << (depth & 0x07)) - 1;
    // Use the bitmask to check if the lower bits are all zeros
    let is_diff = (diff & !mask) == 0;

    match mmi == idx {
        true => is_diff,
        false => mmi > idx
    }
}

pub(crate) fn bits_copy(src: &Id, dst: &mut Id, depth: i32) {
    if depth < 0 {
        return;
    }

    let idx = (depth >> 3) as usize;
    if idx > 0 {
        dst.0[..idx].copy_from_slice(&src.0[..idx]);
    }

    let mask: u8 = (1 << (depth & 0x07)) - 1;
    dst.0[idx] &= !mask;
    dst.0[idx] |= src.0[idx] & mask;
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", self.to_hex())?;
        Ok(())
    }
}
