use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use isatty::stdout_isatty;
use log::Level;

pub enum SearchBy {
    Name,
    NameDesc,
    Maintainer,
}

#[derive(Debug, Fail)]
pub enum ConfigError {
    #[fail(display = "TargetDir path not absolue: {}", path)]
    TargetDirNotAbsolute {
        path: String,
    },
    #[fail(display = "TargetDir not valid directory: {}", path)]
    TargetDirNotDir {
        path: String,
    },
    #[fail(display = "Invalid MaxThreads Argument: {}", val)]
    InvalidMaxThreadArg {
        val: String,
    },
    #[fail(display = "Invalid ConnectTimeout Argument: {}", val)]
    InvalidConnectTimeoutArg {
        val: String,
    },
    #[fail(display = "Invalid Color Argument: {}", val)]
    InvalidColorArg {
        val: String,
    },
    #[fail(display = "Invalid option for 'by': {}", val)]
    InvalidSearchByArg {
        val: String,
    },
}

bitflags! {
    #[derive(Default)]
    pub struct OpMask: u32 {
        const SEARCH   = 1;
        const INFO     = 1 << 1;
        const DOWNLOAD = 1 << 2;
        const UPDATE   = 1 << 3;
    }
}

pub enum SortOrder {
    Forward,
    Reverse
}

pub struct Config {
    pub aur_domain: String,
    pub search_by: SearchBy,

    pub working_dir: PathBuf,
    pub delim: String,
    pub format: String,

    pub opmask: OpMask,
    pub loglevel: Level,

    pub color: bool,
    pub sortorder: SortOrder,
    pub force: bool,
    pub getdeps: bool,
    pub literal: bool,
    pub quiet: bool,
    pub skiprepos: bool,
    pub frompkgbuild: bool,
    pub maxthreads: u64,
    pub timeout: u64,

    pub ignore_pkgs: Vec<String>,
    pub ignore_repos: Vec<String>
}

impl Config {
    pub fn new() -> Self {
        Config {
            aur_domain: String::from("aur.archlinux.org"),
            search_by: SearchBy::NameDesc,

            working_dir: PathBuf::new(),
            delim: String::from("  "),
            format: String::new(),

            opmask: OpMask::default(),
            loglevel: Level::Info,

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
    pub fn parse_config_files(&mut self, path_buf: &PathBuf) -> Result<(), ConfigError> {
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
                                return Err(ConfigError::TargetDirNotDir { path: val.to_string() });
                            } else if !self.working_dir.is_absolute() {
                                return Err(ConfigError::TargetDirNotAbsolute { path: val.to_string() });
                            }
                        },
                        "MaxThreads" => match val.parse() {
                            Ok(val) => self.maxthreads = val,
                            Err(_) => {
                                return Err(ConfigError::InvalidMaxThreadArg {
                                    val: val.to_string()
                                });
                            }
                        }
                        "ConnectTimeout" => match val.parse() {
                            Ok(val) => self.timeout = val,
                            Err(_) => {
                                return Err(ConfigError::InvalidConnectTimeoutArg {
                                    val: val.to_string()
                                });
                            }
                        }
                        "Color" => {
                            self.set_color(val)?;
                        }
                        _ => eprintln!("ignoring unkkown option: {}", key),
                    }
                }
            }
        }
        Ok(())
    }

    pub fn set_color(&mut self, color: &str) -> Result<(), ConfigError> {
        let color = color.trim();
        // Handle auto, always, never
        match color {
            "auto"   => self.color = stdout_isatty(),
            "always" => self.color = true,
            "never"  => self.color = false,
            _        => {
                return Err(ConfigError::InvalidColorArg { val: color.to_string() });
            },
        }
        Ok(())
    }

    pub fn set_search_by(&mut self, by: &str) -> Result<(), ConfigError> {
        let by = by.trim();

        match by {
            "maintainer" => self.search_by = SearchBy::Maintainer,
            "name-desc"  => self.search_by = SearchBy::NameDesc,
            "name"       => self.search_by = SearchBy::Name,
            _            => {
                return Err(ConfigError::InvalidSearchByArg { val: by.to_string() });
            },
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
