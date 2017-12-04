extern crate reqwest;
extern crate clap;
extern crate itertools;
extern crate serde;
#[macro_use]
extern crate serde_json;

use std::fs::File;
use std::io::prelude::*;
use clap::{Arg, App};
use std::env;
use reqwest::header::Authorization;

const GITHUB_API_URL: &'static str = "https://api.github.com";

fn read_file(filepath: &str) -> String {
    let mut file = File::open(filepath)
        .expect("Could not open file");

    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Could not read content");
    
    return content;
}

fn get_filenames(gist: &serde_json::Value) -> String {
    let mut filenames: String = "".to_string();
    for file in gist["files"].as_object().unwrap() {
        if filenames == "" {
            filenames = file.0.to_string();
        } else {
            filenames = format!("{}, {}", filenames, file.0);
        }
    }
    return filenames;
}

fn request(url: &str, data: Option<serde_json::Value>, token: String) -> reqwest::Response {
    let url = format!("{}/{}", GITHUB_API_URL, url);
    let client = reqwest::Client::new();
    
    if token != "" {
        if let Some(d) = data {
            client.post(&url)
                .json(&d)
                .header(Authorization(token.to_owned()))
                .send()
                .expect("Failed to send auth POST")
        } else {
            client.get(&url)
                .header(Authorization(token.to_owned()))
                .send()
                .expect("Failed to send auth GET")
        }
    } else {
        if let Some(d) = data {
            client.post(&url)
                .json(&d)
                .send()
                .expect("Failed to send POST")
        } else {
            client.get(&url)
                .send()
                .expect("Failed to send GET")
        }
    }
}

fn list_all_gists(token: String) {
    let mut res = request("gists", None, token);
    let json: serde_json::Value = res.json()
        .expect("Coult not convert to json");  
    let gists = json.as_array().unwrap();

    for gist in gists {
        println!("ID: {} | Created: {}\nFiles: {}", 
                 gist["id"],
                 gist["created_at"],
                 get_filenames(gist));
    }
}

fn create_gist(filepath: &str, public: bool, token: String) {
    let description = "test";

    let content = read_file(filepath);

    let gist = json!({
        "description": description,
        "public": public.to_string(),
        "files": {
            filepath: {
                "content": content.to_string()
            }
        }
    });
    let response = request("gists", Some(gist), token);
    println!("{:#?}", response);
}



fn main() {
  let matches = App::new("rgist")
                    .version("0.42.0")
                    .about("Github gist client")
                    .arg(Arg::with_name("input")
                         .help("Filename")
                         .index(1))
                    .arg(Arg::with_name("--public")
                        .short("-p")
                        .long("--public")
                        .help("Set flag to create gist public")
                        .takes_value(false))
                    .arg(Arg::with_name("--list")
                        .short("-l")
                        .long("--list")
                        .help("List all gists")
                        .takes_value(false))
                    .get_matches();
    
    
    let token: String = match env::var("RGIST_TOKEN") {
        Ok(token) => format!("token {}", token.to_string()),
        Err(_) => "".to_string(),
    };

    if let Some(input) = matches.value_of("input") {
        let mut public = false;
        if matches.is_present("--public") {
            public = true;
        }
        create_gist(input, public, token);
    } else if matches.is_present("--list") {
        list_all_gists(token);
    }
}
