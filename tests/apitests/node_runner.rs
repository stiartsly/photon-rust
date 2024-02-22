use std::fs;
use boson::node_runner::NodeRunner;
use std::env;

use boson::default_configuration;

#[cfg(test)]
mod apitests {
    use super::*;

    static mut PATH1: Option<String> = None;
    static mut PATH2: Option<String> = None;
    static mut PATH3: Option<String> = None;

    static mut RUNNER1: Option<NodeRunner> = None;
    static mut RUNNER2: Option<NodeRunner> = None;
    static mut RUNNER3: Option<NodeRunner> = None;


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
            PATH1 = Some(get_storage_path("node1"));
            PATH2 = Some(get_storage_path("node2"));
            PATH3 = Some(get_storage_path("node3"));

            let mut b1 = default_configuration::Builder::new();
            b1.with_listening_port(32222);
            b1.with_ipv4("192.168.1.102");
            b1.with_storage_path(PATH1.as_ref().unwrap().as_str());
            let cfg1 = b1.build().unwrap();

            let mut b2 = default_configuration::Builder::new();
            b2.with_listening_port(32224);
            b2.with_ipv4("192.168.1.102");
            b2.with_storage_path(PATH2.as_ref().unwrap().as_str());
            let cfg2 = b2.build().unwrap();

            let mut b3 = default_configuration::Builder::new();
            b3.with_listening_port(32226);
            b3.with_ipv4("192.168.1.102");
            b2.with_storage_path(PATH3.as_ref().unwrap().as_str());
            let cfg3 = b3.build().unwrap();

            RUNNER1 = Some(NodeRunner::new(cfg1).unwrap());
            RUNNER2 = Some(NodeRunner::new(cfg2).unwrap());
            RUNNER3 = Some(NodeRunner::new(cfg3).unwrap());
        }
    }

    fn teardown() {
        unsafe {
            remove_storage(PATH1.as_ref().unwrap().as_str());
            remove_storage(PATH2.as_ref().unwrap().as_str());
            remove_storage(PATH3.as_ref().unwrap().as_str());

            RUNNER1 = None;
            RUNNER2 = None;
            RUNNER3 = None;
        }
    }

    #[test]
    fn test_find_node() {
        setup();
        unsafe {
            assert_eq!(RUNNER1.as_ref().unwrap().is_running(), false);
            assert_eq!(RUNNER2.as_ref().unwrap().is_running(), false);
            assert_eq!(RUNNER3.as_ref().unwrap().is_running(), false);
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
