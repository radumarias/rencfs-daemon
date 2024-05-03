use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use fuse3::MountOptions;
use fuse3::raw::Session;
use rencfs::encryptedfs::{Cipher, FsError, FsResult};
use rencfs::encryptedfs_fuse3::EncryptedFsFuse3;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{error, info, instrument};

#[derive(Debug, Error)]
pub enum VaultHandlerError {
    #[error("cannot lock vault")]
    CannotLockVault,
    #[error("cannot unlock vault")]
    CannotUnlockVault,
    #[error("cannot change mount point")]
    CannotChangeMountPoint,
    #[error("cannot change data dir")]
    CannotChangeDataDir,
    #[error("cannot find config file: {source}")]
    CannotFindConfigFile { #[from] source: io::Error, backtrace: Backtrace },
    #[error("cannot read config file: {source}")]
    CannotReadConfigFile { #[from] source: serde_yaml::Error, backtrace: Backtrace },
    #[error("cannot save config file")]
    CannotSaveConfigFile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    pub name: String,
    pub mount_point: String,
    pub data_dir: String,
    pub locked: bool,
    pub cipher: Cipher,
    pub derive_key_hash_rounds: u32,
}

pub struct VaultHandler {
    id: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub vaults: Vec<Vault>,
}

impl Config {
    pub fn new() -> Self {
        Self { vaults: Vec::new() }
    }
}

impl VaultHandler {
    pub fn new(id: String) -> Self {
        Self { id }
    }

    #[instrument(skip(self), fields(self.id))]
    pub async fn lock(&mut self, mount_point: Option<String>) -> Result<(), VaultHandlerError> {
        info!("");

        Ok(())
    }

    #[instrument(skip(self), fields(self.id))]
    pub async fn unlock(&mut self) -> Result<(), VaultHandlerError> {
        info!("");

        Ok(())
    }

    #[instrument(skip(self), fields(self.id))]
    pub async fn change_mount_point(&mut self, old_mount_point: String) -> Result<(), VaultHandlerError> {
        info!("");

        Ok(())
    }

    #[instrument(skip(self), fields(self.id))]
    pub async fn change_data_dir(&mut self, old_data_dir: String) -> Result<(), VaultHandlerError> {
        info!("");

        Ok(())
    }
}

pub struct VaultHandlerService {
    handlers: Arc<Mutex<HashMap<String, VaultHandler>>>,
    config_file: PathBuf,
    config: Config,
}

impl VaultHandlerService {
    pub async fn new(config_file: PathBuf) -> Result<Self, VaultHandlerError> {
        Ok(Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            config_file: config_file.clone(),
            config: Self::read_config(&config_file)?,
        })
    }

    fn read_config(config_file: &PathBuf) -> Result<Config, VaultHandlerError> {
        let config: Config = serde_yaml::from_reader(File::open(&config_file)?)?;
        Ok(config)
    }

    fn save_config(&self) -> Result<(), VaultHandlerError> {
        serde_yaml::to_writer(File::create(&self.config_file).map_err(|_| VaultHandlerError::CannotSaveConfigFile)?, &self.config)
            .map_err(|_| VaultHandlerError::CannotSaveConfigFile)
    }

    #[instrument(skip(self), fields(self.id))]
    pub async fn lock(&mut self, id: String) -> Result<(), VaultHandlerError> {
        info!("");

        let mut handlers = self.handlers.lock().await;
        let handler = handlers.entry(id.clone()).or_insert_with(|| VaultHandler::new(id));

        todo!();

        Ok(())
    }

    #[instrument(skip(self), fields(self.id))]
    pub async fn unlock(&mut self, id: String) -> Result<(), VaultHandlerError> {
        info!("");

        let mut handlers = self.handlers.lock().await;
        let handler = handlers.entry(id.clone()).or_insert_with(|| VaultHandler::new(id));

        todo!();

        Ok(())
    }

    #[instrument(skip(self), fields(self.id))]
    pub async fn change_mount_point(&mut self, id: String, old_mount_point: String, new_mount_point: String) -> Result<(), VaultHandlerError> {
        info!("");

        let mut handlers = self.handlers.lock().await;
        let handler = handlers.entry(id.clone()).or_insert_with(|| VaultHandler::new(id));

        todo!();

        Ok(())
    }

    #[instrument(skip(self), fields(self.id))]
    pub async fn change_data_dir(&mut self, id: String, old_data_dir: String, new_data_dir: String) -> Result<(), VaultHandlerError> {
        info!("");

        let mut handlers = self.handlers.lock().await;
        let handler = handlers.entry(id.clone()).or_insert_with(|| VaultHandler::new(id));

        todo!();

        Ok(())
    }

    #[instrument(skip(self), fields(self.id))]
    pub async fn delete(&mut self, id: String) -> Result<(), VaultHandlerError> {
        info!("");

        let mut handlers = self.handlers.lock().await;
        let handler = handlers.entry(id.clone()).or_insert_with(|| VaultHandler::new(id));

        todo!();

        Ok(())
    }

    #[instrument(skip(self), fields(self.id))]
    pub async fn insert(&mut self, vault: Vault) -> Result<(), VaultHandlerError> {
        info!("");

        // let mut handlers = self.handlers.lock().await;
        // let handler = handlers.entry(id.clone()).or_insert_with(|| VaultHandler::new(id));

        todo!();

        Ok(())
    }
}

#[instrument]
pub async fn run_fuse(mountpoint: &str, data_dir: &str, password: &str, cipher: Cipher, derive_key_hash_rounds: u32,
                      allow_root: bool, allow_other: bool, direct_io: bool, suid_support: bool) -> FsResult<()> {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };

    let mount_options = MountOptions::default()
        .uid(uid)
        .gid(gid)
        .read_only(false).
        allow_root(allow_root).
        allow_other(allow_other)
        .clone();
    let mount_path = OsStr::new(mountpoint);

    info!("Mounting FUSE filesystem");
    match EncryptedFsFuse3::new(&data_dir, &password, cipher, derive_key_hash_rounds, direct_io, suid_support) {
        Err(err) => {
            error!("{err}");
            Err(err)
        }
        Ok(fs) => {
            let handle = Session::new(mount_options)
                .mount_with_unprivileged(fs, mount_path)
                .await?
                .await?;

            Ok(())
        }
    }
}
