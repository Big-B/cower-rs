#[macro_use]
extern crate bitmask;
extern crate isatty;
extern crate log;
extern crate time;

pub mod config;
use time::Timespec;

struct AurPkg {
    name: String,
    description: String,
    maintainer: String,
    pkgbase: String,
    upstream_url: String,
    aur_urlpath: String,
    version: String,

    category_id: i64,
    package_id: i64,
    pkgbaseid: i64,
    votes: i64,
    popularity: f64,
    out_of_date: Timespec,
    submitted_s: Timespec,
    modified_s: Timespec,

    licenses: Vec<String>,
    conflicts: Vec<String>,
    depends: Vec<String>,
    groups: Vec<String>,
    makedepends: Vec<String>,
    optdepends: Vec<String>,
    checkdepends: Vec<String>,
    provides: Vec<String>,
    replaces: Vec<String>,
    keywords: Vec<String>,

    ignored: i64,
}

