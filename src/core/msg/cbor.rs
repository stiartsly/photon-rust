use core::result::Result;

#[derive(Debug)]
pub(crate) struct Writer<'a> {
    buf: &'a mut Vec<u8>
}

impl<'a> ciborium_io::Write for Writer<'a> {
    type Error = std::io::Error;

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

#[derive(Debug)]
pub(crate) struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ciborium_io::Read for Reader<'a> {
    type Error = std::io::Error;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let remaining_len = self.data.len() - self.pos;

        if remaining_len <= 0 {
            return Err(Self::Error::from(std::io::ErrorKind::UnexpectedEof));
        }

        if remaining_len >= buf.len() {
            // If there is enough data remaining, copy it to buf
            buf.copy_from_slice(&self.data[self.pos..self.pos + buf.len()]);
            self.pos += buf.len();
        } else {
            // If not enough data is remaining, return an error
            //Err(Error::from(std::io::ErrorKind::UnexpectedEof))
            buf.copy_from_slice(&self.data[self.pos..]);
            self.pos = self.data.len();
        }
        Ok(())
    }
}

impl<'a> Reader<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        Self {
            data: input,
            pos: 0
        }
    }
}
