use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::net::SocketAddr;
use ciborium::Value as CVal;

use crate::{
    version,
    error::Error,
    id::Id,
    node_info::NodeInfo,
    rpccall::RpcCall
};

use super::{
    keys,
    msg::{self, Kind, Method, Msg}
};

impl Msg for Message {
    fn kind(&self) -> Kind {
        Kind::from(self.kind)
    }

    fn method(&self) -> Method {
        Method::from(self.kind)
    }

    fn id(&self) -> &Id {
        &self.id.as_ref().unwrap()
    }

    fn addr(&self) -> &SocketAddr {
        &self.addr.as_ref().unwrap()
    }

    fn txid(&self) -> i32 {
        self.txid
    }

    fn version(&self) -> i32 {
        self.ver
    }

    fn set_id(&mut self, nodeid: Id) {
        self.id = Some(nodeid)
    }

    fn set_addr(&mut self, addr: SocketAddr) {
        self.addr = Some(addr)
    }

    fn set_txid(&mut self, txid: i32) {
        self.txid = txid
    }

    fn set_ver(&mut self, ver: i32) {
        self.ver = ver
    }

    fn associated_call(&self) -> Option<Rc<RefCell<RpcCall>>> {
        match self.associated_call.as_ref() {
            Some(call) => Some(Rc::clone(call)),
            None => None
        }
    }

    fn with_associated_call(&mut self, call: Rc<RefCell<RpcCall>>) {
        self.associated_call = Some(call)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_cbor(&self) -> CVal {
        let mut nodes4 = Vec::new();
        if let Some(ns) = self.nodes4.as_ref() {
            ns.iter().for_each(|item| {
                nodes4.push(item.to_cbor());
            })
        }
        let mut nodes6 = Vec::new();
        if let Some(ns) = self.nodes6.as_ref() {
            ns.iter().for_each(|item| {
                nodes6.push(item.to_cbor())
            })
        }

        let mut reply_part = Vec::new();
        if !nodes4.is_empty() {
            reply_part.push((
                CVal::Text(String::from(keys::KEY_RES_NODES4)),
                CVal::Array(nodes4)
            ));
        }
        if !nodes6.is_empty() {
            reply_part.push((
                CVal::Text(String::from(keys::KEY_RES_NODES6)),
                CVal::Array(nodes6)
            ));
        }
        reply_part.push((
            CVal::Text(String::from(keys::KEY_RES_TOKEN)),
            CVal::Integer(self.token.into())
        ));

        CVal::Map(vec![
            (
                CVal::Text(String::from(keys::KEY_TYPE)),
                CVal::Integer(self.kind.into())
            ),
            (
                CVal::Text(String::from(keys::KEY_TXID)),
                CVal::Integer(self.txid.into())
            ),
            (
                CVal::Text(String::from(keys::KEY_VERSION)),
                CVal::Integer(self.ver.into())
            ),
            (
                CVal::Text(Kind::from(self.kind).to_key().to_string()),
                CVal::Map(reply_part)
            )
        ])
    }

    fn from_cbor(&mut self, input: &CVal) -> bool {
        let root = match input.as_map() {
            Some(root) => root,
            None => return false,
        };

        for (key, val) in root {
            let key = match key.as_text() {
                Some(key) => key,
                None => return false,
            };

            match key {
                keys::KEY_TYPE => {
                    let val = match val.as_integer() {
                        Some(val) => val,
                        None => return false,
                    };
                    self.kind = val.try_into().unwrap();
                },
                keys::KEY_TXID => {
                    let val = match val.as_integer() {
                        Some(val) => val,
                        None => return false,
                    };
                    self.txid = val.try_into().unwrap();
                },
                keys::KEY_VERSION => {
                    let val = match val.as_integer() {
                        Some(val) => val,
                        None => return false,
                    };
                    self.ver = val.try_into().unwrap();
                },
                keys::KEY_RESPONSE => {
                    let map = match val.as_map() {
                        Some(map) => map,
                        None => return false,
                    };

                    for (key, val) in map {
                        let key = match key.as_text() {
                            Some(key) => key,
                            None => return false,
                        };
                        match key {
                            keys::KEY_RES_NODES4 => {
                                let array = match val.as_array() {
                                    Some(array) => array,
                                    None => return false,
                                };

                                let mut nodes = Vec::new();
                                for item in array.iter() {
                                    let ni = match NodeInfo::try_from_cbor(item) {
                                        Ok(ni) => ni,
                                        Err(_) => return false
                                    };
                                    nodes.push(ni);
                                }
                                self.nodes4 = Some(nodes);
                            },
                            keys::KEY_RES_NODES6 => {
                                let array = match val.as_array() {
                                    Some(array) => array,
                                    None => return false,
                                };

                                let mut nodes = Vec::new();
                                for item in array.iter() {
                                    let ni = match NodeInfo::try_from_cbor(item) {
                                        Ok(ni) => ni,
                                        Err(_) => return false
                                    };
                                    nodes.push(ni);
                                }
                                self.nodes6 = Some(nodes);
                            },
                            keys::KEY_RES_TOKEN => {
                                let val = match val.as_integer() {
                                    Some(val) => val,
                                    None => return false,
                                };
                                self.token = val.try_into().unwrap();
                            }
                            _ => return false
                        }
                    }
                },
                _ => return false,
            }
        }
        true
    }

    fn nodes4(&self) -> &[NodeInfo] {
        &self.nodes4.as_ref().unwrap()
    }

    fn token(&self) -> i32 {
        self.token
    }

    fn populate_closest_nodes4(&mut self, nodes: Vec<NodeInfo>) {
        self.nodes4 = Some(nodes)
    }

    fn populate_token(&mut self, token: i32) {
        self.token = token
    }
}

pub(crate) struct Message {
    id: Option<Id>,
    addr: Option<SocketAddr>,

    kind: i32,
    txid: i32,
    ver: i32,

    associated_call: Option<Rc<RefCell<RpcCall>>>,

    nodes4: Option<Vec<NodeInfo>>,
    nodes6: Option<Vec<NodeInfo>>,
    token: i32,

}

impl Message {
    pub(crate) fn new() -> Self {
        Message {
            id: None,
            addr: None,
            kind: msg::msg_type(Kind::Response, Method::FindNode),
            txid: 0,
            ver: 0,

            associated_call: None,

            nodes4: None,
            nodes6: None,
            token: 0,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Box<dyn Msg>, Error> {
        let mut msg = Box::new(Self::new());
        match msg.from_cbor(input) {
            true => Ok(msg as Box<dyn Msg>),
            false => Err(Error::Protocol(format!("Invalid cobor value for find_node request message"))),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "y:{},m:{},t:{},r:{{",
            self.kind(),
            self.method(),
            self.txid
        )?;

        match self.nodes4.as_ref() {
            Some(nodes4) => {
                let mut first = true;
                if !nodes4.is_empty() {
                    write!(f, "n4:")?;
                    for item in nodes4.iter() {
                        if !first {
                            first = false;
                            write!(f, ",")?;
                        }
                        write!(f, "[{}]", item)?;
                    }
                }
            }
            None => {}
        }

        match self.nodes6.as_ref() {
            Some(nodes6) => {
                let mut first = true;
                if !nodes6.is_empty() {
                    write!(f, "n6:")?;
                    for item in nodes6.iter() {
                        if !first {
                            first = false;
                            write!(f, ",")?;
                        }
                        write!(f, "[{}]", item)?;
                    }
                }
            }
            None => {}
        }

        if self.token != 0 {
            write!(f, ",tok:{}", self.token)?;
        }
        write!(f, "}},v:{}", version::formatted_version(self.ver))?;
        Ok(())
    }
}
