use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::net::SocketAddr;
use ciborium::Value as CVal;

use crate::{
    version,
    error,
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
        Kind::from(self._type)
    }

    fn method(&self) -> Method {
        Method::from(self._type)
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
        let mut nodes4 = Vec::new() as Vec<CVal>;
        if let Some(ns) = self.nodes4.as_ref() {
            ns.iter().for_each(|item| {
                nodes4.push(item.to_cbor());
            })
        }
        let mut nodes6 = Vec::new() as Vec<CVal>;
        if let Some(ns) = self.nodes6.as_ref() {
            ns.iter().for_each(|item| {
                nodes6.push(item.to_cbor())
            })
        }

        let mut lookup_res = Vec::new() as Vec<(CVal, CVal)>;
        if !nodes4.is_empty() {
            lookup_res.push((
                CVal::Text(String::from(keys::KEY_RES_NODES4)),
                CVal::Array(nodes4)
            ));
        }
        if !nodes6.is_empty() {
            lookup_res.push((
                CVal::Text(String::from(keys::KEY_RES_NODES6)),
                CVal::Array(nodes6)
            ));
        }
        lookup_res.push((
            CVal::Text(String::from(keys::KEY_RES_TOKEN)),
            CVal::Integer(self.token.into())
        ));

        CVal::Map(vec![
            (
                CVal::Text(String::from(keys::KEY_TYPE)),
                CVal::Integer(self._type.into())
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
                CVal::Text(Kind::from(self._type).to_key().to_string()),
                CVal::Map(lookup_res)
            )
        ])
    }

    fn from_cbor(&mut self, input: &CVal) -> bool {
        if let Some(root) = input.as_map() {
            for (key, val) in root {
                if !key.is_text() {
                    return false;
                }
                if let Some(_key) = key.as_text() {
                    match _key {
                        keys::KEY_TYPE => {
                            if let Some(_val) = val.as_integer() {
                                self._type = _val.try_into().unwrap()
                            }
                        },
                        keys::KEY_TXID => {
                            if let Some(_val) = val.as_integer() {
                                self.txid = _val.try_into().unwrap()
                            }
                        },
                        keys::KEY_VERSION => {
                            if let Some(_val) = val.as_integer() {
                                self.ver = _val.try_into().unwrap()
                            }
                        },
                        keys::KEY_RESPONSE => {
                            if let Some(item) = val.as_map() {
                                for (__key, _val) in item {
                                    if !__key.is_text() {
                                        return false;
                                    }
                                    if let Some(__key) = __key.as_text() {
                                        match __key {
                                            keys::KEY_RES_NODES4 => {
                                                if let Some(__val) = _val.as_array().as_ref() {
                                                    let mut nodes = Vec::new() as Vec<NodeInfo>;
                                                    __val.iter().for_each(|item| {
                                                        nodes.push(NodeInfo::try_from_cbor(item).unwrap())
                                                    });
                                                    self.nodes4 = Some(nodes);
                                                }
                                            },
                                            keys::KEY_RES_NODES6 => {
                                                if let Some(__val) = _val.as_array().as_ref() {
                                                    let mut nodes = Vec::new() as Vec<NodeInfo>;
                                                    __val.iter().for_each(|item| {
                                                        nodes.push(NodeInfo::try_from_cbor(item).unwrap())
                                                    });
                                                    self.nodes6 = Some(nodes);
                                                }
                                            },
                                            keys::KEY_RES_TOKEN => {
                                                if let Some(__val) = _val.as_integer() {
                                                    self.token = __val.try_into().unwrap();
                                                }
                                            }
                                            &_ => {
                                                println!("wrong key: {}", __key);
                                                return false;
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        &_ => {
                            println!("_key: {}", _key);
                            return false;
                        }
                    }
                }
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

    _type: i32,
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
            _type: msg::msg_type(Kind::Response, Method::FindNode),
            txid: 0,
            ver: 0,

            associated_call: None,

            nodes4: None,
            nodes6: None,
            token: 0,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Box<dyn Msg>, error::Error> {
        let mut msg = Box::new(Self::new());
        msg.from_cbor(input);
        Ok(msg as Box<dyn Msg>)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
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
