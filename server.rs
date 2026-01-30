use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::fs;
use std::path::Path;
use std::thread;

const ROOT_DIR: &str = ".";

fn handle_client(mut stream: TcpStream) {
  let mut buffer = [0; 1024];
  if stream.read(&mut buffer).is_err() {
    return;
  }
  let request = String::from_utf8_lossy(&buffer);
  let mut lines = request.lines();
  let first_line = lines.next().unwrap_or("");
  let mut parts = first_line.split_whitespace();
  let method = parts.next().unwrap_or("");
  let path = parts.next().unwrap_or("/");

  // Clean path
  let path = &path[1..]; // remove leading '/'
  let path = if path.is_empty() { "index.html" } else { path };
  let fs_path = Path::new(ROOT_DIR).join(path);

  match method {
    "GET" => {
      if fs_path.exists() && fs_path.is_file() {
        if let Ok(contents) = fs::read(&fs_path) {
          let _ = stream.write_all(format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
            contents.len()
          ).as_bytes());
          let _ = stream.write_all(&contents);
        }
      } else {
        let _ = stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\nFile not found");
      }
    }
    "POST" => {
      // parse the requested file name from query or body
      let file_name = request.split("\r\n\r\n").nth(1).unwrap_or("").trim();
      let fs_path = Path::new(ROOT_DIR).join(file_name);

      // create an empty file
      if let Ok(_) = fs::File::create(&fs_path) {
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\n\r\nCreated");
      } else {
        let _ = stream.write_all(b"HTTP/1.1 500 INTERNAL ERROR\r\n\r\nFailed");
      }
    }
    _ => {
      let _ = stream.write_all(b"HTTP/1.1 405 METHOD NOT ALLOWED\r\n\r\n");
    }
  }
  println!("{} {}", method, path);
}

fn main() {
  let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind port 8000");
  println!("Serving HTTP on http://127.0.0.1:8000/");
  for stream in listener.incoming() {
    if let Ok(stream) = stream {
      thread::spawn(|| handle_client(stream));
    }
  }
}
