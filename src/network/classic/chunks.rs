use super::ClassicPacketServer::{self, LevelInitialize, LevelDataChunk, LevelFinalize};
use super::{BlockId, Block};

pub struct World {
  data: Box<[BlockId]>, // XZY
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
  pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockId {
    self.data[self.pos_to_index(x, y, z)]
  }

  pub fn set_block(&mut self, block: Block) {
    self.data[self.pos_to_index(block.position.x as usize, block.position.y as usize, block.position.z as usize)] = block.id;
  }

  pub fn data(&self) -> &[BlockId] {
    &self.data
  }

  pub fn data_mut(&mut self) -> &mut [BlockId] {
    &mut self.data
  }

  // QUICK AND DIRTY
  pub fn to_packets(&self) -> Vec<ClassicPacketServer> {
    let data = deflate::deflate_bytes_gzip(&self.data);

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
  fn generate(&self, data: &mut [BlockId], width: usize, height: usize, length: usize);
}

pub struct FlatWorldGenerator {
  height: usize,
  below: BlockId,
  surface: BlockId,
  above: BlockId,
}

impl FlatWorldGenerator {
  pub fn new(height: usize, below: BlockId, surface: BlockId, above: BlockId) -> Self {
    Self { height, below, surface, above }
  }
}

impl WorldGenerator for FlatWorldGenerator {
  fn generate(&self, data: &mut [BlockId], width: usize, height: usize, length: usize) {
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