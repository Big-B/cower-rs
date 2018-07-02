use failure::Error;
use url::Url;

#[derive(Copy, Clone)]
pub enum RpcBy {
    SearchByName,
    SearchByNameDesc,
    SearchByMaintainer,
}

#[derive(Fail, Debug)]
pub enum AurTErrors {
    #[fail(display = "No arguments given.")]
    EmptyArgumentsVector,
}

pub struct AurT {
    url_prefix: String,
    rpc_version: i64,
}

impl AurT {
    pub fn new(proto: &str, domain: &str) -> Self {
        let mut url = proto.to_owned();
        url.push_str("://");
        url.push_str(domain);
        AurT {
            url_prefix: url,
            rpc_version: 5,
        }
    }

    pub fn aur_build_rpc_info_url(&self, args: &[&str]) -> Result<Url, Error> {
        // Make sure we were given an argument
        if args.is_empty() {
            Err(Error::from(AurTErrors::EmptyArgumentsVector))
        } else {
            // Setup url object
            let mut url = Url::parse(&self.url_prefix)?;
            url.set_path("rpc.php");

            // Append standard info
            url.query_pairs_mut()
                .append_pair("v", &format!("{}", self.rpc_version))
                .append_pair("type", "info");

            // Append arguments
            for arg in args {
                url.query_pairs_mut().append_pair("arg[]", arg);
            }
            Ok(url)
        }
    }

    pub fn aur_build_rpc_search_url(&self, rpc_by: RpcBy, arg: &str) -> Result<Url, Error> {
        // Setup url object
        let mut url = Url::parse(&self.url_prefix)?;
        url.set_path("rpc.php");

        // Get search string
        let search_by_string = match rpc_by {
            RpcBy::SearchByName => "name",
            RpcBy::SearchByNameDesc => "name-desc",
            RpcBy::SearchByMaintainer => "maintainer",
        };

        // Setup query
        url.query_pairs_mut()
            .append_pair("v", &format!("{}", self.rpc_version))
            .append_pair("type", "search")
            .append_pair("arg", arg)
            .append_pair("by", search_by_string);
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_aur_t() {
        let aur = AurT::new("https", "aur.archlinux.com");
        assert_eq!(aur.rpc_version, 5);
        assert_eq!(aur.url_prefix, "https://aur.archlinux.com");
    }

    #[test]
    fn test_rpc_info_url_with_single_arg() {
        let url = AurT::new("https", "aur.archlinux.com").aur_build_rpc_info_url(&vec!["cower"]);
        assert!(url.is_ok());

        let url = url.unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("aur.archlinux.com"));
        assert_eq!(url.path(), "/rpc.php");
        assert_eq!(url.query(), Some("v=5&type=info&arg%5B%5D=cower"));
        assert_eq!(
            url.as_str(),
            "https://aur.archlinux.com/rpc.php?v=5&type=info&arg%5B%5D=cower"
        );
    }

    #[test]
    fn test_rpc_info_url_with_multiple_arg() {
        use std::borrow::Cow;

        let vec = vec!["cower", "pacaur", "some other package"];
        let url = AurT::new("https", "aur.archlinux.com").aur_build_rpc_info_url(&vec);
        assert!(url.is_ok());

        let url = url.unwrap();
        let mut pairs = url.query_pairs();

        // We subtract 2 for v=5 and type=info
        assert_eq!(vec.len(), pairs.count() - 2);
        assert_eq!(pairs.next(), Some((Cow::Borrowed("v"), Cow::Borrowed("5"))));
        assert_eq!(
            pairs.next(),
            Some((Cow::Borrowed("type"), Cow::Borrowed("info")))
        );
        for arg in vec {
            assert_eq!(
                pairs.next(),
                Some((Cow::Borrowed("arg[]"), Cow::Borrowed(arg)))
            );
        }
    }

    #[test]
    fn test_rpc_info_url_with_no_arg() {
        let url = AurT::new("https", "aur.archlinux.com").aur_build_rpc_info_url(&Vec::new());
        assert!(url.is_err());
    }

    #[test]
    fn test_rpc_search_url() {
        let url = AurT::new("https", "aur.archlinux.com")
            .aur_build_rpc_search_url(RpcBy::SearchByName, "cower");

        assert!(url.is_ok());
    }
}
