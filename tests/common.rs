use std::net::TcpStream;
use std::io::{Read, Write};

pub const SERVER_ADDR: &str = "127.0.0.1:7878";

pub fn send_request(request: &str) -> String {
    // Connect to the (running) server
    let mut stream = TcpStream::connect(SERVER_ADDR).expect("Failed to connect");

    // Send a basic HTTP request
    stream.write_all(request.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();

    // Read the response into a string
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    return response;
}
