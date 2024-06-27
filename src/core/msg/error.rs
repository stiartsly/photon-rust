use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    version,
    error::Error,
};

use super::{
    keys,
    msg::{
        Kind,
        Method,
        Msg,
        Data as MsgData
    }
};

pub(crate) struct Message {
    base_data: MsgData,
    msg: String,
    code: i32,
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
                "e" => {
                    let map = match val.as_map() {
                        Some(map) => map,
                        None => return false,
                    };

                    for (key,val) in map {
                        let key = match key.as_text() {
                            Some(key) => key,
                            None => return false,
                        };
                        match key {
                            "c" => {
                                let val = match val.as_integer() {
                                    Some(val) => val,
                                    None => return false,
                                };
                                self.code = val.try_into().unwrap();
                            },
                            "m" => {
                                let val = match val.as_text() {
                                    Some(val) => val,
                                    None => return false,
                                };
                                self.msg = val.to_string();
                            },
                            _ => {
                                return false
                            }
                        }
                    }
                }
                _=> return false,
            }
        }
        true
    }

    fn ser(&self) -> CVal {
        let val = CVal::Map(vec![
            (
                CVal::Text("c".to_string()),
                CVal::Integer(self.code.into())
            ),
            (
                CVal::Text("m".to_string()),
                CVal::Text(self.msg.clone()),
            )
        ]);
        let mut root = Msg::to_cbor(self);
        if let Some(map) = root.as_map_mut() {
            let key = CVal::Text("e".to_string());
            map.push((key, val));
        }
        root
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Message {
    pub(crate) fn new() -> Self {
        Self::with_txid(Method::Unknown, 0)
    }

    pub(crate) fn with_txid(method: Method, txid: i32) -> Self {
        Message {
            base_data: MsgData::new(Kind::Error, method, txid),
            code: 0,
            msg: String::from(""),
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for error message")
            )),
        }
    }

    pub(crate) fn msg(&self) -> &str {
        &self.msg
    }

    pub(crate) fn code(&self) -> i32 {
        self.code
    }

    pub(crate) fn with_msg(&mut self, str: &str) {
        self.msg = String::from(str);
    }

    pub(crate) fn with_code(&mut self, code: i32) {
        self.code = code
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "y:{},m:{},t:{},e:{{c:{}.m:{}}}v:{}",
            self.kind(),
            self.method(),
            self.txid(),
            self.code(),
            self.msg(),
            version::formatted_version(self.ver())
        )?;
        Ok(())
    }
}
