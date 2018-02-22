#[macro_use]
extern crate clap;

extern crate time;

use clap::{Arg, ArgGroup, App};
use std::path::{Path,PathBuf};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use time::Timespec;

enum SearchBy {
    Name,
    NameDesc,
    Maintainer,
}

enum Operation {
    Search,
    Info,
    Download,
    Update
}

enum LogLevel {
    Info,
    Error,
    Warn,
    Debug,
    Verbose
}

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

struct Config {
    aur_domain: String,
    search_by: SearchBy,

    working_dir: String,
    delim: String,
    format: String,

    opmask: Operation,
    logmask: LogLevel,

    color: i16,
    ignoreood: i16,
    sortorder: i16,
    force: i64,
    getdeps: i64,
    literal: i64,
    quiet: i64,
    skiprepos: i64,
    frompkgbuild: i64,
    maxthreads: i64,
    timeout: i64
}

fn main() {
    parse_config_files();
    handle_command_line_args();
}

/// Get the path to the config file.
/// Will first look for it in the `XDG_CONFIG_HOME` environment variable
/// and then in the caller's home directory
fn get_config_path()->Option<PathBuf> {
    let mut path_buf = PathBuf::new();
    if let Ok(path) = env::var("XDG_CONFIG_HOME") {
        path_buf.push(path);
        path_buf.push("cower/config");
    } else if let Some(path) = env::home_dir() {
        path_buf.push(path);
        path_buf.push(".config/cower/config");
    }

    if path_buf.is_file() {
        Some(path_buf)
    } else {
        None
    }
}

/// Parse the config file
fn parse_config_files() {
    if let Some(path_buf) = get_config_path() {
        if let Ok(file) = File::open(path_buf.as_path()) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                println!("{}", line.unwrap());
            }
        }
    }
}

/// Handle the command line arguments
fn handle_command_line_args() {
    let _matches = App::new("cower")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(Arg::with_name("download")
             .short("d")
             .long("download")
             .help("download target(s) -- pass twice to download AUR dependencies")
            )
        .arg(Arg::with_name("info")
             .short("i")
             .long("info")
             .help("show info for target(s)")
            )
        .arg(Arg::with_name("msearch")
             .short("m")
             .long("msearch")
             .help("show packages maintained by target(s)")
            )
        .arg(Arg::with_name("search")
             .short("s")
             .long("search")
             .help("search for target(s)")
            )
        .arg(Arg::with_name("update")
             .short("u")
             .long("update")
             .help("check for updates against AUR -- can be combined with the -d flag")
            )
        .group(ArgGroup::with_name("Operations")
               .args(&["download", "info", "msearch", "search", "update"])
               .required(true)
               .multiple(true)
              )
        .arg(Arg::with_name("by")
             .long("by")
             .help("search by category")
             .takes_value(true)
             .value_name("search-by")
             .possible_values(&["name", "name-desc", "maintainer"])
            )
        .arg(Arg::with_name("domain")
             .long("domain")
             .help("point cower at a different AUR (default: aur.archlinux.org)")
             .takes_value(true)
             .value_name("fqdn")
            )
        .arg(Arg::with_name("force")
             .long("force")
             .short("f")
             .help("overwrite existing files when downloading")
            )
        .arg(Arg::with_name("ignore")
             .long("ignore")
             .help("ignore a package upgrade (can be used more than once")
             .takes_value(true)
             .value_name("pkg")
             .multiple(true)
            )
        .arg(Arg::with_name("ignorerepo")
             .long("ignorerepo")
             .takes_value(true)
             .value_name("repo")
             .multiple(true)
             .help("ignore some or all binary repos")
            )
        .arg(Arg::with_name("target")
             .short("t")
             .long("target")
             .takes_value(true)
             .help("specify an alternate download directory")
            )
        .arg(Arg::with_name("threads")
             .long("threads")
             .takes_value(true)
             .help("limit number of threads created")
            )
        .arg(Arg::with_name("timeout")
             .long("timeout")
             .takes_value(true)
             .help("specify connection timeout in seconds")
            )
        .arg(Arg::with_name("color")
             .long("color")
             .short("c")
             .takes_value(true)
             .value_name("WHEN")
             .help("use colored output")
             .possible_values(&["never", "always", "auto"])
            )
        .arg(Arg::with_name("debug")
             .long("debug")
             .help("show debug output")
            )
        .arg(Arg::with_name("format")
             .long("format")
             .takes_value(true)
             .value_name("string")
             .help("print package output according to format string")
            )
        .arg(Arg::with_name("ignore-ood")
             .long("ignore-ood")
             .short("o")
             .help("the opposite of --ignore-ood")
            )
        .arg(Arg::with_name("sort")
             .long("sort")
             .takes_value(true)
             .value_name("key")
             .help("sort results in ascending order by key")
            )
        .arg(Arg::with_name("rsort")
             .long("rsort")
             .takes_value(true)
             .value_name("key")
             .help("sort results in descending order by key")
            )
        .arg(Arg::with_name("listdelim")
             .long("listdelim")
             .takes_value(true)
             .value_name("delim")
             .help("change list format delimeter")
            )
        .arg(Arg::with_name("literal")
             .long("literal")
             .help("disable regex search, interpret target as a literal string")
            )
        .arg(Arg::with_name("quiet")
             .long("quiet")
             .short("q")
             .help("output less")
            )
        .arg(Arg::with_name("verbose")
             .long("verbose")
             .short("v")
             .help("output more")
            )
        .get_matches();
}
