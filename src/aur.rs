use url::{Url, ParseError};

pub enum RpcType {
    RpcSearch,
    RpcInfo,
}

pub enum RpcBy {
    SearchByName,
    SearchByNameDesc,
    SearchByMaintainer,
}

pub struct AurT{
    url_prefix: String,
    rpc_version: i64,
}

impl AurT {
    pub fn new(proto: &str, domain: &str) -> Self {
        let mut url = proto.to_owned();
        url.push_str("://");
        url.push_str(domain);
        AurT{url_prefix: url, rpc_version: 5}
    }

    pub fn aur_build_rpc_multi_url(&self, args: &str) -> Result<Url, ParseError> {
        Url::parse(&format!("{}/rpc.php?v={}&type=info&arg={}", self.url_prefix, self.rpc_version,
                            args)) }
}

#[test]
fn test_new_aur_t() {
    let aur = AurT::new("https", "aur.archlinux.com");
    assert_eq!(aur.rpc_version, 5);
    assert_eq!(aur.url_prefix, "https://aur.archlinux.com");
}

#[test]
fn test_rpc_multi_url() {
    let url = AurT::new("https", "aur.archlinux.com").aur_build_rpc_multi_url("cower");
    assert!(url.is_ok());

    let url = url.unwrap();
    assert_eq!(url.scheme(), "https");
    assert_eq!(url.host_str(), Some("aur.archlinux.com"));
    assert_eq!(url.path(), "/rpc.php");
    assert_eq!(url.query(), Some("v=5&type=info&arg=cower"));
}
