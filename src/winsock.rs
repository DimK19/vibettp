// size_of: Returns the byte size of a type (used when passing struct sizes to WinSock functions).
// zeroed: Creates a zero-initialized instance of a struct (common for FFI where padding must be 0).
use std::mem::{size_of, zeroed};
use std::fs;

// null_mut: Used to pass a null (null pointer) to C-style functions that expect optional parameters or indicate error.
use std::ptr::null_mut;
use std::collections::HashMap;
use std::thread;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::Instant;

// Import all constants, types, and functions from WinSock (Windows socket API) via the windows-sys crate.
// use windows_sys::Win32::Networking::WinSock::*;
use windows_sys::Win32::Networking::WinSock::{
    WSACleanup, WSAStartup, WSADATA, SOCKADDR, SOCKADDR_IN, IN_ADDR, IN_ADDR_0,
    socket, bind, listen, accept, recv, send, closesocket, shutdown,
    INVALID_SOCKET, SOCKET_ERROR, SD_SEND,
    AF_INET, SOCK_STREAM, IPPROTO_TCP, SOMAXCONN,
    FD_SET, TIMEVAL, select,
};

// Import a helper function from http.rs that builds a static HTTP response.
// use crate::response::build_response;

// Import a helper from util.rs to convert a port number to network byte order (required by WinSock).
use crate::util::{htons, sanitize_path};

// Import the function that parses a request to extract method and path.
use crate::request::parse_request;
use crate::handlers;
use crate::config::Config;

const MAX_REQUEST_SIZE: usize = 8196; // 8KB
// const MAX_BODY_SIZE: usize = 6144; // 6KB (request line ~ 100B, headers ~ 1-2KB)

// Entry point for the raw TCP server logic. Called by main.rs
pub fn run_server() {

    let raw = fs::read_to_string("config.toml").expect("❌ Failed to read config file");
    let config: Config = toml::from_str(&raw).expect("❌ Failed to parse config");

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

        // --- Step 3: Configure socket address  ---

        /*
        Chosen address: 127.0.0.1 (loopback IP)
        Chosen port: 7878
        Both read from config file
        */
        // this will be in the form [127, 0, 0, 1]
        let ip_bytes: [u8; 4] = config.bind_address.split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect::<Vec<u8>>()
            .try_into()
            .expect("Invalid IP format");

        /*
        Create an IPv4 address struct (SOCKADDR_IN) with the following fields:
        - Address family: IPv4.
        - Port: 7878, converted to network byte order (big endian) using htons.
        - IP address: 127.0.0.1 (loopback), expressed in 4 bytes, converted to a 32-bit
          little-endian integer. S_addr: the actual IPv4 address field.
        - Padding to match C layout. Must be zeroed.
        */
        let addr_in = SOCKADDR_IN {
            sin_family: AF_INET as u16,
            sin_port: htons(config.port), // convert to network byte order
            sin_addr: IN_ADDR {
                S_un: IN_ADDR_0 {
                    S_addr: u32::from_le_bytes(ip_bytes),
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
        println!("🌐 Listening on {}:{}...", config.bind_address, config.port);

        // Set up routing table
        let mut routes: HashMap<&str, fn() -> Vec<u8>> = HashMap::new();
        routes.insert("/", handlers::home);
        routes.insert("/about", handlers::about);

        /*
        Rust threads do not share memory by default. To share data (like how many clients
        are connected), we use atomic types inside Arcs.
        The line below creates a new atomic counter initialized to 0 (number of active clients),
        and wraps it in an Arc (Atomic Reference Counted pointer), so it can be shared across
        threads. AtomicUsize is thread-safe and allows us to increment/decrement from multiple
        threads without locks. Arc enables multiple threads to own a reference to the same atomic
        counter.
        */
        let active_clients = Arc::new(AtomicUsize::new(0));

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
                break;
            }

            /*
            Read the current number of active clients from the atomic counter.
            Ordering::SeqCst means “sequentially consistent memory ordering” (the strongest
            ordering, safest but slowest — good for correctness).
            Used when deciding whether to accept a new connection (e.g., limit to 4 clients max).
            */
            let client_count = active_clients.load(Ordering::SeqCst);

            if client_count >= config.max_clients {
                println!("🚫 Too many clients.");
                let response = handlers::service_unavailable();
                send(
                    client_sock,
                    response.as_ptr(),
                    response.len() as i32,
                    0,
                );
                // For explanation see comment on line 330 (similar case).
                shutdown(client_sock, SD_SEND);
                closesocket(client_sock);
                continue;
            }

            println!("📡 Client connected.");

            /*
            Atomically increment the client count when a new client connects.
            Ensures that even if many threads accept connections at the same time,
            the count is accurate.
            fetch_add returns the previous value, which can be used if needed.
            */
            active_clients.fetch_add(1, Ordering::SeqCst);

            /*
            Clone the Arc, not the underlying AtomicUsize value.
            Now the new thread owns a reference to the shared counter too.

            Why clone? What's clone?
            Arc<T> implements Clone, which increments the reference count.
            Cloning here means "make another reference to the same shared object".
            You need to move the cloned reference into the thread since the original
            cannot be accessed from inside the move closure.

            Why same variable name?
            Shadowing in Rust: let active_clients = active_clients.clone();
            This reuses the same name for the new (cloned) Arc, which is moved into the thread.
            It’s fine and idiomatic in Rust, though you could use a new name
            (e.g., let active_clients_thread = active_clients.clone();) if clarity is needed.
            */
            let active_clients = active_clients.clone();
            let routes = routes.clone();

            // --- Step 7: Read from client ---

            /*
            Spawn a new thread. Each client gets handled in its own thread (classic multithreaded
            server model).
            move closure takes ownership of the captured variables (like active_clients, routes)
            — which is why we cloned them first.
            */
            thread::spawn(move || {
                // --- Begin keep-alive-aware inner loop ---

                // Add a per-request temporal deadline
                let start_time = Instant::now();

                'client_loop: loop {
                    // Create a 8196-byte raw buffer to receive data from the incoming request.
                    let mut buffer = [0u8; MAX_REQUEST_SIZE];

                    let mut keep_alive_requested: bool = false;

                    // Buffer to accumulate partial requests
                    let mut request_data = Vec::new();

                    loop {
                        // Check if the socket is ready for reading with a timeout
                        /*
                        Initialize an empty FD_SET struct (file descriptor set) with all values set to 0.
                        This will hold the list of sockets to monitor using select().
                        */
                        let mut fds = FD_SET {
                            fd_count: 1,
                            fd_array: [client_sock; 64], // fill first element, rest zeroed
                        };

                        /*
                        Construct a TIMEVAL struct, which defines the timeout duration.
                        tv_sec: seconds
                        tv_usec: microseconds
                        */
                        let mut timeout = TIMEVAL {
                            tv_sec: config.timeout_seconds as i32,
                            tv_usec: 0,
                        };

                        /*
                        Call select() to block either until at least one socket in fds is ready to read,
                        or until the timeout occurs
                        Parameters:
                        0: Ignored in WinSock, used in Unix to indicate max socket + 1
                        &mut fds: monitor for read
                        null_mut(): no write monitoring
                        null_mut(): no exception monitoring
                        &mut timeout: how long to wait
                        */
                        let ready = select(0, &mut fds, null_mut(), null_mut(), &mut timeout);

                        /*
                        If select() returns 0, that means timeout - no socket ready within the timeout.
                        If select() returns -1, it means an error occurred.
                        Break the client loop and close the connection.
                        */
                        if ready == 0 {
                            println!("⏱️ Timeout waiting for client data.");
                            let response = handlers::request_timeout();
                            send(
                                client_sock,
                                response.as_ptr(),
                                response.len() as i32,
                                0
                            );
                            break 'client_loop;
                        }
                        else if ready == SOCKET_ERROR {
                            eprintln!("❌ select() failed.");
                            break 'client_loop;
                        }

                        // Check elapsed time
                        if start_time.elapsed().as_secs() > config.timeout_seconds as u64 {
                            println!("⏱️ Client took too long to send full request.");
                            break 'client_loop;
                        }

                        // If select() indicates the socket is ready, proceed to call recv() safely.
                        // Read bytes into the buffer from the client socket.
                        // Returns the number of bytes read.
                        let bytes_received = recv(
                            client_sock,
                            buffer.as_mut_ptr(),
                            buffer.len() as i32,
                            0,
                        );

                        if bytes_received <= 0 {
                            let response = handlers::bad_request();
                            send(
                                client_sock,
                                response.as_ptr(),
                                response.len() as i32,
                                0
                            );
                            println!("🔌 Client disconnected.");
                            break 'client_loop;
                        }

                        request_data.extend_from_slice(&buffer[..bytes_received as usize]);

                        /*
                        recv() pulls up to N bytes (N is the buffer size, in this case 8196).
                        If the client sent more, the first N bytes are copied into the buffer, and the
                        remaining data stays queued in the socket’s internal receive buffer, managed by the
                        operating system. This data will be returned by the next recv() call.

                        Where is that data exactly?
                        The OS keeps a receive queue (buffer) per socket. It typically has a size limit
                        (e.g., 64KB or more depending on OS settings). Until you call recv() again, the data
                        sits there. If you never call recv() again and just close the socket, the OS drops the
                        remaining data.
                        */

                        // Impose limit on request size
                        if request_data.len() >= MAX_REQUEST_SIZE {
                            let response = handlers::content_too_large();
                            send(
                                client_sock,
                                response.as_ptr(),
                                response.len() as i32,
                                0,
                            );

                            /*
                            “Gracefully” shut down the write side of the socket after sending the
                            response, so that the client can finish reading before the connection
                            is torn down. This helps pass the test and the client actually sees the
                            response. Shutdown would happen regardless after breaking.
                            Otherwise, the following error would occur:

                            “thread 'test_413' panicked at tests\common.rs:16:42:
                            called `Result::unwrap()` on an `Err` value: Os { code: 10054, kind:
                            ConnectionReset, message: "An existing connection was forcibly closed by
                            the remote host." }”

                            (It means the server closed the TCP connection abruptly before the client
                            finished reading the response. This is expected when handling
                            payload-too-large (413) by immediately rejecting the request and closing
                            the socket).

                            - shutdown() is a syscall from WinSock to partially close a socket.
                            - SD_SEND is a constant (value 1) telling it to close just the sending side.
                            - Using raw sockets, not TcpStream which has std::net::Shutdown::Write.
                            */
                            shutdown(client_sock, SD_SEND);

                            break 'client_loop;
                        }

                        // Only try parsing once we have complete headers
                        /*
                        - .windows(4): This creates an iterator that returns overlapping slices
                        (windows) of 4 bytes from request_data.
                        - .any(...): An iterator method that returns true if any element of the
                        iterator satisfies the predicate.
                        - |w| w == b"\r\n\r\n": This is the closure (anonymous function) that takes
                        a window w and checks if it equals the byte string b"\r\n\r\n".

                        This approach searches for the 4-byte pattern anywhere in the buffer. It
                        works correctly even if \r\n\r\n is in the middle of the buffer.
                        */
                        if request_data.windows(4).any(|w| w == b"\r\n\r\n") {
                            break; // Found end of headers
                        }
                    }

                    /*
                    | Behavior                      | Valid Practice| Notes                               |
                    | ----------------------------- | ------------- | ----------------------------------- |
                    | Reject if recv() == buf.len() | Yes           | Defensive and efficient             |
                    | Try to read more chunks       | Risky         | Slower, invites abuse unless capped |
                    | Trust Content-Length header   | Dangerous     | Headers can lie or be omitted       |
                    */

                    // Decode and print the raw HTTP request from the client.
                    // Convert request to string, parse, and print it
                    // Print the raw request for inspection.
                    println!(
                        "🔍 Raw request:\n{}",
                        String::from_utf8_lossy(&request_data)
                    );

                    println!("Before parse request");
                    if let Some(req) = parse_request(&request_data) {
                        // --- Step 8: Build and send HTTP response ---

                        println!(
                            "📠 HTTP Version: {} Method: {}, Path: {}",
                            req.version, req.method, req.path
                        );

                        keep_alive_requested = req.keep_alive;

                        // Block disallowed methods
                        if req.method.as_str() != "GET" && req.method.as_str() != "POST" {
                            let response = handlers::method_not_allowed();
                            send(
                                client_sock,
                                response.as_ptr(),
                                response.len() as i32,
                                0,
                            );
                            break 'client_loop;
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
                                let response = handlers::file(body);
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
                                    0
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
                            continue 'client_loop;
                        }
                    }
                    else {
                        println!("⚠️ Failed to parse HTTP request.");
                    }

                    // Close client connection.
                    if !config.keep_alive || !keep_alive_requested {
                        break 'client_loop;
                    }
                }

                // --- Step 9: Clean up sockets and Winsock ---

                // Close both client and server sockets.
                // Cleanup WinSock (equivalent to shutting down the library).
                // (never reached in this loop, but good practice for future shutdown logic)

                closesocket(client_sock);
                println!("🔌 Connection closed.\n");

                // Atomically decrements the number of active clients when this thread is done.
                active_clients.fetch_sub(1, Ordering::SeqCst);
            });
        }

        WSACleanup();
    }
}
