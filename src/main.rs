#![allow(dead_code)]

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use std::error::Error;
use std::result;
type Result<T> = result::Result<T, Box<dyn Error + Send + Sync>>;
const IP: &str = "0.0.0.0:25565";

mod network;
use network::classic;

use classic::chunks::{FlatWorldGenerator, World};
use classic::BlockIds;
use tokio::io::AsyncWriteExt;

//use classic::heartbeat;
use classic::{
  Block, ClassicPacketReader, ClassicPacketServer, /*ClassicPacketBuilder, */ Position,
  PositionYP,
};
#[derive(Clone, Debug)]
pub struct Message {
  pub message: String,
  pub system: bool,
}
// TODO Expand player struct
pub struct Player {
  pub name: String,
  pub position: PositionYP,
  pub operator: bool,
  pub block_changes: Vec<Block>,
  pub chatbox: Vec<Message>,
  pub incoming_packets: Vec<ClassicPacketServer>,
}
pub struct LocalPlayer {
  pub player: Arc<Mutex<Player>>,
  pub id: i8,
}

// TODO Everything.
#[derive(Clone)]
pub struct Game {
  players: Arc<Mutex<Vec<Arc<Mutex<Player>>>>>,
  ops: Arc<Mutex<Vec<String>>>,
  world: Arc<Mutex<World>>,
}

#[tokio::main]
async fn main() -> Result<()> {
  let generator = FlatWorldGenerator::new(64, BlockIds::DIRT, BlockIds::GRASS, BlockIds::AIR);
  let world = World::new(generator, 128, 128, 128);
  let game = Game {
    players: Arc::new(Mutex::new(vec![])),
    ops: Arc::new(Mutex::new(vec![
      "Exopteron".to_string(),
      "Galaxtone".to_string(),
    ])),
    world: Arc::new(Mutex::new(world)),
  };

  /*   // Heartbeat Thread
  tokio::spawn(async {
    let duration = Duration::from_secs(45);
    loop {
      println!("Heartbeat!");
      heartbeat::heartbeat().await?;
      time::sleep(duration).await;
    }
  }); */

  listen(game).await.unwrap();
  Ok(())
}

pub async fn listen(game: Game) -> Result<()> {
  let listener = TcpListener::bind(IP).await?;
  println!("Listening on {}", IP);
  loop {
    let (socket, address) = listener.accept().await.unwrap();
    println!("Connection {:?}", address);
    let game = game.clone();
    tokio::spawn(async move {
      process(socket, game).await.unwrap();
    });
  } // im looking it up
}

// Ok uh figure out something about state-keeping
// during handshake
#[derive(Debug)]
pub struct ConnectingUser {
  pub username: String,
  pub protocol_version: u8,
  pub verification_key: String,
}

use std::fmt::{self, Debug, Display, Formatter};

pub struct OurError(String);

impl OurError {
  pub fn new(s: impl Into<String>) -> Self {
    Self(s.into())
  }
}
impl Debug for OurError {
  fn fmt(&self, f: &mut Formatter<'_>) -> result::Result<(), fmt::Error> {
    Debug::fmt(&self.0, f)
  }
}

impl Display for OurError {
  fn fmt(&self, f: &mut Formatter<'_>) -> result::Result<(), fmt::Error> {
    Display::fmt(&self.0, f)
  }
}

impl Error for OurError {}

// do this in another module, like classic/chunks
async fn world_data() {
  // quick and dirty
}
async fn process(mut socket: TcpStream, game: Game) -> Result<()> {
  let mut test = Box::pin(&mut socket);
  let packet = ClassicPacketReader::read_packet_reader(&mut test).await?;
  let mut user = ConnectingUser {
    username: String::default(),
    protocol_version: 0,
    verification_key: String::default(),
  };
  match packet {
    classic::ClassicPacketClient::PlayerIdentification {
      protocol_ver,
      username,
      verification_key,
    } => {
      user.username = username;
      let players = game.players.lock().await;
      let mut already_logged = false;
      for i in 0..players.len() {
        let lp = players[i].lock().await;
        if (*lp).name == user.username {
          let packet = classic::ClassicPacketServer::DisconnectPlayer {
            reason: "Already logged in!".to_string(),
          };
          socket
            .write_all(&classic::ClassicPacketServer::serialize(packet)?)
            .await?;
          already_logged = true;
        }
        drop(lp);
      }
      drop(players);
      if already_logged == true {
        return Err(Box::new(OurError::new("Already logged in!")));
      }
      user.protocol_version = protocol_ver;
      user.verification_key = verification_key;
      println!(
        "{} connecting from [{}]",
        user.username,
        socket.peer_addr()?.to_string()
      );
    }
    _ => {
      return Err(Box::new(OurError::new("Wrong packet!")));
    }
  }
  let ops = game.ops.lock().await;
  let mut isop = false;
  for op in &*ops {
    if op == &user.username {
      isop = true;
      break;
    }
  }
  drop(ops);
  let server_identification = classic::ClassicPacketServer::server_identification(
    0x07,
    "Ballland".to_string(),
    "a really good motd".to_string(),
    isop,
  )?;
  socket.write_all(&server_identification).await?;
  let world = game.world.lock().await;
  let packets = classic::ClassicPacketServer::serialize_vec(world.to_packets())?;
  drop(world);
  for i in 0..packets.len() {
    socket.write_all(&packets[i]).await?;
  }

  // Custom position relative to center of map!
  let player = Player {
    name: user.username.clone(),
    position: PositionYP::from_pos(128 / 2, 64, 128 / 2),
    operator: isop,
    block_changes: Vec::new(),
    chatbox: Vec::new(),
    incoming_packets: Vec::new(),
  };

  let player = Arc::new(Mutex::new(player));
  let player_main = player.clone();
  let game_main = game.clone();
  let x = player.clone();
  let mut players = game.players.lock().await;
  players.push(x);
  drop(players);
  let ourplayer = player.lock().await;
  let spawn_player = classic::ClassicPacketServer::SpawnPlayer {
    player_id: -1,
    name: ourplayer.name.clone(),
    position: ourplayer.position.clone(),
  };
  let teleport_player = classic::ClassicPacketServer::PlayerTeleport {
    player_id: -1,
    position: ourplayer.position.clone(),
  };
  socket
    .write_all(&classic::ClassicPacketServer::serialize(spawn_player)?)
    .await?;
  socket
    .write_all(&classic::ClassicPacketServer::serialize(teleport_player)?)
    .await?;
  drop(ourplayer);
  /* just replace it with an actual event loop */
  let (mut readhalf, mut writehalf) = socket.into_split();
  let game2 = game.clone();
  let player2 = player.clone();
  let disconnect_1 = Arc::new(Mutex::new(false));
  let disconnect_2 = disconnect_1.clone();
  let writehandle = tokio::spawn(async move {
/*     let player = player2.lock().await;
    let ourname = (*player).name.clone();
    drop(player);
    let message = format!("&e{} joined the game.", ourname);
    let x = game2.players.lock().await;
    let players = x.clone();
    drop(x);
    for i in 0..players.len() {
      let mut lockedplayer = players[i].lock().await;
      lockedplayer.chatbox.push(Message {
        message: message.clone(),
        system: true,
      });
      drop(lockedplayer);
    } */
    let mut players_to_render: Vec<Arc<Mutex<Player>>> = vec![];
    let mut currently_rendering: Vec<LocalPlayer> = vec![];
    let mut free_ids = vec![0; 127];
    for i in 0..127 {
      free_ids[i] = i as i8;
    }
    loop {
      let disconnect = disconnect_1.lock().await;
      if *disconnect {
        drop(disconnect);
        return;
      }
      drop(disconnect);
      let mut player = player2.lock().await;
      // Block change loop
      for _ in 0..player.block_changes.len() {
        let change = player.block_changes.pop().unwrap();
        let packet = classic::ClassicPacketServer::SetBlock {
          block: change.clone(),
        };
        let write = writehalf
          .write_all(&classic::ClassicPacketServer::serialize(packet).unwrap())
          .await;
        if write.is_err() {
          let mut disconnect = disconnect_1.lock().await;
          *disconnect = true;
          drop(disconnect);
          break;
        }
      }
      // Incoming packet loop
/*       for _ in 0..player.incoming_packets.len() {
        let packet = player.incoming_packets.pop().unwrap();
        let write = writehalf
          .write_all(&classic::ClassicPacketServer::serialize(packet).unwrap())
          .await;
        if write.is_err() {
          let mut disconnect = disconnect_1.lock().await;
          *disconnect = true;
          drop(disconnect);
          break;
        }
      } */
      let ourname = (*player).name.clone();
      drop(player);
      let players = game2.players.lock().await;

      // New Player rendering loop
      for i in 0..players.len() {
        let lockedplayer = players[i].try_lock();
        if lockedplayer.is_err() {
          continue;
        }
        let lockedplayer = lockedplayer.unwrap();
        let lpname = (*lockedplayer).name.clone();
        drop(lockedplayer);
        if lpname != ourname {
          let us = player2.lock().await;
          let mut dorender = true;
          for i in 0..players_to_render.len() {
            let lockedplayer2 = players_to_render[i].lock().await;
            if lpname == (*lockedplayer2).name {
              dorender = false;
              drop(lockedplayer2);
              break;
            }
            drop(lockedplayer2);
          }
          if dorender == true {
            players_to_render.push(players[i].clone());
          }
          drop(us);
        }
      }
      drop(players);
      // Player culling loop
      for i in 0..players_to_render.len() {
        let player = players_to_render[i].lock().await;
        let name = (*player).name.clone();
        drop(player);
        let players = game2.players.lock().await;
        let allplrs = players.clone();
        drop(players);
        let mut remove = true;
        for i in 0..allplrs.len() {
          if (*allplrs[i].lock().await).name == name {
            remove = false;
          }
        }
        if remove == true {
          for i in 0..players_to_render.len() {
            let player = players_to_render[i].clone();
            let player = player.lock().await;
            if (*player).name.clone() == name {
              players_to_render.remove(i);
            }
            drop(player);
          }
          let mut id = 0;
          for i in 0..currently_rendering.len() {
            let player = currently_rendering[i].player.clone();
            let player = player.lock().await;
            if (*player).name.clone() == name {
              id = currently_rendering[i].id;
              currently_rendering.remove(i);
            }
            drop(player);
          }
          let packet = classic::ClassicPacketServer::DespawnPlayer { player_id: id };
          free_ids.push(id);
          let write = writehalf
            .write_all(&classic::ClassicPacketServer::serialize(packet).unwrap())
            .await;
          if write.is_err() {
            let mut disconnect = disconnect_1.lock().await;
            *disconnect = true;
            drop(disconnect);
            break;
          }
        }
      }
      // Player spawning loop
      for i in 0..players_to_render.len() {
        let player = players_to_render[i].try_lock();
        if player.is_err() {
          continue;
        }
        let player = player.unwrap();
        let name = (*player).name.clone();
        let position = (*player).position.clone();
        drop(player);
        let mut dorender = true;
        for i in 0..currently_rendering.len() {
          if (*currently_rendering[i].player.lock().await).name == name {
            dorender = false;
            break;
          }
        }
        if dorender == false {
          continue;
        }
        let newid = free_ids.pop().unwrap();
        let packet = classic::ClassicPacketServer::SpawnPlayer {
          player_id: newid,
          name: name,
          position: position,
        };
        let write = writehalf
          .write_all(&classic::ClassicPacketServer::serialize(packet).unwrap())
          .await;
        currently_rendering.push(LocalPlayer {
          player: players_to_render[i].clone(),
          id: newid,
        });
        if write.is_err() {
          let mut disconnect = disconnect_1.lock().await;
          *disconnect = true;
          drop(disconnect);
          break;
        }
      }
      // Other player movement loop
      for i in 0..currently_rendering.len() {
        let player = currently_rendering[i].player.clone();
        let player = player.try_lock();
        if player.is_err() {
          continue;
        }
        let player = player.unwrap();
        let id = currently_rendering[i].id;
        let position = (*player).position.clone();
        drop(player);
        let packet = classic::ClassicPacketServer::PlayerTeleport {
          player_id: id,
          position: position,
        };
        let write = writehalf
          .write_all(&classic::ClassicPacketServer::serialize(packet).unwrap())
          .await;
        if write.is_err() {
          let mut disconnect = disconnect_1.lock().await;
          *disconnect = true;
          drop(disconnect);
          break;
        }
      }

      // Chat loop
      let mut player = player2.lock().await;
      for i in 0..(*player).chatbox.len() {
        let messageclone = (*player).chatbox.pop().unwrap();
        let mut id = 0;
        if messageclone.system == false {
          let messageclone = messageclone.message.split(" ").collect::<Vec<&str>>();
          let sender = messageclone[0];
          let mut chars = sender.chars();
          chars.next();
          chars.next_back();
          let sender = chars.as_str();
          let ourname = (*player).name.clone();
          if ourname == sender {
            id = 0;
          } else {
            for i in 0..currently_rendering.len() {
              if (*currently_rendering[i].player.lock().await).name == sender {
                id = currently_rendering[i].id;
                break;
              }
            }
          }
        } else {
          id = -1;
        }
        let packet = classic::ClassicPacketServer::Message {
          player_id: id,
          message: messageclone.message,
        };
        let write = writehalf
          .write_all(&classic::ClassicPacketServer::serialize(packet).unwrap())
          .await;
        if write.is_err() {
          let mut disconnect = disconnect_1.lock().await;
          *disconnect = true;
          drop(disconnect);
          break;
        }
      }
      drop(player);
    }
  });
  let readhandle = tokio::spawn(async move {
    let mut test = Box::pin(&mut readhalf);
    loop {
      let disconnect = disconnect_2.lock().await;
      if *disconnect {
        drop(disconnect);
        return;
      }
      drop(disconnect);
      let packet = classic::ClassicPacketReader::read_packet_reader(&mut test).await;
      if packet.is_err() {
        let mut disconnect = disconnect_2.lock().await;
        *disconnect = true;
        drop(disconnect);
        return;
      }
      let packet = packet.unwrap();

      loop {
        match packet {
          classic::ClassicPacketClient::PositionAndOrientation {
            player_id,
            position,
          } => {
            let mut ourplayer = player.lock().await;
            ourplayer.position = position;
            drop(ourplayer);
          }
          classic::ClassicPacketClient::Message { message } => {
            let x_plrs = game.players.lock().await;
            let players = x_plrs.clone();
            drop(x_plrs);
            //println!("Locked fine");
            if message.starts_with("/") {
              let mut ourplayer = player.lock().await;
              let message = message.split(" ").collect::<Vec<&str>>();
              match message[0] {
                "/coords" => {
                  let pos = ourplayer.position.clone();
                  ourplayer.chatbox.push(Message {
                    message: format!(
                      "You are at {} {} {} yaw {} pitch {}",
                      pos.x, pos.y, pos.z, pos.yaw, pos.pitch
                    )
                    .to_string(),
                    system: true,
                  });
                }
                "/isop" => {
                  let opstatus = (*ourplayer).operator;
                  ourplayer.chatbox.push(Message {
                    message: format!("{}", opstatus).to_string(),
                    system: true,
                  });
                }
                "/tp" => {
                  let opstatus = (*ourplayer).operator;
                  if opstatus == false {
                    ourplayer.chatbox.push(Message {
                      message: "&cYou do not have permission to run this command.".to_string(),
                      system: true,
                    });
                    break;
                  }
                  if message.len() < 2 {
                    ourplayer.chatbox.push(Message {
                      message: "&Syntax error. Usage: /tp (player)".to_string(),
                      system: true,
                    });
                    break;
                  }
                  if message.len() < 3 {
                    let mut to: PositionYP = PositionYP::default();
                    let mut set = false;
                    drop(ourplayer);
                    for i in 0..players.len() {
                      let lockedplayer = players[i].lock().await;
                      if (*lockedplayer).name.to_lowercase() == message[1].to_lowercase() {
                        to = (*lockedplayer).position.clone();
                        set = true;
                        break;
                      }
                      drop(lockedplayer);
                    }
                    let mut ourplayer = player.lock().await;
                    if set == false {
                      ourplayer.chatbox.push(Message {
                        message: "Couldn't tp you!".to_string(),
                        system: true,
                      });
                      break;
                    }
                    let teleport_player = classic::ClassicPacketServer::PlayerTeleport {
                      player_id: -1,
                      position: to.clone(),
                    };
                    ourplayer.incoming_packets.push(teleport_player);
                  } else {
                    let mut to: PositionYP = PositionYP::default();
                    let mut set = false;
                    drop(ourplayer);
                    for i in 0..players.len() {
                      let lockedplayer = players[i].lock().await;
                      if (*lockedplayer).name.to_lowercase() == message[2].to_lowercase() {
                        to = (*lockedplayer).position.clone();
                        set = true;
                      }
                      drop(lockedplayer);
                      if set == true {
                        break;
                      }
                    }
                    let mut ourplayer = player.lock().await;
                    if set == false {
                      ourplayer.chatbox.push(Message {
                        message: "Couldn't tp!".to_string(),
                        system: true,
                      });
                      break;
                    }
                    drop(ourplayer);
                    for i in 0..players.len() {
                      let mut lockedplayer = players[i].lock().await;
                      if (*lockedplayer).name.to_lowercase() == message[1].to_lowercase() {
                        let teleport_player = classic::ClassicPacketServer::PlayerTeleport {
                          player_id: -1,
                          position: to.clone(),
                        };
                        (*lockedplayer).incoming_packets.push(teleport_player);
                        drop(lockedplayer);
                        break;
                      }
                      drop(lockedplayer);
                    }
                  }
                }
                "/setblock" => {
                  let opstatus = (*ourplayer).operator;
                  if opstatus == false {
                    ourplayer.chatbox.push(Message {
                      message: "&cYou do not have permission to run this command.".to_string(),
                      system: true,
                    });
                    break;
                  }
                  if message.len() < 4 {
                    ourplayer.chatbox.push(Message {
                      message: "&Syntax error. Usage: /setblock (x) (y) (z) (blockid)".to_string(),
                      system: true,
                    });
                    break;
                  }
                  let x = usize::from_str_radix(message[1], 10);
                  if x.is_err() {
                    ourplayer.chatbox.push(Message {
                      message: "&Syntax error. Usage: /setblock (x) (y) (z) (blockid)".to_string(),
                      system: true,
                    });
                    break;
                  }
                  let x = x.unwrap() as i16;

                  let y = usize::from_str_radix(message[2], 10);
                  if y.is_err() {
                    ourplayer.chatbox.push(Message {
                      message: "&Syntax error. Usage: /setblock (x) (y) (z) (blockid)".to_string(),
                      system: true,
                    });
                    break;
                  }
                  let y = y.unwrap() as i16;

                  let z = usize::from_str_radix(message[3], 10);
                  if z.is_err() {
                    ourplayer.chatbox.push(Message {
                      message: "&Syntax error. Usage: /setblock (x) (y) (z) (blockid)".to_string(),
                      system: true,
                    });
                    break;
                  }
                  let id = usize::from_str_radix(message[4], 10);
                  if id.is_err() {
                    ourplayer.chatbox.push(Message {
                      message: "&Syntax error. Usage: /setblock (x) (y) (z) (blockid)".to_string(),
                      system: true,
                    });
                    break;
                  }
                  let id = id.unwrap() as u8;
                  let z = z.unwrap() as i16;
                  let pos = Position { x: x, y: y, z: z };
                  let mut block = Block {
                    position: pos,
                    id: id,
                  };
                  /*                   let x_plrs = game.players.try_lock();
                  if x_plrs.is_err() {
                    ourplayer.chatbox.push(Message {
                      message: "Something went wrong.".to_string(),
                      system: true,
                    });
                    break;
                  }
                  let x_plrs = x_plrs.unwrap();
                  let players = x_plrs.clone(); */
                  //(*ourplayer).block_changes.push(block.clone());
                  let ourname = (*ourplayer).name.clone();
                  drop(ourplayer);
                  for i in 0..players.len() {
                    let mut lockedplayer = players[i].lock().await;
                    if (*lockedplayer).name != ourname {
                      (*lockedplayer).block_changes.push(block.clone());
                    }
                    drop(lockedplayer);
                  }
                  // Either god or galaxtone knows why this works, but without this, the world is edited wrong and re-logging makes blocks appear in the wrong location.
                  block.position.x += 4;
                }
                "/kick" => {
                  ourplayer.chatbox.push(Message {
                    message: "&cdunnit work".to_string(),
                    system: true,
                  });
                  break;
                }
                _ => {
                  ourplayer.chatbox.push(Message {
                    message: "&cUnknown command.".to_string(),
                    system: true,
                  });
                }
              }
            } else {
              let mut ourplayer = player.lock().await;
              let ourname = (*ourplayer).name.clone();
              let prefix = format!("<{}> ", ourname);
let index = std::cmp::min(message.len(), 64 - prefix.len());
let tosend = format!("{}{}", prefix, &message[0..index]);
drop(ourplayer);
if message.len() > index {
  let tosend = format!("> {}", &message[index..]);
  for i in 0..players.len() {
    let mut lockedplayer = players[i].lock().await;
    lockedplayer.chatbox.push(Message {
      message: tosend.clone(),
      system: false,
    });
    drop(lockedplayer);
  }
}
for i in 0..players.len() {
  let mut lockedplayer = players[i].lock().await;
  lockedplayer.chatbox.push(Message {
    message: tosend.clone(),
    system: false,
  });
  drop(lockedplayer);
}
break;
              let message = format!("<{}> {}", ourname, message);
              println!("{}", message);
              if message.len() >= 64 {
                ourplayer.chatbox.push(Message {
                  message: "Message too long!".to_string(),
                  system: true,
                });
                drop(ourplayer);
                break;
              }
              drop(ourplayer);
              for i in 0..players.len() {
                let mut lockedplayer = players[i].lock().await;
                lockedplayer.chatbox.push(Message {
                  message: message.clone(),
                  system: false,
                });
                drop(lockedplayer);
              }
            }
          }
          classic::ClassicPacketClient::SetBlock {
            coords,
            mode,
            block_type,
          } => {
            let mut world = game.world.lock().await;
            let mut block: Block;
            match mode {
              0x01 => {
                block = Block {
                  id: block_type,
                  position: coords,
                };
              }
              _ => {
                block = Block {
                  id: 0,
                  position: coords,
                };
              }
            }
            let x = game.players.lock().await;
            let players = x.clone();
            drop(x);
            let ourplayer = player.lock().await;
            let ourname = (*ourplayer).name.clone();
            drop(ourplayer);
            for i in 0..players.len() {
              let mut lockedplayer = players[i].lock().await;
              if (*lockedplayer).name != ourname {
                (*lockedplayer).block_changes.push(block.clone());
              }
              drop(lockedplayer);
            }
            // Either god or galaxtone knows why this works, but without this, the world is edited wrong and re-logging makes blocks appear in the wrong location.
            block.position.x += 4;
            world.set_block(block);
          }
          _ => {}
        }
        break;
      }
    }
  });
  readhandle.await.unwrap();
  writehandle.await.unwrap();
  let mut allplayers = game_main.players.lock().await;
  let ourplayer = player_main.lock().await;
  let ourname = (*ourplayer).name.clone();
  drop(ourplayer);
  for i in 0..allplayers.len() {
    let lockedplayer = allplayers[i].clone();
    let lockedplayer = lockedplayer.lock().await;
    if (*lockedplayer).name == ourname {
      allplayers.remove(i);
      println!("Removed {} from the player pool.", ourname);
      break;
    }
  }
  println!("Disconnecting user {}", ourname);
  drop(allplayers);
  let message = format!("&e{} left the game.", ourname.clone());
  let x = game_main.players.lock().await;
  let players = x.clone();
  for i in 0..players.len() {
    let mut lockedplayer = players[i].lock().await;
    lockedplayer.chatbox.push(Message {
      message: message.clone(),
      system: true,
    });
    drop(lockedplayer);
  }
  drop(x);
  Ok(())
}
