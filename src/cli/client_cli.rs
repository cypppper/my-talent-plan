use std::{fs::OpenOptions, io::{BufRead, BufReader, BufWriter, Read, Write}, net::{SocketAddr, TcpListener, TcpStream}, thread::sleep, time::Duration};
use crate::error::Result;


pub struct KvsClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl KvsClient {
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        let tcp_reader = TcpStream::connect(addr)?;
        let tcp_writer = tcp_reader.try_clone()?;
        Ok(Self {
            reader: BufReader::new(tcp_reader),
            writer: BufWriter::new(tcp_writer),    
        })
    }
    
    pub fn set(&mut self, key: &String, value: &String, ipaddr: SocketAddr) {
        let cmd = format!("*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n", 
            key.len(), key, 
            value.len(), value
        );
        self.writer.write_all(cmd.as_bytes()).unwrap();
        self.writer.flush().unwrap();
        
        let mut buf = String::new();
        self.reader.read_line(&mut buf).unwrap();
        if buf.as_bytes() == b"+OK\r\n" {
            // println!("SET OK");
        } else {
            error!("SET get error msg {}", buf);
            eprintln!("SET get error msg {}", buf);
            std::process::exit(-1);
        }
    }

    pub fn get(&mut self, key: &String, ipaddr: SocketAddr) {
        // let mut stream = Self::connect(ipaddr).unwrap();
        let cmd = format!("*2\r\n$3\r\nGET\r\n${}\r\n{}\r\n", 
            key.len(), key, 
        );
        self.writer.write_all(cmd.as_bytes()).unwrap();
        self.writer.flush().unwrap();

        let mut buf = String::new();
        self.reader.read_line(&mut buf).unwrap();
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
                self.reader.read_line(&mut buf).unwrap();
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

    pub fn remove(&mut self, key: &String, ipaddr: SocketAddr) {
        let cmd = format!("*2\r\n$2\r\nRM\r\n${}\r\n{}\r\n", 
            key.len(), key, 
        );
        self.writer.write_all(cmd.as_bytes()).unwrap();
        self.writer.flush().unwrap();

        let mut buf = String::new();
        self.reader.read_line(&mut buf).unwrap();
        if buf.as_bytes() == b"+OK\r\n" {
            // println!("RM OK");
        } else {
            error!("RM get error msg {}", buf);
            eprintln!("RM get error msg {}", buf);
            std::process::exit(-1);
        }
    }

}
