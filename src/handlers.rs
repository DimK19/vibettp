use crate::response::build_response;
use crate::response::HTTPStatus;

pub fn home() -> Vec<u8> {
    // A fixed HTTP 200 OK response with simple HTML body
    build_response(HTTPStatus::Ok, "OK", "text/html", "<h1>Welcome home!</h1>")
}

pub fn about() -> Vec<u8> {
    build_response(HTTPStatus::Ok, "OK", "text/html", "<h1>About us</h1>")
}

pub fn file(body: &str) -> Vec<u8> {
    build_response(HTTPStatus::Ok, "OK", "text/html", body)
}

pub fn bad_request() -> Vec<u8> {
    build_response(HTTPStatus::BadRequest, "Bad Request", "text/plain", "400 Bad Request")
}

pub fn not_found() -> Vec<u8> {
    build_response(HTTPStatus::NotFound, "Not Found", "text/plain", "404 Not Found")
}

pub fn method_not_allowed() -> Vec<u8> {
    build_response(HTTPStatus::MethodNotAllowed, "Method Not Allowed", "text/plain", "405 Method Not Allowed")
}

pub fn request_timeout() -> Vec<u8> {
    build_response(HTTPStatus::RequestTimeout, "Request Timeout", "text/plain", "408 Request Timeout")
}

pub fn content_too_large() -> Vec<u8> {
    build_response(HTTPStatus::ContentTooLarge, "Content Too Large", "text/plain", "413 Content Too Large")
}

pub fn service_unavailable() -> Vec<u8> {
    build_response(HTTPStatus::ServiceUnavailable, "Service Unavailable", "text/plain", "503 Service Unavailable")
}
