use std::backtrace::Backtrace;
use std::panic::catch_unwind;
use std::str::FromStr;

use anyhow::Result;
use keyring::Entry;
use rencfs::encryptedfs::Cipher;
use tokio::task;
use tracing::{error, info, instrument, Level};

use rencfs_daemon::daemon::Daemon;
use rencfs_daemon::log_init;
use rencfs_daemon::storage::get_config_dir;
use rencfs_daemon::vault_handler::{Config, Vault};

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    // todo: take from configs
    let level = Level::from_str("TRACE").unwrap();
    let guard = log_init(level);

    info!("Starting daemon");

    let res = task::spawn_blocking(|| {
        catch_unwind(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                daemon_run_async().await
            })
        })
    }).await;
    match res {
        Ok(Ok(Ok(_))) => Ok(()),
        Ok(Ok(Err(err))) => {
            error!("{err}");
            drop(guard);
            Err(err)
        }
        Ok(Err(err)) => {
            error!("{err:#?}");
            drop(guard);
            std::panic!("{err:#?}");
        }
        Err(err) => {
            error!("{err}");
            drop(guard);
            std::panic!("{err}");
        }
    }
}

#[instrument]
async fn daemon_run_async() -> Result<()> {
    let daemon = Daemon::new();
    let addr = "[::1]:50051".parse()?;
    daemon.start(addr, get_config_dir().join("config.yaml")).await?;
    Ok(())
}