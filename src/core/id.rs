use std::fmt;

const ID_BYTES: usize = 32;

pub struct Id {
    bytes: [u8; ID_BYTES],
}

impl Id {
    pub fn zero() -> Self {
        Id {
            bytes: [0; ID_BYTES],
        }
    }

    pub fn of_hex(hex_id: &str) -> Result<Self, &'static str> {
        let decoded = hex::decode(hex_id).map_err(|_| "Decoding failed")?;
        if decoded.len() != ID_BYTES {
            return Err("Invalid hex ID length");
        }

        let mut bytes = [0; ID_BYTES];
        bytes.copy_from_slice(&decoded);
        Ok(Id { bytes })
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    pub fn distance(&self, to: &Id) -> Self {
        let mut buf = [0; ID_BYTES];
        for i in 0..ID_BYTES {
            buf[i] = self.bytes[i] ^ to.bytes[i];
        }
        Id { bytes: buf }
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
