#![windows_subsystem = "windows"] // CLIを表示しない（アタッチされないので標準出力は出ない）

mod logic;

use clap::Parser;
use wry::{
  application::{
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
  },
  webview::WebViewBuilder,
  
};
use serde::{Serialize, Deserialize};
use anyhow::Context as _;
use logic::*;

#[derive(clap::Parser, Serialize, Deserialize, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(value_enum, short, long)]
  window_type: Option<WindowType>,
}

#[derive(clap::ValueEnum, Serialize, Deserialize, Clone, Debug)]
pub enum WindowType {
  AllocConsole,
  AttachConsole
}

impl Args {
  /* `--help` オプションが渡された場合コンソールをアタッチ */
  pub fn parse_with_attach_console() -> Self {
    if std::env::args().any(|arg| arg == "--help" || arg == "-h"|| arg == "--version" || arg == "-V" ) {
      unsafe { let _ = windows::Win32::System::Console::AttachConsole(u32::MAX); }
    }
    let args = Args::parse();
    args.attach_console();
    args
  }
  pub fn to_value(&self) -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::to_value(&self).context("err")?)
  }

  pub fn attach_console(&self) {
    match self.window_type {
      Some(WindowType::AllocConsole) => {
        unsafe { 
          let _ = windows::Win32::System::Console::AllocConsole();
          let handle = windows::Win32::System::Console::GetStdHandle(windows::Win32::System::Console::STD_OUTPUT_HANDLE).unwrap();
          let mut lpmode : windows::Win32::System::Console::CONSOLE_MODE = Default::default();
          let _ = windows::Win32::System::Console::GetConsoleMode(handle, &mut lpmode);
          let _ = windows::Win32::System::Console::SetConsoleMode(handle, lpmode | windows::Win32::System::Console::ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
      },
      Some(WindowType::AttachConsole) => {
        unsafe { let _ = windows::Win32::System::Console::AttachConsole(u32::MAX); }
      },
      _=>{
      }
    };
  }
}


fn main() -> anyhow::Result<()> {
  
  let args = Args::parse_with_attach_console(); 
  println!("args {:?}", args);
  println!("args To Value {:?}", args.to_value());

  let html = include_str!("index_vue.html");

  let rgba = image::load_from_memory(include_bytes!("image/icon.png")).unwrap();
  let icon = wry::application::window::Icon::from_rgba(rgba.into_bytes(), 256, 256).ok();

  let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
  let proxy = event_loop.create_proxy();
  let window = WindowBuilder::new()
    .with_title("wuzei")
    .with_window_icon(icon)
    .build(&event_loop).context("err")?;

  let webview = WebViewBuilder::new(window).context("err")?
    .with_html(html).context("err")? // .with_url("http://tauri.app")?
    .with_ipc_handler(move |_window, arg| ipc_handler(&proxy, arg) ) // wry 0.35からはargのみ
    .with_devtools(true)
    .with_initialization_script(&*include_str!("resource/rxjs.umd.min.js"))
    .with_initialization_script(&*include_str!("resource/vue.global.js"))
    .with_file_drop_handler(move |_window, arg|{
      println!("{arg:?}");
      true
    })
    .build().context("err")?;

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;
    event_handler(&webview, event, control_flow);
  });

}
