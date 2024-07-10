use tokio_postgres::{ NoTls, Socket };
use tokio_postgres::tls::{ NoTlsStream };

pub async fn start_connection(
    postgres_username: &str,
    postgres_ip: &str,
    postgres_password: &str,
    postgres_database_name: &str,
    postgres_port: &str
) -> Result<
    (tokio_postgres::Client, tokio_postgres::Connection<Socket, NoTlsStream>),
    tokio_postgres::Error
> {
    //Postgres address ⬇️
    let s: String = format!(
        "postgres://{}:{}@{}:{}/{}",
        postgres_username,
        postgres_password,
        postgres_ip,
        postgres_port,
        postgres_database_name
    );

    let (client, connection) = tokio_postgres::connect(&s[..], NoTls).await?;

    Ok((client, connection))
}
// pub fn read();
