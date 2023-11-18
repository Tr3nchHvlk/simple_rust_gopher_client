use regex::Regex;
use std::{fmt::{Display, Debug}, path::PathBuf, cmp::Ordering};

// Item types
// ======
// Info
//     i => information text
// Binaries
//     0 => plain text
//     4 => BinHexed Macintosh file
//     5 => DOS binary
//     6 => UNIX uuencoded file
//     9 => generic binary
//     g => .gif image
//     I => generic image
//     s => .wav sound file
// Directories
//     1 => directory/menu
//     7 => search result
// Error
//     3 => error

#[derive(Clone, PartialEq)]
pub struct Referer {
    pub port: u16,
    pub path: String,
    pub domain: String,
}

impl Referer {
    pub fn new(domain: &str, port: u16, path: &str) -> Self {
        return Self {
            port: port,
            path: String::from(path),
            domain: String::from(domain),
        }
    }
}

impl Display for Referer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}{}", self.domain, self.port, self.path)
    }
}

#[derive(Clone)]
pub enum Item {
    INFO {
        tag: char,
        from: Referer,
        message: String,
        domain: String, 
        port: u16,
    },
    DATA {
        tag: char,
        size: u64,
        caption: String,
        referer: Referer,
        location: String,
        domain: String,
        port: u16,
    },
    UNKNOWN {
        from: Referer,
        unparsed: String,
    }
}

impl Item {
    pub fn from_str(from: &Referer, unparsed: &str) -> Result<Self, String> {
        if unparsed.len() == 0 {
            // return Err("Parse error: Must not be an empty string!\n".to_string())
            return Ok(Item::UNKNOWN {
                from: from.clone(), 
                unparsed: String::from(unparsed)
            })
        }

        let fst_char = unparsed.chars().collect::<Vec<char>>()[0];

        let matcher = Regex::new("^(.*?)\t([^\t]*)\t([^\t/]*)\t([0-9]*)$").unwrap();

        if let Some(captured) 
            = matcher.captures(&unparsed[1..unparsed.len()])
        {
            match fst_char {
                'i' | '3' => {
                    if let Ok(port) = String::from(captured.get(4).unwrap().as_str()).parse::<u16>() {
                        return Ok(Self::INFO {
                            tag: fst_char,
                            from: from.clone(),
                            message: String::from(captured.get(1).unwrap().as_str()),
                            port: port,
                            domain: String::from(captured.get(3).unwrap().as_str()),
                        })
                    } else {
                        return Err("Unknown regex error on INFO!".to_string())
                    }
                }
                '0' | '1' | '4' | '5' | '7' | '9' | 'g' | 'I' | 's' => {
                    if let Ok(port) = String::from(captured.get(4).unwrap().as_str()).parse::<u16>() {
                        return Ok(Self::DATA {
                            tag: fst_char,
                            size: 0,
                            caption: String::from(captured.get(1).unwrap().as_str()),
                            referer: from.clone(),
                            location: String::from(captured.get(2).unwrap().as_str()),
                            domain: String::from(captured.get(3).unwrap().as_str()),
                            port: port,
                        })
                    } else {
                        return Err("Unknown regex error on DATA!".to_string())
                    }   
                }
                _ => {
                    return Ok(Item::UNKNOWN {
                        from: from.clone(), 
                        unparsed: String::from(unparsed)
                    })
                }
            }
        } else {
            return Ok(Item::UNKNOWN {
                from: from.clone(), 
                unparsed: String::from(unparsed)
            })
        }
    }

    pub fn get_size(&self) -> Option<u64> {
        if let Self::DATA { tag, size, caption, referer, location, domain, port } = self {
            return Some(*size)
        } else {
            return None
        }
    }

    pub fn update_size(&mut self, new_size: u64) -> &mut Self {
        if let Self::DATA { tag, ref mut size, caption, referer, location, domain, port } = self {
            *size = new_size;
        }
        return self
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::INFO { tag: t1, from: f1, message: m1, domain: d1, port: p1 }, 
                Self::INFO { tag: t2, from: f2, message: m2, domain: d2, port: p2 }
            ) => {
                return (t1 == t2)  && (p1 == p2) && (f1 == f2) && (d1 == d2) && (m1 == m2)
            }

            (
                Self::DATA { tag: t1, size: _, caption: c1, referer: r1, location: l1, domain: d1, port: p1 },
                Self::DATA { tag: t2, size: _, caption: c2, referer: r2, location: l2, domain: d2, port: p2 },
            ) => {
                return (p1 == p2) && (d1 == d2) && (l1 == l2) 
            }
            
            (
                Self::UNKNOWN { from: f1, unparsed: u1 }, 
                Self::UNKNOWN { from: f2, unparsed: u2 },
            ) 
                => { return (f1 == f2) && (u1 == u2) }
            _ => false,
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::INFO { tag, from, message, domain, port } => {
                write!(f, 
                    "==INFO: type {}==\nFrom: {}\nMessage: {}\nDomain: {}\nPort: {}\n", 
                    tag, from, message, domain, port
                )
            }
            Self::DATA { tag, size, caption, referer, location, domain, port } => {
                write!(f, 
                    "==DATA: type {}==\nCaption: {}\nLocation: {}\nSize: {}\nDomain: {}\nPort: {}\n", 
                    tag, caption, location, size, domain, port
                )
            }
            Self::UNKNOWN { from, unparsed } => {
                write!(f, "==Unknown==\nFrom: {}\nUnparsed content:\n{}\n", from, unparsed)
            }
        }
    }
}

impl Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::INFO { tag, port, from, domain, message } => {
                write!(f, 
                    "==INFO: type {}==\nFrom: {}\nMessage: {}\nDomain: {}\nPort: {}\n", 
                    tag, from, message, domain, port
                )
            }
            Self::DATA { tag, size, caption, referer, location, domain, port } => {
                write!(f, 
                    "==DATA: type {}==\nCaption: {}\nLocation: {}\nSize: {}\nDomain: {}\nPort: {}\n", 
                    tag, caption, location, size, domain, port
                )
            }
            Self::UNKNOWN { from, unparsed } => {
                write!(f, "==Unknown==\nFrom: {}\nUnparsed content:\n{}\n", from, unparsed)
            }
        }
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if let (Some(self_size), Some(other_size)) = (self.get_size(), other.get_size()) {
            return self_size.partial_cmp(&other_size)
        } else {
            return None
        }
    }
}

impl Eq for Item {}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let (Some(self_size), Some(other_size)) = (self.get_size(), other.get_size()) {
            return self_size.cmp(&other_size)
        } else {
            match (self, other) {
                (Item::DATA { tag, size, caption, referer, location, domain, port }, _) => {
                    return Ordering::Greater;
                }
                (_, Self::DATA { tag, size, caption, referer, location, domain, port }) => {
                    return Ordering::Less;
                }
                _ => { return Ordering::Equal; }
            }
        }
    }
}

// #[derive(Clone)]
// pub struct Error {
//     id: u16,
//     message: String
// }

// impl Error {
//     pub fn new(id: u16, message: &str) -> Self {
//         return Self {
//             id: id,
//             message: String::from(message)
//         }
//     }
// }

// impl PartialEq for Error {
//     fn eq(&self, other: &Self) -> bool {
//         return self.id == other.id
//     }
// }