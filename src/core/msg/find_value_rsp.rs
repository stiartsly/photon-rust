use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use ciborium::Value as CVal;

use crate::{
    Id,
    NodeInfo,
    Value,
    error::Error,
    value::PackBuilder,
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

    value: Option<Rc<Value>>,
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

        for (k,v) in root {
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
                    let mut pk = None;
                    let mut recipient = None;
                    let mut nonce = None;
                    let mut sig = None;
                    let mut data = None;
                    let mut seq = -1;

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
                            },
                            "k" => { // publickey
                                let id = match Id::try_from_cbor(v) {
                                    Ok(id) => id,
                                    Err(_) => return false,
                                };
                                pk = Some(id);
                            },
                            "rec" => { // recipient
                                let id = match Id::try_from_cbor(v) {
                                    Ok(id) => id,
                                    Err(_) => return false,
                                };
                                recipient = Some(id);
                            },
                            "n" => { // nonce
                                let v = match v.as_bytes() {
                                    Some(v) => v,
                                    None => return false,
                                };
                                nonce = Some(v);
                            },
                            "s" => { // signature
                                let v = match v.as_bytes() {
                                    Some(v) => v,
                                    None => return false,
                                };
                                sig = Some(v);
                            },
                            "seq" => { // sequence number
                                let v = match v.as_integer() {
                                    Some(v) => v,
                                    None => return false
                                };
                                seq = v.try_into().unwrap();
                            },
                            "v" => { // value
                                let v = match v.as_bytes() {
                                    Some(v) => v,
                                    None => return false,
                                };
                                data = Some(v);
                            },

                            _ => return false
                        }
                    }

                    let data = match data.take() {
                        Some(data) => data,
                        None => return false,
                    };

                    let mut b = PackBuilder::default(data);
                    if let Some(pk) = pk.take() {
                        b.with_pk(pk);
                    }
                    if let Some(rec) = recipient.take() {
                        b.with_recipient(rec);
                    }
                    if let Some(nonce) = nonce.take() {
                        b.with_nonce(nonce);
                    }
                    if let Some(sig) = sig.take() {
                        b.with_sig(sig);
                    }
                    b.with_seq(seq);
                    self.value = Some(Rc::new(b.build()));
                },
                _ => return false,
            }
        }

        true
    }

    fn ser(&self) -> CVal {
        let mut val = LookupResponse::to_cbor(self);
        if let Some(map) = val.as_map_mut() {
            let value = match self.value.as_ref() {
                Some(val) => val,
                None => {
                    panic!("value is missing.");
                },
            };
            if let Some(pk) = value.public_key() {
                map.push((
                    CVal::Text("k".to_string()),
                    CVal::Bytes(pk.as_bytes().to_vec())
                ));
            }
            if let Some(rec) = value.recipient() {
                map.push((
                    CVal::Text("rec".to_string()),
                    CVal::Bytes(rec.as_bytes().to_vec()),
                ));
            }
            if let Some(nonce) = value.nonce() {
                map.push((
                    CVal::Text("n".to_string()),
                    CVal::Bytes(nonce.as_bytes().to_vec()),
                ));
            }
            if value.sequence_number() >= 0 {
                map.push((
                    CVal::Text("seq".to_string()),
                    CVal::Integer(value.sequence_number().into()),
                ));
            }
            if let Some(sig) = value.signature() {
                map.push((
                    CVal::Text("s".to_string()),
                    CVal::Bytes(sig.to_vec()),
                ));
            }
            map.push((
                CVal::Text("v".to_string()),
                CVal::Bytes(value.data().to_vec())
            ));
        }

        let mut root = Msg::to_cbor(self);
        if let Some(map) = root.as_map_mut() {
            map.push((
                CVal::Text(String::from("r")),
                val
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
        Self {
            base_data: MsgData::new(Kind::Response, Method::FindValue, txid),
            lookup_data: LookupResponseData::new(),
            value: None,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for find_value_rsp message")
            )),
        }
    }

    pub(crate) fn value(&self) -> &Option<Rc<crate::value::Value>> {
        &self.value
    }

    // pub(crate) fn has_value(&self) -> bool {
    //    self.value.is_some()
    // }

    pub(crate) fn populate_value(&mut self, value: Rc<crate::value::Value>) {
        self.value = Some(value)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(val) = self.value.as_ref() {
            write!(f, "{}", val)?;
        }
        Ok(())
    }
}
