use std::io;
use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};
const PIPE_NAME: &str = r"\\.\pipe\mynamedpipe";
use tokio::io::{AsyncWriteExt, Interest};
use anyhow::{bail, Ok};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

  tokio::spawn(async {
    loop {
      let server : NamedPipeServer = ServerOptions::new().create(PIPE_NAME).unwrap();
      match hoge(&server).await {
        Ok(ReturnEnum::Ok(n)) => { println!("dat : {}", n) },
        Ok(ReturnEnum::Continue) => { continue; },
        Err(e) => { bail!(e) }
      }
    }
  });

  println!("hello, world");
  Ok(())
}

enum ReturnEnum {
  Ok(String),
  Continue,
}
async fn hoge(server : &NamedPipeServer) -> anyhow::Result<ReturnEnum> {
  let _connected = server.connect().await?;
  println!("connect");
  let ready = server.ready(Interest::READABLE | Interest::WRITABLE).await?;
  let mut dst = String::new(); 
  if ready.is_readable() {
    let mut data = vec![0; 1024];
    dst = match server.try_read(&mut data) {
      Ok(n) => { 
        println!("read {} bytes", n);
        String::from_utf8_lossy(&data[0..n]).into_owned()
      }
      Err(e) if e.kind() == io::ErrorKind::WouldBlock => { return Ok(ReturnEnum::Continue); }
      Err(e) => bail!(e)
    }
  }
  
  if ready.is_writable() {
    match server.try_write(b"hello world\r\n") {
      Ok(n) => { println!("write {} bytes", n); }
      Err(e) if e.kind() == io::ErrorKind::WouldBlock => { return Ok(ReturnEnum::Continue); }
      Err(e) => bail!(e)
    }
  }  

  server.disconnect()?;
  Ok(ReturnEnum::Ok(dst))
}