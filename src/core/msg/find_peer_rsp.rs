use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt;
use ciborium::Value as CVal;

use crate::{
    version,
    id::Id,
    error::Error,
    peer::{Peer, PackBuilder},
    node_info::NodeInfo,
};

use super::{
        msg::{
        Kind,
        Method,
        Msg,
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

    peers: Vec<Box<Peer>>,
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
                    let val = match val.as_integer() {
                        Some(val) => val,
                        None => return false,
                    };
                    let txid = val.try_into().unwrap();
                    self.set_txid(txid);
                },
                "v" => {
                    let val = match val.as_integer() {
                        Some(val) => val,
                        None => return false,
                    };
                    let ver = val.try_into().unwrap();
                    self.set_ver(ver);
                },
                "r" => {
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
                            "n4" => {
                                let array = match val.as_array() {
                                    Some(array) => array,
                                    None => return false,
                                };

                                let mut nodes = Vec::new();
                                for item in array.iter() {
                                    match NodeInfo::try_from_cbor(item) {
                                        Ok(ni) => nodes.push(ni),
                                        Err(_) => return false
                                    };
                                }
                                self.populate_closest_nodes4(nodes);
                            },
                            "n6" => {
                                let array = match val.as_array() {
                                    Some(array) => array,
                                    None => return false,
                                };

                                let mut nodes = Vec::new();
                                for item in array.iter() {
                                    match NodeInfo::try_from_cbor(item) {
                                        Ok(ni) => nodes.push(ni),
                                        Err(_) => return false
                                    };
                                }
                                self.populate_closest_nodes6(nodes);
                            },
                            "tok" => {
                                let val = match val.as_integer() {
                                    Some(val) => val,
                                    None => return false,
                                };
                                let token = val.try_into().unwrap();
                                self.populate_token(token);
                            },
                            "p" => {
                                let array = match val.as_array() {
                                    Some(val) => val,
                                    None => return false,
                                };

                                let peer_id = match Id::try_from_cbor(&array[0]) {
                                    Ok(val) => val,
                                    Err(_) => return false ,
                                };
                                for item in array.iter() {
                                    if item.is_null() {
                                        // Do nothing.
                                    } else if item.is_bytes() {
                                        // DO nothing;
                                    } else if item.is_array() {
                                        let arr = match item.as_array() {
                                            Some(val) => val,
                                            None => return false,
                                        };

                                        let node_id = match Id::try_from_cbor(&arr[0]) {
                                            Ok(id) => id,
                                            Err(_) => return false,
                                        };
                                        let _ = match Id::try_from_cbor(&arr[1]) {
                                            Ok(id) => Some(id),
                                            Err(_) => None,
                                        };
                                        let port = match arr[2].as_integer() {
                                            Some(val) => val.try_into().unwrap(),
                                            None => return false,
                                        };
                                        let alt = match arr[3].as_text() {
                                            Some(val) => Some(val),
                                            None => None,
                                        };
                                        let sig = match arr[4].as_bytes() {
                                            Some(val) => val,
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
                                        self.peers.push(Box::new(b.build()));
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

#[allow(dead_code)]
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

    pub(crate) fn has_peers(&self) -> bool {
        !self.peers.is_empty()
    }

    pub(crate) fn peers(&self) -> &[Box<Peer>] {
        &self.peers
    }

    pub(crate) fn populate_peers(&mut self, peers: Vec<Box<Peer>>) {
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
