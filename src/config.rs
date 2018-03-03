use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

enum SearchBy {
    Name,
    NameDesc,
    Maintainer,
}

bitmask! {
    mask OpMask: u32 where flags Operation {
        Search   = 1,
        Info     = 1 << 1,
        Download = 1 << 2,
        Update   = 1 << 3
    }
}

bitmask! {
    mask LogMask: u32 where flags LogLevel {
        Info    = 1,
        Error   = 1 << 1,
        Warn    = 1 << 2,
        Debug   = 1 << 3,
        Verbose = 1 << 4
    }
}

enum SortOrder {
    Forward,
    Reverse
}

pub struct Config {
    aur_domain: String,
    search_by: SearchBy,

    working_dir: String,
    delim: String,
    format: String,

    opmask: OpMask,
    logmask: LogMask,

    color: i16,
    ignoreood: i16,
    sortorder: SortOrder,
    force: i64,
    getdeps: i64,
    literal: i64,
    quiet: i64,
    skiprepos: i64,
    frompkgbuild: i64,
    maxthreads: i64,
    timeout: i64
}

impl Config {
    pub fn new() -> Self {
        Config {
            aur_domain: String::from("aur.archlinux.org"),
            search_by: SearchBy::NameDesc,

            working_dir: String::new(),
            delim: String::from("  "),
            format: String::new(),

            opmask: OpMask::none(),
            logmask: LogLevel::Error|LogLevel::Warn|LogLevel::Info,

            color: 0,
            ignoreood: 0,
            sortorder: SortOrder::Forward,
            force: 0,
            getdeps: 0,
            literal: 0,
            quiet: 0,
            skiprepos: 0,
            frompkgbuild: 0,
            maxthreads: 10,
            timeout: 10,
        }
    }

    /// Parse the config file
    pub fn parse_config_files(&mut self, path_buf: &PathBuf) {
        if let Ok(file) = File::open(path_buf.as_path()) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                println!("{}", line.unwrap());
            }
        }
    }
}

