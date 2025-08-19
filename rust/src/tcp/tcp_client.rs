use std::io::Error;
use tokio::net::TcpStream;

pub struct TcpClient {
    host_address: String,
    host_port: u16,
}

impl TcpClient {
    pub fn new(host_address: String, host_port: u16) -> Self {
        TcpClient {
            host_address,
            host_port,
        }
    }

    pub async fn connect(&self) -> Result<(), Error> {
        let host_addr = format!("{}:{}", self.host_address, self.host_port);
        let stream = TcpStream::connect(&host_addr).await?;

        println!("TCP Connected to {}", host_addr);

        Ok(())
    }
}
