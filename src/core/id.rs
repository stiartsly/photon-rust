use std::fmt;
use libc::c_void;
use libsodium_sys::randombytes_buf;

const ID_BYTES: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Id {
    bytes: [u8; ID_BYTES],
}

impl Id {
    pub fn zero() -> Self {
        Id { bytes: [0; ID_BYTES] }
    }

    pub fn random() -> Self {
        let mut data = [0; ID_BYTES];
        unsafe {
            let ptr = data.as_mut_ptr() as *mut c_void;
            randombytes_buf(ptr, ID_BYTES);
        }
        Id { bytes: data }
    }

    pub fn of_hex(hex_id: &str) -> Result<Self, &'static str> {
        let decoded = hex::decode(hex_id).map_err(|_| "Decoding failed")?;
        if decoded.len() != ID_BYTES {
            return Err("Invalid hex ID length");
        }

        let bytes: Result<[u8; 32], _> = decoded.try_into();
        match bytes {
            Ok(array) => Ok(Id { bytes: array }),
            Err(_) => Err("Conversion to [u8; 32] failed"),
        }
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    pub fn distance(&self, to: &Id) -> Self {
        let mut data = [0; ID_BYTES];
        for i in 0..ID_BYTES {
            data[i] = self.bytes[i] ^ to.bytes[i];
        }
        Id { bytes: data }
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
