#[repr(u16)]
#[derive(Copy, Clone, Debug)]
pub enum HTTPStatus {
    Ok = 200,
    BadRequest = 400,
    NotFound = 404,
    MethodNotAllowed = 405,
    RequestTimeout = 408,
    ContentTooLarge = 413,
    ServiceUnavailable = 503
}

/*
Build a full HTTP response from a status line and body string.

# Arguments

* `status_line` - A string slice that specifies the HTTP status line (e.g., "HTTP/1.1 200 OK").
* `body` - The HTML or plain text body of the HTTP response.

# Returns

* A `String` representing the complete HTTP response to be sent to the client.
*/
pub fn build_response(
    status_code: HTTPStatus,
    reason_phrase: &str,
    content_type: &str,
    body: &str
) -> Vec<u8> {
    // Compose the HTTP response headers and body
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
        status_code as u16, // cast to int instead of implementing ‘Display’ trait for the enum (something like repr)
        reason_phrase,
        body.len(),
        content_type,
        body
    );

    // Return response as bytes for sending
    return response.into_bytes();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_formatting() {
        let resp = build_response(HTTPStatus::Ok, "OK", "text/html", "200 OK");
        let text = String::from_utf8_lossy(&resp);
        assert!(text.contains("200 OK"));
    }
}
