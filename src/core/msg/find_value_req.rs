use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use ciborium::Value as CVal;

use crate::{
    version,
    id::Id,
    error::Error,
};

use super::{
    msg::{
        Kind, Method, Msg,
        Data as MsgData
    },
    lookup_req::{
        Msg as LookupRequest,
        Data as LookupRequestData
    },
};

pub(crate) struct Message {
    base_data: MsgData,
    lookkup_data: LookupRequestData,

    seq: i32,
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
                    for (k,v) in map {
                        let k = match k.as_text() {
                            Some(k) => k,
                            None => return false,
                        };
                        match k {
                            "w" => {
                                let v = match v.as_integer() {
                                    Some(v) => v,
                                    None => return false,
                                };
                                let want:i32 = v.try_into().unwrap();
                                self.with_want4((want & 0x01) != 0);
                                self.with_want6((want & 0x02) != 0);
                                self.with_want_token((want & 0x04) != 0);
                            },
                            "t" => {
                                let id = match Id::try_from_cbor(v) {
                                    Ok(id) => id,
                                    Err(_) => return false,
                                };
                                self.with_target(Rc::new(id))
                            },
                            "seq" => {
                                let v = match v.as_integer() {
                                    Some(v) => v,
                                    None => return false,
                                };
                                let seq = v.try_into().unwrap();
                                if seq >= 0 {
                                    self.seq = seq;
                                }
                            }
                            _ => return false,
                        }
                    }
                },
                _ => return false,
            }
        }
        true
    }

    fn ser(&self) -> CVal {
        let mut val = LookupRequest::to_cbor(self);
        if let Some(map) = val.as_map_mut() {
            map.push((
                CVal::Text(String::from("seq")),
                CVal::Integer(self.seq.into())
            ));
        }

        let mut root = Msg::to_cbor(self);
        if let Some(map) = root.as_map_mut() {
            map.push((
                CVal::Text(Kind::Request.to_key().to_string()),
                val
            ));
        }
        root
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LookupRequest for Message {
    fn data(&self) -> &LookupRequestData {
        &self.lookkup_data
    }

    fn data_mut(&mut self) -> &mut LookupRequestData {
        &mut self.lookkup_data
    }
}

impl Message {
    pub(crate) fn new() -> Self {
        Self::with_txid(0)
    }

    pub(crate) fn with_txid(txid: i32) -> Self {
        Self {
            base_data: MsgData::new(Kind::Request, Method::FindValue, txid),
            lookkup_data: LookupRequestData::new(),
            seq: -1
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for find_value_req message")
            )),
        }
    }

    pub(crate) fn seq(&self) -> i32 {
        self.seq
    }

    pub(crate) fn with_seq(&mut self, seq: i32) {
        self.seq = seq
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "y:{},m:{},t:{},q:{{t:{},w:{}}}",
            self.kind(),
            self.method(),
            self.txid(),
            self.target(),
            self.want()
        )?;
        if self.seq >= 0 {
            write!(f, ",seq:{}", self.seq)?;
        }

        write!(f,
            ",v:{}",
            version::formatted_version(self.ver())
        )?;
        Ok(())
    }
}
