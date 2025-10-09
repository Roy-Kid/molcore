use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

use polars::prelude::*;

use crate::core::Frame;
use crate::core::frame::MapLike;


pub fn open_txt(path: &str) -> std::io::Result<BufReader<File>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file))
}

pub fn open_gz(path: &str) -> std::io::Result<BufReader<flate2::read::GzDecoder<File>>> {
    let file = File::open(path)?;
    let decoder = flate2::read::GzDecoder::new(file);
    Ok(BufReader::new(decoder))
}

pub trait Reader {
    type R: BufRead;
    fn new(reader: Self::R) -> Self;
}

pub trait FrameReader: Reader {
    fn read_frame(&mut self) -> std::io::Result<Option<Frame>>;
}

pub trait TrajectoryReader: Reader
where
    Self::R: Seek,
{
    fn build_index(&mut self) -> std::io::Result<BTreeMap<usize, u64>>;
    /// Read a frame at a given byte offset (from build_index)
    fn read_step(&mut self, offset: usize) -> std::io::Result<Option<Frame>>;
}

// ForcefieldReader

pub struct XYZFrameReader<R: BufRead> {
    reader: R,
}

pub fn read_xyz_frame<R: BufRead>(reader: &mut R) -> std::io::Result<Option<Frame>> {

    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None); // EOF
        }
        if !line.trim().is_empty() {
            break;
        }
    }
    let natoms: usize = line.trim().parse().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid natoms: {e}"))
    })?;

    line.clear();
    reader.read_line(&mut line)?;
    let comment = line.trim_end_matches(['\n', '\r']).to_string();

    let mut element = Vec::with_capacity(natoms);
    let mut x = Vec::with_capacity(natoms);
    let mut y = Vec::with_capacity(natoms);
    let mut z = Vec::with_capacity(natoms);


    for _ in 0..natoms {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF while reading XYZ atoms",
            ));
        }
        let mut it = line.split_whitespace();
        let el = it.next().ok_or_else(|| invalid("missing element"))?;
        let sx = it.next().ok_or_else(|| invalid("missing x"))?;
        let sy = it.next().ok_or_else(|| invalid("missing y"))?;
        let sz = it.next().ok_or_else(|| invalid("missing z"))?;
        element.push(el.to_string());
        x.push(sx.parse().map_err(|_| invalid("bad x"))?);
        y.push(sy.parse().map_err(|_| invalid("bad y"))?);
        z.push(sz.parse().map_err(|_| invalid("bad z"))?);
    }

    let atoms = DataFrame::new(vec![
        Series::new("element", element),
        Series::new("x", x),
        Series::new("y", y),
        Series::new("z", z),
    ]).map_err(to_io)?;

    let mut frame = Frame::default();
    frame.insert("atoms", atoms);
    
    frame.meta.insert("natoms".into(), MetaType::Int64(natoms as i64));
    frame.meta.insert("comment".into(), MetaType::String(comment));

    Ok(Some(frame))
}

#[inline]
fn invalid(msg: &'static str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
}
#[inline]
fn to_io<E: std::fmt::Display>(e: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}

impl<R: BufRead> Reader for XYZFrameReader<R> {
    type R = R;
    fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: BufRead> FrameReader for XYZFrameReader<R> {
    fn read_frame(&mut self) -> std::io::Result<Option<Frame>> {
        read_xyz_frame(&mut self.reader)
    }
}

pub struct XYZTrajectoryReader<R: BufRead> {
    reader: R,
}

impl<R: BufRead + Seek> Reader for XYZTrajectoryReader<R> {
    type R = R;
    fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: BufRead + Seek> TrajectoryReader for XYZTrajectoryReader<R> {
    /// Build an index mapping frame numbers to byte offsets.
    fn build_index(&mut self) -> std::io::Result<BTreeMap<usize, u64>> {
        let mut index = BTreeMap::new();
        let mut frame_no: usize = 0;

        loop {
            // Skip blank lines and capture the start of next frame
            let mut start = self.reader.stream_position()?;
            let mut header = String::new();
            loop {
                header.clear();
                let n = self.reader.read_line(&mut header)?;
                if n == 0 {
                    return Ok(index);
                }
                if !header.trim().is_empty() {
                    break;
                }
                start = self.reader.stream_position()?;
            }

            // Validate natoms
            let natoms: usize = match header.trim().parse() {
                Ok(v) => v,
                Err(_) => {
                    // Not a valid frame start; try to continue scanning
                    continue;
                }
            };

            // Record frame start offset
            index.insert(frame_no, start);
            frame_no += 1;

            // Skip comment line
            let mut _comment = String::new();
            self.reader.read_line(&mut _comment)?;

            // Skip natoms lines
            let mut tmp = String::new();
            for _ in 0..natoms {
                tmp.clear();
                let n = self.reader.read_line(&mut tmp)?;
                if n == 0 {
                    break;
                }
            }
        }
    }

    fn read_step(&mut self, offset: usize) -> std::io::Result<Option<Frame>> {
        self.reader.seek(SeekFrom::Start(offset as u64))?;
        read_xyz_frame(&mut self.reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn sample_xyz_two_frames() -> Cursor<Vec<u8>> {
        let data = b"3\nwater\nO 0.0 0.1 0.2\nH 0.9 0.0 0.0\nH -0.9 0.0 0.0\n\n2\nH2\nH 0.0 0.0 0.0\nH 0.7 0.0 0.0\n";
        Cursor::new(data.to_vec())
    }

    #[test]
    fn test_read_single_frame() {
        let mut cursor = sample_xyz_two_frames();
        let mut reader = XYZFrameReader::new(&mut cursor);
        // Adapt type: XYZFrameReader expects R: BufRead; &mut Cursor implements BufRead
        let mut frame_reader = XYZFrameReader::new(cursor);
        let frame_opt = frame_reader.read_frame().expect("read_frame should succeed");
        let frame = frame_opt.expect("should have a frame");

        // Check meta
        assert_eq!(MapLike::<i64>::get(&frame.meta, "natoms"), Some(3));
        assert_eq!(MapLike::<String>::get(&frame.meta, "comment"), Some("water".to_string()));

        // Check atoms component
    let atoms = frame.get("atoms").expect("atoms component");
        assert_eq!(atoms.height(), 3);
        let elem = atoms.column("element").unwrap().utf8().unwrap();
        assert_eq!(elem.get(0), Some("O"));
        assert_eq!(elem.get(1), Some("H"));
        assert_eq!(elem.get(2), Some("H"));
    }

    #[test]
    fn test_build_index_and_random_access() {
        let cursor = sample_xyz_two_frames();
        let mut traj = XYZTrajectoryReader::new(cursor);
        let index = traj.build_index().expect("index");
        assert_eq!(index.len(), 2);
        let first_off = *index.get(&0).unwrap();
        let second_off = *index.get(&1).unwrap();
        assert!(second_off > first_off);

        let f1 = traj.read_step(first_off as usize).unwrap().unwrap();
        let f2 = traj.read_step(second_off as usize).unwrap().unwrap();
        assert_eq!(MapLike::<i64>::get(&f1.meta, "natoms"), Some(3));
        assert_eq!(MapLike::<i64>::get(&f2.meta, "natoms"), Some(2));
    }
}