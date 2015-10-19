//! `cargo add`

#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts, trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces, unused_qualifications)]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

extern crate docopt;
extern crate rustc_serialize;
extern crate pad;
extern crate toml;

use std::error::Error;
use std::process;
use std::io::{self, Write};

extern crate cargo_edit;
use cargo_edit::Manifest;

mod list;
mod list_error;
mod tree;

use list::list_section;
use tree::parse_lock_file as list_tree;

static USAGE: &'static str = r"
Usage:
    cargo list [<section>] [options]
    cargo list (-h|--help)
    cargo list --version

Options:
    --manifest-path=<path>  Path to the manifest to list dependencies of.
    --tree                  List dependencies recursively as tree.
    -h --help               Show this help page.

Display a crate's dependencies using its Cargo.toml file.
";

#[derive(Debug, RustcDecodable)]
/// Docopts input args.
struct Args {
    arg_section: Option<String>,
    flag_manifest_path: Option<String>,
    flag_tree: bool,
    flag_version: bool,
}

impl Args {
    pub fn get_section(&self) -> &str {
        let section = self.arg_section.as_ref().map(|s| &s[..]).unwrap_or("dependencies");

        match section {
            // Handle shortcuts
            "deps" => "dependencies",
            "dev-deps" => "dev-dependencies",
            "build-deps" => "build-dependencies",
            // No shortcut
            field => field,
        }
    }
}

fn handle_list(args: &Args) -> Result<(), Box<Error>> {
    let listing = if args.flag_tree {
        let manifest = try!(Manifest::open_lock_file(&args.flag_manifest_path
                                                          .as_ref()
                                                          .map(|s| &s[..])));
        list_tree(&manifest)
    } else {
        let manifest = try!(Manifest::open(&args.flag_manifest_path.as_ref().map(|s| &s[..])));
        list_section(&manifest, args.get_section())
    };

    listing.map(|listing| println!("{}", listing)).or_else(|err| {
        println!("Could not list your stuff.\n\nERROR: {}", err);
        Err(err)
    })
}

fn main() {
    let args = docopt::Docopt::new(USAGE)
                   .and_then(|d| d.decode::<Args>())
                   .unwrap_or_else(|err| err.exit());

    if args.flag_version {
        println!("cargo-list version {}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    if let Err(err) = handle_list(&args) {
        write!(io::stderr(), "{}", err).unwrap();
        process::exit(1);
    }
}