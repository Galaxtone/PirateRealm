use std::io::{Error, ErrorKind};
use std::convert::From;
//use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use std::pin::Pin;

pub mod chunks;
#[derive(Clone)]
pub struct Position {
  pub x: i16,
  pub y: i16,
  pub z: i16
}
#[derive(Clone)]
pub struct Block {
  pub position: Position,
  pub id: BlockId
}
// To clone it in main.rs fo both packets
#[derive(Clone, Debug)]
pub struct PositionYP {
  pub x: i16,
  pub y: i16,
  pub z: i16,
  pub yaw: u8,
  pub pitch: u8,
}

impl PositionYP {
  pub const FEET_DISTANCE: i16 = 51;
  pub fn from_pos(x: i16, y: i16, z: i16) -> Self {
    PositionYP {
      x: (x << 5) + 16,
      y: (y << 5) + PositionYP::FEET_DISTANCE,
      z: (z << 5) + 16,
      yaw: 0,
      pitch: 0,
    }
  }
}
impl Default for PositionYP {
  fn default() -> Self {
  Self {x: 0, y: 0, z: 0, yaw: 0, pitch: 0}
  }
}

// Constants for use, probably temporary as runtime ability to get block name will be needed in the future.
pub type BlockId = u8;

// Being used as a PSUEDO-Enum
#[allow(non_snake_case)]
pub mod BlockIds {
  use super::BlockId;
  // This is technically a sub-module,
  // so i need to import from the parent
  
  pub const AIR: BlockId = 0;
  pub const STONE: BlockId = 1;
  pub const GRASS: BlockId = 2;
  pub const DIRT: BlockId = 3;
  pub const COBBLESTONE: BlockId = 4;
  pub const PLANKS: BlockId = 5;
}

/// All client packets
pub enum ClassicPacketClient {
  PlayerIdentification {protocol_ver: u8, username: String, verification_key: String},
  PositionAndOrientation {player_id: u8, position: PositionYP},
  SetBlock {coords: Position, mode: u8, block_type: u8},
  Message {message: String},
  Other,
}


/// All server packets
pub enum ClassicPacketServer {
  ServerIdentification {protocol_ver: u8, servername: String, motd: String},
  Ping,
  LevelInitialize, // TODO the below packet should be abstracted into a single property for data Vec with a function like read_fixed_bytes that discards padding bytes if length < 1024
  LevelDataChunk { chunk_length: i16, chunk_data: Box<[u8]>, percent_complete: u8},
  LevelFinalize { width: usize, height: usize, length: usize}, //FIXME
  // skipping a few
  SpawnPlayer { player_id: i8, name: String, position: PositionYP},
  // remember, teleports are relative to eye level, add +51 directly to the final number to be relative to FEET, like normal minecraft. Don't confuse this as going from feet to head!
  PlayerTeleport { player_id: i8, position: PositionYP},
  SetBlock {block: Block},
  DespawnPlayer { player_id: i8 },
  Message { player_id: i8, message: String},
  DisconnectPlayer { reason: String },
}

impl ClassicPacketServer {
  pub fn serialize(packet: ClassicPacketServer) -> std::io::Result<Vec<u8>> {
    match packet {
      ClassicPacketServer::Ping => {
        let builder = ClassicPacketBuilder::new();
        return Ok(builder.build(0x01)?);
      },
      ClassicPacketServer::SpawnPlayer {player_id, name, position} => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        builder.insert_string(&name);
        builder.insert_short(position.x);
        builder.insert_short(position.y);
        builder.insert_short(position.z);
        builder.insert_byte(position.yaw);
        builder.insert_byte(position.pitch);
        return Ok(builder.build(0x07)?);
      }
      ClassicPacketServer::PlayerTeleport { player_id, position } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        builder.insert_short(position.x);
        builder.insert_short(position.y);
        builder.insert_short(position.z);
        builder.insert_byte(position.yaw);
        builder.insert_byte(position.pitch);
        return Ok(builder.build(0x08)?);
      }
      ClassicPacketServer::DespawnPlayer { player_id } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        return Ok(builder.build(0x0c)?);
      }
      ClassicPacketServer::DisconnectPlayer { reason } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_string(&reason);
        return Ok(builder.build(0x0e)?);
      }
      ClassicPacketServer::Message { player_id, message } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        builder.insert_string(&message);
        return Ok(builder.build(0x0d)?);
      }
      ClassicPacketServer::SetBlock { block } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_short(block.position.x);
        builder.insert_short(block.position.y);
        builder.insert_short(block.position.z);
        builder.insert_byte(block.id);
        return Ok(builder.build(0x06)?);
      }
      _ => {
        return Err(Error::new(ErrorKind::Other, format!("Unknown packet")));
      }
    }
  }
  pub fn serialize_vec(vec: Vec<ClassicPacketServer>) -> std::io::Result<Vec<Vec<u8>>> {
    let mut vec2 = vec![];
    for packet in vec {
      match packet {
        ClassicPacketServer::LevelInitialize => {
          let builder = ClassicPacketBuilder::new();
          vec2.push(builder.build(0x02)?);
        }
        ClassicPacketServer::LevelDataChunk {chunk_length, chunk_data, percent_complete} => {
          let mut builder = ClassicPacketBuilder::new();
          builder.insert_short(chunk_length as i16);
          builder.insert_bytearray(chunk_data.to_vec());
          builder.insert_byte(percent_complete);
          vec2.push(builder.build(0x03)?);
        }
        ClassicPacketServer::LevelFinalize {width, height, length} => {
          let mut builder = ClassicPacketBuilder::new();
          builder.insert_short(width as i16);
          builder.insert_short(height as i16);
          builder.insert_short(length as i16);
          vec2.push(builder.build(0x04)?);
        }
        _ => {
            return Err(Error::new(ErrorKind::Other, format!("Unknown packet")));
        }
      }
    }
    Ok(vec2)
  }
  pub fn server_identification(protocol_ver: u8, server_name: String, motd: String, is_op: bool) -> std::io::Result<Vec<u8>> {
    let mut builder = ClassicPacketBuilder::new();
    builder.insert_byte(protocol_ver);
    builder.insert_string(&server_name);
    builder.insert_string(&motd);
    match is_op {
      true => {
        builder.insert_byte(0x64);
      }
      false => {
        builder.insert_byte(0x00);
      }
    }
    return Ok(builder.build(0x00)?);
  }
  pub fn spawn_player(player_id: i8, player_name: String, position: PositionYP) -> std::io::Result<Vec<u8>> {
    let mut builder = ClassicPacketBuilder::new();
    builder.insert_sbyte(player_id);
    builder.insert_string(&player_name);
    builder.insert_short(position.x);
    builder.insert_short(position.y);
    builder.insert_short(position.z);
    builder.insert_byte(position.yaw);
    builder.insert_byte(position.pitch);
    return Ok(builder.build(0x07)?);
  }
}
pub struct ClassicPacketReader {

}
impl ClassicPacketReader {
  pub async fn read_packet_reader<'a>(reader: &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>) -> std::io::Result<ClassicPacketClient> {
    let id = ClassicPacketUtils::read_byte(reader).await?;
    match id {
      0x00 => {
        let protocol_ver = ClassicPacketUtils::read_byte(reader).await?;
        let username = ClassicPacketUtils::read_string(reader).await?;
        let verification_key = ClassicPacketUtils::read_string(reader).await?;
        let unused = ClassicPacketUtils::read_byte(reader).await?;
        drop(unused);
        let packet = ClassicPacketClient::PlayerIdentification {protocol_ver: protocol_ver, username: username, verification_key: verification_key};
        return Ok(packet);
      }
      0x08 => {
        let pid = ClassicPacketUtils::read_byte(reader).await?;
        let x = ClassicPacketUtils::read_short(reader).await?;
        let y = ClassicPacketUtils::read_short(reader).await?;
        let z = ClassicPacketUtils::read_short(reader).await?;
        let yaw = ClassicPacketUtils::read_byte(reader).await?;
        let pitch = ClassicPacketUtils::read_byte(reader).await?;
        let coords = PositionYP {x: x, y: y, z: z, yaw: yaw, pitch: pitch};
        let packet = ClassicPacketClient::PositionAndOrientation {player_id: pid, position: coords};
        return Ok(packet);
      }
      0x05 => {
        let x = ClassicPacketUtils::read_short(reader).await?;
        let y = ClassicPacketUtils::read_short(reader).await?;
        let z = ClassicPacketUtils::read_short(reader).await?;
        let mode = ClassicPacketUtils::read_byte(reader).await?;
        let blocktype = ClassicPacketUtils::read_byte(reader).await?;
        let coords = Position {x: x, y: y, z: z};
        let packet = ClassicPacketClient::SetBlock {coords: coords, mode: mode, block_type: blocktype};
        return Ok(packet);
      }
      0x0d => {
        let x = ClassicPacketUtils::read_byte(reader).await?;
        drop(x);
        let message = ClassicPacketUtils::read_string(reader).await?;
        let packet = ClassicPacketClient::Message { message: message };
        return Ok(packet);
      }
      id => {
        return Err(Error::new(ErrorKind::Other, format!("Unknown packet id {}!", id)));
      }
    }
  }
}
pub struct ClassicPacketUtils {

}
impl ClassicPacketUtils {
  async fn read_byte<'a>(reader:  &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>) -> std::io::Result<u8> {
    let mut byte = [0; 1];
    reader.read_exact(&mut byte).await?;
    return Ok(byte[0]);
  }
  async fn read_short<'a>(reader:  &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>) -> std::io::Result<i16> {
    let mut byte = [0; 2];
    reader.read_exact(&mut byte).await?;
    let short = i16::from_be_bytes(byte);
    return Ok(short);
  }
  async fn read_string<'a>(reader:  &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>) -> std::io::Result<String> {
    let mut byte = [0; 64];
    reader.read_exact(&mut byte).await?;
    let string = String::from_utf8_lossy(&byte).to_string();
    return Ok(string.trim_matches(char::from(0x20)).to_string());
  }
}
#[derive(Clone)]
pub enum Element {
  Byte { byte: u8 },
  SByte { byte: i8 },
  StringElement { string: String },
  Short { short: i16 },
  Bytes { bytes: Vec<u8> }
}
pub struct ClassicPacketBuilder {
  elements: Vec<Element>,
}
impl ClassicPacketBuilder {
  pub fn new() -> Self {
    return Self {
      elements: Vec::new(),
    };
  }
  pub fn insert_string(&mut self, string: &str) {
    self.elements.push(Element::StringElement {
      string: string.to_string(),
    });
  }
  pub fn insert_bytearray(&mut self, bytes: Vec<u8>) {
    self.elements
      .push(Element::Bytes { bytes: bytes });
  }
  pub fn insert_sbyte(&mut self, byte: i8) {
    self.elements.push(Element::SByte { byte: byte });
  }
  pub fn insert_byte(&mut self, byte: u8) {
    self.elements.push(Element::Byte { byte: byte });
  }
  pub fn insert_short(&mut self, short: i16) {
    self.elements.push(Element::Short { short: short });
  }
  pub fn build(mut self, id: u8) -> std::io::Result<Vec<u8>> {
    let mut packet = vec![id];
    packet.append(&mut self.internal_builder()?);
    return Ok(packet);
  }
  fn internal_builder(&mut self) -> std::io::Result<Vec<u8>> {
    let mut packet = vec![];
    for element in self.elements.clone() {
      match element.clone() {
        Element::StringElement { string } => {
          if string.len() > 64 {
            return Err(Error::new(ErrorKind::Other, "String too large!"));
          }
          let mut string = string.as_bytes().to_vec();
          for _ in 0..64 - string.len() {
            string.push(0x20);
          }
          packet.append(&mut string);
        }
        Element::Byte { byte } => {
          packet.push(byte);
        }
        Element::SByte { byte } => {
          packet.push(byte.to_le_bytes()[0]);
        }
        Element::Short { short } => {
          packet.append(&mut short.to_be_bytes().to_vec());
        }
        Element::Bytes { mut bytes } => {
          if bytes.len() > 1024 {
            return Err(Error::new(ErrorKind::Other, "Bytes too large!"));
          }
          for _ in bytes.len()..1024 {
            bytes.push(0x00);
          }
          packet.append(&mut bytes);
        }
      }
    }
    return Ok(packet);
  }
}
