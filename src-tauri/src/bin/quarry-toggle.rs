use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

fn get_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .or_else(|_| std::env::var("TMPDIR"))
        .unwrap_or_else(|_| "/tmp".to_string());
    
    PathBuf::from(runtime_dir).join("quarry.sock")
}

fn main() {
    let socket_path = get_socket_path();
    
    let mut stream = match UnixStream::connect(&socket_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect to quarry: {}", e);
            eprintln!("Is quarry running?");
            std::process::exit(1);
        }
    };
    
    let command = std::env::args().nth(1).unwrap_or_else(|| "toggle".to_string());
    
    let cmd = match command.as_str() {
        "toggle" => r#"{"Toggle":null}"#,
        "show" => r#"{"Show":null}"#,
        "hide" => r#"{"Hide":null}"#,
        "ping" => r#"{"Ping":null}"#,
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Available commands: toggle, show, hide, ping");
            std::process::exit(1);
        }
    };
    
    if let Err(e) = stream.write_all(format!("{}\n", cmd).as_bytes()) {
        eprintln!("Failed to send command: {}", e);
        std::process::exit(1);
    }
    
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    if reader.read_line(&mut response).is_ok() {
        std::process::exit(0);
    }
}
