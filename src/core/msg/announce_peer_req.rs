use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use ciborium::Value as CVal;

use crate::{
    unwrap,
    version,
    Id,
    Peer,
    error::Error,
    peer::PackBuilder,
};

use super::{
    msg::{
        Kind, Method, Msg,
        Data as MsgData
    },
};

pub(crate) struct Message {
    base_data: MsgData,

    token: i32,
    peer: Option<Box<Peer>>,
}

impl Msg for Message {
    fn data(&self) -> &MsgData {
        &self.base_data
    }

    fn data_mut(&mut self) -> &mut MsgData {
        &mut self.base_data
    }

    fn from_cbor(&mut self, input: &CVal)-> bool {
        let mut peer_id = None;
        let mut proxy_id = None;
        let mut port = 0u16;
        let mut alt = None;
        let mut sig = None;

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
                            "t" => {
                                let id = match Id::try_from_cbor(v) {
                                    Ok(v) => v,
                                    Err(_) => return false,
                                };
                                peer_id = Some(id);
                            },
                            "x" => {
                                let id = match Id::try_from_cbor(v) {
                                    Ok(v) => v,
                                    Err(_) => return false,
                                };
                                proxy_id = Some(id);
                            },
                            "p" => {
                                let v = match v.as_integer() {
                                    Some(v) => v.try_into().unwrap(),
                                    None => return false,
                                };
                                port = v;
                            },
                            "alt" => {
                                let v = match v.as_text() {
                                    Some(v) => v,
                                    None => return false,
                                };
                                alt = Some(v);
                            },
                            "sig" => {
                                let v = match v.as_bytes() {
                                    Some(v) => v,
                                    None => return false,
                                };
                                sig = Some(v);
                            },
                            "tok" => {
                                let v = match v.as_integer() {
                                    Some(v) => v,
                                    None => return false,
                                };
                                self.token = v.try_into().unwrap();
                            }
                            _ => return false,
                        }
                    }
                },
                _ => return false,
            }
        }

        let mut b = PackBuilder::new();
        b.with_port(port);
        if let Some(peerid) = peer_id.take() {
            b.with_peerid(peerid);
        }
        if let Some(proxyid) = proxy_id.take() {
            b.with_nodeid(proxyid);
        }
        if let Some(alt) = alt.take() {
            b.with_alternative_url(alt);
        }
        if let Some(sig) = sig.take() {
            b.with_sigature(sig);
        }

        self.peer = Some(Box::new(b.build()));
        true
    }

    fn ser(&self) -> CVal {
        let mut req_map = vec![
            (
                CVal::Text(String::from("t")),
                unwrap!(self.peer).id().to_cbor(),
            ),
            (
                CVal::Text(String::from("w")),
                CVal::Integer(self.token.into()),
            ),
            (
                CVal::Text(String::from("p")),
                CVal::Integer(unwrap!(self.peer).port().into()),
            ),
            (
                CVal::Text(String::from("sig")),
                CVal::Bytes(unwrap!(self.peer).signature().to_vec()),
            )
        ];

        if unwrap!(self.peer).is_delegated() {
            req_map.push(
                (
                    CVal::Text(String::from("x")),
                    unwrap!(self.peer).origin().to_cbor(),
                )
            )
        }

        if let Some(alt) = unwrap!(self.peer).alternative_url() {
            req_map.push(
                (
                    CVal::Text(String::from("alt")),
                    CVal::Text(alt.to_string()),
                )
            )
        }
        let mut root = Msg::to_cbor(self);
        if let Some(map) = root.as_map_mut() {
            map.push((
                CVal::Text(Kind::Request.to_key().to_string()),
                CVal::Map(req_map)
            ));
        }
        root
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[allow(dead_code)]
impl Message {
    pub(crate) fn new() -> Self {
        Self::with_txid(0)
    }

    pub(crate) fn with_txid(txid: i32) -> Self {
        Self {
            base_data: MsgData::new(Kind::Request, Method::AnnouncePeer, txid),
            token: 0,
            peer: None,
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for announce_peer_req message")
            )),
        }
    }

    pub(crate) fn token(&self) -> i32 {
        self.token
    }

    pub(crate) fn peer(&self) -> &Box<Peer> {
        self.peer.as_ref().unwrap()
    }

    pub(crate) fn with_token(&mut self, token: i32) {
        self.token = token
    }

    pub(crate) fn with_peer(&mut self, peer: Box<Peer>) {
        self.peer = Some(peer)
    }

    pub(crate) fn target(&self) -> &Id {
        unwrap!(self.peer).id()
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "y:{},m:{},t:{},q: {{",
            self.kind(),
            self.method(),
            self.txid()
        )?;

        // TODO:

        write!(f,
            "}},v:{}",
            version::formatted_version(self.ver())
        )?;
        Ok(())
    }
}
