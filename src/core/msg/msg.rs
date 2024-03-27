use std::any::Any;
use std::rc::Rc;
use std::fmt;
use std::net::SocketAddr;
use ciborium;
use std::fmt::Debug;
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

pub(crate) trait Msg: Debug + Display {
    fn kind(&self) -> Kind;
    fn method(&self) -> Method;

    // Common methods
    fn id(&self) -> &Id;
    fn addr(&self) -> &SocketAddr;
    fn remote_id(&self) -> &Id;
    fn remote_addr(&self) -> &SocketAddr;
    fn txid(&self) -> i32;
    fn version(&self) -> i32;
    fn set_id(&mut self, _: &Id);
    fn set_addr(&mut self, _: &SocketAddr);
    fn set_remote_id(&mut self, _: &Id);
    fn set_remote_addr(&mut self, _: &SocketAddr);
    fn set_txid(&mut self, _: i32);
    fn set_ver(&mut self, _: i32);
    fn associated_call(&self) -> Option<Box<RpcCall>>;
    fn with_associated_call(&mut self, _: Box<RpcCall>);

    // Methods for Node/Value/Peer Lookup as query condition.
    fn target(&self) -> &Id { panic!() }
    fn want4(&self) -> bool { panic!() }
    fn want6(&self) -> bool { panic!() }
    fn want_token(&self) -> bool { panic!() }
    fn with_target(&mut self, _: &Id) { panic!() }
    fn with_want4(&mut self) { panic!() }
    fn with_want6(&mut self) { panic!() }
    fn with_token(&mut self) { panic!() }

    // Common methods for Node/Value/Peer Lookup as query result.
    fn nodes4(&self) -> &[NodeInfo] { panic!() }
    fn nodes6(&self) -> &[NodeInfo] { panic!() }
    fn token(&self) -> i32 { panic!() }

    // Common methos for Node/Value/Peer Lookup as query result.
    fn populate_closest_nodes4(&mut self, _: Box<dyn FnOnce() -> Vec<NodeInfo> +'static>) { panic!() }
    fn populate_closest_nodes6(&mut self, _: Box<dyn FnOnce() -> Vec<NodeInfo> +'static>) { panic!() }
    fn populate_token(&mut self, _: bool, _: Box<dyn FnOnce() -> i32>) { panic!() }

    // Methods for FindValue as query condition
    fn seq(&self) -> i32 { panic!() }
    fn with_seq(&mut self, _: i32) { panic!() }

    // Methods for FindPeer as query result.
    fn has_peers(&self) -> bool { panic!() }
    fn peers(&self) -> &[Box<Peer>] { panic!() }
    fn populate_peers(&mut self, _: Box<dyn FnMut() -> Option<Vec<Box<Peer>>>>) { panic!()}

    // Methods for Lookup Peer as query result.
    fn value(&self) -> &Option<Box<Value>> { panic!() }
    fn populate_value(&mut self, _: Box<dyn FnMut() -> Option<Box<Value>>>) { panic!() }


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

    //
}

pub(crate) fn deser(id: &Id, from: &SocketAddr, buf: &[u8]) -> Result<Rc<dyn Msg>, Error> {
    let mut _type: i32 = 0;
    let reader = cbor::Reader::new(buf);
    println!("desr buf len: {}", buf.len());
    let value: ciborium::value::Value = ciborium::de::from_reader(reader).unwrap();

    if let Some(root) = value.as_map() {
        for (key, val) in root.iter() {
            if key.as_text().unwrap() == keys::KEY_TYPE {
                _type = val.as_integer().unwrap().try_into().unwrap();
            }
        }
    } else {
        // TODO:
    }

    if !Kind::is_valid(_type) ||!Method::is_valid(_type) {
        return Err(Error::Protocol(format!(
            "Invalid message kind {} or method {}",
            Kind::from(_type),
            Method::from(_type)
        )))
    }

    let msg = match Kind::from(_type) {
        Kind::Error => {
            Rc::new(error::Message::from(&value)) as Rc<dyn Msg>
        },
        Kind::Request => match Method::from(_type) {
            Method::Ping => Rc::new(ping_req::Message::from(&value)) as Rc<dyn Msg>,
            Method::FindNode => Rc::new(find_node_req::Message::from(id, from, &value)) as Rc<dyn Msg>,
            Method::AnnouncePeer => Rc::new(announce_peer_req::Message::from(&value)) as Rc<dyn Msg>,
            Method::FindPeer => Rc::new(find_peer_req::Message::from(&value)) as Rc<dyn Msg>,
            Method::StoreValue => Rc::new(store_value_req::Message::from(&value)) as Rc<dyn Msg>,
            Method::FindValue => Rc::new(find_value_req::Message::from(&value)) as Rc<dyn Msg>,
            _ => { return Err(Error::Protocol(
                format!("Invalid request message: {}, ignored it", Method::from(_type))
            ))}
        },
        Kind::Response => match Method::from(_type) {
            Method::Ping => Rc::new(ping_rsp::Message::from(&value)) as Rc<dyn Msg>,
            Method::FindNode => Rc::new(find_node_rsp::Message::from(&value)) as Rc<dyn Msg>,
            Method::AnnouncePeer => Rc::new(announce_peer_rsp::Message::from(&value)) as Rc<dyn Msg>,
            Method::FindPeer => Rc::new(find_peer_rsp::Message::from(&value)) as Rc<dyn Msg>,
            Method::StoreValue => Rc::new(store_value_rsp::Message::from(&value)) as Rc<dyn Msg>,
            Method::FindValue => Rc::new(find_value_rsp::Message::from(&value)) as Rc<dyn Msg>,
            _ => { return Err(Error::Protocol(
                format!("Invalid response message: {}, ignored it", Method::from(_type))
            ))}
        }
    };
    Ok(msg)
}

pub(crate) fn serialize(msg: &Box<dyn Msg>) -> Vec<u8> {
    let mut value = msg.to_cbor();
    let mut encoded = Vec::new() as Vec<u8>;
    let writer = cbor::Writer::new(encoded.as_mut());
    let _ = ciborium::ser::into_writer(&mut value, writer);
    println!("serializer, len: {}", encoded.len());
    encoded.push(0x0);
    encoded
}

pub(crate) fn msg_type(kind: Kind, method: Method) -> i32 {
    kind as i32 | method as i32
}
