
use crate::Network;

#[derive(Clone, Debug)]
pub struct Compound<T> {
    v4: Option<T>,
    v6: Option<T>,
}

impl<T> Compound<T> {
    pub fn new() -> Self {
        Self {
            v4: None,
            v6: None
        }
    }

    pub fn v4(&self) -> Option<&T> {
        self.v4.as_ref()
    }

    pub fn v6(&self) -> Option<&T> {
        self.v6.as_ref()
    }

    pub fn value(&self, network: Network) -> Option<&T> {
        match network {
            Network::Ipv4 => self.v4(),
            Network::Ipv6 => self.v6(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.v4.is_none() && self.v6.is_none()
    }

    pub fn has_value(&self) -> bool {
        self.v4.is_some() || self.v6.is_some()
    }

    pub fn is_completed(&self) -> bool {
        self.v4.is_some() && self.v6.is_some()
    }

    pub fn set_value(&mut self, network: Network, value: T) {
        match network {
            Network::Ipv4 => self.v4 = Some(value),
            Network::Ipv6 => self.v6 = Some(value)
        }
    }
}