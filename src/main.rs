extern crate clap;
extern crate libc;
extern crate regex;
extern crate toml;

use serde::Deserialize;

use std::collections;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::process::Command;

use clap::{App, AppSettings, Arg};

fn strlen(s: *mut i8) -> usize {
    unsafe { libc::strlen(s) }
}

fn strerror(e: i32) -> String {
    let str_err = unsafe { libc::strerror(e) };
    let str_len = strlen(str_err);
    unsafe { String::from_raw_parts(str_err as *mut u8, str_len, str_len) }
}

fn setegid(gid: u32) -> Result<(), String> {
    let rc = unsafe { libc::setegid(gid as libc::gid_t) };
    if rc == 0 {
        return Ok(());
    }
    return Err(strerror(rc));
}

fn user_by_uid(uid: u32) -> Result<String, String> {
    let path = "/etc/passwd";
    let fd = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("Failed to open {} file: {}", path, e)),
    };

    let mut r = BufReader::new(fd);
    loop {
        let mut line = String::new();
        let res = r.read_line(&mut line);
        let sline = line.trim_end();
        match res {
            Ok(n) => {
                if n == 0 {
                    return Err(format!("{} username not found", uid));
                }
                let v: Vec<&str> = sline.split(':').collect();
                if v[2].parse::<u32>().unwrap() == uid {
                    return Ok(String::from(v[0]));
                }
            }
            Err(e) => return Err(format!("Failed to read line: {}", e)),
        }
    }
}

fn getid(path: &str, uog: &str) -> Result<u32, String> {
    let fd = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("Failed to open {} file: {}", path, e)),
    };

    let mut r = BufReader::new(fd);
    loop {
        let mut line = String::new();
        let res = r.read_line(&mut line);
        let sline = line.trim_end();
        match res {
            Ok(n) => {
                if n == 0 {
                    return Err(format!("{} id not found", uog));
                }
                let v: Vec<&str> = sline.split(':').collect();
                if v[0] == uog {
                    return Ok(v[2].parse::<u32>().unwrap());
                }
            }
            Err(e) => return Err(format!("Failed to read line: {}", e)),
        }
    }
}

fn getuid() -> u32 {
    unsafe { libc::getuid() }
}

#[derive(Deserialize)]
struct Config {
    policies: collections::BTreeMap<String, Vec<String>>,
}

fn load_config() -> Result<Config, String> {
    let conf_str = match fs::read_to_string(CONFIG_PATH) {
        Err(e) => return Err(format!("Failed to read config file: {}", e)),
        Ok(s) => s,
    };

    match toml::from_str::<Config>(&conf_str) {
        Err(e) => return Err(format!("Failed to parse config : {}", e)),
        Ok(c) => return Ok(c),
    };
}

const CONFIG_PATH: &str = "/etc/sudocker/sudockers.toml";
const DOCKER_GROUP: &str = "docker";
const ENV: &str = "/bin:/usr/bin";

fn main() -> Result<(), String> {
    let matches = App::new("sudocker")
        .setting(AppSettings::TrailingVarArg)
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Restrict docker command usage for unprivileged users")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("CMD")
                .help("Docker command to run")
                .multiple(true),
        )
        .get_matches();

    // gid of docker group
    let gid = getid("/etc/group", DOCKER_GROUP)?;
    // current user uid
    let uid = getuid();
    // get username from uid
    let username = user_by_uid(uid)?;
    // setting gid to docker one's
    setegid(gid)?;

    // load configuration file
    let config = load_config()?;

    // build up the command line
    let mut cmd: Vec<&str> = matches.values_of("CMD").unwrap_or_default().collect();
    // what we want to execute
    let wish = {
        cmd.insert(0, "docker");
        cmd.join(" ")
    };

    if let Some(allowed) = config.policies.get(&username) {
        for scmd in allowed {
            // we build up a safe regex
            let safe_scmd = {
                let mut s = String::new();
                if !scmd.starts_with("^") {
                    s.insert(0, '^');
                }
                s.push_str(scmd);
                if !scmd.ends_with("$") {
                    s.push('$');
                }
                s
            };

            // compile regex
            let rule = match regex::Regex::new(&safe_scmd) {
                Err(e) => return Err(format!("Failed to compile policy \"{}\": {}", scmd, e)),
                Ok(re) => re,
            };
            if rule.is_match(&wish) {
                // spawning command line
                let mut child = {
                    if cmd.len() > 1 {
                        match Command::new(&cmd[0])
                            .env("PATH", ENV)
                            .args(&cmd[1..])
                            .spawn()
                        {
                            Err(e) => {
                                return Err(format!(
                                    "Failed to execute command \"{}\" : {}",
                                    wish, e
                                ))
                            }
                            Ok(c) => c,
                        }
                    } else {
                        match Command::new(&cmd[0]).env("PATH", ENV).spawn() {
                            Err(e) => {
                                return Err(format!(
                                    "Failed to execute command \"{}\" : {}",
                                    wish, e
                                ))
                            }
                            Ok(c) => c,
                        }
                    }
                };
                child.wait().unwrap();
                return Ok(());
            }
        }
        // not allowed to executed wished command
        return Err(format!("Not allowed to execute command: {}", wish));
    }
    return Err(format!("No docker command allowed for {}", username));
}
