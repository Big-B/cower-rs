#[macro_use]
extern crate clap;
extern crate stderrlog;
extern crate ferris_says;
extern crate log;

extern crate cower_rs;

use clap::{Arg, ArgGroup, App};
use std::path::PathBuf;
use std::{env, str};
use cower_rs::config::{Config, Operation, SearchBy, SortOrder};
use std::error::Error;
use log::Level;
use std::io::BufWriter;
use std::string::FromUtf8Error;
use ferris_says::say;

fn main() {
    let mut config = Config::new();

    // Check for a config file
    if let Some(path) = get_config_path() {
        // Parse config file
        if !config.parse_config_files(&path) {
            // if bad values, die
            std::process::exit(-1);
        }
    }

    // Handle command line arguments
    if let Err(e) = handle_command_line_args(&mut config) {
        //eprintln!(format!("Error handling command line args: {}", e.description()));
        eprintln!("Error handling command line args: {}", e.description());
        std::process::exit(-1);
    }
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

fn get_version_string() -> Result<String, FromUtf8Error> {
    // Make cower cow
    let mut version = String::new();
    version.push_str("\n  ");
    version.push_str(crate_version!());
    version.push_str("\n");
    version.push_str("     \\\n");
    version.push_str("      \\\n");
    version.push_str("        ,__, |    |\n");
    version.push_str("        (oo)\\|    |___\n");
    version.push_str("        (__)\\|    |   )\\_\n");
    version.push_str("          U  |    |_w |  \\\n");
    version.push_str("             |    |  ||   *\n");
    version.push_str("\n");
    version.push_str("             Cower-rs.\n\n");

    // Get max line size
    let len = version.split('\n').map(|s| s.len()).max().unwrap_or(30);

    // Make a buf writer for ferris_says
    let mut buf = BufWriter::new(Vec::new());
    say(&version.into_bytes(), len, &mut buf).unwrap();

    // Prefix output with a newline to fix clap not using one
    let mut fin = String::new();
    fin.push_str("\n");
    fin.push_str(String::from_utf8(buf.into_inner().unwrap())?.as_str());
    Ok(fin)
}

/// Handle the command line arguments
fn handle_command_line_args(config: &mut Config) -> Result<(),std::num::ParseIntError> {
    let matches = App::new("cower-rs")
        .version(get_version_string().unwrap().as_str())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(Arg::with_name("download")
             .short("d")
             .long("download")
             .multiple(true)
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
             .possible_values(&["name", "version", "maintainer", "votes",
                              "popularity", "outofdate", "lastmodified",
                              "firstsubmitted"])
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

    // Operations
    if matches.is_present("search") {
        config.opmask.set(Operation::Search);
    }

    if matches.is_present("update") {
        config.opmask.set(Operation::Update);
    }

    if matches.is_present("info") {
        config.opmask.set(Operation::Info);
    }

    // Can be passed more than once
    if matches.is_present("download") {
        config.opmask.set(Operation::Download);
        if matches.occurrences_of("download") > 1 {
            config.getdeps = true;
        }
    }

    if matches.is_present("msearch") {
        config.opmask.set(Operation::Search);
        config.search_by = SearchBy::Maintainer;
    }

    // Options
    if let Some(color) = matches.value_of("color") {
        config.set_color(color);
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

    if matches.is_present("sort") {
        unimplemented!();
    }

    if let Some(ignore) = matches.values_of("ignore") {
        config.ignore_pkgs = ignore.map(String::from).collect();
    }

    if let Some(ignore) = matches.values_of("ignorerepo") {
        config.ignore_repos = ignore.map(String::from).collect();
    }

    if let Some(by) = matches.value_of("by") {
        config.set_search_by(by);
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

    Ok(())
}
