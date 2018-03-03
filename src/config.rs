use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use isatty::stdout_isatty;

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

    working_dir: PathBuf,
    delim: String,
    format: String,

    opmask: OpMask,
    logmask: LogMask,

    color: bool,
    sortorder: SortOrder,
    force: bool,
    getdeps: bool,
    literal: bool,
    quiet: bool,
    skiprepos: bool,
    frompkgbuild: bool,
    maxthreads: u64,
    timeout: u64,

    ignore_pkgs: Vec<String>,
    ignore_repos: Vec<String>
}

impl Config {
    pub fn new() -> Self {
        Config {
            aur_domain: String::from("aur.archlinux.org"),
            search_by: SearchBy::NameDesc,

            working_dir: PathBuf::new(),
            delim: String::from("  "),
            format: String::new(),

            opmask: OpMask::none(),
            logmask: LogLevel::Error|LogLevel::Warn|LogLevel::Info,

            color: false,
            sortorder: SortOrder::Forward,
            force: false,
            getdeps: false,
            literal: false,
            quiet: false,
            skiprepos: false,
            frompkgbuild: false,
            maxthreads: 10,
            timeout: 10,

            ignore_pkgs: Vec::new(),
            ignore_repos: Vec::new()
        }
    }

    /// Parse the config file
    pub fn parse_config_files(&mut self, path_buf: &PathBuf) -> bool {
        let mut ret = true;
        if let Ok(file) = File::open(path_buf.as_path()) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = String::from(line.unwrap().trim());

                // If the line is empty or a comment, then move on to
                // the next line
                if line.is_empty() {
                    continue;
                }
                if let Some('#') = line.chars().next() {
                    continue;
                }

                // Get args
                let line:Vec<&str> = line.split('=').collect();
                if line.len() == 2 {
                    let key = line[0].trim();
                    let val = line[1].trim();

                    // Match against possible configuration options
                    match key {
                        "IgnoreRepo" => self.ignore_repos.push(String::from(val)),
                        "IgnorePkg" => self.ignore_pkgs.push(String::from(val)),
                        "TargetDir" => {
                            // Must be an absolute path to a directory
                            self.working_dir.push(val);
                            if !self.working_dir.is_dir() {
                                eprintln!("error: TargetDir cannot be a relative path");
                                ret = false;
                            } else if !self.working_dir.is_absolute() {
                                eprintln!("error: failed to resolve option to TargetDir");
                                ret = false;
                            }
                        },
                        "MaxThreads" => match val.parse() {
                            Ok(val) => self.maxthreads = val,
                            Err(_) => {
                                eprintln!("error: invalid option to MaxThreads: {}", val);
                                ret = false;
                            }
                        }
                        "ConnectTimeout" => match val.parse() {
                            Ok(val) => self.timeout = val,
                            Err(_) => {
                                eprintln!("error: invalid option to ConnectTimeout: {}", val);
                                ret = false;
                            }
                        }
                        "Color" => {
                            // Handle auto, always, never
                            match val {
                                "auto" => self.color = stdout_isatty(),
                                "always" => self.color = true,
                                "never" => self.color = false,
                                _ => {
                                    eprintln!("error: invalid option to Color: {}", val);
                                    ret = false;
                                },
                            }
                        }
                        _ => eprintln!("ignoring unkkown option: {}", key),
                    }
                }
            }
        }
        ret
    }
}

