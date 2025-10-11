use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Result};
use flate2::read::GzDecoder;
use polars::prelude::DataFrame;


pub fn open_txt(path: &str) -> Result<BufReader<File>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file))
}

pub fn open_gz(path: &str) -> Result<BufReader<GzDecoder<File>>> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(file);
    Ok(BufReader::new(decoder))
}

pub trait Reader {
    type R: BufRead;
    type FrameLike;
    fn new(reader: Self::R) -> Self;
}

pub trait FrameReader: Reader {
    fn read_frame(&mut self) -> Result<Option<Self::FrameLike>>;
}

pub trait TrajectoryReader: Reader
{
    fn build_index(&mut self) -> Result<BTreeMap<usize, u64>>;
    /// Read a frame at a given byte offset (from build_index)
    fn read_step(&mut self, offset: usize) -> Result<Option<Self::FrameLike>>;
}