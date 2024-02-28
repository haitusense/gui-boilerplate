#![windows_subsystem = "windows"] // CLIを表示しない（アタッチされないので標準出力は出ない）

mod logic;
mod builder;
use logic::*;
use builder::*;

use clap::Parser;
use serde::{Serialize, Deserialize};
use anyhow::Context;
use colored::*;
use include_dir::{include_dir, Dir};
use wry::{
  application::{
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
  },
  webview::WebViewBuilder,
};

#[derive(clap::ValueEnum, Serialize, Deserialize, Clone, Debug)]
enum ConsoleType {
  AllocConsole,
  AttachConsole
}

#[derive(clap::Parser, Serialize, Deserialize, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(value_enum, short, long)]
  console_type: Option<ConsoleType>,

  #[arg(short, long)]
  working_dir: Option<String>,

  #[arg(short, long)]
  start_url: Option<String>,

  #[arg(short, long)]
  no_register_javascript: bool,

  #[arg(long, default_value="wuzeiNamedPipe")]
  namedpipe: String
}

impl Args {
  #[allow(dead_code)]
  pub fn to_value(&self) -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::to_value(&self).context("err")?)
  }

  /* `--help` オプションが渡された場合コンソールをアタッチ */
  pub fn parse_with_attach_console() -> Self {
    if std::env::args().any(|arg| arg == "--help" || arg == "-h"|| arg == "--version" || arg == "-V" ) {
      unsafe { let _ = windows::Win32::System::Console::AttachConsole(u32::MAX); }
    }
    let args = Args::parse();
    args.attach_console();
    args
  }
  
  pub fn attach_console(&self) {
    match self.console_type {
      Some(ConsoleType::AllocConsole) => {
        unsafe { 
          let _ = windows::Win32::System::Console::AllocConsole();
          let handle = windows::Win32::System::Console::GetStdHandle(windows::Win32::System::Console::STD_OUTPUT_HANDLE).unwrap();
          let mut lpmode : windows::Win32::System::Console::CONSOLE_MODE = Default::default();
          let _ = windows::Win32::System::Console::GetConsoleMode(handle, &mut lpmode);
          let _ = windows::Win32::System::Console::SetConsoleMode(handle, lpmode | windows::Win32::System::Console::ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
      },
      Some(ConsoleType::AttachConsole) => {
        unsafe { let _ = windows::Win32::System::Console::AttachConsole(u32::MAX); }
      },
      _=>{
      }
    };
  }
}

static RESOURCE: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/resource");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse_with_attach_console(); 
  // println!("args {:?} {:?}", args, args.to_value());

  /* read resource */
  let rgba = image::load_from_memory(include_bytes!("image/icon.png")).context("image::load_from_memory")?;
  let icon = wry::application::window::Icon::from_rgba(rgba.into_bytes(), 256, 256).ok();

  /* setting */
  println!("{}", "set window".blue());
  let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

  let window = WindowBuilder::new()
    .with_content_protection(false)
    .with_title("wuzei")
    .with_window_icon(icon)
    .build(&event_loop).context("err")?;

  /* 別スレッド */
  // std::thread::spawn(move ||{})
  // tokio::spawn(async move { back_ground_worker(&proxy_3); });
  let proxy = event_loop.create_proxy();
  tokio::spawn(async move { 
    back_ground_worker_pipe(&proxy, args.namedpipe).await.unwrap();
  });

  /* setting webview */
  let webview = WebViewBuilder::new(window).context("err")?
    .with_devtools(true)
    .resist_javascript(args.no_register_javascript)
    .resist_handler(&event_loop)
    .resist_navigate(args.working_dir, args.start_url).context("err")?
    .build().context("err")?;

  // use wry::webview::WebviewExtWindows;
  // let a = webview.controller();

  /* setting event_loop */
  println!("{}", "run event_loop".blue());
  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;
    event_handler(&webview, event, control_flow);
  });

}

