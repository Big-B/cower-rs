#![feature(test)]
#[macro_use]
extern crate bitflags;
extern crate ferris_says;
extern crate isatty;
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate time;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
extern crate url;
extern crate test;

pub mod alpm;
pub mod aur;
pub mod config;
pub mod package;

use ferris_says::say;
use std::io::BufWriter;
use std::string::FromUtf8Error;

pub fn get_version_string() -> Result<String, FromUtf8Error> {
    // Make cower cow
    let mut version = String::new();
    version.push_str("\n  ");
    version.push_str(env!("CARGO_PKG_VERSION"));
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
