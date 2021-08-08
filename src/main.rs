#![allow(dead_code)]

use tokio::net::{TcpListener, TcpStream};
use tokio::time::{self, Duration};
use std::sync::{Arc, Mutex};

use std::error::Error;
use std::result;
type Result<T> = result::Result<T, Box<dyn Error + Send + Sync>>;
const IP: &str = "0.0.0.0:25565";

mod network;
use network::classic;

use tokio::io::AsyncWriteExt;
use classic::chunks::{World, FlatWorldGenerator};
use classic::BlockIds;

//use classic::heartbeat;
use classic::{ClassicPacketReader, /*ClassicPacketBuilder, Position,*/ PositionYP};

// TODO Expand player struct
pub struct Player {
  pub name: String,
  pub position: PositionYP,
  pub operator: bool
}

// TODO Everything.
/*
pub struct Game {
  players: Arc<Mutex<Vec<Arc<Mutex<Player>>>>>,
  ops: Arc<Mutex<Vec<String>>>
}*/

#[tokio::main]
async fn main() -> Result<()> {
  //let game = Game { players: Arc::new(Mutex::new(vec![])), ops: Arc::new(Mutex::new(vec!["Exopteron".to_string(), "Galaxtone".to_string()])) };

/*   // Heartbeat Thread
  tokio::spawn(async {
    let duration = Duration::from_secs(45);
    loop {
      println!("Heartbeat!");
      heartbeat::heartbeat().await?;
      time::sleep(duration).await;
    }
  }); */

  listen(/*&game*/).await.unwrap();
  Ok(())
}

pub async fn listen(/*game: &Game*/) -> Result<()> {
  let listener = TcpListener::bind(IP).await?;
  println!("Listening on {}", IP);
  loop {
    let (socket, address) = listener.accept().await.unwrap();
    println!("Connection {:?}", address);
    tokio::spawn(async move {
      process(socket, /*&game*/).await.unwrap();
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
async fn process(mut socket: TcpStream/*, game: &Game*/) -> Result<()> {
  let mut test =  Box::pin(&mut socket);
  let packet = ClassicPacketReader::read_packet_reader(&mut test).await?;
  let mut user = ConnectingUser {username: String::default(), protocol_version: 0, verification_key: String::default()};
  match packet {
    classic::ClassicPacketClient::PlayerIdentification {
      protocol_ver,
      username,
      verification_key,
    } => {
      user.username = username;
      user.protocol_version = protocol_ver; 
      user.verification_key = verification_key;
      println!("{} connecting from [{}]", user.username, socket.peer_addr()?.to_string());
    },
    _ => {
      return Err(Box::new(OurError::new("Wrong packet!")));
    },
  }
  /*let ops = game.ops.lock().unwrap();
  let mut isop = false;
  for op in &*ops {
    if op == &user.username {
      isop = true;
      break;
    }
  }
  drop(ops);*/
  let server_identification = classic::ClassicPacketServer::server_identification(0x07, "Ballland".to_string(), "a really good motd".to_string(), false)?;
  socket.write_all(&server_identification).await?;
  let generator = FlatWorldGenerator::new(64, BlockIds::DIRT, BlockIds::GRASS, BlockIds::AIR);
  let world = World::new(generator, 128, 128, 128);
  let packets = classic::ClassicPacketServer::serialize_vec(world.to_packets())?;
  for i in 0..packets.len() {
    socket.write_all(&packets[i]).await?;
  }


  // Custom position relative to center of map!
  let player = Player {name: user.username.clone(), position:
    PositionYP::from_pos(128 / 2, 64, 128 / 2), operator: false };


  //let player = Arc::new(Mutex::new(player));
  //let x = player.clone();
  /*let mut players = game.players.lock().unwrap();
  players.push(x);
  drop(players);*/
  let ourplayer = player;//.lock().unwrap();
  let spawn_player = classic::ClassicPacketServer::SpawnPlayer { player_id: -1, name: ourplayer.name.clone(), position: ourplayer.position.clone() };
  let teleport_player = classic::ClassicPacketServer::PlayerTeleport { player_id: -1, position: ourplayer.position.clone() };
  socket.write_all(&classic::ClassicPacketServer::serialize(spawn_player)?).await?;
  socket.write_all(&classic::ClassicPacketServer::serialize(teleport_player)?).await?;
  drop(ourplayer);
  
  /* just replace it with an actual event loop */
  

  println!("{} disconnected from [{}]", user.username, socket.peer_addr()?.to_string());
  Ok(())
}