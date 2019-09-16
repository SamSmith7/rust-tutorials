extern crate server;

use std::io::prelude::*;
use std::fs::File;
use std::net::TcpListener;
use std::net::TcpStream;
use server::ThreadPool;


fn main() {

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4).unwrap();

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting Down");
}

fn get_page(uri: &str) -> String {

    let mut file = File::open(uri).unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();
    contents
}

fn match_request(buffer: &[u8]) -> String {

    let valid_req = b"GET / HTTP/1.1\r\n";

    if buffer.starts_with(valid_req) {

        let contents = get_page("hello.html");
        format!("HTTP/1.1 200 OK \r\n\r\n{}", contents)

    } else {

        let contents = get_page("404.html");
        format!("HTTP/1.1 404 NOT FOUND\r\n\r\n{}", contents)
    }
}

fn handle_connection(mut stream: TcpStream) {

    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    let response = match_request(&buffer);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

}
