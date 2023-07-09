use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream, UdpSocket},
};

use shared::{MAGIC_BYTES, MULTICAST_SOCKET};

const CLIENT_LOCAL_ADDRESS: (Ipv4Addr, u16) = (Ipv4Addr::UNSPECIFIED, 0);

fn prepend_magic_bytes(body: &[u8]) -> Vec<u8> {
    let mut msg = Vec::from(MAGIC_BYTES);
    msg.extend_from_slice(body);
    msg
}

async fn communicate(mut stream: TcpStream, socket: SocketAddr) {
    println!("Connected to {:?}", socket);
    loop {
        let mut buf = Vec::new();
        if stream.read_buf(&mut buf).await.is_ok() {
            if let Ok(msg) = String::from_utf8(buf) {
                println!("{msg}")
            }
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // We need to loop here in case the thread for TCP connections fails entirely
    let (stream, socket) = loop {
        // We pass a 0 as the port to let the OS designate it
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
        // Discover what port was assigned to the TCP listener
        let tcp_port = match listener.local_addr()? {
            std::net::SocketAddr::V4(addr) => addr.port(),
            std::net::SocketAddr::V6(addr) => addr.port(),
        };

        // Create a UDP socket for multicast
        let socket = UdpSocket::bind(CLIENT_LOCAL_ADDRESS).await?;
        // Prepend the magic bytes to the port number
        let msg = prepend_magic_bytes(&tcp_port.to_le_bytes());

        let heartbeat = tokio::spawn(async move {
            loop {
                // Send a single UDP packet to the multicast server
                socket.send_to(&msg, MULTICAST_SOCKET).await.unwrap();
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        // Spawn a TCP listener on designated port
        let tcp_handle = tokio::spawn(async move {
            println!("Listening for TCP connections on port {tcp_port}!");
            loop {
                if let Ok(res) = listener.accept().await {
                    return res;
                }
            }
        });

        // Wait for an incoming TCP connection
        if let Ok(res) = tcp_handle.await {
            break res;
        } else {
            heartbeat.abort();
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    };

    communicate(stream, socket).await;

    Ok(())
}
