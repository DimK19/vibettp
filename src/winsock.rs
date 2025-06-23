// size_of: Returns the byte size of a type (used when passing struct sizes to WinSock functions).
// zeroed: Creates a zero-initialized instance of a struct (common for FFI where padding must be 0).
use std::mem::{size_of, zeroed};

// null_mut: Used to pass a null (null pointer) to C-style functions that expect optional parameters or indicate error.
use std::ptr::null_mut;

// Import all constants, types, and functions from WinSock (Windows socket API) via the windows-sys crate.
// use windows_sys::Win32::Networking::WinSock::*;
use windows_sys::Win32::Networking::WinSock::{
    WSACleanup, WSAStartup, WSADATA, SOCKADDR, SOCKADDR_IN, IN_ADDR, IN_ADDR_0,
    socket, bind, listen, accept, recv, send, closesocket,
    INVALID_SOCKET, SOCKET_ERROR,
    AF_INET, SOCK_STREAM, IPPROTO_TCP, SOMAXCONN,
};

// Import a helper function from http.rs that builds a static HTTP response.
use crate::response::build_response;

// Import a helper from util.rs to convert a port number to network byte order (required by WinSock).
use crate::util::{htons, sanitize_path};

// Import the function that parses a request to extract method and path.
use crate::request::parse_request;

use std::collections::HashMap;
use crate::handlers;

// Entry point for the raw TCP server logic. Called by main.rs
pub fn run_server() {
    // Unsafe block. Required for raw C-style FFI (Foreign Function Interface) work.
    unsafe {
        // Everything inside here could violate Rust’s safety guarantees if misused.

        // --- Step 1: Initialize WinSock with version 2.2 ---

        // Create a zero-initialized WSAData struct to receive startup information about WinSock.
        let mut wsa_data: WSADATA = zeroed();

        // Initialize WinSock with version 2.2 (0x0202). Return non-zero on error.
        if WSAStartup(0x202, &mut wsa_data) != 0 {
            // Log an error and exit if initialization fails.
            eprintln!("WSAStartup failed");
            return;
        }

        // --- Step 2: Create a TCP socket (IPv4, stream-based) ---

        /*
        Create a new socket:
         - AF_INET: IPv4
         - SOCK_STREAM: TCP (not UDP)
         - IPPROTO_TCP: TCP protocol
        Return a socket handler (integer).
        */
        let sock = socket(AF_INET as i32, SOCK_STREAM as i32, IPPROTO_TCP as i32);

        // Check if socket creation failed
        if sock == INVALID_SOCKET {
            // Log error, clean up, exit
            eprintln!("Socket creation failed");
            WSACleanup();
            return;
        }

        // --- Step 3: Configure socket address: 127.0.0.1:7878 ---

        /*
        Create an IPv4 address struct (SOCKADDR_IN) with the following fields:
        - Address family: IPv4.
        - Port: 7878, converted to network byte order (big endian) using htons.
        - IP address: 127.0.0.1 (loopback), expressed in 4 bytes, converted to a 32-bit
          little-endian integer. S_addr: the actual IPv4 address field.
        - Padding to match C layout. Must be zeroed.
        */
        let mut addr_in = SOCKADDR_IN {
            sin_family: AF_INET as u16,
            sin_port: htons(7878), // convert to network byte order
            sin_addr: IN_ADDR {
                S_un: IN_ADDR_0 {
                    S_addr: u32::from_le_bytes([127, 0, 0, 1]), // loopback IP
                },
            },
            sin_zero: [0; 8], // padding, must be zeroed
        };

        // --- Step 4: Bind the socket to the address ---

        // Bind the socket to IP/port.
        if bind(
            sock,
            // Cast the address struct to the generic SOCKADDR type (what WinSock expects).
            &addr_in as *const _ as *const SOCKADDR,
            // Pass the size of the struct.
            size_of::<SOCKADDR_IN>() as i32,
        ) != 0 { // Returns non-zero on failure
            // Log error, close socket, and exit if bind fails.
            eprintln!("Bind failed");
            closesocket(sock);
            WSACleanup();
            return;
        }

        // --- Step 5: Begin listening for connections ---

        // Start listening for incoming connections.
        // SOMAXCONN is the max number of pending connections in queue.
        if listen(sock, SOMAXCONN.try_into().unwrap()) != 0 {
            // Log error and exit on failure.
            eprintln!("Listen failed");
            closesocket(sock);
            WSACleanup();
            return;
        }

        // Inform user that the server is live.
        println!("🌐 Listening on 127.0.0.1:7878...");

        // Set up routing table
        let mut routes: HashMap<&str, fn() -> Vec<u8>> = HashMap::new();
        routes.insert("/", handlers::home);
        routes.insert("/about", handlers::about);

        // --- Step 6: Accept a client connection ---

        // Loop forever to handle one connection at a time.
        loop {
            // Prepare a buffer to receive the client's address upon connection.
            let mut client_addr: SOCKADDR_IN = zeroed();
            let mut addr_len = size_of::<SOCKADDR_IN>() as i32;

            // Block and wait for an incoming connection.
            // Returns a new socket specific to the client.
            let client_sock = accept(
                sock,
                &mut client_addr as *mut _ as *mut SOCKADDR,
                &mut addr_len,
            );

            // Error handling if accept fails.
            if client_sock == INVALID_SOCKET {
                eprintln!("Accept failed");
                closesocket(sock);
                WSACleanup();
                return;
            }

            println!("📡 Client connected.");

            // --- Step 7: Read from client ---

            // Create a 1024-byte raw buffer to receive data from the incoming request.
            let mut buffer = [0u8; 1024];

            // Read bytes into the buffer from the client socket.
            // Returns the number of bytes read.
            let bytes_received = recv(
                client_sock,
                buffer.as_mut_ptr(),
                buffer.len() as i32,
                0,
            );

            // If data was received, decode and print the raw HTTP request from the client.
            if bytes_received > 0 {
                // Convert request to string, parse, and print it
                // Print the raw request for inspection.
                let request_data = &buffer[..bytes_received as usize];
                println!(
                    "🔍 Raw request:\n{}",
                    String::from_utf8_lossy(request_data)
                );

                if let Some(req) = parse_request(request_data) {
                    // --- Step 8: Build and send HTTP response ---

                    println!(
                        "📠 HTTP Version: {} Method: {}, Path: {}",
                        req.version, req.method, req.path
                    );

                    // Block disallowed methods
                    if req.method.as_str() != "GET" && req.method.as_str() != "POST" {
                        let response = handlers::method_not_allowed();
                        send(
                            client_sock,
                            response.as_ptr(),
                            response.len() as i32,
                            0,
                        );
                        closesocket(client_sock);
                        println!("🔌 Connection closed.\n");
                        continue;
                    }


                    // Try route match first
                    // Get the appropriate handler function
                    if let Some(handler) = routes.get(req.path.as_str()) {
                        // Create the HTTP response body using the helper function.
                        let response = handler();

                        // Send the response over the client socket.
                        send(
                            client_sock,
                            response.as_ptr(),
                            response.len() as i32,
                            0,
                        );
                    }
                    // Fallback to static file serving
                    else if let Some(safe_path) = sanitize_path(&req.path) {
                        if let Ok(contents) = std::fs::read(&safe_path) {
                            let body = std::str::from_utf8(&contents).unwrap_or("Invalid UTF-8 in file");
                            let response = build_response(200, "OK", "text/html", body);
                            send(
                                client_sock,
                                response.as_ptr(),
                                response.len() as i32,
                                0,
                            );
                        }
                        else {
                            let response = handlers::not_found();
                            send(
                                client_sock,
                                response.as_ptr(),
                                response.len() as i32,
                                0,
                            );
                        }
                    }
                    // Malicious path or error
                    else {
                        let response = handlers::bad_request();
                        send(
                            client_sock,
                            response.as_ptr(),
                            response.len() as i32,
                            0
                        );
                        closesocket(client_sock);
                        println!("🔌 Connection closed.\n");
                        continue;
                    }
                }
                else {
                    println!("⚠️ Failed to parse HTTP request.");
                }
            }

            // Close client connection.
            closesocket(client_sock);
            println!("🔌 Connection closed.\n");
        }

        // --- Step 9: Clean up sockets and Winsock ---

        // Close both client and server sockets.
        // Cleanup WinSock (equivalent to shutting down the library).
        // (never reached in this loop, but good practice for future shutdown logic)
        /*
        closesocket(client_sock);
        closesocket(sock);
        WSACleanup();
        */
    }
}
