#![feature(const_str_as_bytes)]
#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate dirs;
extern crate log;
extern crate regex;
extern crate stderrlog;
extern crate tempdir;

extern crate cower_rs;

use clap::{App, Arg, ArgGroup, ArgMatches};
use cower_rs::aur::*;
use cower_rs::config::*;
use cower_rs::package::*;
use cower_rs::*;
use failure::Error;
use log::Level;
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::{env, str};

#[derive(Debug, Fail)]
pub enum CowerError {
    #[fail(display = "Invalid Operation")]
    InvalidOperation,
    #[fail(display = "Invalid Regex: {}", regex)]
    InvalidRegexes { regex: String },
}

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

    if config.srcinfo {
        let files: Vec<PathBuf> = config.args.iter().map(|s| PathBuf::from(s)).collect();
        config.args = load_targets_from_files(files)?;
    } else if config.args.contains(&String::from("-")) {
        unimplemented!();
    }

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
    } else if let Some(path) = dirs::home_dir() {
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
        .arg(
            Arg::with_name("from-srcinfo")
                .long("from-srcinfo")
                .short("p")
                .help("use .SRCINFO files to determine targets"),
        )
        .arg(Arg::with_name("args").multiple(true))
        .get_matches();

    // Operations
    parse_operations(config, &matches);

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
            "version" => sort_cmpver,
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

    if matches.is_present("from-srcinfo") {
        config.srcinfo = true;
    }

    if let Some(args) = matches.values_of("args") {
        config.args = args.map(String::from).collect();
    }

    check_operation_combinations(&config)?;

    // Handle regexes
    if allow_regex(&config) {
        // Check for valid regexes from args
        let regexes: Vec<String> = config
            .args
            .iter()
            .filter(|arg| Regex::new(arg).is_err())
            .cloned()
            .collect();

        // If there are bad regexes, let's print them out
        // and report them
        if !regexes.is_empty() {
            return Err(Error::from(CowerError::InvalidRegexes {
                regex: regexes.join("\n").trim().to_string(),
            }));
        }
    }

    Ok(())
}

/// Parse out the operations portion of the commandline arguments
fn parse_operations(config: &mut Config<AurPkg>, args: &ArgMatches) {
    if args.is_present("search") {
        config.opmask.insert(OpMask::SEARCH);
    }

    if args.is_present("update") {
        config.opmask.insert(OpMask::UPDATE);
    }

    if args.is_present("info") {
        config.opmask.insert(OpMask::INFO);
    }

    // Can be passed more than once
    if args.is_present("download") {
        config.opmask.insert(OpMask::DOWNLOAD);
        if args.occurrences_of("download") > 1 {
            config.getdeps = true;
        }
    }

    if args.is_present("msearch") {
        config.opmask.insert(OpMask::SEARCH);
        config.search_by = SearchBy::Maintainer;
    }
}

/// Ensure that the mode argument combinations are valid
fn check_operation_combinations(config: &Config<AurPkg>) -> Result<(), Error> {
    let info = OpMask::INFO;
    let search = OpMask::SEARCH;
    let updown = OpMask::UPDATE | OpMask::DOWNLOAD;

    // Check the combinations, ensure they're valid
    if config.opmask.contains(info) && config.opmask.intersects(!info)
        || config.opmask.contains(search) && config.opmask.intersects(!search)
        || config.opmask.contains(updown) && config.opmask.intersects(!updown)
    {
        Err(Error::from(CowerError::InvalidOperation))
    } else {
        Ok(())
    }
}

/// Determine whether or not regexes as arguments are valid inputs
fn allow_regex(config: &Config<AurPkg>) -> bool {
    config.opmask.contains(OpMask::SEARCH)
        && !config.literal
        && config.search_by != SearchBy::Maintainer
}

/// Get all the dependencies from the given files and return them in a
/// deduped list.
fn load_targets_from_files(files: Vec<PathBuf>) -> Result<Vec<String>, Error> {
    let mut all_deps = Vec::new();
    for file in files {
        let mut f = File::open(file)?;
        let deps = get_dependencies_from_srcinfo(f)?;

        // Get all the strings, split them at the special characters '<' and '>' and keep
        // the prefix, discard the suffix (version number)
        all_deps.append(
            &mut deps
                .iter()
                .map(|s| (s.split_at(s.find(|c: char| c == '<' || c == '>').unwrap_or(s.len()))).0)
                .map(|s| s.to_owned())
                .collect::<Vec<String>>(),
        );
    }
    all_deps.sort_unstable();
    all_deps.dedup();
    Ok(all_deps)
}

/// Get the dependencies from a .SRCINFO file
fn get_dependencies_from_srcinfo<T>(file: T) -> Result<Vec<String>, Error>
where
    T: Read,
{
    let mut r = BufReader::new(file);
    let mut line = String::new();
    let mut deps = Vec::new();

    // Get each line and split it into its parts
    while r.read_line(&mut line)? != 0 {
        {
            let params: Vec<&str> = line.split('=').map(|s| s.trim()).collect();

            // Bounds check
            if params.len() > 1 {
                // Get the dependencies
                match params[0] {
                    "depends" | "checkdepends" | "makedepends" => deps.push(params[1].to_owned()),
                    _ => (),
                };
            }
        }
        line.clear();
    }

    Ok(deps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    const SRCINFO: &[u8] = r##"
# Generated by mksrcinfo v8
# Wed Mar 28 18:45:02 UTC 2018
pkgbase = aurutils
	pkgdesc = helper tools for the arch user repository
	pkgver = 1.5.3
	pkgrel = 10
	url = https://github.com/AladW/aurutils
	arch = any
	license = custom:ISC
	makedepends = git
	depends = pacman>=5
	depends = git
	depends = jq
	depends = pacutils>=0.4
	optdepends = devtools: systemd-nspawn support
	optdepends = vifm: build file interaction
	optdepends = aria2: threaded downloads
	optdepends = parallel: threaded downloads
	optdepends = expac: aursift script
	optdepends = repose: repo-add alternative
	source = aurutils-1.5.3.tar.gz::https://github.com/AladW/aurutils/archive/1.5.3.tar.gz
	source = aurutils-1.5.3.tar.gz.asc::https://github.com/AladW/aurutils/releases/download/1.5.3/1.5.3.tar.gz.asc
	source = 0001-aurbuild-backport-fix-for-236.patch
	source = 0002-aursync-make-L-optional-281.patch
	source = 0003-aurbuild-update-default-options.patch
	source = 0004-aurfetch-specify-git-work-tree-git-dir-274.patch
	source = 0005-specify-absolute-paths-for-GIT_DIR-GIT_WORK_TREE.patch
	source = 0006-aurfetch-aursearch-use-aria2-no-conf.patch
	source = 0007-aurchain-do-not-sort-results-when-appending.patch
	sha256sums = a09088a460e352179dbf799d915e866af47aa280474a9c943f8e6885490734c5
	sha256sums = SKIP
	sha256sums = 8bf1fe675284a8e91aa37bdbf035c0158f910446fdd10d21a705e89ff711c883
	sha256sums = 75326f1f932b545754eb05ef62ad637874367d276ee584ff9544f0c0178e39b8
	sha256sums = bb03ef84bd3e7b28af9d2a829a61869c4845bdce65c897d267e691091033fe8a
	sha256sums = 40efaedd46cb98e0af0faf8cd61dc36eaa2638cf429d280beaf5c37f09a4369b
	sha256sums = 2fc7599245c53cad4b3b404a9ecf0ef122cf6be66d18a156e83ebfd1923b5359
	sha256sums = 8f4c9ea372827db3a4d4aa8e67e4fd962384197fc1684ba50e4f739d2917402f
	sha256sums = 1cb14e6605e38a1bc127d7ea576a02dfbc2d3c0e009597980fe4040a65b347f2

pkgname = aurutils
"##.as_bytes();

    #[test]
    fn test_get_deps() {
        let deps = get_dependencies_from_srcinfo(SRCINFO);
        assert!(deps.is_ok());
        let deps = deps.unwrap();
        assert_eq!(deps.len(), 5);
        assert!(deps.contains(&"pacman>".to_owned()));
        // Will contain two instances of "git"
        assert!(deps.contains(&"git".to_owned()));
        assert!(deps.contains(&"jq".to_owned()));
        assert!(deps.contains(&"pacutils>".to_owned()));
    }

    #[test]
    fn test_load_targets_from_files() {
        let dir = TempDir::new("cower_test_dir").unwrap();
        let file_path = dir.path().join("test_load_targets_from_files.txt");
        let paths = vec![file_path.clone()];

        let mut f = File::create(file_path).unwrap();
        f.write_all(SRCINFO).unwrap();
        f.sync_all().unwrap();

        let deps = load_targets_from_files(paths);
        assert!(deps.is_ok());
        let deps = deps.unwrap();

        // Should be deduped, which should get rid of one entry, git
        assert_eq!(deps.len(), 4);

        assert!(deps.contains(&"pacman".to_owned()));
        assert!(deps.contains(&"git".to_owned()));
        assert!(deps.contains(&"jq".to_owned()));
        assert!(deps.contains(&"pacutils".to_owned()));
        assert!(false);
    }
}
