use std::{io::{BufRead, BufReader, BufWriter, Write}, net::{SocketAddr, TcpListener, TcpStream}, thread};

#[test]
fn test_cmd() {
    let key = format!("a");
    let value = format!{"b"};
    let cmd = String::from(
        format!("*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n", 
        key.len(), key, 
        value.len(), value
    ));
    println!("{cmd}");
}

fn gen_cmd() -> String {
    let key = format!("a");
    let value = format!{"b"};
    let cmd = String::from(
        format!("*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n", 
        key.len(), key, 
        value.len(), value
    ));
    cmd
}

fn main(){
    let addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();
    let handle1 = thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(b"test\r\ntest\r\n").unwrap();
        let mut reader2 = BufReader::new(stream);
        let mut buf = String::new( );
        reader2.read_line(&mut buf);
        println!("got line!!!!!!! {}, len: {}", buf, buf.len());
        assert_eq!(buf.as_bytes(), b"testttt\r\n");
    });
    for stream in listener.incoming() {
        let mut reader = BufReader::new(stream.as_ref().unwrap());
        let mut read_buf = String::new();
        reader.read_line(&mut read_buf).unwrap();
        println!("get read line1{}", read_buf);
        reader.read_line(&mut read_buf).unwrap();
        println!("get read line2{}", read_buf);
        let mut writer= BufWriter::new(stream.as_ref().unwrap());
        writer.write_all(b"testttt\r\n").unwrap();
        break;
    }
    handle1.join();

}