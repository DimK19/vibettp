// Build a fixed HTTP 200 OK response with simple HTML body
pub fn build_response() -> Vec<u8> {
    let body = "<h1>Hello, world!</h1>";

    // Format HTTP headers and body
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        body.len(),
        body
    );

    // Return response as bytes for sending
    response.into_bytes()
}
