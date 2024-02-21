//use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
//use boson::id::Id;
//use boson::node::{Node};
use std::rc::Rc;
use boson::node_runner::NodeRunner;
use std::env;

use boson::default_configuration;

#[cfg(test)]
mod apitests {
    use super::*;

    struct TestContext {
        node1: Rc<NodeRunner>,
        node2: Rc<NodeRunner>,
        node3: Rc<NodeRunner>
    }

    static mut CONTEXT: Option<TestContext> = None;

    fn setup() {
        match env::current_dir() {
            Ok(current_dir) => {
                println!("Current directory: {:?}", current_dir);
            }
            Err(err) => {
                eprintln!("Failed to get current directory: {}", err);
            }
        }

        unsafe {
            let mut b1 = default_configuration::Builder::new();
            b1.with_listening_port(32222);
            b1.with_ipv4("192.168.1.102");
            let cfg1 = b1.build().unwrap();

            let mut b2 = default_configuration::Builder::new();
            b2.with_listening_port(32224);
            b2.with_ipv4("192.168.1.102");
            let cfg2 = b2.build().unwrap();

            let mut b3 = default_configuration::Builder::new();
            b3.with_listening_port(32226);
            b3.with_ipv4("192.168.1.102");
            let cfg3 = b3.build().unwrap();

            CONTEXT = Some(TestContext {
                node1: Rc::new(NodeRunner::new(cfg1).unwrap()),
                node2: Rc::new(NodeRunner::new(cfg2).unwrap()),
                node3: Rc::new(NodeRunner::new(cfg3).unwrap())
            });
        }
    }

    fn teardown() {
        unsafe {
            CONTEXT = None;
        }
    }

    #[test]
    fn initialize_context() {
        setup();
    }

    #[test]
    fn cleanup_context() {
        teardown();
    }

    #[test]
    fn test_find_node() {
    }

    #[test]
    fn test_find_value() {
    }

    #[test]
    fn test_find_peer() {
    }



}
