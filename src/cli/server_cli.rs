use std::{
        io::{BufRead, BufReader, Write}, 
        net::{SocketAddr, TcpListener, TcpStream}
    };

use crate::{thread_pool::ThreadPool, KvsEngine};

pub struct KvsServer<E: KvsEngine, P: ThreadPool>
{
    addr: SocketAddr,
    engine: E,
    thread_pool: P,
}

// statically dispatch
impl<E: KvsEngine, P: ThreadPool> KvsServer<E, P> {
    pub fn new(addr: SocketAddr, engine: E, pool: P) -> Self {
        Self {
            addr,
            engine,
            thread_pool: pool,
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

    // server start to run
    pub fn start(&self) {
        let listener = TcpListener::bind(self.addr).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let engine = self.engine.clone();
            self.thread_pool.spawn(|| Self::handle_cmd(engine, stream, vec![]));
        }
    }

    fn handle_cmd(engine: E, mut stream: TcpStream, result: Vec<String>) {
        let result = Self::parse_cmd(&stream);
        // response
        match result[0].as_str() {
            "SET" => {
                assert!(result.len() == 3);
                engine.set(result[1].clone(), result[2].clone()).unwrap();
                Self::write(&mut stream, b"+OK\r\n");
            }
            "GET" => {
                assert!(result.len() == 2);
                let value = engine.get(result[1].clone()).unwrap();
                if value.is_none() {
                    Self::write(&mut stream, b"$-1\r\n"); 
                } else {
                    let value = value.unwrap();
                    Self::write(&mut stream, format!("${}\r\n{}\r\n", value.len(), value).as_bytes());
                }
            }
            "RM" => {
                assert!(result.len() == 2);
                let result = engine.remove(result[1].clone());
                if result.is_ok() {
                    Self::write(&mut stream, b"+OK\r\n");
                } else {
                    Self::write(&mut stream, format!("-ERR: {}", result.unwrap_err().to_string()).as_bytes());
                }
            }
            "SHUTDOWN" => {}
            default => {
                unreachable!()
            }
        }
    }

    fn write(stream: &mut TcpStream, content: &[u8]) {
        stream.write_all(content).unwrap();
    }
}
