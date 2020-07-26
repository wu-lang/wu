use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{FetchOptions, RemoteCallbacks};

use toml::Value;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Write;

use std::path::Path;

use colored::Colorize;

pub fn new(name: Option<&str>) {
    if let Some(name) = name {
        if Path::new(name).exists() {
            wrong(&format!("path '{}' already exists", name));
        } else {
            fs::create_dir_all(format!("{}/src", name)).unwrap();

            let mut init = File::create(&format!("{}/init.wu", name)).unwrap();
            init.write_all(b"import src\n").unwrap();

            let mut wu_toml = File::create(&format!("{}/wu.toml", name)).unwrap();
            wu_toml.write_all(b"[dependencies]\n").unwrap();

            File::create(&format!("{}/src/init.wu", name)).unwrap();
        }
    } else {
        let mut wu_toml = File::create("wu.toml").unwrap();
        wu_toml.write_all(b"[dependencies]").unwrap();

        File::create("src/init.wu").unwrap();
    }
}

pub fn get() {
    if Path::new("wu.toml").exists() {
        let mut config = File::open("wu.toml").unwrap();

        let mut contents = String::new();
        config.read_to_string(&mut contents).unwrap();

        match toml::from_str::<Value>(&contents) {
            Ok(value) => match value.get("dependencies") {
                Some(depends) => match *depends {
                    Value::Table(ref t) => {
                        let mut modules = Vec::new();

                        let mut dep_path = "libs/".to_string();

                        if let Some(ref path) = value.get("libpath") {
                            if let Value::String(ref path) = path {
                                dep_path = format!("{}", path);
                            } else {
                                wrong("Expected string `libpath` value")
                            }
                        }

                        for member in t {
                            if !Path::new(&dep_path).exists() {
                                fs::create_dir_all(&dep_path).unwrap();
                            }

                            if let Value::String(ref url) = *member.1 {
                                let path = &format!("{}/{}", dep_path, member.0);

                                if Path::new(path).exists() {
                                    fs::remove_dir_all(path).unwrap()
                                }

                                println!(
                                    "{}",
                                    format!(
                                        "{} {} => `{}`",
                                        "Cloning".green().bold(),
                                        member.0,
                                        dep_path
                                    )
                                );
                                clone(&format!("https://github.com/{}", url), path);

                                modules.push(format!("import {}", member.0))
                            } else {
                                wrong("Expected string URL value")
                            }
                        }
                    }

                    _ => wrong(r#"Expected key e.g. `a = "b"`"#),
                },
                _ => (),
            },

            Err(_) => wrong("Something went wrong in 'wu.toml'"),
        }
    } else {
        wrong("Couldn't find 'wu.toml'");
    }
}

fn clone(url: &str, path: &str) {
    let cb = RemoteCallbacks::new();
    let co = CheckoutBuilder::new();
    let mut fo = FetchOptions::new();

    fo.remote_callbacks(cb);

    match RepoBuilder::new()
        .fetch_options(fo)
        .with_checkout(co)
        .clone(url, Path::new(path))
    {
        Ok(_) => (),
        Err(why) => wrong(&format!("Failed to download '{}' :: {}", url, why)),
    }

    println!()
}

fn wrong(message: &str) {
    println!("{} {}", "wrong:".red().bold(), message)
}
