// Declare modules
mod winsock;
mod util;
mod response;
mod request;
mod handlers;
mod config;

use winsock::run_server;

fn main() {
    // Start the raw Winsock server
    run_server();
}
