use std::fs;
use std::rc::Rc;
use boson::node_runner::NodeRunner;
use std::env;

use boson::default_configuration;

#[cfg(test)]
mod apitests {
    use super::*;

    struct TestContext {
        path1: String,
        path2: String,
        path3: String,

        node1: Rc<NodeRunner>,
        node2: Rc<NodeRunner>,
        node3: Rc<NodeRunner>
    }

    static mut CONTEXT: Option<TestContext> = None;

    fn get_storage_path(input: &str) -> String {
        let path = env::current_dir().unwrap().join(input);

        if !fs::metadata(&path).is_ok() {
            match fs::create_dir(&path) {
                Ok(_) => {},
                Err(e) => {
                    panic!("Failed to create directory: {}", e);
                }
            }
        }
        path.display().to_string()
    }

    fn remove_storage(input: &str) {
        if fs::metadata(&input).is_ok() {
            match fs::remove_dir_all(&input) {
                Ok(_) => {},
                Err(e) => {
                    panic!("Failed to remove directory: {}", e);
                }
            }
        }
    }

    fn setup() {
        unsafe {
            let path1 = get_storage_path("node1");
            let path2 = get_storage_path("node2");
            let path3 = get_storage_path("node3");

            let mut b1 = default_configuration::Builder::new();
            b1.with_listening_port(32222);
            b1.with_ipv4("192.168.1.102");
            b1.with_storage_path(path1.as_str());
            let cfg1 = b1.build().unwrap();

            let mut b2 = default_configuration::Builder::new();
            b2.with_listening_port(32224);
            b2.with_ipv4("192.168.1.102");
            b2.with_storage_path(path2.as_str());
            let cfg2 = b2.build().unwrap();

            let mut b3 = default_configuration::Builder::new();
            b3.with_listening_port(32226);
            b3.with_ipv4("192.168.1.102");
            b2.with_storage_path(path3.as_str());
            let cfg3 = b3.build().unwrap();

            CONTEXT = Some(TestContext {
                path1,
                path2,
                path3,
                node1: Rc::new(NodeRunner::new(cfg1).unwrap()),
                node2: Rc::new(NodeRunner::new(cfg2).unwrap()),
                node3: Rc::new(NodeRunner::new(cfg3).unwrap())
            });

            println!("Context is some(): {}", CONTEXT.is_some());
        }
    }

    fn teardown() {
        unsafe {
            let ctx = CONTEXT.as_ref().unwrap();
            remove_storage(ctx.path1.as_str());
            remove_storage(ctx.path2.as_str());
            remove_storage(ctx.path3.as_str());

            CONTEXT = None;
        }
    }

    #[test]
    fn test_find_node() {
        setup();
        unsafe {
            CONTEXT.as_ref().unwrap().node1.is_running();
            CONTEXT.as_ref().unwrap().node2.is_running();
            CONTEXT.as_ref().unwrap().node3.is_running();
        }
        teardown()
    }

    #[test]
    fn test_find_value() {
    }

    #[test]
    fn test_find_peer() {
    }



}
