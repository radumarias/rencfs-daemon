use std::backtrace::Backtrace;
use std::error::Error;
use std::ffi::OsStr;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use anyhow::Context;

use fuse3::MountOptions;
use fuse3::raw::Session;
use rencfs::encryptedfs::{Cipher, FsError};
use rencfs::encryptedfs_fuse3::EncryptedFsFuse3;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use tracing::{error, info, instrument};

use crate::daemon::daemon_service_server::{DaemonService, DaemonServiceServer};
use crate::daemon::DaemonError::Start;
use crate::vault_handler;
use crate::vault_handler::{Vault, VaultHandler, VaultHandlerError, VaultHandlerService};

tonic::include_proto!("rencfs_daemon");

#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("error starting daemon: {source}")]
    Start { #[from] source: tonic::transport::Error, backtrace: Backtrace },
    #[error("vault handler error: {source}")]
    VaultHandler { #[from] source: VaultHandlerError, backtrace: Backtrace },
    #[error("generic error: {msg}")]
    Generic { msg: String, backtrace: Backtrace },
}

pub struct Daemon {}

impl Daemon {
    pub fn new() -> Self {
        Self {}
    }

    #[instrument(skip(self))]
    pub async fn start(&self, addr: SocketAddr, config_file: PathBuf) -> Result<(), DaemonError> {
        info!("Starting server");
        let service = MyDaemonService::new(config_file).await?;

        info!("Listening on {}", addr);
        Server::builder()
            .add_service(DaemonServiceServer::new(service))
            .serve(addr)
            .await?;

        Ok(())
    }
}

pub(crate) struct MyDaemonService {
    handler_service: Mutex<VaultHandlerService>,
}

impl MyDaemonService {
    pub(crate) async fn new(config_file: PathBuf) -> Result<Self, DaemonError> {
        Ok(Self {
            handler_service: Mutex::new(VaultHandlerService::new(config_file).await?)
        })
    }

    async fn handle_handler_empty_response(response: Result<(), VaultHandlerError>) -> Result<Response<EmptyReply>, Status> {
        match response {
            Ok(_) => Ok(Response::new(EmptyReply {})),
            Err(err) => {
                // Err(DaemonServiceError::from(err).into())
                Err(DaemonServiceError::VaultHandlerError.into())
            }
        }
    }
}

#[tonic::async_trait]
impl DaemonService for MyDaemonService {
    async fn echo(&self, request: Request<EmptyRequest>) -> Result<Response<EmptyReply>, Status> {
        Ok(Response::new(EmptyReply {}))
    }

    #[instrument(skip(self), err)]
    async fn lock(&self, request: Request<IdRequest>) -> Result<Response<EmptyReply>, Status> {
        let id = request.into_inner().id;
        info!(id, "Vault lock request received");

        let mut handler = self.handler_service.lock().await;
        MyDaemonService::handle_handler_empty_response(handler.lock(id).await).await
    }

    #[instrument(skip(self), err)]
    async fn unlock(&self, request: Request<IdRequest>) -> Result<Response<EmptyReply>, Status> {
        let id = request.into_inner().id;
        info!(id, "Vault unlock request received");

        let mut handler = self.handler_service.lock().await;
        MyDaemonService::handle_handler_empty_response(handler.unlock(id).await).await
    }

    #[instrument(skip(self), err)]
    async fn change_mount_point(&self, request: Request<StringsIdRequest>) -> Result<Response<EmptyReply>, Status> {
        let request = request.into_inner();
        let id = request.id;
        if request.value.len() != 2 {
            return Err(Status::invalid_argument("Invalid number of arguments"));
        }
        info!(id, old = request.value.get(0), new = request.value.get(0), "Vault change mount point request received");

        let mut handler = self.handler_service.lock().await;
        MyDaemonService::handle_handler_empty_response(handler.change_mount_point(id, request.value.get(0).unwrap().clone(), request.value.get(1).unwrap().clone()).await).await
    }

    #[instrument(skip(self), err)]
    async fn change_data_dir(&self, request: Request<StringsIdRequest>) -> Result<Response<EmptyReply>, Status> {
        let request = request.into_inner();
        let id = request.id;
        if request.value.len() != 2 {
            return Err(Status::invalid_argument("Invalid number of arguments"));
        }
        info!(id, old = request.value.get(0), new = request.value.get(0), "Vault change data dir request received");

        let mut handler = self.handler_service.lock().await;
        MyDaemonService::handle_handler_empty_response(handler.change_data_dir(id, request.value.get(0).unwrap().clone(), request.value.get(1).unwrap().clone()).await).await
    }

    #[instrument(skip(self), err)]
    async fn remove(&self, request: Request<IdRequest>) -> Result<Response<EmptyReply>, Status> {
        let request = request.into_inner();
        let id = request.id;
        info!(id, "Vault delete request received");

        let mut handler = self.handler_service.lock().await;
        MyDaemonService::handle_handler_empty_response(handler.delete(id).await).await
    }

    #[instrument(skip(self), err)]
    async fn insert(&self, request: Request<InsertRequest>) -> Result<Response<EmptyReply>, Status> {
        let request = request.into_inner();
        info!("Vault insert request received");

        let mut handler = self.handler_service.lock().await;
        let cipher = Cipher::from_str(&request.cipher);
        if cipher.is_err() {
            return Err(Status::invalid_argument("Invalid cipher"));
        }
        let cipher = cipher.unwrap();
        let vault = Vault {
            name: request.name.clone(),
            mount_point: request.mount_point.clone(),
            data_dir: request.data_dir.clone(),
            locked: true,
            cipher,
            derive_key_hash_rounds: request.derive_key_hash_rounds,
        };
        MyDaemonService::handle_handler_empty_response(handler.insert(vault).await).await
    }
}

#[derive(Debug, Error, Serialize, Deserialize, Clone)]
pub enum DaemonServiceError {
    #[error("vault handler error:")]
    VaultHandlerError,
    // #[error("{0}")]
    // VaultHandlerError(#[from] VaultHandlerError),
}

static CUSTOM_ERROR: &str = "x-custom-tonic-error-vault_service_error";

impl TryFrom<Status> for DaemonServiceError {
    type Error = ();

    fn try_from(status: Status) -> Result<Self, Self::Error> {
        match status.code() {
            tonic::Code::Internal => {
                if let Some(err) = status.metadata().get(CUSTOM_ERROR) {
                    Ok(serde_json::from_str(err.to_str().unwrap()).unwrap())
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}

impl From<DaemonServiceError> for Status {
    fn from(e: DaemonServiceError) -> Self {
        let mut status = Status::internal(format!("internal error: {}", e));

        status.metadata_mut().insert(CUSTOM_ERROR,
                                     serde_json::to_string(&e)
                                         .unwrap_or("could not serialize: {e}".to_string())
                                         .parse()
                                         .unwrap_or(tonic::metadata::MetadataValue::from_static("unable to create metadata value")));
        status
    }
}
