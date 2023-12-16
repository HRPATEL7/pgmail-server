use pgmail_server::TlsAcceptor;
use tokio::fs;

pub async fn create_tls_acceptor() -> Result<TlsAcceptor,native_tls::Error> {
    let contents = fs::read("E:/edgex/pgmail-server/src/identity.p12").await;
    let identity = native_tls::Identity::from_pkcs12(&contents.unwrap(), "mypass")?;
    // Configure the TLS acceptor
    let tls_acceptor = TlsAcceptor::from(native_tls::TlsAcceptor::builder(identity).build()?);
    Ok(tls_acceptor)
}