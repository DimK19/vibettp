use std::path::{Path, PathBuf};
use std::fs;

use crate::config::Config;

// Converts a u16 port number to network byte order (big endian)
// htons = "host to network short"
pub fn htons(port: u16) -> u16 {
    port.to_be()
}

/*
Prevent a user from requesting files outside the public directory using sneaky paths like:
GET /../secret.txt
GET /../../etc/passwd
GET /symlink-to-root/etc/shadow
These are attacks. We need to sanitize the path so users can only access files inside
the safe directory: (e.g. public/).

In other words, prevent malicious paths like GET /../../etc/passwd from escaping the public/
directory and accessing sensitive files.

Returns a file system path, not a URL.
It returns a PathBuf, which is an owned file system path like:

C:\project\myapp\public\index.html       // on Windows
/home/user/myapp/public/index.html       // on Linux/macOS

So after sanitization, it’s a safe, absolute file path you can use to:
Open and read the file,
Serve it as a static file,
Or check whether it exists.

Why Not a URL?
Because:
URLs (like "/index.html") are what the browser sends.
File system paths (like "public/index.html") are what your server uses to locate and load files.
This function converts the URL into a safe file path, after validating it doesn’t escape the
intended directory.

What This Code Does (Step-by-Step)
Suppose:
base = /home/user/project/public ← this is your safe "public" folder
The user sends: GET /../etc/passwd
So: requested = ../etc/passwd ← user is trying to escape!


*/
pub fn sanitize_path(url_path: &str) -> Option<PathBuf> {
    println!("🔍 Entered sanitize_path()");
    println!("📥 Raw URL path: {:?}", url_path);

    // Disallow backslashes (Windows-specific), null bytes, or path traversal
    if url_path.contains("..") || url_path.contains('\\') || url_path.contains('\0') {
        println!("⛔️ Rejected: Malicious characters found.");
        return None;
    }

    /*
    trim_start_matches('/') removes the leading slash from the path
    (e.g. "/about.html" → "about.html"). This is necessary because Path::new("/about.html")
    would create an absolute path like C:/about.html, skipping your public/ directory.
    Path::new(...) turns the resulting string into a Path object (but it's still relative).
    requested might now be "index.html" or "images/logo.png".
    */
    let requested = Path::new(url_path.trim_start_matches('/'));
    println!("📂 Cleaned relative path: {:?}", requested);

    /*
    Prepend the public/ directory to whatever the user requested.
    For example, "index.html" becomes "public/index.html".
    join automatically handles things like slashes, so this will work for nested paths too
    (e.g. "css/style.css" → "public/css/style.css").
    */
    // let full_path = Path::new("public").join(requested);

    /*
    .canonicalize() resolves the path into an absolute, normalized version,
    resolving symlinks (like shortcuts), double periods (..), single periods (.), etc.
    For example, "public/../secret.txt" becomes something like C:\project\secret.txt.
    If this fails (e.g., file doesn’t exist or is invalid), it returns an Err, which .ok()?
    converts into None via the ? operator. So, the following line either
    Resolves the full path,
    Or returns None if it's invalid.
    */
    // let canonical = full_path.canonicalize().ok()?;

    /*
    Canonize the trusted root directory (public/) to get its absolute path.
    For example: C:\project\public.
    Path::new("public").canonicalize() fails — likely because the "public" directory does
    not exist, and .canonicalize() silently returns None via .ok()?, exiting the function early.
    .ok()? returns None without panicking or printing if the directory doesn't exist
    or is inaccessible. Why this happens:
    Rust’s design leans heavily on the idea that, if you don’t want to handle the error,
    then the caller should. But .ok()? is especially bad because it converts a Result into an
    Option and then silently exits. That’s often the opposite of what you want in a debug or
    server scenario, where visibility is critical. Rust gives you tools to make these things
    explicit (match, if let Err(e), etc.), but it defaults to implicit behaviour that can be painful.
    */
    // let base = Path::new("C:\\Users\\KYRIAKOS\\Desktop").canonicalize().ok()?;
    let raw = fs::read_to_string("config.toml").expect("❌ Failed to read config file");
    let config: Config = toml::from_str(&raw).expect("❌ Failed to parse config");

    println!("📂 Root directory: {}", config.root_directory);
    let base = match Path::new(&config.root_directory).canonicalize() {
        Ok(path) => {
            println!("🛡 Canonical base dir: {:?}", path);
            path // Cannot be return path; here because this is the result of match
        }
        Err(e) => {
            eprintln!("❌ Failed to canonicalize base directory: {}", e);
            return None;
        }
    };

    /*
    Join and normalize the full target path without requiring existence
    What's going on here:
    base.join(requested) → combines the paths:
    /home/user/project/public + ../etc/passwd = /home/user/project/public/../etc/passwd

    .components() → breaks the path into pieces:
    ["home", "user", "project", "public", "..", "etc", "passwd"]

    .collect::<PathBuf>() → rebuilds the path while removing .. and .
    (".." means “go up one directory”)

    So:
    "public", ".." → cancel each other out
    Resulting in:
    /home/user/project/etc/passwd
    Now we check:
    if normalized.starts_with(&base)
    → This checks whether /home/user/project/etc/passwd starts with /home/user/project/public
    → It does not.
    So we reject it by returning None.

    What This Means
    This line detects and blocks traversal outside your safe folder.

    Bad Path Example
    Input path: ../etc/passwd

    normalized = /home/user/project/etc/passwd
    starts_with(base)? → No
    → return None
    BLOCKED

    Good Path Example
    Input path: images/logo.png

    normalized = /home/user/project/public/images/logo.png
    starts_with(base)? → Yes
    → return Some(normalized)
    ALLOWED
    */
    let normalized = base.join(requested).components().collect::<PathBuf>();
    println!("📌 Normalized full path: {:?}", normalized);
    /*
    Check if the requested path is inside the public/ directory.
    Prevent directory traversal attacks like ../../etc/passwd, which would escape the base dir.
    */
    if normalized.starts_with(&base) {
        println!("✅ Safe: Path is within base.");
        return Some(normalized);
    } else {
        println!("🚫 Unsafe: Path escapes base.");
        return None;
    }

    /*
    📠 HTTP Version: HTTP/1.1 Method: GET, Path: /hello
    🔍 Entered sanitize_path()
    📥 Raw URL path: "/hello"
    📂 Cleaned relative path: "hello"
    🛡 Canonical base dir: "\\\\?\\C:\\Users\\KYRIAKOS\\Desktop"
    📌 Normalized full path: "\\\\?\\C:\\Users\\KYRIAKOS\\Desktop\\hello"
    ✅ Safe: Path is within base.
    🔌 Connection closed.

    What is the junk before C:\\ in the paths?
    That \\?\ prefix is not junk — it’s a Windows “verbatim path prefix”.
    It tells the Windows API to disable all path normalization rules like interpreting ..,
    collapsing //, or parsing C:/ differently.
    It allows very long paths (over 260 characters).
    You’ll see this when calling Path::canonicalize() on Windows — it's how Rust (via stdlib)
    safely interacts with the Win32 API.
    */
}
