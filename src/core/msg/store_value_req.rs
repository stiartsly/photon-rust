use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use ciborium::Value as CVal;

use crate::{
    Id,
    Value,
    error::Error,
    value::PackBuilder,
};

use super::{
    msg::{
        Kind, Method, Msg,
        Data as MsgData,
    },

};

pub(crate) struct Message {
    base_data: MsgData,

    token: i32,
    expected_seq: i32,
    value: Option<Box<Value>>,
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
                "q" => {
                    let mut pk = None;
                    let mut recipient = None;
                    let mut nonce = None;
                    let mut sig = None;
                    let mut seq = 0;
                    let mut data = None;

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
                            "k" => { // publickey
                                let id = match Id::try_from_cbor(v) {
                                    Ok(v) => v,
                                    Err(_) => return false,
                                };
                                pk = Some(id);
                            },
                            "rec" => { // recipient
                                let id = match Id::try_from_cbor(v) {
                                    Ok(v) => v,
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
                            "cas" => { // expected sequence number.
                                let v = match v.as_integer() {
                                    Some(v) => v,
                                    None => return false
                                };
                                self.expected_seq = v.try_into().unwrap();
                            },
                            "tok" => {  // token
                                let v = match v.as_integer() {
                                    Some(v) => v,
                                    None => return false,
                                };
                                self.token = v.try_into().unwrap();
                            },
                            "v" => { // value
                                let v = match v.as_bytes() {
                                    Some(v) => v,
                                    None => return false,
                                };
                                data = Some(v);
                            },
                            _ => return false,
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
                    self.value = Some(Box::new(b.build()));
                },
                _ => return false,
            }
        }
        true
    }

    fn ser(&self) -> CVal {
        let mut val = CVal::Map(vec![]);
        if let Some(map) = val.as_map_mut() {
            map.push((
                CVal::Text("tok".to_string()),
                CVal::Integer(self.token.into())
            ));

            let value = match self.value.as_ref() {
                Some(val) => val,
                None => {
                    panic!("missing value field");
                }
            };
            if let Some(pk) = value.public_key() {
                map.push((
                    CVal::Text(String::from("k")),
                    CVal::Bytes(pk.as_bytes().into())
                ));
            }
            if let Some(rec) = value.recipient() {
                map.push((
                    CVal::Text(String::from("rec")),
                    CVal::Bytes(rec.as_bytes().into())
                ));
            }
            if let Some(nonce) = value.nonce() {
                map.push((
                    CVal::Text(String::from("n")),
                    CVal::Bytes(nonce.as_bytes().into())
                ));
            }
            if let Some(sig) = value.signature() {
                map.push((
                    CVal::Text(String::from("s")),
                    CVal::Bytes(sig.into()),
                ))
            }
            if value.sequence_number() >= 0 {
                map.push((
                    CVal::Text(String::from("seq")),
                    CVal::Integer(value.sequence_number().into())
                ));
            }
            if self.expected_seq >= 0 {
                map.push((
                    CVal::Text(String::from("cas")),
                    CVal::Integer(self.expected_seq.into())
                ));
            }
            map.push((
                CVal::Text(String::from("v")),
                CVal::Bytes(value.data().into()),
            ));
        }

        let mut root = Msg::to_cbor(self);
        if let Some(map) = root.as_map_mut() {
            map.push((
                CVal::Text(String::from("q")),
                val
            ));
        }
        root
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Message {
    pub(crate) fn new() -> Self {
        Self::with_txid(0)
    }

    pub(crate) fn with_txid(txid: i32) -> Self {
        Self {
            base_data: MsgData::new(Kind::Request, Method::StoreValue, txid),
            token: 0,
            expected_seq: -1,
            value: None,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for store_value_req message")
            )),
        }
    }

    pub(crate) fn token(&self) -> i32 {
        self.token
    }

    // pub(crate) fn with_token(&mut self, token: i32) {
    //    self.token = token
    //}

    pub(crate) fn value(&self) -> &Option<Box<Value>> {
        &self.value
    }

    // pub(crate) fn with_value(&mut self, value: Box<Value>) {
    //    self.value = Some(value)
    //}
}

impl fmt::Display for Message {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!();
    }
}
