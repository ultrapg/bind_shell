# Reverse Shell (Bind Shell)

A lightweight remote administration tool written in Rust. Uses a **bind shell** architecture where the server listens for incoming connections and the client connects to issue commands — similar to SSH.

## How It Works

```
┌─────────────────┐          TCP         ┌──────────────────────┐
│  CLIENT.EXE     │ ◄──────────────────► │  SERVER.EXE          │
│  (Remote)       │    connect/send      │  (Target)            │
│                 │    receive/display   │                      │
│  You type:      │                      │  - Listens on port   │
│    whoami       │ ──────command──────► │  - Spawns cmd.exe    │
│                 │                      │  - Executes command  │
│  You see:       │ ◄─────output───────  │  - Sends output back │
│    desktop\user │                      │                      │
└─────────────────┘                      └──────────────────────┘
```

- **server.exe** — runs on the target machine, opens a port, spawns a persistent `cmd.exe` process
- **client.exe** — runs on your machine, connects to the server, sends commands and displays output

All commands execute inside a single persistent `cmd.exe` shell, so `cd`, environment variables, and command chaining (`&&`) maintain state across commands.

## Features

- **Persistent shell** — single `cmd.exe` process lives for the entire session; directory changes and variables persist
- **Firewall evasion** — client auto-tries common allowed ports (443, 80, 8080, 8443, 4444, 53, 993, 995) when no port is specified
- **Multi-port fallback** — client scans multiple ports to find the server
- **Multi-client** — server accepts multiple simultaneous connections
- **Zero dependencies** — pure Rust standard library, no external crates
- **Small binaries** — ~200 KB each, statically compiled
- **Cross-platform target** — compiles for any platform Rust supports (though `cmd.exe` is Windows-specific)

## Requirements

- [Rust](https://rustup.rs/) (edition 2021)
- Windows (server uses `cmd.exe`; client can run on any OS)

## Building

```bash
# Build server (target machine)
cd server
cargo build --release

# Build client (your machine)
cd ../client
cargo build --release
```

Binaries are output to `server/target/release/server.exe` and `client/target/release/client.exe`.

## Usage

### 1. Start the server on the target machine

```bash
server\target\release\server 4444
```

The server listens on `0.0.0.0:4444` and waits for a client connection.

### 2. Connect from your machine

```bash
client\target\release\client 192.168.1.100 4444
```

Replace `192.168.1.100` with the IP address of the target machine.

### 3. Type commands

Once connected, type commands directly into the client console:

```
whoami
desktop-xyz\user

ipconfig

Windows IP Configuration
Ethernet adapter Ethernet0:
   IPv4 Address. . . . . . . . . . . : 192.168.1.100
   ...

cd C:\Users
dir
Volume in Laufwerk C: hat keine Bezeichnung.
 ...

exit
```

Type `exit` to close the connection.

### Port Selection

If no port is specified, the client tries these ports in order (firewall evasion):

```
443, 80, 8080, 8443, 4444, 53, 993, 995
```

You can specify multiple ports as a comma-separated list:

```bash
client 192.168.1.100 443,80,4444
```

You can also run the server on a privileged port if you have admin rights:

```bash
server 443    # requires admin on Windows
```

## Firewall Evasion

The client's multi-port fallback system helps bypass restrictive firewalls:

1. **Port 443 (HTTPS)** — almost always allowed outbound
2. **Port 80 (HTTP)** — universally permitted
3. **Port 8080 (HTTP alternate)** — common development port
4. **Port 8443 (HTTPS alternate)** — common secure development port
5. **Port 4444** — default listener
6. **Port 53 (DNS)** — rarely blocked
7. **Port 993 (IMAPS)** — commonly allowed
8. **Port 995 (POP3S)** — commonly allowed

The client reconnects automatically every 5 seconds if the server is unreachable.

## Technical Details

- **Bind shell** — server listens, client connects (the reverse of a reverse shell)
- **Persistent child process** — server spawns `cmd.exe /Q` (quiet mode, echo off) and pipes its stdin/stdout/stderr through the TCP socket
- **Threading** — server uses three threads per client: stdout forwarding, stderr forwarding, and command input reading
- **Binary size** — release builds are ~200 KB with no external dependencies
- **Language** — Rust 2021 edition, pure `std` library

## Project Structure

```
reverse-shell/
├── client/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs          # Client: connects, reads stdin, sends commands
├── server/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs          # Server: listens, spawns cmd.exe, relays I/O
└── README.md
```

## License

GNU General Public License v3.0
