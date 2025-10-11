use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Result};
use flate2::read::GzDecoder;


/// Open a plain text file and return a buffered reader.
pub fn open_txt(path: &str) -> Result<BufReader<File>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file))
}

/// Open a gzip-compressed file and return a buffered reader over the decompressed stream.
pub fn open_gz(path: &str) -> Result<BufReader<GzDecoder<File>>> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(file);
    Ok(BufReader::new(decoder))
}

/// Generic reader for data sources returning frame-like records.
pub trait Reader {
    /// Underlying buffered reader type.
    type R: BufRead;
    /// The frame type produced by this reader.
    type FrameLike;
    /// Construct a new reader from the underlying buffered reader.
    fn new(reader: Self::R) -> Self;
}

/// A reader that can read one logical frame at a time.
pub trait FrameReader: Reader {
    /// Read a single frame from the current stream position. Returns Ok(None) on EOF.
    fn read_frame(&mut self) -> Result<Option<Self::FrameLike>>;
}

/// A reader over a trajectory-like file supporting random access by step.
pub trait TrajectoryReader: Reader
{
    /// Build an index mapping step numbers to byte offsets in the underlying stream.
    fn build_index(&mut self) -> Result<BTreeMap<usize, u64>>;
    /// Read a frame at a given byte offset (from build_index)
    fn read_step(&mut self, offset: usize) -> Result<Option<Self::FrameLike>>;
}