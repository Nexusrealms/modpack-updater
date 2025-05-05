use std::{io::Write, net::TcpStream, path::Path, sync::Arc};

use russh::{client, keys::ssh_key, ChannelId};
use tokio::{io::AsyncWriteExt, runtime::Runtime};
use russh_sftp::client::SftpSession;
use crate::{generate, mrpack, FtpLocation, PackSource};
struct Client;

impl client::Handler for Client {
    async fn check_server_key(
        &mut self,
        server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        println!("check_server_key: {:?}", server_public_key);
        Ok(true)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        println!("data on channel {:?}: {}", channel, data.len());
        Ok(())
    }

    type Error = IdkEroor;
}
pub fn run_over_sftp(location: FtpLocation, source: PackSource) -> Result<(), &'static str> {
    let rt = Runtime::new().unwrap();
    println!("Started run");
    rt.block_on(async move {
        println!("Started tracing");
        let config = russh::client::Config::default();
        let sh = Client {};
        let mut session = russh::client::connect(
            Arc::new(config),
            (location.address.as_str(), location.port as u16),
            sh,
        )
        .await
        .unwrap();
        if session
            .authenticate_password(location.name, location.password)
            .await
            .unwrap()
            .success()
        {
            let channel = session.channel_open_session().await.unwrap();
            channel.request_subsystem(true, "sftp").await.unwrap();
            let sftp = SftpSession::new(channel.into_stream()).await.unwrap();
            println!("current path: {:?}", sftp.canonicalize(".").await.unwrap());
            sftp.create("test.txt").await.unwrap().write("hello".as_bytes()).await.unwrap();

        } else {
            println!("Something happened idk");
        }
    });
    Ok(())
}
#[derive(Debug)]
struct IdkEroor;
impl From<russh::Error> for IdkEroor {
    fn from(_: russh::Error) -> Self {
        IdkEroor
    }
}
