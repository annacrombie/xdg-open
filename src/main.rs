extern crate regex;
extern crate mime_guess as mime;
extern crate toml;
extern crate dirs;
extern crate clap;

#[macro_use] extern crate lazy_static;

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::ffi::OsStr;
use std::fs;
use std::process::{Stdio, Command};

use clap::{Arg, App, AppSettings, ArgMatches};
use mime::Mime;
use regex::Regex;
use toml::Value;

struct FileMatch {
    exp: Regex,
    mime: mime::Mime
}

lazy_static! {
    static ref RE: [FileMatch; 1] = [
        FileMatch { exp:  Regex::new(r"https?://.*").unwrap(),
                    mime: "text/html".parse().unwrap() },
    ];
}

fn mime_map_file() -> PathBuf {
    match env::var("MIME_MAP_FILE") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            let mut data_dir = dirs::data_dir().unwrap_or(PathBuf::from("."));
            data_dir.push("mime_map.toml");
            data_dir
        }
    }
}

fn get_mime(path: &str) -> Option<Mime> {
    let ext = match Path::new(path).extension() {
        Some(extension) => extension,
        None            => OsStr::new("")
    }.to_string_lossy();

    mime::get_mime_type_opt(&ext)
}

fn check_regex(path: &str) -> Option<Mime> {
    for fm in RE.into_iter() {
        if fm.exp.is_match(path) {
            return Some(fm.mime.clone());
        }
    };

    return None
}

fn read_toml() -> Value {
    let mmf = mime_map_file();
    match fs::read_to_string(&mmf) {
        Ok(string) => string.parse::<Value>().unwrap(),
        Err(_) => panic!("couldn't parse {:?}, does it exist?", mmf)
    }
}

fn spawn(cmd: &str, file: &str) {
    let split: Vec<&str> = cmd.split(" ").collect();
    let (cmd_path, other_args) = split.split_at(1);

    Command::new(cmd_path.first().unwrap())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(other_args)
            .arg(file)
            .spawn()
            .expect("couldn't spawn action");
}

fn parse_options() -> ArgMatches<'static> {
    App::new("xdg-open (not)")
        .version("0.1.0")
        .author("Stone Tickle")
        .about("lightweight clone of xdg-open, looks for a mime map file in\n$XDG_DATA_HOME/mime_map.toml or $MIME_MAP_FILE")
        .setting(AppSettings::TrailingVarArg)
        .arg(Arg::with_name("manual")
             .help("does nothing")
             .long("manual"))
        .arg(Arg::with_name("paths")
             .required(true)
             .multiple(true))
        .arg(Arg::with_name("verbose")
             .help("set verbosity")
             .short("V")
             .long("verbose"))
        .get_matches()
}

fn main() {
    let options = parse_options();
    let paths: Vec<&str> = options.values_of("paths").unwrap().collect();
    let map = read_toml();
    for path in paths {
        if options.is_present("verbose") { println!("processing {:?}:", path); }

        let mime: Mime = match check_regex(path) {
            Some(m) => m,
            None => match get_mime(path) {
                Some(mm) => mm,
                None => "application/octet-stream".parse().unwrap()
            }
        };

        if options.is_present("verbose") { println!("  mime type: {:?}", mime); }

        let action = match mime.clone() {
            Mime(top, bot, _) => {
                match map.get(top.to_string()) {
                    Some(val) => val.get(bot.to_string()),
                    None => None
                }
            }
        };

        match action {
            Some(thing) => {
                if options.is_present("verbose") {
                    println!("  taking action: {:?}", thing.as_str().unwrap());
                }
                spawn(thing.as_str().unwrap(), path);
            },
            None => {
                eprintln!("I don't know how to open '{}' ({})", path, &mime.to_string());
                continue;
            }
        }
    }
}
