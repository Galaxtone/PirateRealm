/*
Pirate Realm, An experimental classicube server.
Copyright (c) 2021  Galaxtone, Exopteron

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
// TODO Make CountingReader and Cursor into one AtomicCursor
use super::classic::{Packet::{self, LevelInitialize, LevelDataChunk, LevelFinalize}};
use super::{BlockID, Block, ClassicPacketWriter};
use std::pin::Pin;
use flate2::Compression;
use flate2::read::GzEncoder;
use flate2::GzBuilder;
use std::io::{Read, Write};
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use std::cell::Cell;
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use bytes::{Bytes, BytesMut};
#[derive(Clone)]
pub struct World {
  data: BytesMut, // XZY
  pub width: usize,
  pub height: usize,
  pub length: usize,
}
pub struct SendableWorld {
  pub len: usize,
  pub width: usize,
  pub height: usize,
  pub length: usize,
  pub data: Arc<&'static [u8]>,
}

use std::io::Result as IoResult;
pub struct CountingReader<'a> {
  inner: &'a mut (dyn Read + Send),
  pub count: Arc<AtomicUsize>,
}

impl<'a> CountingReader<'a> {
  pub fn new(read: &'a mut (dyn Read + Send)) -> (Self, Arc<AtomicUsize>) {
    let count = Arc::new(AtomicUsize::new(0));
    (Self { inner: read, count: count.clone() }, count.clone())
  }

}

impl<'a> Read for CountingReader<'a> {
  fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
    let count = self.inner.read(buf)?;
    self.count.fetch_add(1, Ordering::SeqCst);
    Ok(count)
  }
}

impl World {
  pub fn new(generator: impl WorldGenerator, width: usize, height: usize, length: usize) -> Self {
    let size = width * height * length;
    let mut data = vec![0; size + 4];
    // Big-endian length of blocks array number
    // TODO use bytes to simplify into .put_be_i32() or something
    data[0] = (size >> 24) as u8;
    data[1] = (size >> 16) as u8;
    data[2] = (size >> 8) as u8;
    data[3] = size as u8;
    generator.generate(&mut data[4..], width, height, length);
    Self { data: BytesMut::from(&data[..]), width, height, length }
  }
  pub fn from_file(width: usize, height: usize, length: usize) -> World {
    use nbt::decode::read_compound_tag;
    use std::io::Cursor;
    println!("");
    let mut cursor = std::io::Cursor::new(include_bytes!("../serverworldlol.cw.uncompressed"));
    let root_tag = read_compound_tag(&mut cursor).unwrap();
    let world = root_tag.get_i8_vec("BlockArray").unwrap();
    let width = root_tag.get_i16("X").unwrap() as usize;
    let height = root_tag.get_i16("Y").unwrap() as usize;
    let length = root_tag.get_i16("Z").unwrap() as usize;
    let mut newworld = vec![];
    for i in 0..world.len() {
      newworld.push(world[i] as u8);
    }
    let size = width * height * length;
    let mut data = vec![0; 4];
    data[0] = (size >> 24) as u8;
    data[1] = (size >> 16) as u8;
    data[2] = (size >> 8) as u8;
    data[3] = size as u8;
    data.append(&mut newworld);
    //let data = data.into_boxed_slice();
    Self { data: BytesMut::from(&data[..]), width, height, length }
  }
  pub fn pos_to_index(&self, x: usize, y: usize, z: usize) -> usize {
    (z + y * self.length) * self.width + x
  }

  // TODO position struct type stuff
  pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<BlockID> {
    let x = self.data.get(self.pos_to_index(x, y, z));
    match x {
      Some(x) => {
        return Some(*x);
      }
      None => {
        return None;
      }
    }
  }

  pub fn set_block(&mut self, block: Block) -> Option<()> {
    let pos = self.pos_to_index(block.position.x as usize, block.position.y as usize, block.position.z as usize);
    let p2 = pos.clone();
    drop(pos);
    let test = self.data.get(p2);
    if test.is_some() {
      self.data[p2] = block.id;
      return Some(());
    } else {
      return None;
    }
  }

  pub fn data(&self) -> &[BlockID] {
    &self.data
  }
  pub fn new_data(&self) -> BytesMut {
    self.data.clone()
  }

  pub fn data_mut(&mut self) -> &mut [BlockID] {
    &mut self.data
  }
      // QUICK AND DIRTY
      pub async fn to_packets(&mut self, mut writer: &mut Pin<Box<impl tokio::io::AsyncWriteExt>>) {
        //let mut encoder = GzEncoder::new(self.data(), Compression::fast());
        let len = self.data().len();
        //let mut reader = &mut self.data;
        let mut data = self.data();
        let (mut reader, counter) = CountingReader::new(&mut data);
        let mut encoder = GzBuilder::new().comment("Rust>Java").read(&mut reader, Compression::fast());
        let serialized = ClassicPacketWriter::serialize(LevelInitialize).unwrap();
        writer.write_all(&serialized).await;
        let mut i: u32 = 0;
        loop {
          let mut x = [0; 1024];
          let res = encoder.read_exact(&mut x);
          if res.is_err() {
            let count = counter.load(Ordering::SeqCst);
            let mut chunk_data = vec![];
            encoder.read_to_end(&mut chunk_data);
            //println!("Reader: {:?}", counter.load(Ordering::SeqCst));
            if chunk_data.len() == 0 {
              let serialized = ClassicPacketWriter::serialize(LevelFinalize { width: self.width, height: self.height, length: self.length}).unwrap();
              writer.write_all(&serialized).await;
              return;
            }
            let chunk_data = chunk_data.into_boxed_slice();
            let len = chunk_data.len();
            let serialized = ClassicPacketWriter::serialize(LevelDataChunk { chunk_length: len as i16, chunk_data, percent_complete: 255}).unwrap();
            writer.write_all(&serialized).await;
          } else {
            let count = counter.load(Ordering::SeqCst);
            //println!("Reader: {:?}", count);
            //println!("Sending");
            //let i = i << 10;
            let mut chunk_data = Box::new(x);
            let serialized = ClassicPacketWriter::serialize(LevelDataChunk { chunk_length: 1024, chunk_data,
              percent_complete: (count * 255 / len) as u8,
            }).unwrap();
            writer.write_all(&serialized).await;
          }
          i += 1;
        }
      }
}
pub trait WorldGenerator {
  fn generate(&self, data: &mut [BlockID], width: usize, height: usize, length: usize);
}

pub struct FlatWorldGenerator {
  height: usize,
  below: BlockID,
  surface: BlockID,
  above: BlockID,
}
pub struct PerlinWorldGenerator {
    height: usize,
    below: BlockID,
    surface: BlockID,
    above: BlockID,
  }

impl FlatWorldGenerator {
  pub fn new(height: usize, below: BlockID, surface: BlockID, above: BlockID) -> Self {
    Self { height, below, surface, above }
  }
}
impl PerlinWorldGenerator {
    pub fn new(height: usize, below: BlockID, surface: BlockID, above: BlockID) -> Self {
      Self { height, below, surface, above }
    }
  }
impl WorldGenerator for FlatWorldGenerator {
  fn generate(&self, data: &mut [BlockID], width: usize, height: usize, length: usize) {
    let area = width * length;
    for y in 0..height {
      let yi = area * y;
      if y < self.height - 1 {
        for i in 0..area { data[yi + i] = self.below; }
      } else if y < self.height {
        for i in 0..area { data[yi + i] = self.surface; }
      } else {
        for i in 0..area { data[yi + i] = self.above; }
      }
    }
  }
}
impl WorldGenerator for PerlinWorldGenerator {
    fn generate(&self, data: &mut [BlockID], width: usize, height: usize, length: usize) {
        extern crate perlin_noise as perlin;
use perlin::PerlinNoise;
let perlin = PerlinNoise::new();
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64 - 0.5;
                let ny = y as f64 / height as f64 - 0.5;
                data[perlin.get2d([nx,ny]) as usize] = self.surface;
            }
        }
    }
  }