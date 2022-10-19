use libunftp::ServerError;
use std::path::PathBuf;
use unftp_sbe_fs::ServerExt;

pub async fn start_server(path: String) -> Result<(), ServerError> {
    let ftp_home = PathBuf::from(path);
    let server = libunftp::Server::with_fs(ftp_home)
        .greeting("Welcome to FTP server")
        .passive_ports(50000..65535);
    server.listen("0.0.0.0:2121").await?;

    Ok(())
}
