use std::env;
use std::io::{self, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const FALLBACK_PORTS: &[u16] = &[443, 80, 8080, 8443, 4444, 53, 993, 995];

fn try_connect(host: &str, ports: &[u16]) -> Option<TcpStream> {
    for &port in ports {
        let socket_addr = format!("{}:{}", host, port);
        let addr = match socket_addr.to_socket_addrs() {
            Ok(mut addrs) => match addrs.next() {
                Some(a) => a,
                None => continue,
            },
            Err(_) => continue,
        };
        match TcpStream::connect_timeout(&addr, Duration::from_secs(3)) {
            Ok(stream) => {
                println!("[+] Verbunden mit {}:{}", host, port);
                return Some(stream);
            }
            Err(_) => continue,
        }
    }
    None
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Client (SSH-ähnlich) v0.2");
        println!("");
        println!("VERWENDUNG:");
        println!("  {} <server-ip> [port]", args[0]);
        println!("");
        println!("BEISPIELE:");
        println!("  {} 192.168.1.100", args[0]);
        println!("  {} 192.168.1.100 4444", args[0]);
        println!("  {} example.com 443", args[0]);
        println!("");
        println!("Ohne Port-Angabe werden mehrere Ports probiert:");
        println!("  {:?}", FALLBACK_PORTS);
        return;
    }

    let host = &args[1];
    let ports: Vec<u16> = if args.len() >= 3 {
        args[2]
            .split(',')
            .filter_map(|p| p.trim().parse().ok())
            .collect()
    } else {
        FALLBACK_PORTS.to_vec()
    };

    let mut stream = loop {
        match try_connect(host, &ports) {
            Some(s) => break s,
            None => {
                println!("[-] Server nicht erreichbar. Neuer Versuch in 5s...");
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    };

    println!("[+] Shell ist bereit. Tippe Befehle ein (exit = beenden).\r\n");

    let reader = BufReader::new(stream.try_clone().unwrap());

    // Thread: Server-Ausgabe -> stdout
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut r = reader;
        loop {
            match r.read(&mut buf) {
                Ok(0) => {
                    println!("\n[-] Server-Verbindung geschlossen.");
                    break;
                }
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    print!("{}", text);
                    io::stdout().flush().unwrap();
                }
                Err(_) => break,
            }
        }
    });

    // Hauptthread: stdin -> Server
    let mut input = String::new();
    loop {
        input.clear();
        match io::stdin().read_line(&mut input) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let cmd = input.trim();
        if cmd.is_empty() {
            continue;
        }
        if cmd.eq_ignore_ascii_case("exit") {
            let _ = stream.write_all(b"exit\n");
            break;
        }
        if stream.write_all(cmd.as_bytes()).is_err() {
            println!("[-] Fehler beim Senden. Verbindung verloren.");
            break;
        }
        if stream.write_all(b"\n").is_err() {
            break;
        }
        stream.flush().unwrap();
    }
}
