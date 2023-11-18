use chrono::Utc;
use std::{str, fs};
use std::sync::Mutex;
use path_clean::{clean, PathClean};
use std::path::PathBuf;
use std::time::Duration;
use regex::{Regex, RegexBuilder};
use std::io::{Write, Bytes, Read, ErrorKind};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

use super::types::{Item, Referer};
use crate::cli::CLI_ARGS;

#[derive(Clone)]
pub struct Request {
    port: u16,
    path: String,
    domain: String,
    conn_timeout: Duration,
    resp_timeout: Duration,
}

impl Request {
    pub fn new(domain: &str, port: u16) -> Self {
        return Self { 
            port: port,
            path: "/".to_string(),
            domain: String::from(domain),
            conn_timeout: Duration::from_secs(CLI_ARGS.conn_timeout),
            resp_timeout: Duration::from_secs(CLI_ARGS.resp_timeout),
        }
    }

    pub fn from_item(item: &Item) -> Result<Self, String> {
        if let Item::DATA { tag, size, caption, referer, location, domain, port } = item {
            let mut loc = clean(location).to_str().unwrap().replace(r"\", r"/");
            if loc.is_empty() { loc = "/".to_string(); }
            return Ok(Self {
                port: *port,
                path: loc,
                domain: domain.clone(),
                conn_timeout: Duration::from_secs(CLI_ARGS.conn_timeout),
                resp_timeout: Duration::from_secs(CLI_ARGS.resp_timeout),
            })
        } else {
            return Err("Request initiation error: Invalid item!".to_string())
        }
    }
    
    pub fn update_path(&mut self, new_path: &str) -> &mut Self {
        self.path = clean(new_path).to_str().unwrap().replace(r"\", r"/");
        if self.path.is_empty() { self.path = "/".to_string(); }
        return self
    }

    // send request to server and collect response
    pub fn send(&self) -> Result<ResponseBuilder, String> {
        if let Some(sock_addr) = (String::from(&self.domain), self.port).to_socket_addrs().unwrap().next() {
            if let Ok(mut stream) = TcpStream::connect_timeout(
                &sock_addr, 
                self.conn_timeout
            ) {
                // without domain
                let req_buff = format!("{}\r\n", self.path);
                
                // with domain
                // let req_buff = format!("{}\t{}\r\n\r\n", self.path, self.domain);

                if !CLI_ARGS.disable_verbose {
                    println!("==Request sent==\nQuery: {:?}\nTarget: {}\nServer: {}:{}\nTime: {}\n", 
                        req_buff, 
                        self.path, 
                        self.domain, 
                        self.port, 
                        Utc::now().format("%Y-%m-%d %H:%M:%S %Z").to_string()
                    );
                }

                if let Ok(_) = stream.write(req_buff.as_bytes()) {
                    let mut resp_buff: Vec<u8> = Vec::new();
                    loop {
                        let mut chunk: Vec<u8> = vec![0; 4096]; // upsize to 6144 or 8192 if network condition allows
                        stream.set_read_timeout(Some(self.resp_timeout)).unwrap();

                        match stream.read(&mut chunk) {
                            Ok(0) => { break; }
                            Ok(resp_size) => {
                                resp_buff.extend_from_slice(&chunk[0..resp_size])
                            }
                            Err(error) => {
                                if error.kind() == ErrorKind::WouldBlock {
                                    println!("Error: response timed out\n");
                                }
                                return Err("Unknown read error ...".to_string());
                            }
                        }
                    }

                    if let Err(_) = stream.shutdown(std::net::Shutdown::Both) {
                        println!("==Stream shutdown failed!==");
                    }

                    return Ok(ResponseBuilder {
                        raw: resp_buff,
                        referer: Referer::new(&self.domain, self.port, &self.path)
                    })
                } else { return Err("Stream write failed ...".to_string()) }
            } else { return Err("Connection timed out ...".to_string()) }
        } else { return Err("Ip address parse error ...".to_string()) }
    }

    // use this only for download large files
    pub fn download(&self, dest_prefix: &str) -> Result<u64, (u16, String)> {
        let full_path = clean(format!("{}/{}", dest_prefix, self.path));
        if let (Some(dir_prefix), Some(file_name)) = (full_path.parent(), full_path.file_name()) {
            return self.download_as(dir_prefix.to_str().unwrap(), file_name.to_str().unwrap())
        } else {
            return Err((0, "Download path parse error: check the prefix variable or use function download_as().".to_string()))
        }
    }

    // use this only for download large files
    pub fn download_as(&self, dir_prefix: &str, file_name: &str) -> Result<u64, (u16, String)> {
        let full_path = clean(format!("{}/{}", dir_prefix, file_name));

        if let (Ok(()), Ok(mut file)) = 
            (fs::create_dir_all(&dir_prefix), fs::File::create(&full_path)) 
        {
            if let Some(sock_addr) = (String::from(&self.domain), self.port).to_socket_addrs().unwrap().next() {
                if let Ok(mut stream) = TcpStream::connect_timeout(
                    &sock_addr, 
                    self.conn_timeout
                ) {
                    // without domain
                    let req_buff = format!("{}\r\n", self.path);
                    
                    // with domain
                    // let req_buff = format!("{}\t{}\r\n\r\n", self.path, self.domain);
    
                    if !CLI_ARGS.disable_verbose {
                        println!("==Request sent==\nQuery: {:?}\nTarget: {}\nServer: {}:{}\nTime: {}\n", 
                            req_buff, 
                            self.path, 
                            self.domain, 
                            self.port, 
                            Utc::now().format("%Y-%m-%d %H:%M:%S %Z").to_string()
                        );
                    }
    
                    if let Ok(_) = stream.write(req_buff.as_bytes()) {
                        let mut file_size: u64 = 0;
                        loop {
                            let mut chunk: Vec<u8> = vec![0; 2048];
                            stream.set_read_timeout(Some(self.resp_timeout)).unwrap();
    
                            match stream.read(&mut chunk) {
                                Ok(0) => { break; }
                                Ok(resp_size) => {
                                    file.write_all(&chunk[0..resp_size]).unwrap();
                                    file_size += resp_size as u64;
                                }
                                Err(error) => {
                                    if error.kind() == ErrorKind::WouldBlock {
                                        println!("Error: response timed out\n");
                                    }
                                    return Err((4, "Unknown read error ...".to_string()));
                                }
                            }
                        }
    
                        if let Err(_) = stream.shutdown(std::net::Shutdown::Both) {
                            println!("==Stream shutdown failed!==");
                        }

                        if !CLI_ARGS.disable_verbose {
                            println!("\n==File downloaded==\nFolder:{}\nName:{}\n", &dir_prefix, &file_name);
                        }

                        return Ok(file_size)
                    } else { return Err((3, "Stream write failed!".to_string())) }
                } else { return Err((2, "Connection timed out!".to_string())) }
            } else { return Err((1, "Ip address parse error!".to_string())) }
        } else {
            return Err((0, "File path invalid!".to_string()))
        }
    }
}

pub struct ResponseBuilder {
    pub raw: Vec::<u8>,
    pub referer: Referer,
}

impl ResponseBuilder {
    pub fn as_items(&self) -> Result<Vec<Item>, String> {
        if let Ok(content) = str::from_utf8(&self.raw) {
            // check if content ends with /r/n
            let matcher = RegexBuilder::new("^(.*)\r??\n??.\r\n$").dot_matches_new_line(true).build().unwrap();

            if let Some(captured) = matcher.captures(content) {
                let items: Vec<Item> = captured.get(1).unwrap()
                    .as_str().split("\r\n")
                    .map(|line| {Item::from_str(&self.referer, line).unwrap()})
                    .collect();
                
                return Ok(items)
            }

            return Err("Parse error: Response message incomplete or mal-formatted!".to_string())
        } else {
            return Err("Message does not support utf8 encoding or is corrupted".to_string())
        }
    }

    pub fn save_to_file(&mut self, dest_prefix: &str) -> Result<u64, String> {
        let full_path = clean(format!("{}/{}", dest_prefix, self.referer.path));
        if let Some(dir_prefix) = full_path.parent() {
            if let (Ok(()), Ok(mut file)) = (
                fs::create_dir_all(&dir_prefix), fs::File::create(&full_path)
            ) {
                file.write_all(&self.raw).unwrap();
                return Ok(self.raw.len() as u64)
            }
        }
        return Err("File path invalid!".to_string())
    }

    pub fn save_as_file(&mut self, dir_prefix: &str, file_name: &str) -> Result<u64, String> {
        let full_path = clean(format!("{}/{}", dir_prefix, file_name));

        if let (Ok(()), Ok(mut file)) = (
            fs::create_dir_all(&dir_prefix), fs::File::create(&full_path)
        ) {
            file.write_all(&self.raw).unwrap();
            return Ok(self.raw.len() as u64)
        } else {
            return Err("File path invalid!".to_string())
        }
    }

    pub fn save_to_txt(&mut self, dest_prefix: &str) -> Result<u64, String> {
        if let Ok(content) = str::from_utf8(&self.raw) {
            // check if content ends with ./r/n
            let matcher = RegexBuilder::new("^(.*).\r\n$").dot_matches_new_line(true).build().unwrap();

            if let Some(captured) = matcher.captures(content) {
                let full_path = clean(format!("{}/{}", dest_prefix, self.referer.path));
                if let Some(dir_prefix) = full_path.parent() {
                    if let (Ok(()), Ok(mut file)) = (
                        fs::create_dir_all(&dir_prefix), 
                        fs::File::create(&full_path)
                    ) {
                        let write_buff = captured.get(1).unwrap().as_str().as_bytes();
                        file.write_all(write_buff).unwrap();

                        if !CLI_ARGS.disable_verbose {
                            println!("\n==File saved==\nFolder:{}\nName:{}\n", 
                                &dir_prefix.to_str().unwrap(), 
                                &full_path.file_name().unwrap().to_str().unwrap()
                            );
                        }
                        return Ok(write_buff.len() as u64)
                    }
                }
                return Err("File path invalid!".to_string())
            }
        }
        return self.save_to_file(dest_prefix)
    }

    pub fn save_as_txt(&mut self, dir_prefix: &str, file_name: &str) -> Result<u64, String> {
        if let Ok(content) = str::from_utf8(&self.raw) {
            // check if content ends with ./r/n
            let matcher = RegexBuilder::new("^(.*).\r\n$").dot_matches_new_line(true).build().unwrap();
            if let Some(captured) = matcher.captures(content) {
                let full_path = clean(format!("{}/{}", dir_prefix, file_name));

                if let (Ok(()), Ok(mut file)) = (
                    fs::create_dir_all(&dir_prefix), fs::File::create(&full_path)
                ) {
                    let write_buff = captured.get(1).unwrap().as_str().as_bytes();
                    file.write_all(write_buff).unwrap();

                    if !CLI_ARGS.disable_verbose {
                        println!("\n==File saved==\nFolder:{}\nName:{}\n", dir_prefix, file_name);
                    }
                    return Ok(write_buff.len() as u64)
                } else {
                    return Err("File path invalid!".to_string())
                }
            }
        }
        return self.save_as_file(dir_prefix, file_name)
    }

    pub fn as_utf8_str(&self) -> Result<String, String> {
        if let Ok(content) = str::from_utf8(&self.raw) {
            return Ok(String::from(content))
        } else {
            return Err("Message does not support utf8 encoding or is corrupted!".to_string())
        }
    }
}