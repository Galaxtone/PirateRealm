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

// HINT: Game struct holds many distinct properties
// together to transport together
// POSSIBLE: The game struct may be removed
// if we instead end up immediately making
// managing message-receiving tasks
// for all properties

// HINT: Make properties as big as possible while
// containing entirely similiar attributes that are
// used together
// Good e.g. PlayerBodies + PlayerNames => Players
// Bad e.g. World + Config => Worofig

// HINT: Message passing is god and it's optimised.
mod chunks;
use chunks::{FlatWorldGenerator, World};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
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
  pub const SPONGE: BlockId = 19;
}

/* ================================================ maths.rs ================================================ */
#[derive(Clone, Debug)]
pub struct BlockPosition {
  x: usize,
  y: usize,
  z: usize,
}

/* pub struct Position(f32, f32, f32);

impl Position {
  pub fn distance_from_squared(&self, other: Position) -> f32 {
    //(self.x - other.x).pow(2) + (self.y - other.y).pow(2) + (self.z - other.z).pow(2)
  }
  // uh put it in a queue for the time being, technically an unbounded broadcast is a queue
  // anyway go and make assumptions and ill correct em... hopefully
  // puts a set block packet in a queue to send to other players, to make it easier on us
  // wut i have no idea where to start, help pls you
  // chose the logic for this one you help me get started
  // line 157 <-- @Galaxtone
  //take me to the line of code to start at or around thee, just click and ill follow
  pub fn distance_from_block_squared(&self, other: BlockPosition) -> usize {
    //(self.x as usize - other.x).pow(2) + (self.y as usize - other.y).pow + (self.z as usize - other.z).pow(2)
  }
  // we still don't have a solution to the MagicContainer for storing all the players
  // no that's where we store all the players in the Game object, just a Vec? Arc<Mutex<Vec<Player>>> ?
  pub fn distance_from(&self, other: Position) -> f32 {
    self.distance_from_squared(other).sqrt()
  }
}

pub struct PlayerTransform {
  pos: Position,
  yaw: f32,
  pitch: f32,
  size: f32,
}

impl PlayerTransform {
  pub fn looking_at(&self, target: BlockPosition) -> bool {
    /*insert fancy vector maths*/
  }

  pub fn distance_from_player(&self, other: PlayerTransform) -> f32 {
  }
  pub fn distance_from_squared(&self, other: BlockPosition) -> usize {
    //(other.x - self.x.floor()).pow(2) + (other.y - self.y.floor()).pow(2) + (other.z - self.z.floor())

    return ((other.x as f32 - self.x as usize).pow(2) + (target.y as usize - self.y as usize).pow(2) + (target.z as usize + self.z as usize).pow(2)) / 2;
  }
}

fn f32_to_fixed(x: f32) -> i16 {
  return (x * 32).round() as i16
} */
// players will be able to change their scale with the ScaleExtension
// and your calculations will need scale to properly raytrace when we implement it
// so really it should include scale and be: PlayerPositionRotationScale
// which is short for Transformation or "PlayerTransform"
// including scale, this is what game engines use.

// exclude scale, transform makes sense
// and players might be able to scale their size in the near-future
// and if your doing calculations you'll want scale...

// just leave it whatever, we decide later.
// let's write some proper code

// yeah, although PlayerPositionRotation isn't too bad
// it'd be short if you shorthand Pos and Rot like PosRot
#[derive(Clone)]
pub struct PlayerPosition {
  x: u16,
  y: u16,
  z: u16,
  yaw: u8,
  pitch: u8,
}
impl PlayerPosition {
  pub const FEET_DISTANCE: u16 = 51;
  pub fn from_pos(x: u16, y: u16, z: u16) -> Self {
    PlayerPosition {
      x: (x << 5) + 16,
      y: (y << 5) + PlayerPosition::FEET_DISTANCE,
      z: (z << 5) + 16,
      yaw: 0,
      pitch: 0,
    }
  }
  pub fn distance_to(&self, target: BlockPosition) -> f64 {

    return (((self.x as f64 / 32.0) - target.x as f64).powf(2.0)
      + ((self.y as f64 / 32.0) - target.y as f64).powf(2.0)
      + ((self.z as f64 / 32.0) - target.z as f64).powf(2.0))
    .sqrt();
  }
  pub fn distance_to_plr(&self, target: PlayerPosition) -> f64 {

    return (((self.x as f64 / 32.0) - (target.x as f64 / 32.0)).powf(2.0)
      + ((self.y as f64 / 32.0) - (target.y as f64 / 32.0)).powf(2.0)
      + ((self.z as f64 / 32.0) - (target.z as f64 / 32.0)).powf(2.0))
    .sqrt();
  }
}
impl Default for PlayerPosition {
  fn default() -> Self {
    Self {
      x: 0,
      y: 0,
      z: 0,
      yaw: 0,
      pitch: 0,
    }
  }
}
type BlockID = u8;
#[derive(Clone, Debug)]
pub struct Block {
  position: BlockPosition,
  id: BlockID,
}

pub struct PlayerData {
  position: PlayerPosition,
}
pub struct Player {
  data: PlayerData,
  op: bool,
  // unique identifier, shorthand
  id: u32,
  protocol: Protocol,
  name: String,
  message_send: stdmpsc::Sender<PlayerCommand>,
}
pub enum Protocol {
  Classic { plr_id: u8 },
}

use std::sync::mpsc as stdmpsc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct GMTS {
  // GMTS short for: Game Managing Task Senders
  world_send: mpsc::Sender<WorldCommand>,
  players_send: mpsc::Sender<PlayersCommand>,
  tempcrntid_send: mpsc::Sender<TempCrntIdCommand>,
  heartbeat_send: mpsc::Sender<HeartbeatCommand>,
}

// block is already defined
#[derive(Clone)]
pub enum PlayerCommand {
  SetBlock {
    block: Block,
  },
  SpawnPlayer {
    position: PlayerPosition,
    id: i8,
    name: String,
  },
  DespawnPlayer {
    id: i8,
  },
  PlayerTeleport {
    position: PlayerPosition,
    id: i8,
  },
  Message {
    id: i8,
    message: String,
  },
  Disconnect {
    reason: String,
  },
}
pub enum HeartbeatCommand {}
pub enum WorldCommand {
  GetBlock {
    pos: BlockPosition,
    res_send: oneshot::Sender<Option<Block>>,
  },
  SetBlock {
    block: Block,
    res_send: oneshot::Sender<Option<()>>,
  },
  SetBlockP {
    block: Block,
    players_send: mpsc::Sender<PlayersCommand>,
    sender_id: u32,
    res_send: oneshot::Sender<Option<()>>,
  },
  GetWorld {
    res_send: oneshot::Sender<World>,
  },
}

pub enum PlayersCommand {
  /*   GetUserPos {
    user_id: u32,
    res_send: oneshot::Sender<PlayerPosition>
  }, */
  NewUser {
    user: Player,
    pos: PlayerPosition,
    res_send: oneshot::Sender<()>,
  },
  RemoveUser {
    user_id: u32,
    res_send: oneshot::Sender<()>,
  },
  PassMessageToAll {
    message: PlayerCommand,
    res_send: oneshot::Sender<()>,
  },
  SpawnAllPlayers {
    my_id: u32,
    res_send: oneshot::Sender<()>,
  },
  UpdatePosition {
    my_id: u32,
    position: PlayerPosition,
    res_send: oneshot::Sender<Option<()>>,
  },
  UpdatePositionUnchecked {
    my_id: u32,
    position: PlayerPosition,
    res_send: oneshot::Sender<Option<()>>,
  },
  GetPosition {
    id: u32,
    res_send: oneshot::Sender<Option<PlayerPosition>>,
  },
  PassMessageToID {
    id: u32,
    message: PlayerCommand,
    res_send: oneshot::Sender<Option<()>>,
  },
  IsOperator {
    id: u32,
    res_send: oneshot::Sender<bool>,
  },
  KickUserByName {
    username: String,
    reason: String,
    res_send: oneshot::Sender<Option<()>>,
  },
}
// Honestly the code looks as ugly as a raw Future implemntation
// We could probably write some macros to insert the ugly code... in the future.

pub enum TempCrntIdCommand {
  FetchFreeID {
    res_send: oneshot::Sender<u32>,
  },
  ReturnFreeID {
    id: u32,
    res_send: oneshot::Sender<()>,
  },
}

use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  Builder::new()
  .format(|buf, record| {
      writeln!(buf,
          "[{} {}] - {}",
          Local::now().format("%H:%M:%S"),
          record.level(),
          record.args()
      )
  })
  .filter(None, LevelFilter::Info)
  .init();
  let gmts = setup_gmts();
  let cgmts = gmts.clone();
  let console = tokio::spawn(async move {
    loop {
      let mut command = String::new();
      let x = std::io::stdin().read_line(&mut command);
      if x.is_err() {
        log::error!("Error reading command!");
        continue;
      }
      let command = command.trim();
      let command: Vec<&str> = command.split(" ").collect();
      
      match command[0] {
        "say" => {
          let to_say = &command[1..].join(" ");
          let to_say = format!("&d[Server] {}", to_say);
          send_message(&to_say, -69, &cgmts).await;
        }
        "stop" => {
          log::info!("Stopping server..");
          let message = PlayerCommand::Disconnect { reason: "Server closed".to_string() };
          pass_message_to_all(message, &cgmts).await;
          std::process::exit(0);
        }
        _ => {
          log::warn!("Unknown command \"{}\".", command[0]);
        }
      }
    }
  });
  // Pass around immutable references, and clone the sender.

  //example(&gmts);

  let listener = TcpListener::bind("0.0.0.0:25565").await?;
  log::info!("Server listening on 0.0.0.0:25565");
  loop {
    let possible = listener.accept().await;
    if possible.is_err() {
      continue;
    }
    let (stream, _) = possible.unwrap();
    let gmts = gmts.clone();
    tokio::spawn(async move {
      if let Err(e) = incoming_connection_handler(stream, &gmts).await {
        eprintln!("An error occured. {:?}", e);
      }
    });
  }
}
async fn incoming_connection_handler(
  mut stream: TcpStream,
  gmts: &GMTS,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut test = Box::pin(&mut stream);
  let packet = ClassicPacketReader::read_packet_reader(&mut test).await?;
  let (msg_send, recv) = stdmpsc::channel::<PlayerCommand>();
  let our_id: u32;
  let our_username: String;
  let our_p_ver: u8;
  let mut op = false;
  match packet {
    classic::Packet::PlayerIdentification {
      p_ver,
      user_name,
      v_key: _,
    } => {
      our_p_ver = p_ver;
      our_username = user_name.clone();
      if our_username.len() >= 20 {
        let packet = classic::Packet::Disconnect {
          reason: "Name too long!".to_string(),
        };
        stream
          .write_all(&ClassicPacketWriter::serialize(packet)?)
          .await?;
        return Ok(());
      }
      let (res_send, res_recv) = oneshot::channel();
      let x = gmts
        .tempcrntid_send
        .send(TempCrntIdCommand::FetchFreeID { res_send })
        .await;
      if x.is_err() {
        panic!("This shouldn't fail!");
      }
      our_id = res_recv.await?;
      let data = PlayerData {
        position: PlayerPosition::from_pos(128 / 2, 64, 128 / 2),
      };
      let operators = vec!["Exopteron"];
      for operator in &operators {
        if &user_name == operator {
          op = true;
        }
      }
      let (res_send, res_recv) = oneshot::channel();
      let x = gmts
        .players_send
        .send(PlayersCommand::KickUserByName {
          username: user_name.clone(),
          reason: "You logged in from another location".to_string(),
          res_send,
        })
        .await;
      if x.is_err() {
        panic!("This shouldn't fail!");
      }
      res_recv.await?;
      let player = Player {
        data: data,
        op,
        id: our_id,
        protocol: Protocol::Classic {
          plr_id: our_id as u8,
        },
        name: user_name,
        message_send: msg_send.clone(),
      };
      let (res_send, res_recv) = oneshot::channel();
      let pos = player.data.position.clone();
      let x = gmts
        .players_send
        .send(PlayersCommand::NewUser {
          user: player,
          pos,
          res_send,
        })
        .await;
      if x.is_err() {
        panic!("Shouldn't fail!");
      }
      res_recv.await?;
    }
    _ => {
      return Ok(());
    }
  }
  let server_identification = ClassicPacketWriter::server_identification(
    0x07,
    "Ballland".to_string(),
    "a really good motd -hax".to_string(),
    op,
  )?;
  stream.write_all(&server_identification).await.unwrap();
  log::info!("{}[{}] logging in with entity id {} protocol version {}", our_username, stream.peer_addr()?.to_string(), our_id, our_p_ver);
  send_message(&format!("&e{} logging in...", our_username), -1, &gmts).await;
  let mut world = get_world(&gmts).await?;
  log::info!("Sending world to {}", our_username);
  world.to_packets(&mut Box::pin(&mut stream)).await.expect("Shouldn't fail!");
  log::info!("World sent to {}", our_username);
  /*   let world_packets = ClassicPacketWriter::serialize_vec(world)?;
  for packet in world_packets {
    stream.write_all(&packet).await?;
  } */
  //let (whsend, mut whrecv) = mpsc::channel::<classic::Packet>(10);
  let (mut readhalf, mut writehalf) = stream.into_split();
  let mut test = Box::pin(&mut readhalf);
  /*   let writehandle = tokio::spawn(async move {
    loop {
      let recv = whrecv.recv().await;
      if recv.is_none() {
        continue;
      }
      let packet = recv.unwrap();
      let packet = ClassicPacketWriter::serialize(packet).unwrap();
      writehalf.write_all(&packet).await;
    }
  }); */
  let disconnect = std::sync::Arc::new(tokio::sync::Mutex::new(false));
  let disconnect_1 = disconnect.clone();
  let disconnect_2 = disconnect.clone();
  let teleport_player = classic::Packet::PlayerTeleportS {
    player_id: -1,
    position: PlayerPosition::from_pos(128 / 2, 64, 128 / 2),
  };
  let iswrite = writehalf
    .write_all(&ClassicPacketWriter::serialize(teleport_player)?)
    .await;
  if iswrite.is_err() {
    let (res_send, res_recv) = oneshot::channel();
    let x = gmts
      .players_send
      .send(PlayersCommand::RemoveUser {
        user_id: our_id,
        res_send,
      })
      .await;
    if x.is_err() {
      panic!("This shouldn't fail!");
    }
    res_recv.await.unwrap();
    let (res_send, res_recv) = oneshot::channel();
    let x = gmts
      .tempcrntid_send
      .send(TempCrntIdCommand::ReturnFreeID {
        id: our_id,
        res_send,
      })
      .await;
    if x.is_err() {
      panic!("This shouldn't fail!");
    }
    res_recv.await.unwrap();
    println!("Early disconnect.");
    send_message(&format!("&e{} left the game.", our_username), -1, &gmts).await;
    return Ok(());
  }
  let gmts2 = gmts.clone();
  log::info!("{} joined the game.", our_username);
  send_message(&format!("&e{} joined the game.", our_username), -1, &gmts).await;
  let messagehandle = tokio::spawn(async move {
    let (res_send, res_recv) = oneshot::channel();
    let x = gmts2
      .players_send
      .send(PlayersCommand::SpawnAllPlayers {
        my_id: our_id,
        res_send,
      })
      .await;
    if x.is_err() {
      panic!("Shouldn't fail!");
    }
    let recvr = res_recv.await;
    if recvr.is_err() {
      let mut disc = disconnect_1.lock().await;
      *disc = true;
      drop(disc);
    }
    loop {
      let disc = disconnect_1.lock().await;
      if *disc {
        break;
      }
      drop(disc);
      let recv = recv.try_recv();
      //println!("Going");
      match recv {
        Ok(msg) => {
          match msg {
            PlayerCommand::SetBlock { block } => {
              let packet = classic::Packet::SetBlockS { block };
              let packet = ClassicPacketWriter::serialize(packet).unwrap();
              let write = writehalf.write_all(&packet).await;
              if write.is_err() {
                let mut disc = disconnect_1.lock().await;
                *disc = true;
                drop(disc);
                break;
              }
              //let packet = ClassicPacketWriter::serialize(packet)?;
            }
            PlayerCommand::SpawnPlayer { position, id, name } => {
              let packet = classic::Packet::SpawnPlayer {
                player_id: id,
                name,
                position,
              };
              let packet = ClassicPacketWriter::serialize(packet).unwrap();
              let write = writehalf.write_all(&packet).await;
              if write.is_err() {
                let mut disc = disconnect_1.lock().await;
                *disc = true;
                drop(disc);
                break;
              }
            }
            PlayerCommand::DespawnPlayer { id } => {
              let packet = classic::Packet::DespawnPlayer { player_id: id };
              let packet = ClassicPacketWriter::serialize(packet).unwrap();
              let write = writehalf.write_all(&packet).await;
              if write.is_err() {
                let mut disc = disconnect_1.lock().await;
                *disc = true;
                drop(disc);
                break;
              }
            }
            PlayerCommand::PlayerTeleport { position, id } => {
              let packet = classic::Packet::PlayerTeleportS {
                player_id: id,
                position: position,
              };
              let packet = ClassicPacketWriter::serialize(packet).unwrap();
              let write = writehalf.write_all(&packet).await;
              if write.is_err() {
                let mut disc = disconnect_1.lock().await;
                *disc = true;
                drop(disc);
                break;
              }
            }
            PlayerCommand::Message { id, message } => {
              let packet = classic::Packet::Message {
                player_id: id,
                message,
              };
              let packet = ClassicPacketWriter::serialize(packet).unwrap();
              let write = writehalf.write_all(&packet).await;
              if write.is_err() {
                let mut disc = disconnect_1.lock().await;
                *disc = true;
                drop(disc);
                break;
              }
            }
            PlayerCommand::Disconnect { reason } => {
              let packet = classic::Packet::Disconnect { reason };
              let packet = ClassicPacketWriter::serialize(packet).unwrap();
              let write = writehalf.write_all(&packet).await;
              if write.is_err() {
                let mut disc = disconnect_1.lock().await;
                *disc = true;
                drop(disc);
                break;
              }
            }
          }
        }
        Err(_) => {
          continue;
        }
      }
    }
  });
  loop {
    let disc = disconnect_2.lock().await;
    if *disc {
      break;
    }
    drop(disc);
    //println!("Started");
    let packet = ClassicPacketReader::read_packet_reader(&mut test).await;
    if packet.is_err() {
      let mut disc = disconnect_2.lock().await;
      *disc = true;
      drop(disc);
      break;
    }
    let packet = packet.unwrap();
    match packet {
      classic::Packet::SetBlockC {
        coords,
        mode,
        block_type,
      } => {
        if mode == 0x00 {
          let block = Block {
            position: coords,
            id: 0x00,
          };
          let (res_send, res_recv) = oneshot::channel();
          let mps = gmts
            .world_send
            .send(WorldCommand::SetBlockP {
              block,
              sender_id: our_id,
              players_send: gmts.players_send.clone(),
              res_send,
            })
            .await;
          if mps.is_err() {
            let mut disc = disconnect_2.lock().await;
            *disc = true;
            drop(disc);
            break;
          }
          let mps = res_recv.await;
          if mps.is_err() {
            let mut disc = disconnect_2.lock().await;
            *disc = true;
            drop(disc);
            break;
          }
          let mps = mps.unwrap();
          if mps.is_none() {
            println!("Error placing block.");
          }
        } else {
          let block = Block {
            position: coords,
            id: block_type,
          };
          let (res_send, res_recv) = oneshot::channel();
          let mps = gmts
            .world_send
            .send(WorldCommand::SetBlockP {
              block,
              sender_id: our_id,
              players_send: gmts.players_send.clone(),
              res_send,
            })
            .await;
          if mps.is_err() {
            let mut disc = disconnect_2.lock().await;
            *disc = true;
            drop(disc);
            break;
          }
          let mps = res_recv.await;
          if mps.is_err() {
            let mut disc = disconnect_2.lock().await;
            *disc = true;
            drop(disc);
            break;
          }
        }
      }
      classic::Packet::PositionAndOrientationC { position, .. } => {
        let (res_send, res_recv) = oneshot::channel();
        let x = gmts
          .players_send
          .send(PlayersCommand::UpdatePosition {
            my_id: our_id,
            position,
            res_send,
          })
          .await;
        if x.is_err() {
          panic!("Shouldn't fail!");
        }
        res_recv.await.unwrap();
      }
      classic::Packet::MessageC { message } => {
        if message.starts_with("/") {
          let command = message.split(" ").collect::<Vec<&str>>();
          log::info!("{} issued command \"{}\"", our_username, command[0]);
          match command[0] {
            "/setblock" => {
              if !op {
                let (res_send2, res_recv2) = oneshot::channel();
                let x = gmts
                  .players_send
                  .send(PlayersCommand::PassMessageToID {
                    id: our_id,
                    message: PlayerCommand::Message {
                      id: -1,
                      message: "&cInsufficient permission.".to_string(),
                    },
                    res_send: res_send2,
                  })
                  .await;
                if x.is_err() {
                  panic!("Shouldn't fail!");
                }
                res_recv2.await.expect("Shouldn't fail!");
              } else {
                if command.len() < 5 {
                  let (res_send2, res_recv2) = oneshot::channel();
                  let x = gmts
                    .players_send
                    .send(PlayersCommand::PassMessageToID {
                      id: our_id,
                      message: PlayerCommand::Message {
                        id: -1,
                        message: "&cUsage: /setblock (x) (y) (z) (id)".to_string(),
                      },
                      res_send: res_send2,
                    })
                    .await;
                  if x.is_err() {
                    panic!("Shouldn't fail!");
                  }
                  res_recv2.await.expect("Shouldn't fail!");
                } else {
                  loop {
                    let x = usize::from_str_radix(command[1], 10);
                    let x = match x {
                      Ok(x) => x,
                      Err(_) => {
                        let (res_send2, res_recv2) = oneshot::channel();
                        let x = gmts
                          .players_send
                          .send(PlayersCommand::PassMessageToID {
                            id: our_id,
                            message: PlayerCommand::Message {
                              id: -1,
                              message: "&cUsage: /setblock (x) (y) (z) (id)".to_string(),
                            },
                            res_send: res_send2,
                          })
                          .await;
                        if x.is_err() {
                          panic!("Shouldn't fail!");
                        }
                        res_recv2.await.expect("Shouldn't fail!");
                        break;
                      }
                    };
                    let y = usize::from_str_radix(command[2], 10);
                    let y = match y {
                      Ok(x) => x,
                      Err(_) => {
                        let (res_send2, res_recv2) = oneshot::channel();
                        let x = gmts
                          .players_send
                          .send(PlayersCommand::PassMessageToID {
                            id: our_id,
                            message: PlayerCommand::Message {
                              id: -1,
                              message: "&cUsage: /setblock (x) (y) (z) (id)".to_string(),
                            },
                            res_send: res_send2,
                          })
                          .await;
                        if x.is_err() {
                          panic!("Shouldn't fail!");
                        }
                        res_recv2.await;
                        break;
                      }
                    };
                    let z = usize::from_str_radix(command[3], 10);
                    let z = match z {
                      Ok(x) => x,
                      Err(_) => {
                        let (res_send2, res_recv2) = oneshot::channel();
                        let x = gmts
                          .players_send
                          .send(PlayersCommand::PassMessageToID {
                            id: our_id,
                            message: PlayerCommand::Message {
                              id: -1,
                              message: "&cUsage: /setblock (x) (y) (z) (id)".to_string(),
                            },
                            res_send: res_send2,
                          })
                          .await;
                        if x.is_err() {
                          panic!("Shouldn't fail!")
                        }
                        res_recv2.await.expect("Shouldn't fail!");
                        break;
                      }
                    };
                    let id = usize::from_str_radix(command[4], 10);
                    let id = match id {
                      Ok(x) => x,
                      Err(_) => {
                        let (res_send2, res_recv2) = oneshot::channel();
                        let x = gmts
                          .players_send
                          .send(PlayersCommand::PassMessageToID {
                            id: our_id,
                            message: PlayerCommand::Message {
                              id: -1,
                              message: "&cUsage: /setblock (x) (y) (z) (id)".to_string(),
                            },
                            res_send: res_send2,
                          })
                          .await;
                        if x.is_err() {
                          panic!("Shouldn't fail!");
                        }
                        res_recv2.await.expect("Shouldn't fail!");
                        break;
                      }
                    };
                    let blockpos = BlockPosition { x, y, z };
                    let block = Block {
                      position: blockpos,
                      id: id as u8,
                    };
                    let (res_send, res_recv) = oneshot::channel();
                    let mps = gmts
                      .world_send
                      .send(WorldCommand::SetBlockP {
                        block,
                        sender_id: our_id,
                        players_send: gmts.players_send.clone(),
                        res_send,
                      })
                      .await;
                    let mps = res_recv.await;
                    if mps.is_err() {
                      let mut disc = disconnect_2.lock().await;
                      *disc = true;
                      drop(disc);
                      break;
                    }
                    break;
                  }
                }
              }
            }
            _ => {
              let (res_send2, res_recv2) = oneshot::channel();
              gmts
                .players_send
                .send(PlayersCommand::PassMessageToID {
                  id: our_id,
                  message: PlayerCommand::Message {
                    id: -1,
                    message: "&cUnknown command.".to_string(),
                  },
                  res_send: res_send2,
                })
                .await;
              res_recv2.await;
            }
          }
        } else {
          let prefix = format!("<{}> ", our_username);
          let index = std::cmp::min(message.len(), 64 - prefix.len());
          send_message(
            &format!("{}{}", prefix, &message[0..index]),
            (our_id as u8) as i8,
            &gmts,
          )
          .await;
          if message.len() > index {
            send_message(
              &format!("> {}", &message[index..]),
              (our_id as u8) as i8,
              &gmts,
            )
            .await;
          }
        }
        /*         let message = PlayerCommand::Message { id: (our_id as u8) as i8, message};
        gmts.players_send.send(PlayersCommand::PassMessageToAll { message, res_send }).await;
        res_recv.await.unwrap(); */
      }
      _ => {}
    }
  }
  let (res_send, res_recv) = oneshot::channel();
  gmts
    .players_send
    .send(PlayersCommand::RemoveUser {
      user_id: our_id,
      res_send,
    })
    .await;
  res_recv.await.unwrap();
  let (res_send, res_recv) = oneshot::channel();
  gmts
    .tempcrntid_send
    .send(TempCrntIdCommand::ReturnFreeID {
      id: our_id,
      res_send,
    })
    .await;
  res_recv.await.unwrap();
  log::info!("{} left the game.", our_username);
  send_message(&format!("&e{} left the game.", our_username), -1, &gmts).await;
  Ok(())
}
async fn send_message(message: &str, id: i8, gmts: &GMTS) {
  log::info!("[CHAT]: {}", message);
  let (res_send, res_recv) = oneshot::channel();
  let message = PlayerCommand::Message {
    id: (id as u8) as i8,
    message: message.to_string(),
  };
  gmts
    .players_send
    .send(PlayersCommand::PassMessageToAll { message, res_send })
    .await;
  res_recv.await.unwrap();
}
async fn pass_message_to_all(message: PlayerCommand, gmts: &GMTS) {
  let (res_send, res_recv) = oneshot::channel();
  gmts
    .players_send
    .send(PlayersCommand::PassMessageToAll { message, res_send })
    .await;
  res_recv.await.unwrap();
}
async fn get_world(gmts: &GMTS) -> Result<World, Box<dyn std::error::Error>> {
  let (res_send, res_recv) = oneshot::channel();
  gmts
    .world_send
    .send(WorldCommand::GetWorld { res_send })
    .await;
  return Ok(res_recv.await?);
}
/*  async fn example(gmts: &GMTS) {
  let my_world_send = gmts.world_send.clone();
  let (res_send, res_recv) = oneshot::channel();
  my_world_send.send(WorldCommand::GetBlock {
    pos: BlockPosition {x: 20, y: 40, z: 60},
    res_send,
  });
  let block = res_recv.await.unwrap();
  // ...
} */
fn setup_gmts() -> GMTS {
  // Initialize World Managing Task
  let (world_send, mut recv) = mpsc::channel::<WorldCommand>(10);
  tokio::spawn(async move {
    //let generator = FlatWorldGenerator::new(64, BlockIds::SPONGE, BlockIds::SPONGE, BlockIds::AIR);
    //let mut world = World::new(generator, 128, 128, 128);
    let mut world = World::from_file();
    log::info!("Finished initializing world");
    loop {
      match recv.recv().await.unwrap() {
        WorldCommand::GetBlock { pos, res_send } => {
          let id = world.get_block(pos.x, pos.y, pos.z);
          if id.is_none() {
            res_send.send(None).expect("Shouldn't fail");
          } else {
            let block = Block {
              position: pos,
              id: id.unwrap(),
            };
            res_send.send(Some(block));
          }
        }
        WorldCommand::SetBlock {
          mut block,
          res_send,
        } => {
          block.position.x += 4;
          res_send.send(world.set_block(block));
        }
        WorldCommand::GetWorld { res_send } => {
          match res_send.send(world.clone()) {
            Err(e) => {
              panic!("Shouldn't fail!");
            }
            Ok(()) => {

            }
          }
        }
        WorldCommand::SetBlockP {
          mut block,
          sender_id,
          players_send,
          res_send,
        } => {
            //let distance = coords.distance_to(block.position.clone());
            if true == false {
/*               println!("Reach hacking! Distance: {:?}", distance);
              let (res_send2, res_recv2) = oneshot::channel();
              let could = world.get_block(block.position.x, block.position.y, block.position.z);
              if could.is_none() {
                res_send.send(None).expect("Shouldn't fail");
              } else {
                block.id = could.unwrap();
                let x = players_send
                  .send(PlayersCommand::PassMessageToID {
                    id: sender_id,
                    message: PlayerCommand::SetBlock { block },
                    res_send: res_send2,
                  })
                  .await;
                if x.is_err() {
                  panic!("Shouldn't fail");
                }
                res_recv2.await.expect("Shouldn't fail!");
                res_send.send(Some(())).expect("Shouldn't fail!");
              } */
            } else {
              block.position.x += 4;
              let x = world.set_block(block.clone());
              match x {
                Some(_) => {
                  block.position.x -= 4;
                  let (res_send2, res_recv2) = oneshot::channel();
                  let x = players_send
                    .send(PlayersCommand::PassMessageToAll {
                      message: PlayerCommand::SetBlock { block },
                      res_send: res_send2,
                    })
                    .await;
                  if x.is_err() {
                    panic!("Shouldn't fail!");
                  }
                  res_recv2.await.expect("Shouldn't fail");
                  res_send.send(Some(())).expect("Shouldn't fail");
                }
                None => {
                  res_send.send(None).expect("Shouldn't fail");
                }
              }
            }
        }
      }
    }
  });
  // Initialize Players Managing Task
  let (players_send, mut recv) = mpsc::channel::<PlayersCommand>(10);
  tokio::spawn(async move {
    let mut players: HashMap<u32, Player> = HashMap::new();
    let mut user_ids: HashMap<u32, String> = HashMap::new();
    loop {
      match recv.recv().await.unwrap() {
        PlayersCommand::NewUser { user, pos, res_send } => {
          let id = user.id;
          let name = user.name.clone();
          for player in &mut players {
            let x = player.1.message_send.send(PlayerCommand::SpawnPlayer {
              position: pos.clone(),
              id: (id as u8) as i8,
              name: name.clone(),
            });
            if x.is_err() {
              println!("Shouldn't fail");
            }
          }
          user_ids.insert(id, name);
          players.insert(id, user);
          res_send.send(()).expect("Shouldn't fail");
        }
        PlayersCommand::RemoveUser { user_id, res_send } => {
          players.remove(&user_id);
          user_ids.remove(&user_id);
          for player in &mut players {
            player.1.message_send.send(PlayerCommand::DespawnPlayer {
              id: (user_id as u8) as i8,
            });
          }
          res_send.send(());
        }
        PlayersCommand::PassMessageToAll { message, res_send } => {
          for player in &mut players {
            player.1.message_send.send(message.clone());
          }
          res_send.send(());
        } // ...
        PlayersCommand::SpawnAllPlayers { my_id, res_send } => {
          let us = players.get(&my_id);
          if us.is_none() {
            res_send.send(());
          } else {
            let us = us.unwrap();
            for player in &players {
              if player.1.id != us.id {
                us.message_send.send(PlayerCommand::SpawnPlayer {
                  position: player.1.data.position.clone(),
                  id: (player.1.id as u8) as i8,
                  name: player.1.name.clone(),
                });
              } else {
                us.message_send.send(PlayerCommand::SpawnPlayer {
                  position: player.1.data.position.clone(),
                  id: -1,
                  name: player.1.name.clone(),
                });
              }
            }
            res_send.send(());
          }
        }
        PlayersCommand::IsOperator { id, res_send } => {
          let us = players.get(&id);
          if us.is_none() {
            res_send.send(false);
          } else {
            let us = us.unwrap();
            res_send.send(us.op);
          }
        }
        PlayersCommand::UpdatePosition {
          my_id,
          position,
          res_send,
        } => {
          let us = players.get_mut(&my_id);
          if us.is_none() {
            res_send.send(None).expect("Shouldn't fail");
          } else {
            let us = us.unwrap();
            let distance = us.data.position.distance_to_plr(position.clone());
            if true == false {
              println!("{} moved wrongly!", us.name);
              let x = us.message_send.send(PlayerCommand::PlayerTeleport {
                position: us.data.position.clone(),
                id: -1,
              });
              if x.is_err() {
                panic!("Shouldn't fail!");
              }
              res_send.send(None).expect("Shouldn't fail");
            } else {
              us.data.position = position.clone();
              let id = us.id.clone();
              drop(us);
              for player in &players {
                if player.1.id != id {
                  let x = player.1.message_send.send(PlayerCommand::PlayerTeleport {
                    position: position.clone(),
                    id: (my_id as u8) as i8,
                  });
                  if x.is_err() {
                    panic!("Shouldn't fail!");
                  }
                }
              }
              res_send.send(Some(())).expect("Shouldn't fail");
            }
          }
        }
        PlayersCommand::UpdatePositionUnchecked { my_id, position, res_send } => {
          let us = players.get_mut(&my_id);
          if us.is_none() {
            res_send.send(None).expect("Shouldn't fail");
          } else {
            let us = us.unwrap();
            us.data.position = position.clone();
            let id = us.id.clone();
            drop(us);
            for player in &players {
              if player.1.id != id {
                let x = player.1.message_send.send(PlayerCommand::PlayerTeleport {
                  position: position.clone(),
                  id: (my_id as u8) as i8,
                });
                if x.is_err() {
                  panic!("Shouldn't fail!");
                }
              }
            }
            res_send.send(Some(())).expect("Shouldn't fail");
          }
        }
        PlayersCommand::GetPosition { id, res_send } => {
          let user = players.get(&id);
          if user.is_none() {
            res_send.send(None);
          } else {
            let user = user.unwrap();
            res_send.send(Some(user.data.position.clone()));
          }
        }
        PlayersCommand::PassMessageToID {
          id,
          message,
          res_send,
        } => {
          let user = players.get(&id);
          if user.is_none() {
            res_send.send(None).expect("Shouldn't fail");
          } else {
            let user = user.unwrap();
            user.message_send.send(message).expect("Shouldn't fail");
            res_send.send(Some(())).expect("Shouldn't fail");
          }
        }
        PlayersCommand::KickUserByName {
          username,
          reason,
          res_send,
        } => {
          let mut f = false;
          for (id, user) in &user_ids {
            if user == &username {
              let user = players.get(&id).unwrap();
              user.message_send.send(PlayerCommand::Disconnect { reason }).expect("Shouldn't fail");
              f = true;
              break;
            }
          }
          if f {
            res_send.send(Some(())).expect("Shouldn't fail");
          } else {
            res_send.send(None).expect("Shouldn't fail");
          }
        }
      }
    }
  });
  // Initialize Temp Crnt Id Managing Task
  let (temp_crnt_id_send, mut recv) = mpsc::channel::<TempCrntIdCommand>(10);
  tokio::spawn(async move {
    let mut ids = vec![0; 127];
    for i in 0..127 {
      ids[i] = i;
    }
    loop {
      match recv.recv().await.unwrap() {
        TempCrntIdCommand::FetchFreeID { res_send } => {
          res_send.send(ids.pop().unwrap() as u32).expect("Shouldn't fail");
        } // ...
        TempCrntIdCommand::ReturnFreeID { id, res_send } => {
          ids.push(id as usize);
          res_send.send(()).expect("Shouldn't fail");
        }
      }
    }
  });
  let (heartbeat_send, mut recv) = mpsc::channel::<HeartbeatCommand>(10);
  tokio::spawn(async move {
    use rand::rngs::OsRng;
    use rand::RngCore;
    let mut bytes: Vec<u8> = vec![0; 15];
    let mut rng = OsRng::new().unwrap();
    rng.fill_bytes(&mut bytes);
    let salt = base_62::encode(&bytes);
    let salt2 = salt.clone();
    let heartbeat_thread = tokio::spawn(async move {
      loop {
        break;
/*         use std::io::{Read, Write};
        let request = format!("GET /heartbeat.jsp?port=25565&max=10&name=Epic%20Server&public=True&version=7&salt={salt}&users=0 HTTP/1.1\r\nHost: www.classicube.net\r\nConnection: close\r\n\r\n", salt = salt2);
        extern crate native_tls;
        use native_tls::TlsConnector;
        let connector = TlsConnector::new().unwrap();
        let mut tlsstream = std::net::TcpStream::connect("classicube.net:443").unwrap();
        let mut tlsstream = connector.connect("classicube.net", tlsstream).unwrap();
        tlsstream.write_all(request.as_bytes()).unwrap();
        let mut buf = vec![];
        tlsstream.read_to_end(&mut buf).unwrap();
        println!("Response: {:?}", String::from_utf8_lossy(&buf).to_string());
        tokio::time::sleep(tokio::time::Duration::from_secs(45)).await; */
      }
    });
    loop {
      match recv.recv().await.unwrap() {}
    }
  });

  GMTS {
    world_send,
    players_send,
    tempcrntid_send: temp_crnt_id_send,
    heartbeat_send,
  }
}

/* ================================================ classic.rs ================================================ */
pub mod classic {
  use super::{Block, BlockPosition, PlayerPosition};
  // common:               p_ver           name             string        isop, client just says theyre not op
  // it's unused, doesn't need to be there
  // anything marked as "unused" is a reverse engineering byproduct, it should more accurately be:
  // "unchanging" as that's what they observed to give it the name of "unused"
  // and an unchanged permission is valid.
  pub enum Packet {
    PlayerIdentification {
      p_ver: u8,
      user_name: String,
      v_key: String, /*technically, not going to leave it in, just for a point*/
    },
    ServerIdentification {
      p_ver: u8,
      server_name: String,
      motd: String,
      is_op: u8,
    },
    LevelInitialize,
    LevelDataChunk {
      chunk_length: i16,
      chunk_data: Box<[u8]>,
      percent_complete: u8,
    },
    LevelFinalize {
      width: usize,
      height: usize,
      length: usize,
    },
    PositionAndOrientationC {
      player_id: u8,
      position: PlayerPosition,
    },
    SetBlockC {
      coords: BlockPosition,
      mode: u8,
      block_type: u8,
    },
    MessageC {
      message: String,
    },
    SetBlockS {
      block: Block,
    },
    PlayerTeleportS {
      player_id: i8,
      position: PlayerPosition,
    },
    SpawnPlayer {
      player_id: i8,
      name: String,
      position: PlayerPosition,
    },
    DespawnPlayer {
      player_id: i8,
    },
    Message {
      player_id: i8,
      message: String,
    },
    Disconnect {
      reason: String,
    },
  }
}

// I reserve the right to change the signedness of a value when negatives have no purpose, and the name of packets to something of equivalent meaning
// because of the reasons stated in reserve_f_u_exo.txt
/*
+------------+------+--------+-------
|    Type    | Size |  Rust  | Note
+------------+------+--------+-------
| Byte       | 1    |  u8    | Used when needed, in java values are stored in shorts, to avoid signedness.
| SByte      | 1    |  i8    | Single byte signed integer (-128 to 127) == i8
| Short      | 2    |  u16   | Unsigned integer, network order (BE)
| SShort     | 2    |  i16   | Signed integer, network order (BE)
| String     | 64   |  &str  | US-ASCII/ISO646-US encoded string padded with spaces (0x20) == String
| Byte array | 1024 |  &[u8] | Binary data padded with null bytes (0x00) == &[u8]
*/

// who classicoomer
// ======= Old packet enums =======
/*
/// All client packets
pub enum ClassicPacketClient {
  PlayerIdentification {protocol_ver: u8, username: String, verification_key: String},
  PositionAndOrientation {player_id: u8, position: PositionYP},
  SetBlock {coords: Position, mode: u8, block_type: u8},
  Message {message: String},
  Other,
}


/// All server packets
#[derive(Clone)]
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
*/

// put seiralize below here
// copy and pasted the old one LOL it works fine
use std::pin::Pin;
use tokio::io::AsyncReadExt;
pub struct ClassicPacketWriter {}
impl ClassicPacketWriter {
  pub fn serialize(packet: classic::Packet) -> std::io::Result<Vec<u8>> {
    match packet {
      classic::Packet::LevelInitialize => {
        let builder = ClassicPacketBuilder::new();
        return builder.build(0x02);
      }
      classic::Packet::LevelDataChunk {
        chunk_length,
        chunk_data,
        percent_complete,
      } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_short(chunk_length as i16);
        builder.insert_bytearray(chunk_data.to_vec());
        builder.insert_byte(percent_complete);
        return builder.build(0x03);
      }
      classic::Packet::LevelFinalize {
        width,
        height,
        length,
      } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_short(width as i16);
        builder.insert_short(height as i16);
        builder.insert_short(length as i16);
        return builder.build(0x04);
      }
      classic::Packet::SetBlockS { block } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_short(block.position.x as i16);
        builder.insert_short(block.position.y as i16);
        builder.insert_short(block.position.z as i16);
        builder.insert_byte(block.id);
        return Ok(builder.build(0x06)?);
      }
      classic::Packet::PlayerTeleportS {
        player_id,
        position,
      } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        builder.insert_short(position.x as i16);
        builder.insert_short(position.y as i16);
        builder.insert_short(position.z as i16);
        builder.insert_byte(position.yaw);
        builder.insert_byte(position.pitch);
        return Ok(builder.build(0x08)?);
      }
      classic::Packet::SpawnPlayer {
        player_id,
        name,
        position,
      } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        builder.insert_string(&name);
        builder.insert_short(position.x as i16);
        builder.insert_short(position.y as i16);
        builder.insert_short(position.z as i16);
        builder.insert_byte(position.yaw);
        builder.insert_byte(position.pitch);
        return Ok(builder.build(0x07)?);
      }
      classic::Packet::DespawnPlayer { player_id } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        return Ok(builder.build(0x0c)?);
      }
      classic::Packet::Message { player_id, message } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_sbyte(player_id);
        builder.insert_string(&message);
        return Ok(builder.build(0x0d)?);
      }
      classic::Packet::Disconnect { reason } => {
        let mut builder = ClassicPacketBuilder::new();
        builder.insert_string(&reason);
        return Ok(builder.build(0x0e)?);
      }
      _ => {
        return Err(std::io::Error::new(
          ErrorKind::Other,
          format!("Unknown packet!"),
        ));
      }
    }
  }
  pub fn server_identification(
    protocol_ver: u8,
    server_name: String,
    motd: String,
    is_op: bool,
  ) -> std::io::Result<Vec<u8>> {
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
  pub fn serialize_vec(vec: Vec<classic::Packet>) -> std::io::Result<Vec<Vec<u8>>> {
    let mut vec2 = vec![];
    for packet in vec {
      match packet {
        classic::Packet::LevelInitialize => {
          let builder = ClassicPacketBuilder::new();
          vec2.push(builder.build(0x02)?);
        }
        classic::Packet::LevelDataChunk {
          chunk_length,
          chunk_data,
          percent_complete,
        } => {
          let mut builder = ClassicPacketBuilder::new();
          builder.insert_short(chunk_length as i16);
          builder.insert_bytearray(chunk_data.to_vec());
          builder.insert_byte(percent_complete);
          vec2.push(builder.build(0x03)?);
        }
        classic::Packet::LevelFinalize {
          width,
          height,
          length,
        } => {
          let mut builder = ClassicPacketBuilder::new();
          builder.insert_short(width as i16);
          builder.insert_short(height as i16);
          builder.insert_short(length as i16);
          vec2.push(builder.build(0x04)?);
        }
        _ => {
          return Err(std::io::Error::new(
            ErrorKind::Other,
            format!("Unknown packet"),
          ));
        }
      }
    }
    Ok(vec2)
  }
}
pub struct ClassicPacketReader {}
impl ClassicPacketReader {
  pub async fn read_packet_reader<'a>(
    reader: &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>,
  ) -> std::io::Result<classic::Packet> {
    let id = ClassicPacketUtils::read_byte(reader).await?;
    match id {
      0x00 => {
        let protocol_ver = ClassicPacketUtils::read_byte(reader).await?;
        let username = ClassicPacketUtils::read_string(reader).await?;
        let verification_key = ClassicPacketUtils::read_string(reader).await?;
        let unused = ClassicPacketUtils::read_byte(reader).await?;
        drop(unused);
        let packet = classic::Packet::PlayerIdentification {
          p_ver: protocol_ver,
          user_name: username,
          v_key: verification_key,
        };
        return Ok(packet);
      }
      0x08 => {
        let pid = ClassicPacketUtils::read_byte(reader).await?;
        let x = ClassicPacketUtils::read_short(reader).await?;
        let y = ClassicPacketUtils::read_short(reader).await?;
        let z = ClassicPacketUtils::read_short(reader).await?;
        let yaw = ClassicPacketUtils::read_byte(reader).await?;
        let pitch = ClassicPacketUtils::read_byte(reader).await?;
        let coords = PlayerPosition {
          x: x as u16,
          y: y as u16,
          z: z as u16,
          yaw: yaw,
          pitch: pitch,
        };
        let packet = classic::Packet::PositionAndOrientationC {
          player_id: pid,
          position: coords,
        };
        return Ok(packet);
      }
      0x05 => {
        let x = ClassicPacketUtils::read_short(reader).await?;
        let y = ClassicPacketUtils::read_short(reader).await?;
        let z = ClassicPacketUtils::read_short(reader).await?;
        let mode = ClassicPacketUtils::read_byte(reader).await?;
        let blocktype = ClassicPacketUtils::read_byte(reader).await?;
        let coords = BlockPosition {
          x: x as usize,
          y: y as usize,
          z: z as usize,
        };
        let packet = classic::Packet::SetBlockC {
          coords: coords,
          mode: mode,
          block_type: blocktype,
        };
        return Ok(packet);
      }
      0x0d => {
        let x = ClassicPacketUtils::read_byte(reader).await?;
        drop(x);
        let message = ClassicPacketUtils::read_string(reader).await?;
        let packet = classic::Packet::MessageC { message: message };
        return Ok(packet);
      }
      id => {
        return Err(std::io::Error::new(
          ErrorKind::Other,
          format!("Unknown packet id {}!", id),
        ));
      }
    }
  }
}
pub struct ClassicPacketUtils {}
impl ClassicPacketUtils {
  async fn read_byte<'a>(
    reader: &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>,
  ) -> std::io::Result<u8> {
    let mut byte = [0; 1];
    reader.read_exact(&mut byte).await?;
    return Ok(byte[0]);
  }
  async fn read_short<'a>(
    reader: &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>,
  ) -> std::io::Result<i16> {
    let mut byte = [0; 2];
    reader.read_exact(&mut byte).await?;
    let short = i16::from_be_bytes(byte);
    return Ok(short);
  }
  async fn read_string<'a>(
    reader: &mut Pin<Box<impl tokio::io::AsyncRead + 'a>>,
  ) -> std::io::Result<String> {
    let mut byte = [0; 64];
    reader.read_exact(&mut byte).await?;
    let string = String::from_utf8_lossy(&byte).to_string();
    return Ok(string.trim_matches(char::from(0x20)).to_string());
  }
}
use std::io::ErrorKind;
#[derive(Clone)]
pub enum Element {
  Byte { byte: u8 },
  SByte { byte: i8 },
  StringElement { string: String },
  Short { short: i16 },
  Bytes { bytes: Vec<u8> },
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
    self.elements.push(Element::Bytes { bytes: bytes });
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
            return Err(std::io::Error::new(ErrorKind::Other, "String too large!"));
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
            return Err(std::io::Error::new(ErrorKind::Other, "Bytes too large!"));
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

/* ============================================================================== ALERT: GALAXTONE IS GAY ==============================================================================
/* Section [Bumboy] Start */
// have names in it, for easy ctrl+F
/* Section [Bumboy] End */

// hey can we keep things in one file and split it up afterwards
// i guess, might be ugly for now

// game struct will house global variables and elements that all threads can get at

// "commands" to the socket might as well be the packets themselves
// it's implicit that it writes what it receives and mspc<T>, T takes ANYTHING not just enums
// I'll add an enum for packets, and leave Bytes to the serialization department

// Vec(s)? or ClassicPacketServer(s)
pub fn main() {

  let (my_sender, my_receiver) = oneshot::new();
  player_manager_sender.send(GetCoords { player_id: 5, sender: my_sender })

  pub enum ExampleCommand {
    use tokio::sync::oneshot::Sender;
    GetCoords {
      player_id: u32,
      sender: Sender,
    },
    GetName {
      player_id: u32,
      sender: Sender,
    }
  }

  tokio::task::spawn(async {
    // Future gremlin magic is in effect here.
  });
}
*/
