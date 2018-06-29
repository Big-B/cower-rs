#[macro_use]
extern crate clap;
extern crate failure;
extern crate log;
extern crate stderrlog;

extern crate cower_rs;

use clap::{App, Arg, ArgGroup};
use cower_rs::aur::*;
use cower_rs::config::*;
use cower_rs::package::*;
use cower_rs::*;
use failure::Error;
use log::Level;
use std::path::PathBuf;
use std::{env, str};

fn main() -> Result<(), Error> {
    let mut config = Config::new(package::sort_name);

    // Check for a config file
    if let Some(path) = get_config_path() {
        // Parse config file
        config.parse_config_files(&path)?;
    }

    // Handle command line arguments
    handle_command_line_args(&mut config)?;

    // Get an Aur object
    let _aur = AurT::new("https", &config.aur_domain);

    if config.frompkgbuild {}

    // Everything worked out
    Ok(())
}

/// Get the path to the config file.
/// Will first look for it in the `XDG_CONFIG_HOME` environment variable
/// and then in the caller's home directory
pub fn get_config_path() -> Option<PathBuf> {
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

/// Handle the command line arguments
fn handle_command_line_args(config: &mut Config<AurPkg>) -> Result<(), Error> {
    let matches = App::new("cower-rs")
        .version(get_version_string().unwrap().as_str())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::with_name("download")
                .short("d")
                .long("download")
                .multiple(true)
                .help("download target(s) -- pass twice to download AUR dependencies"),
        )
        .arg(
            Arg::with_name("info")
                .short("i")
                .long("info")
                .help("show info for target(s)"),
        )
        .arg(
            Arg::with_name("msearch")
                .short("m")
                .long("msearch")
                .help("show packages maintained by target(s)"),
        )
        .arg(
            Arg::with_name("search")
                .short("s")
                .long("search")
                .help("search for target(s)"),
        )
        .arg(
            Arg::with_name("update")
                .short("u")
                .long("update")
                .help("check for updates against AUR -- can be combined with the -d flag"),
        )
        .group(
            ArgGroup::with_name("Operations")
                .args(&["download", "info", "msearch", "search", "update"])
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("by")
                .long("by")
                .help("search by category")
                .takes_value(true)
                .value_name("search-by")
                .possible_values(&["name", "name-desc", "maintainer"]),
        )
        .arg(
            Arg::with_name("domain")
                .long("domain")
                .help("point cower at a different AUR (default: aur.archlinux.org)")
                .takes_value(true)
                .value_name("fqdn"),
        )
        .arg(
            Arg::with_name("force")
                .long("force")
                .short("f")
                .help("overwrite existing files when downloading"),
        )
        .arg(
            Arg::with_name("ignore")
                .long("ignore")
                .help("ignore a package upgrade (can be used more than once")
                .takes_value(true)
                .value_name("pkg")
                .multiple(true),
        )
        .arg(
            Arg::with_name("ignorerepo")
                .long("ignorerepo")
                .takes_value(true)
                .value_name("repo")
                .multiple(true)
                .help("ignore some or all binary repos"),
        )
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .help("specify an alternate download directory"),
        )
        .arg(
            Arg::with_name("threads")
                .long("threads")
                .takes_value(true)
                .help("limit number of threads created"),
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .takes_value(true)
                .help("specify connection timeout in seconds"),
        )
        .arg(
            Arg::with_name("color")
                .long("color")
                .short("c")
                .takes_value(true)
                .value_name("WHEN")
                .help("use colored output")
                .possible_values(&["never", "always", "auto"]),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .help("show debug output"),
        )
        .arg(
            Arg::with_name("format")
                .long("format")
                .takes_value(true)
                .value_name("string")
                .help("print package output according to format string"),
        )
        .arg(
            Arg::with_name("ignore-ood")
                .long("ignore-ood")
                .short("o")
                .help("the opposite of --ignore-ood"),
        )
        .arg(
            Arg::with_name("sort")
                .long("sort")
                .takes_value(true)
                .value_name("key")
                .help("sort results in ascending order by key")
                .possible_values(&[
                    "name",
                    "version",
                    "maintainer",
                    "votes",
                    "popularity",
                    "outofdate",
                    "lastmodified",
                    "firstsubmitted",
                ]),
        )
        .arg(
            Arg::with_name("rsort")
                .long("rsort")
                .takes_value(true)
                .value_name("key")
                .help("sort results in descending order by key"),
        )
        .arg(
            Arg::with_name("listdelim")
                .long("listdelim")
                .takes_value(true)
                .value_name("delim")
                .help("change list format delimeter"),
        )
        .arg(
            Arg::with_name("literal")
                .long("literal")
                .help("disable regex search, interpret target as a literal string"),
        )
        .arg(
            Arg::with_name("quiet")
                .long("quiet")
                .short("q")
                .help("output less"),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .help("output more"),
        )
        .arg(Arg::with_name("args").multiple(true))
        .get_matches();

    // Operations
    if matches.is_present("search") {
        config.opmask.insert(OpMask::SEARCH);
    }

    if matches.is_present("update") {
        config.opmask.insert(OpMask::UPDATE);
    }

    if matches.is_present("info") {
        config.opmask.insert(OpMask::INFO);
    }

    // Can be passed more than once
    if matches.is_present("download") {
        config.opmask.insert(OpMask::DOWNLOAD);
        if matches.occurrences_of("download") > 1 {
            config.getdeps = true;
        }
    }

    if matches.is_present("msearch") {
        config.opmask.insert(OpMask::SEARCH);
        config.search_by = SearchBy::Maintainer;
    }

    // Options
    if let Some(color) = matches.value_of("color") {
        config.set_color(color)?;
    }

    if matches.is_present("force") {
        config.force = true;
    }

    if matches.is_present("quiet") {
        config.quiet = true;
    }

    if let Some(path) = matches.value_of("target") {
        config.working_dir = PathBuf::from(path);
    }

    if matches.is_present("debug") {
        config.loglevel = Level::Debug;
    }

    if matches.is_present("verbose") {
        config.loglevel = Level::Trace;
    }

    if let Some(format) = matches.value_of("format") {
        config.format = String::from(format);
    }

    if matches.is_present("rsort") {
        config.sortorder = SortOrder::Reverse;
    }

    if let Some(sort) = matches.value_of("sort") {
        config.sort_func = match sort {
            "name" => sort_name,
            "version" => unimplemented!(),
            "maintainer" => sort_cmpmaint,
            "votes" => sort_cmpvotes,
            "popularity" => sort_cmppopularity,
            "outofdate" => sort_cmpood,
            "lastmodified" => sort_cmplastmod,
            "firstsubmitted" => sort_cmpfirstsub,
            _ => unreachable!(),
        }
    }

    if let Some(ignore) = matches.values_of("ignore") {
        config.ignore_pkgs = ignore.map(String::from).collect();
    }

    if let Some(ignore) = matches.values_of("ignorerepo") {
        config.ignore_repos = ignore.map(String::from).collect();
    }

    if let Some(by) = matches.value_of("by") {
        config.set_search_by(by)?;
    }

    if let Some(domain) = matches.value_of("domain") {
        config.aur_domain = String::from(domain);
    }

    if let Some(delim) = matches.value_of("listdelim") {
        config.delim = String::from(delim);
    }

    if matches.is_present("literal") {
        config.literal = true;
    }

    if let Some(threads) = matches.value_of("threads") {
        config.maxthreads = threads.parse()?
    }

    if let Some(timeout) = matches.value_of("timeout") {
        config.timeout = timeout.parse()?
    }

    if let Some(args) = matches.values_of("args") {
        config.args = args.map(String::from).collect();
    }

    Ok(())
}
