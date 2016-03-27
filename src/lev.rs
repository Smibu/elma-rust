//! Read and write Elma level files.

use std::io::{ Cursor, Read, Write };
use std::fs::File;
use std::ffi::CString;
use byteorder::{ ByteOrder, ReadBytesExt, WriteBytesExt, LittleEndian };
use super::{ cstring_read, Position, read_n };
use super::rand::Rng;

// Magic arbitrary number; signifies end-of-data. Followed by Top10 list(s).
const EOD: i32 = 0x0067103A;
// Magic arbitrary number; signifies end-of-file.
const EOF: i32 = 0x00845D52;

/// Type of object.
pub enum ObjectType {
    Apple,
    Exit,
    Killer,
    Player
}

/// Object struct. Every level requires one ObjectType::Player Object and at least one ObjectType::Exit Object.
pub struct Object {
    /// Position. See Position struct.
    pub position: Position<f64>,
    /// Type of Object, see ObjectType.
    pub object_type: ObjectType,
    /// Applies to ObjectType::Apple only.
    ///
    /// 0 = normal
    /// 1 = gravity up
    /// 2 = gravity down
    /// 3 = gravity left
    /// 4 = gravity right
    // TODO: enum with gravity
    pub gravity: i32,
    /// Applies to ObjectType::Apple only. Valid values are 1 to 9.
    pub animation: i32
}

/// Polygon struct.
#[derive(Debug, PartialEq)]
pub struct Polygon {
    /// Grass polygon.
    pub grass: bool,
    /// Vector with all vertices, see Position struct.
    pub vertices: Vec<Position<f64>>
}

impl Polygon {
    pub fn new () -> Polygon {
        Polygon {
            grass: false,
            vertices: vec![]
        }
    }
}

/// Picture struct.
pub struct Picture {
    /// Picture name.
    pub name: CString,
    /// Texture name.
    pub texture: CString,
    /// Mask name.
    pub mask: CString,
    /// Position. See Position struct.
    pub position: Position<f64>,
    /// Z-distance
    pub distance: i32,
    /// Clipping.
    ///
    /// 0 = unclipped
    /// 1 = ground
    /// 2 = sky
    // TODO: make enum
    pub clip: i32
}

/// Top10 list entry struct.
#[derive(Debug)]
pub struct ListEntry {
    /// Player 1 name.
    pub name_1: CString,
    /// Player 2 name.
    pub name_2: CString,
    /// Time.
    pub time: i32
}

/// Level struct that contains all level information.
pub struct Level {
    /// Elma or Across level.
    pub version: String,
    /// Raw binary data of a loaded or finalized constructed level.
    raw: Vec<u8>,
    /// Random number that links level file to replay files.
    pub link: i32,
    /// Contains four integrity checks (See create_integrity()).
    pub integrity: [f64; 4],
    /// Level name.
    pub name: CString,
    /// LGR file name.
    pub lgr: CString,
    /// Ground texture name.
    pub ground: CString,
    /// Sky texture name.
    pub sky: CString,
    /// Vector with all polygons (See Polygon).
    pub polygons: Vec<Polygon>,
    /// Vector with all objects (See Object).
    pub objects: Vec<Object>,
    /// Vector with all pictures (See Picture).
    pub pictures: Vec<Picture>,
    /// Vector of Top10 single-player names and times.
    pub top10_single: Vec<ListEntry>,
    /// Vector of Top10 multi-player names and times.
    pub top10_multi: Vec<ListEntry>
}

impl Level {
    /// Returns a new Level struct.
    ///
    /// # Examples
    ///
    /// ```
    /// let level = elma::lev::Level::new();
    /// ```
    pub fn new () -> Level {
        Level {
            version: "Elma".to_string(),
            raw: vec![],
            link: 0,
            integrity: [0.0f64; 4],
            name: CString::new("").unwrap(),
            lgr: CString::new("default").unwrap(),
            ground: CString::new("ground").unwrap(),
            sky: CString::new("sky").unwrap(),
            polygons: vec![],
            objects: vec![],
            pictures: vec![],
            top10_single: vec![],
            top10_multi: vec![]
        }
    }

    /// Loads a level file and returns a Level struct.
    ///
    /// # Examples
    ///
    /// ```
    /// let level = elma::lev::Level::load_level("tests/test.lev");
    /// ```
    pub fn load_level (filename: &str) -> Level {
        let mut level = Level::new();
        let mut file = File::open(filename).unwrap();
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).unwrap();
        level.raw = buffer;
        level.parse_level();
        level
    }

    /// Parses the raw binary data into Level struct fields.
    fn parse_level (&mut self) {
        let mut buffer = Cursor::new(&self.raw);
        let mut _data :Vec<u8>;

        // Elma = POT14, Across = POT06.
        // TODO: make Across compatible in 2025.
        let version = read_n(&mut buffer, 5);
        self.version = match version.as_slice() {
            [80, 79, 84, 49, 52] => "Elma".to_string(),
            [80, 79, 84, 48, 54] => "Across".to_string(),
            _ => panic!("Not a valid level file.")
        };

        // Link.
        _data = read_n(&mut buffer, 2); // Never used
        self.link = buffer.read_i32::<LittleEndian>().unwrap();

        // Integrity checksums.
        for i in 0..4 {
            self.integrity[i] = buffer.read_f64::<LittleEndian>().unwrap();
        }

        // Level name.
        self.name = cstring_read(read_n(&mut buffer, 51));
        // LGR name.
        self.lgr = cstring_read(read_n(&mut buffer, 16));
        // Ground texture name.
        self.ground = cstring_read(read_n(&mut buffer, 10));
        // Sky texture name.
        self.sky = cstring_read(read_n(&mut buffer, 10));

        // Polygons.
        let poly_count = (buffer.read_f64::<LittleEndian>().unwrap() - 0.4643643).round() as u16;
        for _ in 0..poly_count {
            let grass = buffer.read_i32::<LittleEndian>().unwrap() > 0;
            let vertex_count = buffer.read_i32::<LittleEndian>().unwrap();
            let mut vertices: Vec<Position<f64>> = vec![];
            for _ in 0..vertex_count {
                let x = buffer.read_f64::<LittleEndian>().unwrap();
                let y = buffer.read_f64::<LittleEndian>().unwrap();
                vertices.push(Position {
                    x: x,
                    y: y
                });
            }
            self.polygons.push(Polygon {
                grass: grass,
                vertices: vertices
            });
        }

        // Objects.
        let object_count = (buffer.read_f64::<LittleEndian>().unwrap() - 0.4643643).round() as u16;
        for _ in 0..object_count {
            let x = buffer.read_f64::<LittleEndian>().unwrap();
            let y = buffer.read_f64::<LittleEndian>().unwrap();
            let position = Position { x: x, y: y };
            let object_type = match buffer.read_i32::<LittleEndian>().unwrap() {
                1 => ObjectType::Exit,
                2 => ObjectType::Apple,
                3 => ObjectType::Killer,
                4 => ObjectType::Player,
                _ => panic!("Not a valid object type")
            };
            let gravity = buffer.read_i32::<LittleEndian>().unwrap();
            let animation = buffer.read_i32::<LittleEndian>().unwrap() + 1;

            self.objects.push(Object {
                position: position,
                object_type: object_type,
                gravity: gravity,
                animation: animation
            });
        }

        // Pictures.
        let picture_count = (buffer.read_f64::<LittleEndian>().unwrap() - 0.2345672).round() as u16;
        for _ in 0..picture_count {
            let name = cstring_read(read_n(&mut buffer, 10));
            let texture = cstring_read(read_n(&mut buffer, 10));
            let mask = cstring_read(read_n(&mut buffer, 10));
            let x = buffer.read_f64::<LittleEndian>().unwrap();
            let y = buffer.read_f64::<LittleEndian>().unwrap();
            let distance = buffer.read_i32::<LittleEndian>().unwrap();
            let clip = buffer.read_i32::<LittleEndian>().unwrap();

            self.pictures.push(Picture {
                name: name,
                texture: texture,
                mask: mask,
                position: Position { x: x, y: y },
                distance: distance,
                clip: clip
            });
        }

        // EOD marker expected at this point.
        let expected = buffer.read_i32::<LittleEndian>().unwrap();
        if expected != EOD { panic!("EOD marker mismatch: x0{:x} != x0{:x}", expected, EOD); }

        // First decrypt the top10 blocks.
        let decrypted_top10_data = crypt_top10(read_n(&mut buffer, 688));

        // Single-player list.
        let single = &decrypted_top10_data[0..344];
        let times = LittleEndian::read_i32(&single[0..4]);
        for n in 0..times {
            let time_offset: usize = (4 + n * 4) as usize;
            let time_end: usize = time_offset + 4;
            let name_offset: usize = (44 + n * 15) as usize;
            let name_end: usize = name_offset + 15;
            // All of this pains me even though I don't understand it...
            let mut name = Vec::new();
            name.extend_from_slice(&single[name_offset..name_end]);
            self.top10_single.push(ListEntry {
                time: LittleEndian::read_i32(&single[time_offset..time_end]),
                name_1: cstring_read(name),
                name_2: CString::new("").unwrap()
            });
        }

        // Multi-player list.
        let multi = &decrypted_top10_data[344..688];
        let times = LittleEndian::read_i32(&multi[0..4]);
        for n in 0..times {
            let time_offset: usize = (4 + n * 4) as usize;
            let time_end: usize = time_offset + 4;
            let name_offset: usize = (44 + n * 15) as usize;
            let name_end: usize = name_offset + 15;
            let name2_offset: usize = (194 + n * 15) as usize;
            let name2_end: usize = name2_offset + 15;
            // All of this pains me even though I don't understand it...
            let mut name = Vec::new();
            let mut name2 = Vec::new();
            name.extend_from_slice(&multi[name_offset..name_end]);
            name2.extend_from_slice(&multi[name2_offset..name2_end]);
            self.top10_multi.push(ListEntry {
                time: LittleEndian::read_i32(&multi[time_offset..time_end]),
                name_1: cstring_read(name),
                name_2: cstring_read(name2)
            });
        }

        // EOF marker expected at this point.
        let expected = buffer.read_i32::<LittleEndian>().unwrap();
        if expected != EOF { panic!("EOF marker mismatch: x0{:x} != x0{:x}", expected, EOF); }
    }

    /// Combines the Level struct fields to generate the raw binary data.
    fn convert_to_raw (&self) {
        // TODO: convert
    }

    /// Converts all struct fields into raw binary form and returns it.
    pub fn get_raw (self) -> Vec<u8> {
        self.convert_to_raw();
        self.raw
    }

    /// Saves level as a file.
    pub fn save_lev (self, filename: &str) {
        self.convert_to_raw();
        let mut file = File::create(&filename).unwrap();
        // TODO: write stuff.
    }
}

/// Decrypt and encrypt top10 list data. Same algorithm for both.
pub fn crypt_top10 (mut top10: Vec<u8>) -> Vec<u8> {
    // Who knows
    let mut ebp8: i16 = 0x15;
    let mut ebp10: i16 = 0x2637;

    for n in 0..688 {
        top10[n] ^= (ebp8 & 0xFF) as u8;
        ebp10 = ebp10.wrapping_add((ebp8.wrapping_rem(0xD3D)).wrapping_mul(0xD3D));
        ebp8 = ebp10.wrapping_mul(0x1F).wrapping_add(0xD3D);
    }

    top10
}
