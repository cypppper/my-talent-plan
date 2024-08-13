use std::{fs::OpenOptions, io::{Read, Write}, net::{SocketAddr, TcpStream}};

struct KvsClient {
    stream: TcpStream,
}

impl KvsClient {
    pub fn new(ipaddr: SocketAddr) -> Self {
        Self {stream: TcpStream::connect(ipaddr).unwrap()}
    }

    pub fn write(&mut self, bytes: &[u8]) {
        self.stream.write_all(bytes).unwrap();
    }

    // pub fn read(&mut self) -> Vec<u8> {
    //     self.stream.read(buf)
    // }
}
