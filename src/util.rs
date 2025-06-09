// Converts a u16 port number to network byte order (big endian)
// htons = "host to network short"
pub fn htons(port: u16) -> u16 {
    port.to_be()
}
