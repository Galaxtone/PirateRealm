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

use hyper::{Request, Body, Client, Uri};
use hyper_tls::HttpsConnector;

//use hyper::body::HttpBody as _;
//use tokio::io::{stdout, AsyncWriteExt as _};

// TODO move into classic.rs, once it's done.

mod urlencode;
use super::Result;

pub async fn heartbeat() -> Result<()> {
  let players = 0.to_string();

  // TODO Add config and remove fixed message content for heartbeat.
  let content = urlencode::serialize_form(&[
    ("name", "Not A Server"),
    ("port", "25565"),
    ("users", "0"),
    ("max", "0"),
    ("public", "true"),
    ("salt", "0123456789abcdef"),
    ("software", "Pirate Realm"),
  ]);

  // TODO Add config for custom heartbeat URL
  let request = Request::post("https://www.classicube.net/server/heartbeat/")
    .version()
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(Body::from(content))?;
  
  let https = HttpsConnector::new();
  let client = Client::builder().build(https);

  let mut response = client.request(request).await?;
  while let Some(chunk) response.body_mut().data()
// I'm currently researching Request's methods

/*

  let mut resp = client.request(request).await?;
  while let Some(chunk) = resp.body_mut().data().await {
    stdout().write_all(&chunk?).await?;
  }

  Uri::from_str();
  println!("7");*/
  Ok(())
  
}