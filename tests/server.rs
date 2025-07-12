use std::net::TcpStream;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

const SERVER_ADDR: &str = "127.0.0.1:7878";

fn send_request(request: &str) -> String {
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

#[test]
fn test_homepage_response() {
    let response = send_request("GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    // Assert expected content
    assert!(response.contains("200 OK"), "Response did not contain 200 OK:\n{}", response);
}

#[test]
fn test_400() {
    let response = send_request("NOT_A_REQUEST");
    assert!(response.contains("400 Bad Request"), "Expected 400, got:\n{}", response);
}

#[test]
fn test_404() {
    let response = send_request("GET /test HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert!(response.contains("404 Not Found"), "Response did not contain 404:\n{}", response);
}

#[test]
fn test_405() {
    let response = send_request("PUT / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert!(response.contains("405 Method Not Allowed"), "Expected 405, got:\n{}", response);
}

#[test]
fn test_413() {
    let mut large_body = "POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 9000\r\n\r\n".to_string();
    large_body.push_str(&"A".repeat(9000));
    let response = send_request(&large_body);
    assert!(response.contains("413 Payload Too Large"), "Expected 413, got:\n{}", response);
}

#[test]
fn test_503() {
    // Spawn 4 clients to saturate the server
    let mut handles = vec![];
    for _ in 0..4 {
        handles.push(thread::spawn(|| {
            let _stream = TcpStream::connect(SERVER_ADDR).unwrap();
            thread::sleep(Duration::from_secs(3)); // Keep connection open
        }));
    }

    // Give server time to increment client count
    thread::sleep(Duration::from_millis(500));

    // Attempt a 5th connection
    let response = send_request("GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert!(response.contains("503 Service Unavailable"), "Expected 503, got:\n{}", response);

    for h in handles {
        let _ = h.join();
    }
}
