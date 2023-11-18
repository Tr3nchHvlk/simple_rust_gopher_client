use regex::Regex;
use path_clean::clean;
use std::path::PathBuf;

mod cli;
mod log;
mod gopher;

use crate::log::produce; 
use crate::cli::CLI_ARGS;
use crate::gopher::Client;

fn main() {
    let host_matcher = Regex::new("^(.*?):*([0-9]*)$").unwrap();

    if let Some(captured) = host_matcher.captures(&CLI_ARGS.host) {
        let port = if let Ok(_port) = String::from(captured.get(2).unwrap().as_str()).parse::<u16>() {_port} else {70};
        let domain = captured.get(1).unwrap().as_str();

        let mut client = Client::new(domain);
        client.update_port(port).scan_all();

        if let Some(ref dl_prefix) = CLI_ARGS.download_path_prefix {
            client.download_all_to(dl_prefix).unwrap();
        }
    }

    produce()
}
