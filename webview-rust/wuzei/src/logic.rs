use colored::Colorize;
use anyhow::bail;
use serde::{Serialize, Deserialize};
use wry::{
  webview::WebView,
  application::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopProxy},
  },
};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Message{
  #[serde(rename = "type")]
  pub type_name: String,
  pub payload: String
}

#[derive(Debug)]
pub enum UserEvent {
  Message(String),
  SubProcess(String),
  NewEvent(String, String)
}


pub fn ipc_handler(proxy:&EventLoopProxy<UserEvent>, arg:String) {
  let src: Message = match serde_json::from_str::<Message>(&arg) {
    Ok(n) => n,
    Err(e) => {
      println!("err {e}");
      Message::default()
    }
  };
  match src.type_name.as_str() {
    "test" => {
      println!("{src:?}");
    },
    "test-2" => {
      println!("{src:?}");
    },
    "message" => { let _ = proxy.send_event(UserEvent::Message(src.payload)).unwrap(); },
    "subprocess" => { let _ = proxy.send_event(UserEvent::SubProcess(src.payload)).unwrap(); },
    _ => println!("{src:?}")
  };
}

pub fn event_handler(webview: &WebView, event:Event<'_, UserEvent>, control_flow:&mut ControlFlow) {
  match event {
    Event::NewEvents(StartCause::Init) => {
      println!("Init");
      let _ = webview.evaluate_script(&*include_str!("resource/app.js"));
    },
    Event::UserEvent(UserEvent::Message(e)) => {
      let dst = format!("{} {}", "message".green(), e);
      println!("{dst}");
      let _ = webview.evaluate_script(&*format!("console.log('{dst}')"));
    },
    Event::UserEvent(UserEvent::NewEvent(key, payload)) => {
      match &payload {
        n if n.starts_with("https://") => {
          println!("https : {n:?}");
        },
        n if n.starts_with("file://") => {
          println!("file : {n:?}");
        },
        _=> { println!("{payload:?}"); }
      };
      let dst = indoc::formatdoc! {r##"
        const raisedEvent = new CustomEvent('{key}', {{
          bubbles: false,
          cancelable: false,
          detail: {{ payload : '{payload}' }}
        }});
        window.chrome.webview.dispatchEvent(raisedEvent);
      "##};
      let _ = webview.evaluate_script(&*dst);
      // webview.evaluate_script_with_callback(js, callback)
    },
    Event::UserEvent(UserEvent::SubProcess(e)) => {
      println!("{} {}", "message".green(), e);
      let dst = std::process::Command::new("powershell")
        .args(&["dotnet", "script test.csx"])
        .spawn().unwrap()
        .wait().unwrap();
      println!("Exit {dst:?}");
    },
    Event::WindowEvent {
      event: WindowEvent::CloseRequested,
      ..
    } => *control_flow = ControlFlow::Exit,
    _ => {}
  }
}

#[allow(dead_code)]
fn run_powershell<F>(func: F) -> anyhow::Result<std::process::ExitStatus> where F: FnOnce(String) -> String {
  use std::io::Write;
  
  let temp = tempfile::tempdir().unwrap();
  let path = temp.path().to_string_lossy().into_owned();
  let file_path = format!(r"{path}\_temp.ps1");

  println!("{} {}", "create temp dir".green(), path);
  println!("{} {}", "create temp file".green(), file_path);
  {
    let mut buffer = std::fs::File::create(&file_path).unwrap();
    write!(&mut buffer, "{}", func(path)).unwrap();  
  }

  println!("{}", "run powershell process...".green());
  let dst = std::process::Command::new("powershell")
    .args(&["-ExecutionPolicy", "Bypass", "-File", file_path.as_str()])
    .spawn().unwrap()
    .wait().unwrap();
  println!("{} {:?}", "cleaning up temp dir".green(), temp);
  // std::fs::Fileから解放、wait()で待つをしないとロック or リリースされてしまう

  Ok(dst)

  // let mut child = Command::new("powershell")
  // .args(&["-Command", "ls"])
  // .spawn()
  // .expect("failed")
  // .wait().unwrap();

}

#[allow(dead_code)]
pub fn back_ground_worker(proxy:&EventLoopProxy<UserEvent>) {
  loop {
      let now = std::time::SystemTime::now()
      .duration_since(std::time::SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_millis();
    if proxy.send_event(UserEvent::Message(format!("{now}"))).is_err() { break; }
    std::thread::sleep(std::time::Duration::from_secs(1));
  }
}

use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};
use tokio::io::Interest;

enum ReturnEnum {
  Ok(String),
  Continue,
}

pub async fn back_ground_worker_pipe(proxy:&EventLoopProxy<UserEvent>, pipename:String) -> anyhow::Result<()> {
  loop {
    match pipe(format!(r##"\\.\pipe\{pipename}"##).as_str()).await {
      Ok(ReturnEnum::Ok(n)) => { 
        if proxy.send_event(UserEvent::NewEvent("namedPipe".to_string(), format!("{n}"))).is_err() { bail!("proxy err") }
      },
      Ok(ReturnEnum::Continue) => { continue; },
      Err(e) => { bail!(e) }
    }
  }
}

async fn pipe(pipename:&str) -> anyhow::Result<ReturnEnum> {
  let server : NamedPipeServer = ServerOptions::new().create(pipename).unwrap();
  let _connected = server.connect().await?;
  println!("connect");
  let ready = server.ready(Interest::READABLE).await?;
  let mut dst = String::new();
  let mut buf = vec![0; 1024];
  if ready.is_readable() {
    dst = match server.try_read(&mut buf) {
      Ok(n) => { String::from_utf8_lossy(&buf[0..n]).into_owned().trim().to_string() }
      Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => { return Ok(ReturnEnum::Continue); }
      Err(e) => bail!(e)
    }
  }
  let ready = server.ready(Interest::WRITABLE).await?;
  if ready.is_writable() {
    match server.try_write(b"Ok\r\n") {
      Ok(n) => { println!("write {} bytes", n); }
      Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => { return Ok(ReturnEnum::Continue); }
      Err(e) => bail!(e)
    }
  }
  server.disconnect()?;
  Ok(ReturnEnum::Ok(dst))
}