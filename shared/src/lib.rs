#![feature(impl_trait_in_fn_trait_return)]

use std::{fmt::Display, future::Future, net::Ipv4Addr, time::Duration};

pub const MAGIC_BYTES_SIZE: usize = 32;
// This is actually just the sha256sum of "Hello World!"
pub const MAGIC_BYTES: [u8; MAGIC_BYTES_SIZE] = [
    0x03, 0xba, 0x20, 0x4e, 0x50, 0xd1, 0x26, 0xe4, 0x67, 0x4c, 0x00, 0x5e, 0x04, 0xd8, 0x2e, 0x84,
    0xc2, 0x13, 0x66, 0x78, 0x0a, 0xf1, 0xf4, 0x3b, 0xd5, 0x4a, 0x37, 0x81, 0x6b, 0x6a, 0xb3, 0x40,
];

pub const MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(239, 2, 2, 2);
pub const MULTICAST_PORT: u16 = 8888;
pub const MULTICAST_SOCKET: (Ipv4Addr, u16) = (MULTICAST_ADDRESS, MULTICAST_PORT);

pub async fn try_until<F, T, E>(
    func: impl Fn() -> F,
    repeat: Duration,
) -> T
where
    F: Future<Output = Result<T, E>>,
    T: Send + 'static,
    E: Display + 'static,
{
    loop {
        match func().await {
            Ok(res) => break res,
            Err(err) => {
                eprintln!("Encountered Error: {err}");
                tokio::time::sleep(repeat).await
            }
        }
    }
}