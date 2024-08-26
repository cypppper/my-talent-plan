use std::{default, io::{BufRead, BufReader, Write}, net::{SocketAddr, TcpListener, TcpStream}};

use crate::KvsEngine;

pub struct KvsServer
{
    listener: TcpListener,
    engine: Box<dyn KvsEngine>,
}

impl KvsServer {
    pub fn new(addr: SocketAddr, engine: Box<dyn KvsEngine>) -> Self {
        Self {
            listener: TcpListener::bind(addr).unwrap(),
            engine,
        }
    }

    fn parse_cmd(stream: &TcpStream) -> Vec<String> {
        let mut reader = BufReader::new(stream);
        let mut results: Vec<String> = Vec::new();
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let argc_str = &line[1..line.len().wrapping_sub(2)];
        let argc = argc_str.parse::<i32>().unwrap();
        
        for _ in 0..argc {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let len_str = &line.as_str()[1..line.len() - 2];
            let len = len_str.parse::<i32>().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert_eq!(line.pop(), Some('\n'));
            assert_eq!(line.pop(), Some('\r'));
            assert_eq!(line.len(), len as usize, "len must equal to read str.len()! ({}, {})", len, line.len());
            results.push(line);
        }
        results
    }

    pub fn start(&mut self) {
        while let Ok((mut stream, _)) = self.listener.accept() {
            self.handle_cmd(&mut stream);
        }
    }

    fn handle_cmd(&mut self, stream: &mut TcpStream) {
        let result = Self::parse_cmd(&stream);
        match result[0].as_str() {
            "SET" => {
                assert!(result.len() == 3);
                self.engine.set(result[1].clone(), result[2].clone()).unwrap();
                Self::write(stream, b"+OK\r\n");
            }
            "GET" => {
                assert!(result.len() == 2);
                let value = self.engine.get(result[1].clone()).unwrap();
                if value.is_none() {
                    Self::write(stream, b"$-1\r\n"); 
                } else {
                    let value = value.unwrap();
                    Self::write(stream, format!("${}\r\n{}\r\n", value.len(), value).as_bytes());
                }
            }
            "RM" => {
                assert!(result.len() == 2);
                let result = self.engine.remove(result[1].clone());
                if result.is_ok() {
                    Self::write(stream, b"+OK\r\n");
                } else {
                    Self::write(stream, format!("-ERR: {}", result.unwrap_err().to_string()).as_bytes());
                }
            }
            default => {
                unreachable!()
            }
        }
    }

    fn write(stream: &mut TcpStream, content: &[u8]) {
        stream.write_all(content).unwrap();
    }
}
