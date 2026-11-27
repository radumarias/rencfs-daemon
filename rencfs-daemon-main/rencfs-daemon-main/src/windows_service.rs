use std::ffi::OsString;
use std::net::SocketAddr;
use std::time::Duration;
use anyhow::Context;
use tokio::runtime::Builder;
use tokio::sync::watch;
use tracing::{error, info};
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{
    self, ServiceControlHandlerResult, ServiceStatusHandle,
};
use windows_service::service_dispatcher;
use crate::daemon::Daemon;
use crate::storage::get_user_config_file;

const SERVICE_NAME: &str = "rencfs-daemon";
const GRPC_ADDRESS: &str = "[::1]:50051";
const STARTUP_WAIT: Duration = Duration::from_secs(10);

windows_service::define_windows_service!(ffi_service_main, service_main);

pub fn run() -> windows_service::Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

fn service_main(_arguments: Vec<OsString>) {
    if let Err(error) = run_service() {
        error!(%error, "Windows service stopped with an error");
    }
}

fn run_service() -> anyhow::Result<()> {
    let (stop_sender, stop_receiver) = watch::channel(false);
    let status_handle =
        service_control_handler::register(SERVICE_NAME, move |control| match control {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                if let Err(error) = stop_sender.send(true) {
                    error!(%error, "Windows service stop receiver is unavailable");
                }
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        })
        .context("cannot register Windows service control handler")?;

    status_handle
        .set_service_status(service_status(ServiceState::StartPending, 1, STARTUP_WAIT))
        .context("cannot report Windows service startup")?;

    let result = run_service_runtime(&status_handle, stop_receiver);
    let exit_code = if result.is_ok() {
        ServiceExitCode::Win32(0)
    } else {
        ServiceExitCode::Win32(1)
    };
    let stop_result = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code,
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    });

    if let Err(error) = result {
        if let Err(status_error) = stop_result {
            error!(%status_error, "Cannot report Windows service stopped after failure");
        }
        return Err(error);
    }

    stop_result.context("cannot report Windows service stopped")?;
    Ok(())
}

fn run_service_runtime(
    status_handle: &ServiceStatusHandle,
    mut stop_receiver: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("cannot create Tokio runtime")?;
    let config_file = get_user_config_file()?;
    let address: SocketAddr = GRPC_ADDRESS.parse()?;

    runtime.block_on(async {
        let daemon = Daemon::new();
        let status_handle = status_handle;
        let shutdown = async move {
            match stop_receiver.changed().await {
                Ok(()) => {
                    if let Err(error) = status_handle.set_service_status(service_status(
                        ServiceState::StopPending,
                        1,
                        STARTUP_WAIT,
                    )) {
                        error!(%error, "Cannot report Windows service stop pending");
                    }
                }
                Err(error) => error!(%error, "Windows service stop channel closed"),
            }
        };

        daemon
            .start_with_shutdown(address, config_file, shutdown, || {
                status_handle
                    .set_service_status(service_status(
                        ServiceState::Running,
                        0,
                        Duration::default(),
                    ))
                    .context("cannot report Windows service running")?;
                info!("Windows service gRPC endpoint is ready");
                Ok(())
            })
            .await
            .context("Windows service gRPC server failed")
    })
}

fn service_status(
    current_state: ServiceState,
    checkpoint: u32,
    wait_hint: Duration,
) -> ServiceStatus {
    ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state,
        controls_accepted: if matches!(current_state, ServiceState::Running) {
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN
        } else {
            ServiceControlAccept::empty()
        },
        exit_code: ServiceExitCode::Win32(0),
        checkpoint,
        wait_hint,
        process_id: None,
    }
}
