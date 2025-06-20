// Represents a basic HTTP request with method and path only.
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
}

// Parses a raw HTTP request buffer into a Request struct.
pub fn parse_request(buffer: &[u8]) -> Option<Request> {
    // Convert raw bytes to UTF-8 string (fallible).
    // match is switch
    let request_str = match std::str::from_utf8(buffer) {
        Ok(s) => s,
        Err(_) => return None,
    };
    /*
    Explanation of Ok
    Rust does not have exceptions. Instead, functions that may fail have a return type that
    indicates so. Exempli gratia:
    fn get_weather(location: LatLng) -> Result<WeatherReport, io::Error>
    The Result type indicates possible failure. The function will return either a success result
    Ok(wr), where wr is a new WeatherReport value, or an error result Err(ev), where ev is an io::Error.

    std::str::from_utf8(buffer) tries to convert raw bytes (&[u8]) into a UTF-8 string slice (&str).
    It returns a Result<&str, Utf8Error>.
    */

    // Split the request string into lines.
    // The first line typically looks like: "GET /index.html HTTP/1.1"
    let mut lines = request_str.lines();

    // The first line is the request line: METHOD PATH VERSION
    /*
    Explanation of Some
    Rust references are never null. There is no analogue to C’s NULL or C++’s nullptr.
    There is no default initial value for a reference (any variable that has not been initialised
    cannot be used, regardless of type) and Rust will not convert integers to references (outside
    of code within an unsafe block). So zero cannot be converted to a reference.

    In Rust, if you need a value that may be either a reference to something or “null”, you use
    the type Option<&T>. At the machine level, Rust represents None as a null pointer and Some(r),
    where r is a value of type &T, as the non-zero address. Thus, Option<&T> is just as efficient
    as a nullable pointer in C or C++, yet safer: its type requires checking whether it is None
    before it can be used.
    */
    if let Some(request_line) = lines.next() {
        // Split by whitespace to extract method and path.
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_string();
        let path = parts.next()?.to_string();
        let version = parts.next()?.to_string();

        // Return a populated Request struct if successful.
        return Some(Request { method, path, version });
    }

    /*
    Explanation of if above
    1. lines.next()
       lines is an iterator over lines of a string (created using .lines()).
       .next() tries to get the next item from the iterator.
       It returns an Option<&str>:
       Some("GET / HTTP/1.1") → if a line is available
       None → if there are no more lines
    2. if let Some(request_line) = ...
       This is a pattern match combined with an if.
       It means: “If lines.next() returns a Some value (id est, not None), bind its value
       to the varaible request_line and run the block.”
    */

    // If the format is wrong, return None.
    return None;
}
