use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    version,
    id::Id,
    error::Error,
};

use super::{
    keys,
    msg::{
        Kind,
        Method,
        Msg,
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

        for (key, val) in root {
            let key = match key.as_text() {
                Some(key) => key,
                None => return false,
            };
            match key {
                keys::KEY_TYPE => {},
                keys::KEY_TXID => {
                    let txid = match val.as_integer() {
                        Some(txid) => txid,
                        None => return false,
                    };
                    self.set_txid(txid.try_into().unwrap());
                },
                keys::KEY_VERSION => {
                    let ver = match val.as_integer() {
                        Some(ver) => ver,
                        None => return false,
                    };
                    self.set_ver(ver.try_into().unwrap());
                },
                keys::KEY_REQUEST => {
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
                            keys::KEY_REQ_WANT => {
                                let val = match val.as_integer() {
                                    Some(val) => val,
                                    None => return false,
                                };
                                let _want: i32 = val.try_into().unwrap();
                                self.with_want4((_want & 0x01) != 0);
                                self.with_want6((_want & 0x01) != 0);
                            },
                            keys::KEY_REQ_TARGET => {
                                let id = match Id::try_from_cbor(val) {
                                    Ok(id) => id,
                                    Err(_) => return false,
                                };
                                self.with_target(id)
                            },
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
        let mut root = Msg::to_cbor(self);
        if let Some(map) = root.as_map_mut() {
            let key = CVal::Text(Kind::Request.to_key().to_string());
            let val = LookupRequest::to_cbor(self);
            map.push((key, val));
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
        Message {
            base_data: MsgData::new(Kind::Request, Method::FindPeer, txid),
            lookkup_data: LookupRequestData::new(),
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for find_peer_req message")
            )),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "y:{},m:{},t:{},q:{{t:{},w:{}}},v:{}",
            self.kind(),
            self.method(),
            self.txid(),
            self.target(),
            self.want(),
            version::formatted_version(self.ver())
        )?;
        Ok(())
    }
}
