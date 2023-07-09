#![feature(split_array)]
#![feature(async_closure)]

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, UdpSocket},
};

use shared::{try_until, MAGIC_BYTES, MAGIC_BYTES_SIZE, MULTICAST_ADDRESS, MULTICAST_PORT};

fn remove_header(msg: &[u8]) -> Option<&[u8]> {
    if msg.len() < MAGIC_BYTES_SIZE {
        return None;
    }
    let (header, body) = msg.split_array_ref::<MAGIC_BYTES_SIZE>();
    if *header != MAGIC_BYTES {
        return None;
    }
    Some(body)
}

async fn handle(src: SocketAddr, msg: Vec<u8>) {
    if let Some(body) = remove_header(&msg) {
        let tcp_port = u16::from_le_bytes(*body.split_array_ref().0);
        let target_ip = match src {
            std::net::SocketAddr::V4(addr) => IpAddr::V4(*addr.ip()),
            std::net::SocketAddr::V6(addr) => IpAddr::V6(*addr.ip()),
        };
        match TcpStream::connect((target_ip, tcp_port)).await {
            Ok(stream) => {
                println!("Established TCP connection with {target_ip}:{tcp_port}");
                communicate(stream).await
            }
            Err(err) => {
                eprintln!("Failed to establish TCP connection with {target_ip}:{tcp_port}");
                eprintln!("Err: {err}")
            }
        }
    }
}

async fn communicate(mut stream: TcpStream) {
    loop {
        let _ = stream.write_all(b"Hello World!").await;
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

#[tokio::main]
pub async fn main() -> ! {
    // Create the UDP socket that we'll be using for multicast
    let bind_udp = async || UdpSocket::bind((Ipv4Addr::UNSPECIFIED, MULTICAST_PORT)).await;
    let socket = try_until(bind_udp, Duration::from_secs(1)).await;

    // Join the multicast group to recieve packets from clients
    let join_mc = async || socket.join_multicast_v4(MULTICAST_ADDRESS, Ipv4Addr::UNSPECIFIED);
    try_until(join_mc, Duration::from_secs(1)).await;

    loop {
        // Await a UDP packet from multicast
        let mut buf = Vec::new();
        if let Ok((_, src)) = socket.recv_buf_from(&mut buf).await {
            tokio::spawn(handle(src, buf));
        }
    }
}
