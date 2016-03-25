//! Read and write Elma replay files.
use std::io::{ Cursor, Read, Write };
use std::fs::File;
use std::ffi::CString;
use byteorder::{ ReadBytesExt, WriteBytesExt, LittleEndian };
use super::{ cstring_read, Position, read_n };

// Magic arbitrary number to signify end of replay file.
const EOR: u32 = 0x00492F75;

/// One frame of replay.
#[derive(Debug, PartialEq)]
pub struct Frame {
    /// Bike position?
    pub bike: Position<f32>,
    /// Left wheel position.
    pub left_wheel: Position<i16>,
    /// Right wheel position.
    pub right_wheel: Position<i16>,
    /// Head position.
    pub head: Position<i16>,
    /// Bike rotation. Range 0..10000.
    pub rotation: i16,
    /// Left wheel rotation. Range 0..255.
    pub left_wheel_rotation: u8,
    /// Right wheel rotation. Range 0..255.
    pub right_wheel_rotation: u8,
    /// Throttle.
    pub throttle: bool,
    /// Right direction. True = right, False = left.
    pub right: bool,
    /// Spring sound effect volume.
    pub volume: i16
}

#[derive(Debug, PartialEq)]
pub struct Event {
    /// Time of event.
    pub time: f64,
    /// Event type.
    pub event_type: [u32; 2]
}

/// Rec struct
pub struct Rec {
    /// Number of Frames in replay.
    pub frame_count: i32,
    /// Whether replay is multi-player or not.
    pub multi: bool,
    /// Whether replay is flag-tag or not.
    pub flag_tag: bool,
    /// Random number to link with level file.
    pub link: u32,
    /// Full level filename.
    pub level: CString,
    /// Vector with Frame structs.
    pub frames: Vec<Frame>,
    /// Events.
    pub events: Vec<Event>
}

impl Rec {
    /// Build a new Rec.
    ///
    /// # Examples
    ///
    /// ```
    /// let rec = elma::rec::Rec::new();
    /// ```
    pub fn new () -> Rec {
        Rec {
            frame_count: 0,
            multi: false,
            flag_tag: false,
            link: 0,
            level: CString::new("").unwrap(),
            frames: vec![],
            events: vec![]
        }
    }

    /// Loads a replay file and returns a Rec struct.
    ///
    /// # Examples
    ///
    /// ```
    /// let rec = elma::rec::Rec::load_replay("tests/test.rec");
    /// ```
    pub fn load_replay (filename: &str) -> Rec {
        Rec::new()
    }
}
