pub mod smtp;
pub mod state;
mod structs;
mod tcphandler;



use tcphandler::create_tls_acceptor;
use tokio::net::TcpListener;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:25").await.unwrap();
    let tls_acceptor = create_tls_acceptor().await.expect("Error creating TLS acceptor");
    loop {
        let (socket, _add) = listener.accept().await.unwrap();
        let tls_acceptor = tls_acceptor.clone();
        print!("{}:{}\n", _add.ip(), _add.port());
        tokio::task::spawn(async move {

            let tls_stream = tls_acceptor.accept(socket).await.unwrap();
            let run = smtp::Server::new(tls_stream, "smtp.pgmail.com".to_string())
                .await
                .unwrap();
            run.serve().await.ok();
        });
    }
}