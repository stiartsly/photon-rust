use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use ciborium::Value as CVal;

use crate::{
    version,
    Id,
    Peer,
    NodeInfo,
    error::Error,
    peer::PackBuilder,
};

use super::{
    msg::{
        Kind, Method, Msg,
        Data as MsgData
    },
    lookup_rsp::{
        Msg as LookupResponse,
        Data as LookuResponseData
    },
};

pub(crate) struct Message {
    base_data: MsgData,
    lookup_data: LookuResponseData,

    peers: Vec<Rc<Peer>>,
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
                            "p" => {
                                let v = match v.as_array() {
                                    Some(v) => v,
                                    None => return false,
                                };

                                let peer_id = match Id::try_from_cbor(&v[0]) {
                                    Ok(v) => v,
                                    Err(_) => return false ,
                                };
                                for item in v.iter() {
                                    if item.is_null() {
                                        // Skip
                                    } else if item.is_bytes() {
                                        // Skip
                                    } else if item.is_array() {
                                        let v = match item.as_array() {
                                            Some(v) => v,
                                            None => return false,
                                        };

                                        let node_id = match Id::try_from_cbor(&v[0]) {
                                            Ok(id) => id,
                                            Err(_) => return false,
                                        };
                                        let _ = match Id::try_from_cbor(&v[1]) {
                                            Ok(id) => Some(id),
                                            Err(_) => None,
                                        };
                                        let port = match v[2].as_integer() {
                                            Some(v) => v.try_into().unwrap(),
                                            None => return false,
                                        };
                                        let alt = match v[3].as_text() {
                                            Some(v) => Some(v),
                                            None => None,
                                        };
                                        let sig = match v[4].as_bytes() {
                                            Some(v) => v,
                                            None => return false,
                                        };

                                        let mut b = PackBuilder::new();
                                        b.with_peerid(peer_id.clone());
                                        b.with_nodeid(node_id);
                                        //if let Some(origin) = origin {
                                           // b.with_origin(origin);
                                        //}
                                        b.with_port(port);
                                        if let Some(alt) = alt {
                                            b.with_alternative_url(alt);
                                        }
                                        b.with_sigature(sig);
                                        self.peers.push(Rc::new(b.build()));
                                    } else {
                                        return false;
                                    }
                                };
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
        let mut array = vec![];

        if self.peers.len() > 0 {
            let peer_id = self.peers[0].id();
            array.push(peer_id.to_cbor());
        }

        self.peers.iter().for_each(|item| {
            let node_id = item.node_id().to_cbor();
            let port = CVal::Integer(item.port().into());
            let sig = CVal::Bytes(item.signature().to_vec());

            let origin:CVal;
            if item.is_delegated() {
                origin = item.origin().to_cbor();
            } else {
                origin = CVal::Null;
            }

            let alt: CVal;
            if let Some(url) = item.alternative_url() {
                alt = CVal::Text(url.to_string());
            } else {
                alt = CVal::Null;
            }

            let mut peer = vec![];
            peer.push(node_id);
            peer.push(origin);
            peer.push(port);
            peer.push(alt);
            peer.push(sig);

            array.push(CVal::Array(peer));
        });

        let mut rsp = LookupResponse::to_cbor(self);
        if let Some(map) = rsp.as_map_mut() {
            let key = CVal::Text(String::from("p"));
            map.push((key, CVal::Array(array)));
        }

        let mut root = Msg::to_cbor(self);
        if let Some(map) = root.as_map_mut() {
            let key = CVal::Text(String::from("r"));
            map.push((key, rsp));
        }
        root
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LookupResponse for Message {
    fn data(&self) -> &LookuResponseData {
        &self.lookup_data
    }

    fn data_mut(&mut self) -> &mut LookuResponseData {
        &mut self.lookup_data
    }
}

impl Message {
    pub(crate) fn new() -> Self {
        Self::with_txid(0)
    }

    pub(crate) fn with_txid(txid: i32) -> Self {
        Self {
            base_data: MsgData::new(Kind::Response, Method::FindPeer, txid),
            lookup_data: LookuResponseData::new(),
            peers: Vec::new(),
        }
    }

    pub(crate) fn from(input: &CVal) -> Result<Rc<RefCell<dyn Msg>>, Error> {
        let mut msg = Self::new();
        match msg.from_cbor(input) {
            true => Ok(Rc::new(RefCell::new(msg))),
            false => Err(Error::Protocol(
                format!("Invalid cobor value for find_peer_rsp message")
            )),
        }
    }

    pub(crate) fn peers(&self) -> &[Rc<Peer>] {
        self.peers.as_ref()
    }

    pub(crate) fn populate_peers(&mut self, peers: Vec<Rc<Peer>>) {
        self.peers = peers
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "y:{},m:{},t:{},r: {{",
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

        let mut first = true;
        if !self.peers.is_empty() {
            write!(f, ",p:")?;
            for item in self.peers.iter() {
                if !first {
                    first = false;
                    write!(f, ",")?;
                }
                write!(f, "[{}]", item)?;
            }
        }

        write!(f,
            "}},v:{}",
            version::formatted_version(self.ver())
        )?;
        Ok(())
    }
}
