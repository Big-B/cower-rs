use failure::Error;
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Query {
    version: u64,
    #[serde(rename = "type")]
    query_type: String,
    resultcount: u64,
    results: Vec<AurPkg>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AurPkg {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Maintainer")]
    maintainer: String,
    #[serde(rename = "PackageBase")]
    pkgbase: String,
    #[serde(rename = "URL")]
    upstream_url: String,
    #[serde(rename = "URLPath")]
    aur_urlpath: String,
    #[serde(rename = "Version")]
    version: String,

    #[serde(rename = "CategoryID", default)]
    category_id: u64,
    #[serde(rename = "ID")]
    package_id: i64,
    #[serde(rename = "PackageBaseID")]
    pkgbaseid: i64,
    #[serde(rename = "NumVotes")]
    votes: i64,
    #[serde(rename = "Popularity")]
    popularity: f64,
    #[serde(rename = "OutOfDate")]
    out_of_date: Option<u64>,
    #[serde(rename = "FirstSubmitted")]
    submitted_s: u64,
    #[serde(rename = "LastModified")]
    modified_s: u64,

    #[serde(rename = "License", default)]
    licenses: Vec<String>,
    #[serde(rename = "Conflicts", default)]
    conflicts: Vec<String>,
    #[serde(rename = "Depends", default)]
    depends: Vec<String>,
    #[serde(rename = "Groups", default)]
    groups: Vec<String>,
    #[serde(rename = "MakeDepends", default)]
    makedepends: Vec<String>,
    #[serde(rename = "OptDepends", default)]
    optdepends: Vec<String>,
    #[serde(rename = "CheckDepends", default)]
    checkdepends: Vec<String>,
    #[serde(rename = "Provides", default)]
    provides: Vec<String>,
    #[serde(rename = "Replaces", default)]
    replaces: Vec<String>,
    #[serde(rename = "Keywords", default)]
    keywords: Vec<String>,
}

pub fn aur_packages_from_json(json: &str) -> Result<Vec<AurPkg>, Error> {
    let p: Query = serde_json::from_str(json)?;
    Ok(p.results)
}

#[test]
fn test_parsing_json() {
    let data = r#"{
        "version":5,
        "type":"multiinfo",
        "resultcount":1,
        "results":[{
            "ID":229417,
            "Name":"cower",
            "PackageBaseID":44921,
            "PackageBase":"cower",
            "Version":"14-2",
            "Description":"A simple AUR agent with a pretentious name",
            "URL":"http:\/\/github.com\/falconindy\/cower",
            "NumVotes":590,
            "Popularity":24.595536,
            "OutOfDate":null,
            "Maintainer":"falconindy",
            "FirstSubmitted":1293676237,
            "LastModified":1441804093,
            "URLPath":"\/cgit\/aur.git\/snapshot\/cower.tar.gz",
            "Depends":[
                "curl",
                "openssl",
                "pacman",
                "yajl"
            ],
            "MakeDepends":[
                "perl"
            ],
            "License":[
                "MIT"
            ],
            "Keywords":[]
        }]
    }"#;

    let input = aur_packages_from_json(data);
    assert!(input.is_ok());

    let input = input.unwrap();
    assert_eq!(input.len(), 1);

    let input = &input[0];
    assert_eq!(input.package_id, 229417);
    assert_eq!(input.name, "cower");
    assert_eq!(input.pkgbaseid, 44921);
    assert_eq!(input.version, "14-2");
    assert_eq!(input.description, "A simple AUR agent with a pretentious name");
    assert_eq!(input.upstream_url, "http://github.com/falconindy/cower");
    assert_eq!(input.votes, 590);
    assert_eq!(input.popularity, 24.595536);
    assert!(input.out_of_date.is_none());
    assert_eq!(input.maintainer, "falconindy");
    assert_eq!(input.submitted_s, 1293676237);
    assert_eq!(input.modified_s, 1441804093);
    assert_eq!(input.aur_urlpath, "/cgit/aur.git/snapshot/cower.tar.gz");

    assert_eq!(input.depends.len(), 4);
    assert_eq!(input.depends[0], "curl");
    assert_eq!(input.depends[1], "openssl");
    assert_eq!(input.depends[2], "pacman");
    assert_eq!(input.depends[3], "yajl");

    assert_eq!(input.makedepends.len(), 1);
    assert_eq!(input.makedepends[0], "perl");

    assert_eq!(input.licenses.len(), 1);
    assert_eq!(input.licenses[0], "MIT");

    assert_eq!(input.keywords.len(), 0);
}

#[test]
fn test_parsing_search() {
    let data = r#"{
        "version":5,
        "type":"search",
        "resultcount":4,
        "results":[{
            "ID":266495,
            "Name":"burgaur",
            "PackageBaseID":91085,
            "PackageBase":"burgaur",
            "Version":"2.2-2",
            "Description":"A delicious AUR helper. Made from cower.",
            "URL":"https:\/\/github.com\/m45t3r\/burgaur",
            "NumVotes":7,
            "Popularity":0.000813,
            "OutOfDate":null,
            "Maintainer":"m45t3r",
            "FirstSubmitted":1425574472,
            "LastModified":1453133491,
            "URLPath":"\/cgit\/aur.git\/snapshot\/burgaur.tar.gz"
        },
        {
            "ID":266497,
            "Name":"burgaur-git",
            "PackageBaseID":91086,
            "PackageBase":"burgaur-git",
            "Version":"2.2-2",
            "Description":"A delicious AUR helper. Made from cower.",
            "URL":"https:\/\/github.com\/m45t3r\/burgaur",
            "NumVotes":1,
            "Popularity":0.004006,
            "OutOfDate":null,
            "Maintainer":"m45t3r",
            "FirstSubmitted":1425574489,
            "LastModified":1453133995,
            "URLPath":"\/cgit\/aur.git\/snapshot\/burgaur-git.tar.gz"
        },
        {
            "ID":404277,
            "Name":"cower-git",
            "PackageBaseID":35888,
            "PackageBase":"cower-git",
            "Version":"17-1",
            "Description":"A simple AUR agent with a pretentious name",
            "URL":"http:\/\/github.com\/falconindy\/cower",
            "NumVotes":81,
            "Popularity":0.385032,
            "OutOfDate":null,
            "Maintainer":"falconindy",
            "FirstSubmitted":1269401179,
            "LastModified":1493040653,
            "URLPath":"\/cgit\/aur.git\/snapshot\/cower-git.tar.gz"
        },
        {
            "ID":404289,
            "Name":"cower",
            "PackageBaseID":44921,
            "PackageBase":"cower",
            "Version":"17-2",
            "Description":"A simple AUR agent with a pretentious name",
            "URL":"http:\/\/github.com\/falconindy\/cower",
            "NumVotes":997,
            "Popularity":13.169459,
            "OutOfDate":null,
            "Maintainer":"falconindy",
            "FirstSubmitted":1293676237,
            "LastModified":1493044041,
            "URLPath":"\/cgit\/aur.git\/snapshot\/cower.tar.gz"
        }]
    }"#;

    let input = aur_packages_from_json(data);
    assert!(input.is_ok());

    let input = input.unwrap();
    assert_eq!(input.len(), 4);
}
