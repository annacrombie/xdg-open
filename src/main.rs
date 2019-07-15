extern crate clap;
extern crate dirs;
extern crate mime_guess as mime;
extern crate regex;
extern crate toml;

#[macro_use]
extern crate lazy_static;

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::{exit, Command};

use clap::{App, AppSettings, Arg, ArgMatches};
use mime::Mime;
use regex::Regex;
use toml::Value;

struct FileMatch {
    exp: Regex,
    mime: mime::Mime,
}

lazy_static! {
    static ref RE: [FileMatch; 1] = [FileMatch {
        exp: Regex::new(r"https?://.*").unwrap(),
        mime: "text/html".parse().unwrap()
    },];
}

fn mime_map_file() -> PathBuf {
    match env::var("MIME_MAP_FILE") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            let mut data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            data_dir.push("mime_map.toml");
            data_dir
        }
    }
}

fn get_mime(path: &str) -> Option<Mime> {
    let ext = match Path::new(path).extension() {
        Some(extension) => extension,
        None => OsStr::new(""),
    }
    .to_string_lossy();

    mime::get_mime_type_opt(&ext)
}

fn check_regex(path: &str) -> Option<Mime> {
    for fm in RE.into_iter() {
        if fm.exp.is_match(path) {
            return Some(fm.mime.clone());
        }
    }

    None
}

fn read_toml() -> Value {
    let mmf = mime_map_file();
    if let Ok(string) = fs::read_to_string(&mmf) {
        string.parse::<Value>().unwrap()
    } else {
        panic!("couldn't parse {:?}, does it exist?", mmf);
    }
}

fn exec(cmd: &str, file: &str) {
    let split: Vec<&str> = cmd.split(' ').collect();
    let (cmd_path, other_args) = split.split_at(1);

    Command::new(cmd_path.first().unwrap())
        .args(other_args)
        .arg(file)
        .exec();
}

fn parse_options() -> ArgMatches<'static> {
    App::new("xdg-open (not)")
        .version("0.1.0")
        .author("Stone Tickle")
        .about(
            "lightweight clone of xdg-open, looks for a mime map file in
$XDG_DATA_HOME/mime_map.toml or $MIME_MAP_FILE",
        )
        .setting(AppSettings::TrailingVarArg)
        .arg(Arg::with_name("manual").help("does nothing").long("manual"))
        .arg(Arg::with_name("path").required(true).multiple(false))
        .arg(
            Arg::with_name("verbose")
                .help("set verbosity")
                .short("V")
                .long("verbose"),
        )
        .get_matches()
}

fn main() {
    let options = parse_options();
    let path: &str = options.value_of("path").unwrap();
    let map = read_toml();

    if options.is_present("verbose") {
        println!("processing {:?}:", path);
    }

    let mime: Mime = match check_regex(path) {
        Some(m) => m,
        None => match get_mime(path) {
            Some(mm) => mm,
            None => "application/octet-stream".parse().unwrap(),
        },
    };

    if options.is_present("verbose") {
        println!("  mime type: {:?}", mime);
    }

    let action = match mime.clone() {
        Mime(top, bot, _) => match map.get(top.to_string()) {
            Some(val) => val.get(bot.to_string()),
            None => None,
        },
    };

    match action {
        Some(thing) => {
            if options.is_present("verbose") {
                println!("  taking action: {:?}", thing.as_str().unwrap());
            }
            exec(thing.as_str().unwrap(), path);
            exit(4);
        }
        None => {
            eprintln!(
                "I don't know how to open '{}' ({})",
                path,
                &mime.to_string()
            );
            exit(3);
        }
    }
}
