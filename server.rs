use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::fs;
use std::path::Path;
use std::thread;

const ROOT_DIR: &str = ".";
const STATFILES_SUBDIR: &str = "stat_files";

fn handle_client(mut stream: TcpStream) {
  let statfiles_dir = Path::new(ROOT_DIR).join(STATFILES_SUBDIR);
  println!("La directory: {}", statfiles_dir.display());
  let mut buffer = [0; 1024];
  let bytes_read = match stream.read(&mut buffer) {
    Ok(n) => n,
    Err(_) => return,
  };
  let request = String::from_utf8_lossy(&buffer[..bytes_read]);
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
      let file_name_raw = path.trim_start_matches('/');
      let file_name: String = file_name_raw
      .chars()
      .map(|c| {
        if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' {
          c
        } else {
          '_'
        }
      })
      .collect();
      eprintln!("This is the file to create: {}", file_name);
      let fs_path = Path::new(&statfiles_dir).join(file_name);
      
      // create an empty file
      match fs::File::create(&fs_path) {
        Ok(_) => {
          let _ = stream.write_all(b"HTTP/1.1 200 OK\r\n\r\nCreated");
        } 
        Err(e) => {
          eprintln!("Failed to create file {:?}: {}", fs_path, e);
          let _ = stream.write_all(b"HTTP/1.1 500 INTERNAL ERROR\r\n\r\nFailed");
        }
      }
    }
    _ => {
      let _ = stream.write_all(b"HTTP/1.1 405 METHOD NOT ALLOWED\r\n\r\n");
    }
  }
  println!("{} {}", method, path);
}

fn main() {
  let listener = TcpListener::bind("127.0.0.1:24375").expect("Failed to bind port 24375");
  println!("Serving HTTP on http://127.0.0.1:24375/");
  for stream in listener.incoming() {
    if let Ok(stream) = stream {
      thread::spawn(|| handle_client(stream));
    }
  }
}
