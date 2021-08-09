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

use super::classic::Packet::{self, LevelInitialize, LevelDataChunk, LevelFinalize};
use super::{BlockID, Block};

use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

pub struct World {
  data: Box<[BlockID]>, // XZY
  width: usize,
  height: usize,
  length: usize,
}


impl World {
  pub fn new(generator: impl WorldGenerator, width: usize, height: usize, length: usize) -> Self {
    let size = width * height * length;
    let mut data = vec![0; size + 4].into_boxed_slice();
    // Big-endian length of blocks array number
    // TODO use bytes to simplify into .put_be_i32() or something
    data[0] = (size >> 24) as u8;
    data[1] = (size >> 16) as u8;
    data[2] = (size >> 8) as u8;
    data[3] = size as u8;
    generator.generate(&mut data[4..], width, height, length);
    Self { data, width, height, length }
  }

  pub fn pos_to_index(&self, x: usize, y: usize, z: usize) -> usize {
    (z + y * self.length) * self.width + x
  }

  // TODO position struct type stuff
  pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockID {
    self.data[self.pos_to_index(x, y, z)]
  }

  pub fn set_block(&mut self, block: Block) {
    self.data[self.pos_to_index(block.position.x as usize, block.position.y as usize, block.position.z as usize)] = block.id;
  }

  pub fn data(&self) -> &[BlockID] {
    &self.data
  }

  pub fn data_mut(&mut self) -> &mut [BlockID] {
    &mut self.data
  }

  // QUICK AND DIRTY
  pub fn to_packets(&self) -> Vec<Packet> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&self.data);

    let data = encoder.finish().unwrap();

    // 2 for initialize and finalize and data.len()/1024 packets for data chunk
    // just thinking ahead for allocation, shrug
    let num_full_data_packets = data.len() >> 10;
    let mut num_data_packets = num_full_data_packets;
    let has_partial_data_packet = data.len() & 0x3FF != 0;
    if has_partial_data_packet {
      num_data_packets += 1;
    }

    let mut packets = Vec::with_capacity(num_data_packets + 2);
    packets.push(LevelInitialize);

    let data: &[u8] = &data;
    for i in 0..num_full_data_packets {
      let i = i << 10;
      let mut chunk_data = vec![0; 1024].into_boxed_slice();
      chunk_data.copy_from_slice(&data[i..i+1024]);
      packets.push(LevelDataChunk { chunk_length: 1024, chunk_data,
        percent_complete: ((i + 1023) / (data.len() * 255)) as u8,
      });
    }

    if has_partial_data_packet {
      let i = num_full_data_packets << 10;
      let len = data.len() - i;
      let mut chunk_data = vec![0; 1024].into_boxed_slice();
      chunk_data[0..len].copy_from_slice(&data[i..data.len()]);
      packets.push(LevelDataChunk { chunk_length: len as i16, chunk_data, percent_complete: 255});
    }

    packets.push(LevelFinalize { width: self.width, height: self.height, length: self.length});
    packets
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