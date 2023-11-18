use std::fs::{OpenOptions, self};
use std::{str, usize};
use chrono::Utc;
use std::sync::Mutex;
use path_clean::clean;
use std::path::PathBuf;
use std::time::Duration;
use regex::{Regex, RegexBuilder};
use std::io::{Write, Bytes, Read, ErrorKind};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

pub mod types;
pub mod io;

use io::{Request, ResponseBuilder};
use types::{Item, Referer};

use crate::add_log;
use crate::cli::CLI_ARGS;

pub struct Client {
    pub port: u16,
    pub domain: String,
    pub items: Vec<Item>,
    pub referers: Vec<Referer>,
}

impl Client {
    pub fn new(domain: &str) -> Self {
        return Self {
            port: 70,
            domain: String::from(domain),
            items: Vec::new(),
            referers: Vec::new(),
        }
    }

    pub fn update_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        return self
    }

    // send an initial request to ping the server
    pub fn ping(&mut self) -> Result<&mut Self, String> {
        match Request::new(&self.domain, self.port).update_path("/").send() {
            Ok(_) => { return Ok(self) }
            Err(e) => { return Err(e) }
        }
    }

    // scan directory and all sub-directories for items
    pub fn scan_all(&mut self) -> &mut Self {
        let time_start = Utc::now();
        println!("===Scanning server directories===\nServer: {}:{}\nTime: {}\n",
            self.domain, 
            self.port, 
            time_start.format("%Y-%m-%d %H:%M:%S %Z").to_string()
        );
        self.scan_re("/", !CLI_ARGS.trace_external);
        let time_end = Utc::now();
        println!("===Server directories scan complete===\nServer: {}:{}\nTime: {}\nTotal runtime: {} ms\n\nUpdating logs...\n",
            self.domain, 
            self.port, 
            time_end.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
            time_end.signed_duration_since(time_start).num_milliseconds()
        );

        let dir_items = self.items.iter()
            .filter(|item| {
                return if let Item::DATA { tag: '1' | '7', size, caption, referer, location, domain, port } = item 
                { true } else { false }
            }).cloned()
            .collect::<Vec<Item>>();

        add_log!("===All directory items===\nTotal: {}\n\n{}", 
            dir_items.len(),
            dir_items.iter().map(|item| {item.to_string()}).collect::<Vec<String>>().join("\n")
        );

        let err_items = self.items.iter()
            .filter(|item| {
                return if let Item::DATA { tag: '3', size, caption, referer, location, domain, port } = item 
                { true } else { false }
            }).cloned()
            .collect::<Vec<Item>>();

        add_log!("===All error items===\nTotal: {}\n\n{}", 
            err_items.len(),
            err_items.iter().map(|item| {item.to_string()}).collect::<Vec<String>>().join("\n")
        );

        let ext_items = self.items.iter()
            .filter(|item| {
                return if let Item::DATA { tag, size, caption, referer, location, domain: dom, port: p } = item 
                    { (dom != &self.domain) || (p != &self.port) } 
                    else 
                    { false }
            }).cloned()
            .collect::<Vec<Item>>();

        add_log!("===All external items===\nTotal: {}\n\n{}", 
            ext_items.len(),
            ext_items.iter().map(|item| {item.to_string()}).collect::<Vec<String>>().join("\n")
        );

        println!("Logs update complete.\n");
        return self
    }

    pub fn scan_re(&mut self, loc_re: &str, internal_only: bool) {
        if let Ok(resp) = Request::new(&self.domain, self.port).update_path(loc_re).send() {
            self.referers.push(resp.referer.clone());

            if let Ok(items) = resp.as_items() {
                let items_filtered = items.iter()
                    .filter(|item| { return !self.items.contains(item) }).cloned()
                    .collect::<Vec<Item>>();
                // println!("==Items filtered==\n{:?}\n", &items_filtered);
                self.items.extend_from_slice(&items_filtered);

                for item in &items_filtered {
                    // Directory item at tag = '1' | '7'
                    if let Item::DATA { tag: '1' | '7', size: _, caption: _, referer: _, location: l, port: p, domain: dom } = item {
                        if internal_only {
                            // only scan ahead if reference is internal
                            if (&self.domain == dom) && (&self.port == p) {
                                self.scan_re(l, internal_only);
                            }
                        } else {
                            self.scan_re(l, internal_only);
                        }
                    }
                }
            }
        }
    }

    // Future expansion
    pub async fn async_scan_directories(&mut self) -> Result<&mut Self, String> {
        todo!()
    }

    // return a combined info text by combining text contents from all info items at a directory level specified by referer
    pub fn get_info_at(&self, referer: Referer) -> String {
        let items_found = self.items.iter()
            .filter(|item| {
                return if let Item::INFO { tag, port, from, domain, message } = item 
                    { from == &referer } 
                    else 
                    { false }
            }).cloned()
            .map(|item| { 
                return if let Item::INFO { tag, port, from, domain, message } = item 
                    { message.clone() } 
                    else 
                    { String::new() }
            })
            .collect::<Vec<String>>();
        return items_found.join("\n")
    }

    // download all items (updated by scan_all) to folder specified by path_prefix
    pub fn download_all_to(&mut self, path_prefix: &str) -> Result<&mut Self, String> {
        let time_start = Utc::now();
        println!("===Starting file downloads===\nTime: {}\n",
            time_start.format("%Y-%m-%d %H:%M:%S %Z").to_string()
        );
        
        let mut text_items = self.items.iter()
            .filter(|item| {
                return if let Item::DATA { tag: '0', size, caption, referer, location, domain, port } = item 
                { true } else { false }
            }).cloned()
            .collect::<Vec<Item>>();

        let mut non_text_items = self.items.iter()
        .filter(|item| {
            return if let Item::DATA { tag, size, caption, referer, location, domain, port } = item 
                { (tag != &'0') && (tag != &'1') && (tag != &'7') } 
                else 
                { false }
        }).cloned()
        .collect::<Vec<Item>>();

        let mut corrupted_downloads: Vec<Item> = Vec::new();
        let corrupted_dest: String = String::from(clean(format!("{}/corrupted", path_prefix)).to_str().unwrap());
        
        for item in &mut text_items {
            if let Ok(mut resp) = Request::from_item(item).unwrap().send() {
                match resp.save_to_txt(path_prefix) {
                    Ok(file_size) => {
                        item.update_size(file_size);
                    }
                    Err(_) => {
                        match resp.save_as_txt(&corrupted_dest, &format!("{}.corrupted", corrupted_downloads.len())) {
                            Ok(file_size) => {
                                item.update_size(file_size);
                                add_log!("===Abnormal download: file name or path invalid===\n{}Relocated to: {}\n", 
                                    item, 
                                    &format!("{}.corrupted", corrupted_downloads.len())
                                );
                                corrupted_downloads.push(item.clone());
                            }
                            Err(_) => { todo!() }
                        }
                    }
                }
            }
        }

        for item in &mut non_text_items {
            match Request::from_item(item).unwrap().download(path_prefix) {
                Ok(file_size) => {
                    item.update_size(file_size);
                }
                Err((0, _)) => {
                    match Request::from_item(item).unwrap()
                        .download_as(
                            &corrupted_dest, 
                            &format!("{}.corrupted", corrupted_downloads.len())
                        ) {
                        Ok(file_size) => {
                            item.update_size(file_size);
                            corrupted_downloads.push(item.clone());
                        }
                        Err(_) => { todo!() }
                    }
                }
                Err(_) => { todo!() }
            }
        }

        let time_end = Utc::now();
        println!("===File downloads complete===\nTime: {}\nTotal runtime: {} ms\n\nUpdating logs...\n",
            time_end.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
            time_end.signed_duration_since(time_start).num_milliseconds()
        );

        add_log!("===All text items===\nTotal: {}\n\n{}", 
            text_items.len(),
            text_items.iter().map(|item| {item.to_string()}).collect::<Vec<String>>().join("\n")
        );

        add_log!("===All binary items===\nTotal: {}\n\n{}", 
            non_text_items.len(),
            non_text_items.iter().map(|item| {item.to_string()}).collect::<Vec<String>>().join("\n")
        );

        let (min_text_item, max_text_item) 
            = (text_items.iter().min().unwrap(), text_items.iter().max().unwrap());

        let mut min_text_read_buff: Vec<u8> = Vec::new();
        let mut max_text_read_buff: Vec<u8> = Vec::new();

        if let Some(i) = corrupted_downloads.iter().position(|item| {item == min_text_item}) {
            if let Ok(mut file) = fs::File::open(clean(format!("{}/{}.corrupted", corrupted_dest, i))) {
                file.read_to_end(&mut min_text_read_buff).unwrap();
            }
        } else {
            if let Item::DATA { tag, size, caption, referer, location, domain, port } = min_text_item {
                if let Ok(mut file) = fs::File::open(clean(format!("{}/{}", path_prefix, location))) {
                    file.read_to_end(&mut min_text_read_buff).unwrap();
                }
            }
        }

        if let Some(i) = corrupted_downloads.iter().position(|item| {item == max_text_item}) {
            if let Ok(mut file) = fs::File::open(clean(format!("{}/{}.corrupted", corrupted_dest, i))) {
                file.read_to_end(&mut max_text_read_buff).unwrap();
            }
        } else {
            if let Item::DATA { tag, size, caption, referer, location, domain, port } = max_text_item {
                if let Ok(mut file) = fs::File::open(clean(format!("{}/{}", path_prefix, location))) {
                    file.read_to_end(&mut max_text_read_buff).unwrap();
                }
            }
        }

        add_log!("===Smallest text file===\n{}", min_text_item);
        add_log!("===Largest text file===\n{}", max_text_item);

        if let Ok(content) = str::from_utf8(&min_text_read_buff) {
            add_log!("===Smallest text file content===\n{}\n", content);
        }
        if let Ok(content) = str::from_utf8(&max_text_read_buff) {
            add_log!("===Largest text file content===\n{}\n", content);
        }

        add_log!("===Size of smallest and largest binary file===\nSmallest: {}\nLargest: {}\n", 
            non_text_items.iter().min().unwrap().get_size().unwrap(),
            non_text_items.iter().max().unwrap().get_size().unwrap()
        );

        println!("Logs update complete.\n");

        return Ok(self)
    }
}