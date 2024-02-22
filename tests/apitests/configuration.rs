use std::net::IpAddr;
use boson::default_configuration;

#[cfg(test)]
mod apitests {
    use super::*;

    #[test]
    fn test_config() {
        let ipv4_str = "192.168.1.102";
        let port = 32222;

        let mut b = default_configuration::Builder::new();
        b.with_listening_port(port);
        b.with_ipv4(ipv4_str);
        b.with_storage_path("node");
        let cfg = b.build().unwrap();

        assert_eq!(cfg.addr4().is_some(), true);
        assert_eq!(cfg.addr6().is_none(), true);
        assert_eq!(cfg.addr4().unwrap().is_ipv4(), true);
        assert_eq!(cfg.addr4().unwrap().port(), 32222);
        assert_eq!(cfg.addr4().unwrap().ip(), IpAddr::V4(ipv4_str.parse().unwrap()));
        assert_eq!(cfg.bootstrap_nodes().len(), 0);
        assert_eq!(cfg.storage_path(), "node");

        cfg.dump();
    }
}
