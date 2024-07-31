use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use ciborium::Value as CVal;

use crate::{
    version,
    NodeInfo,
    error::Error
};

use super::{
    msg::{
        Kind, Method, Msg,
        Data as MsgData
    },
    lookup_rsp::{
        Msg as LookupResponse,
        Data as LookupResponseData
    },
};

pub(crate) struct Message {
    base_data: MsgData,
    lookup_data: LookupResponseData,
}

impl Msg for Message {
    fn data(&self) -> &MsgData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut MsgData {
        &mut self.base_data
    }

    fn from_cbor(&mut self, input: &CVal) -> bool {
        let root = match input.as_map() {
            Some(root) => root,
            None => return false,
        };

        for (k, v) in root {
            let k = match k.as_text() {
                Some(k) => k,
                None => return false,
            };

            match k {
                "y" => {},
                "t" => {
                    let v = match v.as_integer() {
                        Some(v) => v,
                        None => return false,
                    };
                    let txid = v.try_into().unwrap();
                    self.set_txid(txid);
                },
                "v" => {
                    let v = match v.as_integer() {
                        Some(v) => v,
                        None => return false,
                    };
                    let ver = v.try_into().unwrap();
                    self.set_ver(ver);
                },
                "r" => {
                    let map = match v.as_map() {
                        Some(v) => v,
                        None => return false,
                    };

                    for (k, v) in map {
                        let k = match k.as_text() {
                            Some(k) => k,
                            None => return false,
                        };
                        match k {
                            "n4" => {
                                let v = match v.as_array() {
                                    Some(v) => v,
                                    None => return false,
                                };

                                let mut nodes = Vec::new();
                                for item in v.iter() {
                                    match NodeInfo::try_from_cbor(item) {
                                        Ok(ni) => nodes.push(Rc::new(ni)),
                                        Err(_) => return false
                                    };
                                }
                                self.populate_closest_nodes4(nodes);
                            },
                            "n6" => {
                                let v = match v.as_array() {
                                    Some(v) => v,
                                    None => return false,
                                };

                                let mut nodes = Vec::new();
                                for item in v.iter() {
                                    match NodeInfo::try_from_cbor(item) {
                                        Ok(ni) => nodes.push(Rc::new(ni)),
                                        Err(_) => return false
                                    };
                                }
                                self.populate_closest_nodes6(nodes);
                            },
                            "tok" => {
                                let v = match v.as_integer() {
                                    Some(v) => v,
                                    None => return false,
                                };
                                let token = v.try_into().unwrap();
                                self.populate_token(token);
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

    fn ser(&self) -> CVal {
        let mut root = Msg::to_cbor(self);
        if let Some(map) = root.as_map_mut() {
            map.push(
                (CVal::Text(String::from("r")),
                LookupResponse::to_cbor(self)
            ));
        }
        root
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LookupResponse for Message {
    fn data(&self) -> &LookupResponseData {
        &self.lookup_data
    }

    fn data_mut(&mut self) -> &mut LookupResponseData {
        &mut self.lookup_data
    }
}

impl Message {
    pub(crate) fn new() -> Self {
        Self::with_txid(0)
    }

    pub(crate) fn with_txid(txid: i32) -> Self {
        Message {
            base_data: MsgData::new(Kind::Response, Method::FindNode, txid),
            lookup_data: LookupResponseData::new(),
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for find_node_rsp message")
            )),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "y:{},m:{},t:{},r:{{",
            self.kind(),
            self.method(),
            self.txid()
        )?;

        if let Some(nodes4) = self.nodes4() {
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

        if let Some(nodes6) = self.nodes6() {
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

        if self.token() != 0 {
            write!(f, ",tok:{}", self.token())?;
        }
        write!(f,
            "}},v:{}",
            version::formatted_version(self.ver())
        )?;
        Ok(())
    }
}
