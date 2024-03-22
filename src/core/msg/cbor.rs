use core::result::Result;
use std::io::{Error};

#[derive(Debug)]
pub(crate) struct Writer<'a> {
    buf: &'a mut Vec<u8>
}

impl<'a> ciborium_io::Write for Writer<'a> {
    type Error = Error;

    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        if !data.is_empty() {
            self.buf.extend_from_slice(data);
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a> Writer<'a> {
    pub(crate) fn new(input: &'a mut Vec<u8>) -> Self {
        Self {
            buf: input,
        }
    }
}