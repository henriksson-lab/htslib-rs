use std::ffi::c_int;
use std::io::{Read, Write};
use std::net::TcpStream;

// original: check_running (htslib/ref_cache/ping.c:39)
pub fn ref_cache_ping_c_39_check_running(port_num: c_int) -> c_int {
    let request = b"GET /hello HTTP/1.0\r\n\r\n";
    let expected = b"HTTP/1.0 200 OK\r\n";
    let len = expected.len();

    // Connect to the locally running server. This mirrors the C path of
    // getaddrinfo + socket + connect over the loopback interface; std's
    // TcpStream::connect performs the same address resolution and connect.
    let mut stream = match TcpStream::connect(("localhost", port_num as u16)) {
        Ok(s) => s,
        Err(e) => {
            // A refused connection means no server is running: report "not running".
            if e.kind() == std::io::ErrorKind::ConnectionRefused {
                return 0;
            }
            eprintln!("Couldn't connect socket: {e}");
            return -1;
        }
    };

    if let Err(e) = stream.write_all(request) {
        eprintln!("Writing to socket: {e}");
        return -1;
    }

    let mut reply = vec![0u8; 256];
    let mut bytes = 0usize;
    while bytes < reply.len() {
        match stream.read(&mut reply[bytes..]) {
            Ok(0) => break,
            Ok(n) => bytes += n,
            Err(e) => {
                eprintln!("Reading from socket: {e}");
                return -1;
            }
        }
    }

    if bytes < len || &reply[..len] != expected {
        eprintln!(
            "Unexpected reply from server:\n{}",
            String::from_utf8_lossy(&reply[..bytes])
        );
        return -1;
    }

    1
}
