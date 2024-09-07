use std::{fs::OpenOptions, io::{BufRead, BufReader, BufWriter, Read, Write}, net::{SocketAddr, TcpListener, TcpStream}, thread::sleep, time::Duration};

pub struct KvsClient {}

impl KvsClient {
    pub fn new() -> Self {
        Self {}
    }

    fn write(stream: &mut TcpStream, bytes: &String) {
        stream.write_all(bytes.as_bytes()).unwrap();
    }

    fn connect(addr: SocketAddr) -> Option<TcpStream> {
        let result = TcpStream::connect(addr);
        if result.is_ok() {
            return Some(result.unwrap());
        }
        None
    }
    
    pub fn set(&self, key: &String, value: &String, ipaddr: SocketAddr) {
        let mut stream = Self::connect(ipaddr).unwrap();

        let cmd = format!("*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n", 
            key.len(), key, 
            value.len(), value
        );
        Self::write(&mut stream, &cmd);
        
        
        let mut reader = BufReader::new(&stream);
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        if buf.as_bytes() == b"+OK\r\n" {
            // println!("SET OK");
        } else {
            error!("SET get error msg {}", buf);
            eprintln!("SET get error msg {}", buf);
            std::process::exit(-1);
        }
    }

    pub fn shutdown(&self, ipaddr: SocketAddr) {
        let cmd = format!("*1\r\n$8\r\nSHUTDOWN\r\n");
        let mut stream = Self::connect(ipaddr);
        if stream.is_some() {
            Self::write(stream.as_mut().unwrap(), &cmd);
        } else {
            println!("\nstream has end!");
        }
    }

    pub fn get(&self, key: &String, ipaddr: SocketAddr) {
        let mut stream = Self::connect(ipaddr).unwrap();
        let cmd = format!("*2\r\n$3\r\nGET\r\n${}\r\n{}\r\n", 
            key.len(), key, 
        );
        Self::write(&mut stream, &cmd);

        let mut reader = BufReader::new(&stream);
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        if buf.as_str().starts_with("$") {
            // get success
            // 1. find a value
            // 2. did not find
            let num_str = &buf.as_str()[1..buf.len() - 2];
            let num = num_str.parse::<i32>().unwrap();
            if num == -1 {
                println!("[GET] Key not found: {}", key);
            } else {
                buf.clear();
                reader.read_line(&mut buf).unwrap();
                let value = &buf.as_str()[..buf.len() - 2];
                println!("{}", value);
            }
        } else {
            // error occurred
            error!("GET get error msg {}", buf);
            eprintln!("GET get error msg {}", buf);
            std::process::exit(-1);
        }
    }

    pub fn remove(&self, key: &String, ipaddr: SocketAddr) {
        let mut stream = Self::connect(ipaddr).unwrap();
        let cmd = format!("*2\r\n$2\r\nRM\r\n${}\r\n{}\r\n", 
            key.len(), key, 
        );
        Self::write(&mut stream, &cmd);

        let mut reader = BufReader::new(&stream);
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        if buf.as_bytes() == b"+OK\r\n" {
            // println!("RM OK");
        } else {
            error!("RM get error msg {}", buf);
            eprintln!("RM get error msg {}", buf);
            std::process::exit(-1);
        }
    }

}
