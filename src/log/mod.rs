use chrono::Utc;
use lazy_static::lazy_static;
use path_clean::clean;
use std::{sync::Mutex, fs, io::Write};

use crate::cli::CLI_ARGS;

lazy_static! {
    pub static ref BUFF: Mutex<String> = Mutex::new(format!("Boot time: {}\n", 
        Utc::now().format("%Y-%m-%d %H:%M:%S %Z").to_string()
    ));
}

#[macro_export]
macro_rules! add_log {
    ($($arg:tt)*) => {{
        use crate::log::BUFF;
        *BUFF.lock().unwrap() = format!("{}\n{}", 
            *BUFF.lock().unwrap(),
            format!($($arg)*)
        )
    }};
}

pub fn wipe_log() {
    *BUFF.lock().unwrap() = String::new();
}

// write BUFF to a local file 
// (File path prefix specified in CLI_ARGS.log_path_prefix)
pub fn produce() {
    add_log!("End time: {}\n", 
        Utc::now().format("%Y-%m-%d %H:%M:%S %Z").to_string()
    );
    if let Some(ref path_prefix) = CLI_ARGS.log_path_prefix {
        let full_path = clean(format!("{}/log.txt", path_prefix));
        if let (Ok(()), Ok(mut file)) = 
            (fs::create_dir_all(&path_prefix), fs::File::create(&full_path)) 
        {
            println!("===Log file saved===\nLocation: {}", &full_path.to_str().unwrap());
            file.write_all(BUFF.lock().unwrap().as_bytes()).unwrap();
        } else {
            println!("\n===Log file write unsuccessful!===\nPath prefix:{:?}\n", path_prefix);
        }
    }
    // wipe_log();
    *BUFF.lock().unwrap() = format!("Boot time: {}\n", 
        Utc::now().format("%Y-%m-%d %H:%M:%S %Z").to_string()
    );
}
