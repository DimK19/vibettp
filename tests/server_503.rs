use std::thread;
use std::time::Duration;
use std::net::TcpStream;
use std::io::{Read, Write};

mod common;

use common::{send_request, SERVER_ADDR};

#[test]
fn test_503() {
    // Spawn 4 clients to saturate the server
    let mut handles = vec![];
    for _ in 0..4 {
        handles.push(thread::spawn(|| {
            let mut stream = TcpStream::connect(SERVER_ADDR).unwrap();
            let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n";
            stream.write_all(request.as_bytes()).unwrap();
            thread::sleep(Duration::from_secs(3)); // Keep connection open
        }));
    }

    // Give server time to increment client count
    thread::sleep(Duration::from_millis(500));

    // Attempt a 5th connection
    let response = send_request("GET / HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n");
    assert!(response.contains("503 Service Unavailable"), "Expected 503, got:\n{}", response);

    /*
    This is waiting for all the threads to finish (i.e., joining them), and discarding any errors
    if they occur.
    handles is a Vec<JoinHandle<...>>, created by spawning threads via thread::spawn(...).
    h.join() attempts to join the thread — meaning it waits for that thread to complete execution.
    join() returns a Result<T, Box<dyn Any + Send + 'static>>, where:
        Ok(_) means the thread exited normally.
        Err(...) means the thread panicked during execution.
    let _ = h.join(); calls .join() and ignores the result (_ means “we don’t care what the result was”).

    Why do this?
    It ensures all the spawned threads finish before the test ends.
    It avoids the program exiting prematurely and potentially terminating threads mid-execution.
    Discarding the result is fine here because the test logic doesn't depend on the return value -
    it's just making sure the background clients finish their delay (thread::sleep)
    and close their sockets.
    */
    for h in handles {
        if let Err(e) = h.join() {
            eprintln!("Thread panicked: {:?}", e);
        }
    }
}
