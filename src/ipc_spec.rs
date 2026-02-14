use anyhow::Result;
use bitflags::bitflags;
use bytes::Bytes;
use std::io::{Cursor, Read};

pub const IPC_PROTO_VERSION: i32 = 1;
pub const IPC_PATH: &str = "/tmp/xab/xab_uds";

#[repr(i32)]
#[derive(Copy, Clone, Default)]
pub enum IpcCommands {
    // stuff
    #[default]
    NoneInvalid = -1,
    None = 0,

    // set statte
    Restart = 1,
    Shutdown = 2,
    ClientDisconnect = 3,
    ChangeBackground = 4,
    DeleteBackground = 5,
    PauseVideo = 6,
    UnpauseVideo = 7,
    TogglePauseVideo = 8,

    // get state
    GetMonitors = 9,
    GetAllBackgrounds = 10,
    GetCapabilites = 11,
}

// im too lazy to implement monitor names (coming soon TM)
#[repr(C)]
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
        let mut cursor = Cursor::new(&bytes[..]);
        let mut buf4 = [0u8; 4];
        let mut buf1 = [0u8; 1];

        cursor.read_exact(&mut buf4)?;
        let index = i32::from_be_bytes(buf4);

        cursor.read_exact(&mut buf1)?;
        let primary = buf1[0] != 0;

        cursor.read_exact(&mut buf4)?;
        let x = u32::from_be_bytes(buf4);

        cursor.read_exact(&mut buf4)?;
        let y = u32::from_be_bytes(buf4);

        cursor.read_exact(&mut buf4)?;
        let width = u32::from_be_bytes(buf4);

        cursor.read_exact(&mut buf4)?;
        let height = u32::from_be_bytes(buf4);

        Ok(Self {
            index,
            primary,
            x,
            y,
            width,
            height,
        })
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct IpcXabCapabilities: u32 {
        const None = 0;
        const CustomPositioning = 1 << 0;
        const Monitors = 1 << 1;
    }
}

impl Default for IpcXabCapabilities {
    fn default() -> Self {
        Self::None
    }
}
