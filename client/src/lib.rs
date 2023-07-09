#![feature(async_closure)]

use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream, UdpSocket},
};

use shared::{MAGIC_BYTES, MULTICAST_SOCKET, try_until};

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
        if let Ok(n) = stream.read_buf(&mut buf).await {
            if n == 0 {
                return;
            }
            if let Ok(msg) = String::from_utf8(buf) {
                println!("{msg}")
            }
        }
    }
}

async fn heartbeat(socket: UdpSocket, msg: Vec<u8>) {
    loop {
        // Send a single UDP packet to the multicast server
        socket.send_to(&msg, MULTICAST_SOCKET).await.unwrap();
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn tcp_listen(listener: TcpListener, port: u16) -> (TcpStream, SocketAddr) {
    println!("Listening for TCP connections on port {port}!");
    try_until(async || {listener.accept().await}, Duration::ZERO).await
}

async fn get_connection() -> Result<(TcpStream, SocketAddr), Box<dyn std::error::Error>> {
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

    // Spawn a heartbeat that'll send out a multicast message at fixed intervals
    let heartbeat = tokio::spawn(heartbeat(socket, msg));

    // Spawn a TCP listener on designated port
    let tcp_handle = tokio::spawn(tcp_listen(listener, tcp_port));

    // Wait for an incoming TCP connection
    let res = tcp_handle.await;

    // Kill the heartbeat, as we'll be in once of two states:
    // 1. A TCP connection was accepted, at which point we can migrate to that
    // 2. The thread listening for TCP connections was killed, at which point we'll need to create a new listener
    heartbeat.abort();

    Ok(res?)
}

#[tokio::main]
pub async fn main() -> ! {
    loop {
        let (stream, socket) = try_until(get_connection, Duration::from_secs(5)).await;
        communicate(stream, socket).await;
        println!("Connection to server terminated, resuming idle multicast pings!");
    }
}
