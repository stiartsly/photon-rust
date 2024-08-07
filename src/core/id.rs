use bs58::decode;
use hex::FromHexError;
use std::cmp::Ordering;
use std::fmt;
use ciborium::value::Value;

use crate::{
    cryptobox,
    signature,
    error::Error
};

pub const ID_BYTES: usize = 32;
pub const ID_BITS: usize = 256;

#[derive(Default, Clone, PartialEq, Ord, Eq, Debug, Hash)]
pub struct Id([u8; ID_BYTES]);

impl Id {
    pub fn random() -> Self {
        let mut bytes = [0u8; ID_BYTES];
        unsafe {
            libsodium_sys::randombytes_buf(bytes.as_mut_ptr()
                as *mut libc::c_void, ID_BYTES);
        }
        Id(bytes)
    }

    pub fn from_signature_pubkey(publick_key: &signature::PublicKey) -> Self {
        let mut bytes = [0u8; ID_BYTES];
        bytes.copy_from_slice(publick_key.as_bytes());
        Id(bytes)
    }

    pub fn from_encryption_pubkey(public_key: &cryptobox::PublicKey) -> Self {
        let mut bytes = [0u8; ID_BYTES];
        bytes.copy_from_slice(public_key.as_bytes());
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

    pub(crate) fn try_from_cbor(input: &Value) -> Result<Self, Error> {
        let bytes = match input.as_bytes() {
            Some(bytes) => bytes,
            None => return Err(Error::Protocol(format!("Invalid cobor value for Id"))),
        };
        Ok(Self::from_bytes(bytes))
    }

    pub fn try_from_hex(input: &str) -> Result<Self, Error> {
        let mut bytes = [0u8; ID_BYTES];
        let _ = hex::decode_to_slice(input, &mut bytes[..]).map_err(|e| match e {
            FromHexError::InvalidHexCharacter { c, index } => {
                Error::Argument(format!("Invalid hex character {} at {}", c, index))
            },
            FromHexError::OddLength => {
                Error::Argument(format!("Odd hex string length {}", input.len()))
            },
            FromHexError::InvalidStringLength => {
                Error::Argument(format!("Invalid hex string length"))
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
                    Error::Argument(format!("Invalid base58 string length"))
                },
                decode::Error::InvalidCharacter { character, index } => {
                    Error::Argument(format!("Invalid base58 character {} at {}", character, index))
                },
                _ => {
                    Error::Argument(format!("Invalid base58 with unknown error"))
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

    pub fn into_hex(&self) -> String {
        hex::encode(&self.0)
    }

    pub fn into_base58(&self) -> String {
        bs58::encode(self.0)
            .with_alphabet(bs58::Alphabet::DEFAULT)
            .into_string()
    }

    pub fn to_signature_pubkey(&self) -> signature::PublicKey {
        signature::PublicKey::from(self.as_bytes())
    }

    pub fn to_encryption_pubkey(&self) -> cryptobox::PublicKey {
        cryptobox::PublicKey::from_signature_key(&self.to_signature_pubkey()).unwrap()
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

    pub(crate) fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }

    pub(crate) fn three_way_compare(&self, id1: &Self, id2: &Self) -> Ordering {
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

    pub(crate) fn to_cbor(&self) -> Value {
        Value::Bytes(self.0.to_vec())
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
        false => mmi > idx,
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
        write!(f, "{}", self.into_base58())?;
        Ok(())
    }
}

impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Id) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}
