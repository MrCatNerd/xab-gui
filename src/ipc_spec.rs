use anyhow::{Result, ensure};
use bitflags::bitflags;
use bytes::Bytes;

pub const IPC_PROTO_VERSION: i32 = 1;
pub const IPC_PATH: &str = "/tmp/xab/xab_uds";

#[derive(Copy, Clone, Default)]
pub enum IpcCommands {
    // stuff
    #[default]
    NoneInvalid = -1,
    None = 0,

    // ipc stuff
    Restart = 1,
    XabShutdown = 2,
    ClientDisconnect = 3,

    // change stuff
    ChangeBackgrounds = 4,
    PauseVideos = 6,
    UnpauseVideos = 7,
    TogglePauseVideos = 8,

    // get stuff
    GetMonitors = 9,
    GetCapabilites = 10,
}

// im too lazy to implement monitor names (coming soon TM)
#[derive(Default, Debug, Clone, Copy)]
pub struct Monitor {
    pub index: i32,
    pub primary: bool,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Monitor {
    pub fn fullscreen() -> Self {
        Self {
            index: 0,
            primary: true,
            x: 0,
            y: 0,
            // width and height of 0 means fullscreen
            width: 0,
            height: 0,
        }
    }
    pub fn from_bytes(bytes: &Bytes) -> Result<Self> {
        ensure!(bytes.len() >= 21, "Not enough bytes to read Monitor");
        Ok(Self {
            index: i32::from_be_bytes(bytes[0..4].try_into()?),
            primary: bytes[4] != 0,
            x: u32::from_be_bytes(bytes[5..9].try_into()?),
            y: u32::from_be_bytes(bytes[9..13].try_into()?),
            width: u32::from_be_bytes(bytes[13..17].try_into()?),
            height: u32::from_be_bytes(bytes[17..21].try_into()?),
        })
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct IpcXabCapabilities: u32 {
        const None = 0;
        const Multimonitor = 1 << 0;
    }
}

impl Default for IpcXabCapabilities {
    fn default() -> Self {
        Self::None
    }
}
