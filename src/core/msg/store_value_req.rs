use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    id::Id,
    error::Error,
    value::{Value, PackBuilder},
};

use super::{
    msg::{
        Kind,
        Method,
        Msg,
        Data as MsgData
    }
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

        for (key, val) in root {
            let key = match key.as_text() {
                Some(key) => key,
                None => return false,
            };
            match key {
                "y" => {},
                "t" => {
                    let txid = match val.as_integer() {
                        Some(txid) => txid,
                        None => return false,
                    };
                    let txid = txid.try_into().unwrap();
                    self.set_txid(txid);
                },
                "v" => {
                    let ver = match val.as_integer() {
                        Some(ver) => ver,
                        None => return false,
                    };
                    let ver = ver.try_into().unwrap();
                    self.set_ver(ver);
                },
                "q" => {
                    let mut pk = None;
                    let mut recipient = None;
                    let mut nonce = None;
                    let mut sig = None;
                    let mut seq = 0;
                    let mut data = None;

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
                            "k" => { // publickey
                                let id = match Id::try_from_cbor(val) {
                                    Ok(id) => id,
                                    Err(_) => return false,
                                };
                                pk = Some(id);
                            },
                            "rec" => { // recipient
                                let id = match Id::try_from_cbor(val) {
                                    Ok(id) => id,
                                    Err(_) => return false,
                                };
                                recipient = Some(id);
                            },
                            "n" => { // nonce
                                let val = match val.as_bytes() {
                                    Some(val) => val,
                                    None => return false,
                                };
                                nonce = Some(val);
                            },
                            "s" => { // signature
                                let val = match val.as_bytes() {
                                    Some(val) => val,
                                    None => return false,
                                };
                                sig = Some(val);
                            },
                            "seq" => { // sequence number
                                let val = match val.as_integer() {
                                    Some(val) => val,
                                    None => return false
                                };
                                seq = val.try_into().unwrap();
                            },
                            "cas" => { // expected sequence number.
                                let val = match val.as_integer() {
                                    Some(val) => val,
                                    None => return false
                                };
                                self.expected_seq = val.try_into().unwrap();
                            },
                            "tok" => {  // token
                                let val = match val.as_integer() {
                                    Some(val) => val,
                                    None => return false,
                                };
                                self.token = val.try_into().unwrap();
                            },
                            "v" => { // value
                                let val = match val.as_bytes() {
                                    Some(val) => val,
                                    None => return false,
                                };
                                data = Some(val);
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
        unimplemented!()
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
