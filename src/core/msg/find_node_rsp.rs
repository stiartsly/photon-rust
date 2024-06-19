use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    version,
    error::Error,
    node_info::NodeInfo,
};

use super::{
    keys,
    msg::{Kind, Method, Msg, Data as MsgData},
    lookup_rsp::{Msg as LookupResponse, Data as LookupData },
};

pub(crate) struct Message {
    base_data: MsgData,
    lookup_data: LookupData,
}

impl Msg for Message {
    fn data(&self) -> &MsgData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut MsgData {
        &mut self.base_data
    }

    fn to_cbor(&self) -> CVal {
        let mut nodes4 = Vec::new();
        if let Some(ns) = self.nodes4() {
            ns.iter().for_each(|item| {
                nodes4.push(item.to_cbor());
            })
        }
        let mut nodes6 = Vec::new();
        if let Some(ns) = self.nodes6() {
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
            CVal::Integer(self.token().into())
        ));

        CVal::Map(vec![
            (
                CVal::Text(String::from(keys::KEY_TYPE)),
                CVal::Integer(self._type().into())
            ),
            (
                CVal::Text(String::from(keys::KEY_TXID)),
                CVal::Integer(self.txid().into())
            ),
            (
                CVal::Text(String::from(keys::KEY_VERSION)),
                CVal::Integer(self.ver().into())
            ),
            (
                CVal::Text(Kind::Response.to_key().to_string()),
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
                keys::KEY_TYPE => {},
                keys::KEY_TXID => {
                    let val = match val.as_integer() {
                        Some(val) => val,
                        None => return false,
                    };
                    self.set_txid(val.try_into().unwrap());
                },
                keys::KEY_VERSION => {
                    let val = match val.as_integer() {
                        Some(val) => val,
                        None => return false,
                    };
                    self.set_ver(val.try_into().unwrap());
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
                                self.populate_closest_nodes4(nodes);
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
                                self.populate_closest_nodes6(nodes);
                            },
                            keys::KEY_RES_TOKEN => {
                                let val = match val.as_integer() {
                                    Some(val) => val,
                                    None => return false,
                                };
                                self.populate_token(val.try_into().unwrap());
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LookupResponse for Message {
    fn data(&self) -> &LookupData {
        &self.lookup_data
    }

    fn data_mut(&mut self) -> &mut LookupData {
        &mut self.lookup_data
    }
}

#[allow(dead_code)]
impl Message {
    pub(crate) fn new() -> Self {
        Self::with_txid(0)
    }

    pub(crate) fn with_txid(txid: i32) -> Self {
        Message {
            base_data: MsgData::new(Kind::Response, Method::FindNode, txid),
            lookup_data: LookupData::new(),
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
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
            self.txid()
        )?;

        match self.nodes4() {
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

        match self.nodes6() {
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

        if self.token() != 0 {
            write!(f, ",tok:{}", self.token())?;
        }
        write!(f, "}},v:{}", version::formatted_version(self.ver()))?;
        Ok(())
    }
}
