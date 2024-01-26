use std::{fmt, io, net, result};

#[derive(Debug)]
pub struct BosonError {
    kind: Error,
}

#[derive(Debug)]
pub enum Error {
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Generic(msg) => write!(f, "Generic error: {}", msg),
            Error::Io(err, msg) => write!(f, "IO error: {}:{}", err, msg),
            Error::Network(err, msg) => write!(f, "Network error: {}:{}", err, msg),
            Error::Argument(msg) => write!(f, "Invalid argument: {}", msg),
            Error::State(msg) => write!(f, "State error {}", msg),
            Error::Protocol(msg) => write!(f, "DHT error {}", msg),
            Error::Crypto(msg) => write!(f, "Crypto error {}", msg),
        }
    }
}

impl std::error::Error for BosonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            Error::Generic(_) => None,
            Error::Io(err,_) => Some(err),
            Error::Network(err, _) => Some(err),
            Error::Argument(_) => None,
            Error::State(_) => None,
            Error::Protocol(_) => None,
            Error::Crypto(_) => None
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
