/*
Build a full HTTP response from a status line and body string.

# Arguments

* `status_line` - A string slice that specifies the HTTP status line (e.g., "HTTP/1.1 200 OK").
* `body` - The HTML or plain text body of the HTTP response.

# Returns

* A `String` representing the complete HTTP response to be sent to the client.
*/
pub fn build_response(
    status_code: u16,
    reason_phrase: &str,
    content_type: &str,
    body: &str
) -> Vec<u8> {
    // Compose the HTTP response headers and body
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
        status_code,
        reason_phrase,
        body.len(),
        content_type,
        body
    );

    // Return response as bytes for sending
    return response.into_bytes();
}
