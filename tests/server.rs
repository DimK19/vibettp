mod common;
use common::{send_request};

/*
Tests using Rust’s built-in #[test] attribute are executed in parallel by default (via cargo test).
*/
#[test]
fn test_homepage_response() {
    let response = send_request("GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    // Assert expected content
    assert!(response.contains("200 OK"), "Response did not contain 200 OK:\n{}", response);
}

#[test]
fn test_400() {
    let response = send_request("GET /../password.txt HTTP/1.1\r\nHost: localhost\r\n\r\n");
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
    assert!(response.contains("413 Content Too Large"), "Expected 413, got:\n{}", response);
}
