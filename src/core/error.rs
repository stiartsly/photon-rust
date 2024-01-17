use std::{fmt, io, net, result};

#[derive(Debug)]
#[allow(dead_code)]
pub struct BosonError {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Generic(String),
    Io(io::Error, String),
    Network(net::AddrParseError, String),
    Argument(String),
    State(String),
    Protocol(String),
    Crypto(String),
}

impl fmt::Display for BosonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Generic(msg) => write!(f, "Generic error: {}", msg),
            ErrorKind::Io(err, msg) => write!(f, "IO error: {}:{}", err, msg),
            ErrorKind::Network(err, msg) => write!(f, "Network error: {}:{}", err, msg),
            ErrorKind::Argument(msg) => write!(f, "Invalid argument: {}", msg),
            ErrorKind::State(msg) => write!(f, "State error {}", msg),
            ErrorKind::Protocol(msg) => write!(f, "DHT error {}", msg),
            ErrorKind::Crypto(msg) => write!(f, "Crypto error {}", msg),
        }
    }
}

impl std::error::Error for BosonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Generic(_) => None,
            ErrorKind::Io(err,_) => Some(err),
            ErrorKind::Network(err, _) => Some(err),
            ErrorKind::Argument(_) => None,
            ErrorKind::State(_) => None,
            ErrorKind::Protocol(_) => None,
            ErrorKind::Crypto(_) => None
        }
    }
}

impl From<io::Error> for BosonError {
    fn from(_: io::Error) -> Self {
        /*BosonError {
            kind: ErrorKind::Io(err, err.msg()),
        }*/
        unimplemented!()
    }
}

impl From<net::AddrParseError> for BosonError {
    fn from(_: net::AddrParseError) -> Self {
        /*BosonError {
            kind: ErrorKind::Network(err),
        }*/
        unimplemented!()
    }
}

pub type Result<T> = result::Result<T, BosonError>;

#[allow(dead_code)]
fn example_function() -> Result<()> {
    //Err(BosonError::generic("An example error"))
    unimplemented!()
}
