# Minimal WinSock HTTP Server in Rust

A low-level HTTP 1.1 server written in _Rust_, using the raw Windows Sockets API (WinSock).  
This project was built to explore Rust’s systems-level capabilities, including manual memory management, multi-threading, and Foreign Function Interface (FFI) with the Windows API.

## Features

- ⚡ Raw socket operations using the `windows-sys` crate (WinSock FFI)
- 🌐 Configurable IP and port via `config.toml`
- 🧵 Multi-threaded handling of up to 4 concurrent client connections
- 🚦 Sends `503 Service Unavailable` if maximum clients are exceeded
- 🧭 Basic routing support (`/`, `/about`, etc.) using `HashMap`
- 🗂️ Serves static HTML files from the working directory
- ⏳ Timeout and `Keep-Alive` support
- 🔒 Input sanitization to prevent directory traversal
- 🛡️ Defines request size limit for security
- 📛 Specifies allowed HTTP methods
- 🧠 HTTP status codes defined as a Rust `enum`

---

## ⚙️ Installation

> 🪟 **Windows only** (uses `windows-sys` for raw socket operations)

1. **Clone the repository**:
   ```sh
   git clone https://github.com/DimK19/vibettp.git
   cd vibettp
   ```

2. **Create a config.toml file (see example below)**

3. **Build the project**:
   ```sh
   cargo build
   ```

4. **Run the server**:
   ```sh
   cargo run
   ```

### Example `config.toml`
This file is required and must be placed in the project root. It is `.gitignore`d by default.

_config.toml_
```toml
## Root directory for file serving
root_directory = "C:/..."

## Enable HTTP Keep-Alive (persistent connections)
keep_alive = true

## Timeout in seconds before closing inactive client connections
timeout_seconds = 180

## Maximum number of concurrent client connections
max_clients = 4

## IP address to bind the server
## Local IP for LAN (can be found via ipconfig), 127.0.0.1 for loopback
bind_address = "127.0.0.1"
port = 7878
```

### Usage Notes

Server listens only on the configured IP and port.

### Acknowledgements 🤖
Major assistance provided by ChatGPT (GPT-4.5, July 2025) - used extensively for FFI bindings, concurrency design, architecture, and code comments.

