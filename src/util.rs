use std::path::{Path, PathBuf};

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
    println!("Inside sanitize_path!");
    /*
    trim_start_matches('/') removes the leading slash from the path
    (e.g. "/about.html" → "about.html"). This is necessary because Path::new("/about.html")
    would create an absolute path like C:/about.html, skipping your public/ directory.
    Path::new(...) turns the resulting string into a Path object (but it's still relative).
    requested might now be "index.html" or "images/logo.png".
    */
    let requested = Path::new(url_path.trim_start_matches('/'));
    return requested;

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
    */
    let base = Path::new("public").canonicalize().ok()?;

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
    println!("{:?}", &base);
    println!("{:?}", &normalized);
    /*
    Check if the requested path is inside the public/ directory.
    Prevent directory traversal attacks like ../../etc/passwd, which would escape the base dir.
    */
    if normalized.starts_with(&base) {
        return Some(normalized);
    } else {
        return None;
    }
}
