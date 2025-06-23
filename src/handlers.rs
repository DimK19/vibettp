use crate::response::build_response;

pub fn home() -> Vec<u8> {
    // A fixed HTTP 200 OK response with simple HTML body
    build_response(200, "OK", "text/html", "<h1>Welcome home!</h1>")
}

pub fn about() -> Vec<u8> {
    build_response(200, "OK", "text/html", "<h1>About us</h1>")
}

pub fn bad_request() -> Vec<u8> {
    build_response(400, "Bad Request", "text/plain", "400 Bad Request")
}

pub fn not_found() -> Vec<u8> {
    build_response(404, "Not Found", "text/plain", "404 Not Found")
}

pub fn method_not_allowed() -> Vec<u8> {
    build_response(405, "Method Not Allowed", "text/plain", "405 Method Not Allowed")
}

pub fn content_too_large() -> Vec<u8> {
    build_response(413, "Content Too Large", "text/plain", "413 Content Too Large")
}
