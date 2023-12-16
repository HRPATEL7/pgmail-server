use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::state::StateMachine;


pub struct Server {
    socket: pgmail_server::TlsStream<tokio::net::TcpStream>,
    state_machine: StateMachine,
}
impl Server {
    pub async fn new(socket: pgmail_server::TlsStream<tokio::net::TcpStream>, domain: String) -> Result<Self> {
        Ok(Self {
            socket,
            state_machine:StateMachine::new(&domain),
        })
    }
    pub async fn serve(mut self) -> Result<()> {
        self.connection().await.ok();
        let (reader,mut writer)=tokio::io::split(self.socket);
        let mut buffer = BufReader::new(reader);
        let mut line = String::new();
        loop {
            let byte_readd = match buffer.read_line(&mut line).await {
                Ok(data) => data,
                Err(_) => 0,
            };
            if byte_readd == 0 {
                break;
            }
            let response = self.state_machine.command_handler(&line)?;
            println!("{}",std::str::from_utf8(response).unwrap());
            if response != StateMachine::HOLD_YOUR_HORSES {
                writer.write_all(response)
                .await.ok();
            }
            // if response == StateMachine::KTHXBYE{
            //     self.socket.write_all(StateMachine::KTHXBYE).await.ok();
            //     break;
            // }
            line.clear();
        }
        Ok(())
    }
    async fn connection(&mut self) -> Result<()> {
        self.socket
            .write_all(b"220 smtp.pgmail.com\r\n")
            .await
            .map_err(|e| e.into())
    }
}
