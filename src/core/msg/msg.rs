use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::net::SocketAddr;
use ciborium;
use std::fmt::Display;

use crate::id::Id;
use crate::rpccall::RpcCall;
use crate::error::Error;
use crate::node_info::NodeInfo;
use crate::peer::Peer;
use crate::value::Value;

use super::keys;
use super::cbor;
use super::error;
use super::ping_req;
use super::ping_rsp;
use super::find_node_req;
use super::find_node_rsp;
use super::announce_peer_req;
use super::announce_peer_rsp;
use super::find_peer_req;
use super::find_peer_rsp;
use super::store_value_req;
use super::store_value_rsp;
use super::find_value_req;
use super::find_value_rsp;

#[derive(PartialEq)]
pub(crate) enum Kind {
    Error = 0,
    Request = 0x20,
    Response = 0x40,
}

impl Kind {
    const MASK: i32 = 0xE0;
    pub(crate) fn from(_type: i32) -> Kind {
        let kind = _type & Self::MASK;
        match kind {
            0x00 => Kind::Error,
            0x20 => Kind::Request,
            0x40 => Kind::Response,
            _ => panic!("invalid msg kind: {}", kind)
        }
    }

    fn is_valid(_type: i32) -> bool {
        match _type & Self::MASK {
            0x00 => true,
            0x20 => true,
            0x40 => true,
            _ => false,
        }
    }

    pub(crate) fn to_key(&self) -> &'static str {
        match self {
            Kind::Error => "e",
            Kind::Request => "q",
            Kind::Response => "r",
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Kind::Error => "e",
            Kind::Request => "q",
            Kind::Response => "r",
        };
        write!(f, "{}", str)?;
        Ok(())
    }
}

#[derive(PartialEq)]
pub(crate) enum Method {
    Unknown = 0x00,
    Ping = 0x01,
    FindNode = 0x02,
    AnnouncePeer = 0x03,
    FindPeer = 0x04,
    StoreValue = 0x05,
    FindValue = 0x6,
}

impl Method {
    const MASK: i32 = 0x1F;
    pub(crate) fn from(_type: i32) -> Self {
        let method = _type & Self::MASK;
        match _type & Self::MASK {
            0x00 => Method::Unknown,
            0x01 => Method::Ping,
            0x02 => Method::FindNode,
            0x03 => Method::AnnouncePeer,
            0x04 => Method::FindPeer,
            0x05 => Method::StoreValue,
            0x06 => Method::FindValue,
            _ => panic!("invalid msg method: {}", method)
        }
    }

    fn is_valid(_type: i32) -> bool {
        let method = _type & Self::MASK;
        method >= 0 && method < 0x06
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Method::Unknown => "unknown",
            Method::Ping => "ping",
            Method::FindNode => "find_node",
            Method::AnnouncePeer => "announce_peer",
            Method::FindPeer => "find_peer",
            Method::StoreValue => "store_value",
            Method::FindValue => "find_value",
        };
        write!(f, "{}", str)?;
        Ok(())
    }
}

pub(crate) trait Msg: Display {
    fn kind(&self) -> Kind;
    fn method(&self) -> Method;

    // Common methods
    fn id(&self) -> &Id;
    fn addr(&self) -> &SocketAddr;
    fn txid(&self) -> i32;
    fn version(&self) -> i32;
    fn set_id(&mut self, _: Id);
    fn set_addr(&mut self, _: SocketAddr);
    fn set_txid(&mut self, _: i32);
    fn set_ver(&mut self, _: i32);
    fn associated_call(&self) -> Option<Rc<RefCell<RpcCall>>>;
    fn with_associated_call(&mut self, _: Rc<RefCell<RpcCall>>);

    // Methods for Node/Value/Peer Lookup as query condition.
    fn target(&self) -> &Id { panic!() }
    fn want4(&self) -> bool { panic!() }
    fn want6(&self) -> bool { panic!() }
    fn want_token(&self) -> bool { panic!() }
    fn with_target(&mut self, _: Id) { panic!() }
    fn with_want4(&mut self, _: bool) { panic!() }
    fn with_want6(&mut self, _: bool) { panic!() }
    fn with_want_token(&mut self) { panic!() }

    // Common methods for Node/Value/Peer Lookup as query result.
    fn nodes4(&self) -> &[NodeInfo] { panic!() }
    fn nodes6(&self) -> &[NodeInfo] { panic!() }
    fn token(&self) -> i32 { panic!() }

    // Common methos for Node/Value/Peer Lookup as query result.
    fn populate_closest_nodes4(&mut self, _: Vec<NodeInfo>) { panic!() }
    fn populate_closest_nodes6(&mut self, _: Vec<NodeInfo>) { panic!() }
    fn populate_token(&mut self, _:i32) { panic!() }

    // Methods for FindValue as query condition
    fn seq(&self) -> i32 { panic!() }
    fn with_seq(&mut self, _: i32) { panic!() }

    // Methods for FindPeer as query result.
    fn has_peers(&self) -> bool { panic!() }
    fn peers(&self) -> &[Box<Peer>] { panic!() }
    fn populate_peers(&mut self, _: Vec<Box<Peer>>) { panic!()}

    // Methods for Lookup Peer as query result.
    fn value(&self) -> &Option<Box<Value>> { panic!() }
    fn populate_value(&mut self, _: Box<Value>) { panic!() }


    // StoreValue option
    // fn token(&self) -> i32;
    //fn value(&self) -> &Box<Value>;

    // Methods for Error Message.
    fn msg(&self) -> &str { panic!() }
    fn code(&self) -> i32 { panic!() }
    fn with_msg(&mut self, _: &str) { panic!() }
    fn with_code(&mut self, _: i32) { panic!() }

    fn with_value(&mut self, _: Box<Value>) { panic!() }

    fn as_any(&self) -> &dyn Any;

    fn to_cbor(&self) -> ciborium::value::Value;
    fn from_cbor(&mut self, _: &ciborium::value::Value) -> bool;
}

pub(crate) fn deser(buf: &[u8]) -> Result<Rc<RefCell<dyn Msg>>, Error> {
    let mut msg_type = 0;
    let value: ciborium::value::Value;
    let reader = cbor::Reader::new(buf);
    value = ciborium::de::from_reader(reader).unwrap();
    if let Some(root) = value.as_map() {
        for (key, val) in root.iter() {
            if key.as_text().unwrap() == keys::KEY_TYPE {
                msg_type = val.as_integer().unwrap().try_into().unwrap();
            }
        }
    } else {
        return Err(Error::Protocol(
            format!("Invalid content for message deserialization")
        ));
    }

    if !Kind::is_valid(msg_type) ||!Method::is_valid(msg_type) {
        return Err(Error::Protocol(format!(
            "Invalid message kind {} or method {}", Kind::from(msg_type),Method::from(msg_type)
        )));
    }

    match Kind::from(msg_type) {
        Kind::Error => {
            error::Message::from(&value)
        },
        Kind::Request => match Method::from(msg_type) {
            Method::Ping => ping_req::Message::from(&value),
            Method::FindNode => find_node_req::Message::from(&value),
            Method::AnnouncePeer => announce_peer_req::Message::from(&value),
            Method::FindPeer => find_peer_req::Message::from(&value),
            Method::StoreValue => store_value_req::Message::from(&value),
            Method::FindValue => find_value_req::Message::from(&value),
            _ => Err(Error::Protocol(format!(
                "Invalid request message: {}, ignored it", Method::from(msg_type)
            )))
        },
        Kind::Response => match Method::from(msg_type) {
            Method::Ping => ping_rsp::Message::from(&value),
            Method::FindNode => find_node_rsp::Message::from(&value),
            Method::AnnouncePeer => announce_peer_rsp::Message::from(&value),
            Method::FindPeer => find_peer_rsp::Message::from(&value),
            Method::StoreValue => store_value_rsp::Message::from(&value),
            Method::FindValue => find_value_rsp::Message::from(&value),
            _ => Err(Error::Protocol(format!(
                "Invalid response message: {}, ignored it", Method::from(msg_type)
            )))
        }
    }
}

pub(crate) fn serialize(msg: Rc<RefCell<dyn Msg>>) -> Vec<u8> {
    let mut value = msg.borrow().to_cbor();
    let mut encoded = Vec::new() as Vec<u8>;
    let writer = cbor::Writer::new(&mut encoded);
    let _ = ciborium::ser::into_writer(&mut value, writer);
    encoded.push(0x0);
    encoded
}

pub(crate) fn msg_type(kind: Kind, method: Method) -> i32 {
    kind as i32 | method as i32
}
