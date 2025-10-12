use anyhow::{Context, Result, anyhow};
use bytes::{Bytes, BytesMut};
use iced::futures::lock::{Mutex, MutexGuard};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};
use tracing::{debug, error};

use crate::ipc_spec::*;

#[derive(Debug)]
pub struct IpcHandle {
    pub path: String,
    socket: Mutex<UnixStream>,
    pub capabilities: IpcXabCapabilities,
}

impl IpcHandle {
    pub async fn new(path: &str) -> Result<Self> {
        debug!("Initializing Unix Domain Socket");
        let mut socket = UnixStream::connect(path)
            .with_context(|| format!("Failed to connect to socket at {path}"))?;

        // get xab IPC protocol version from server
        let mut buf = [0u8; std::mem::size_of::<i32>()]; // rust is so weird 0_0
        socket
            .read_exact(&mut buf)
            .with_context(|| "Failed to read IPC protocol version")?;

        // version from buf - uses native-endianness
        let version: i32 = i32::from_be_bytes(buf);
        debug!("Server IPC version: {version}");

        // send version back
        buf = IPC_PROTO_VERSION.to_be_bytes();
        socket
            .write_all(&buf)
            .with_context(|| "Failed to send IPC protocol version")?;

        // if version is mismatched - disconnect
        if version != IPC_PROTO_VERSION {
            error!(
                "Mismatch between client and server xab IPC protocol version! (server: {} | client| {})",
                version, IPC_PROTO_VERSION
            );
            socket.shutdown(std::net::Shutdown::Both)?;
            return Err(anyhow!(
                "Mismatch between client and server xab IPC protocol version! (server: {} | client: {})",
                version,
                IPC_PROTO_VERSION
            ));
        } else {
            debug!("Server and Client xab IPC protocol versions match!");
        }

        // read capabilities
        debug!("Getting XAB capabilities");
        socket
            .read_exact(&mut buf)
            .with_context(|| "Failed to read XAB capabilities")?;
        let capabilities = IpcXabCapabilities::from_bits_truncate(u32::from_be_bytes(buf));
        debug!(
            "capabilities: {:?} {:b}",
            capabilities,
            u32::from_be_bytes(buf)
        );

        Ok(Self {
            path: path.to_owned(),
            socket: Mutex::from(socket),
            capabilities,
        })
    }

    /// NOTE: try not deadlocking yourself - by using the guard argument
    pub async fn send_commands<'a>(
        &'a self,
        commands: u32,
        guard: Option<MutexGuard<'a, UnixStream>>,
    ) -> Result<MutexGuard<'a, UnixStream>> {
        let mut socket = if let Some(guard) = guard {
            guard
        } else {
            self.socket.lock().await
        };
        socket.write_all(&commands.to_be_bytes())?;
        Ok(socket)
    }

    /// NOTE: try not deadlocking yourself
    pub async fn send_recv_command(&self, command: IpcCommands) -> Result<Option<Bytes>> {
        // TODO: guard thingy like i did with send_commands
        let mut socket = self.socket.lock().await;
        socket.write_all(&(command as i32).to_be_bytes())?;

        let mut demz_bytes = BytesMut::new();
        socket.read_exact(&mut demz_bytes)?;
        let demz_bytes: Bytes = demz_bytes.freeze();
        Ok(match !demz_bytes.is_empty() {
            true => Some(demz_bytes),
            false => None,
        })
    }

    pub async fn close(&self) -> Result<()> {
        debug!("Closing connection: {}", self.path);

        let socket = self.socket.lock().await;
        let socket = self
            .send_commands(IpcCommands::ClientDisconnect as u32, Some(socket))
            .await?;
        socket.shutdown(std::net::Shutdown::Both)?;
        Ok(())
    }

    pub async fn get_monitors(&self) -> Vec<Monitor> {
        // if xab isn't capable then return fullscreen
        if self.capabilities.contains(IpcXabCapabilities::Multimonitor) {
            let monitors_bytes = self
                .send_recv_command(IpcCommands::GetMonitors)
                .await
                .unwrap()
                .unwrap();

            // NOTE: remember to set step size and the other stuff
            // to the same size at Monitor::from_bytes
            return (0..monitors_bytes.len())
                .step_by(21)
                .filter_map(|i| {
                    if i + 21 <= monitors_bytes.len() {
                        return Monitor::from_bytes(&monitors_bytes.slice(i..i + 21)).ok();
                    }
                    None
                })
                .collect();
        }
        vec![Monitor::fullscreen()]
    }
}
