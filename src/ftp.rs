use std::sync::Arc;

use russh::{client, keys::ssh_key, ChannelId};
use tokio::runtime::Runtime;
use russh_sftp::client::SftpSession;
use crate::{config::{delete_by_config_remote, load_config_remote, write_config_remote}, generate::generate_at_remote, mrpack::update_from_mrpack_to_remote, FtpLocation, PackSource};
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
    rt.block_on(async move {
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
            let mut sftp = SftpSession::new(channel.into_stream()).await.unwrap();
            match source {
                PackSource::None => Err("No pack source set!"),
                _ => {
                    if let Ok(config) = load_config_remote(&mut sftp).await {
                        match delete_by_config_remote(&mut sftp, &config).await {
                            Ok(_) => {}
                            Err(err) => {
                                return Err(err);
                            }
                        };
                    };
                    match update_from_mrpack_to_remote(&source, &mut sftp).await {
                        Ok(config) => write_config_remote(&mut sftp, &config).await,
                        Err(str) => Err(str),
                    }
                }
            }
        } else {
            Err("ASYNCWRONG")
        }
    })
}
pub fn generate_over_sftp(location: FtpLocation) -> Result<(), &'static str> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async move {
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
            let mut sftp = SftpSession::new(channel.into_stream()).await.unwrap();
            generate_at_remote(&mut sftp).await
        } else {
            Err("Setting up SFTP session did not succeed")
        }
    })
}
#[derive(Debug)]
struct IdkEroor;
impl From<russh::Error> for IdkEroor {
    fn from(_: russh::Error) -> Self {
        IdkEroor
    }
}
