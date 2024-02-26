#![windows_subsystem = "windows"] // CLIを表示しない（アタッチされないので標準出力は出ない）

mod logic;

use clap::Parser;
use wry::{
  application::{
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::WindowBuilder,
  },
  webview::WebViewBuilder,
};
use serde::{Serialize, Deserialize};
use anyhow::Context;
use logic::*;
use colored::*;

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

  #[arg(short, long, default_value="wuzeiNamedPipe")]
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

trait WuzeiBuilder : Sized {
  fn resist_javascript(self, flag:bool) -> Self;
  fn resist_navigate(self, working_dir:Option<String>, url:Option<String>) -> anyhow::Result<Self>;
  fn resist_handler(self, event_loop:&EventLoop<UserEvent>) -> Self;
}

impl<'a> WuzeiBuilder for WebViewBuilder<'a> {

  fn resist_javascript(self, flag:bool) -> Self {
    match flag {
      true => self,
      false => {
        println!("{} {}", "resist".blue(), "resource/rxjs.umd.min.js");
        println!("{} {}", "resist".blue(), "resource/vue.global.js");
        self
          .with_initialization_script(&*include_str!("resource/rxjs.umd.min.js"))
          .with_initialization_script(&*include_str!("resource/vue.global.js"))
      }
    }
  } 
  fn resist_navigate(self, working_dir:Option<String>, url:Option<String>) -> anyhow::Result<Self> {
    match working_dir {
      None => { },
      Some(n) => { std::env::set_current_dir(&n)?; }
    };
    let path = std::env::current_dir().context("context")?;
    println!("{} {}", "current dir".blue(), path.display());
    println!("{} {:?}", "start_url".blue(), url);
    let dst = match url {
      None => {
        let html = include_str!("index_vue.html");
        self.with_html(html)
      },
      Some(n) if n.starts_with("https://") || n.starts_with("http://") => {
        self.with_url(n.as_str())
      },
      Some(n) => {
        let content = std::fs::read_to_string(&n).expect("could not read file");
        self.with_html(content)
      }
    };
    Ok(dst.context("failed navigate")?)
  }
  fn resist_handler(self, event_loop:&EventLoop<UserEvent>) -> Self {
    /* 
      - wry0.35からWindowの引数がなくなりargのみ
      - with_file_drop_handlerはwith_new_window_req_handlerの発火を阻害するので使用できない

        .with_file_drop_handler(move |_window, event|{
          println!("{event:?}");
          match event {
            wry::webview::FileDropEvent::Dropped { paths, position } => {
              println!("{paths:?} {position:?}");
            },
            _=>{ }
          }
          false
        })
    */
    println!("{} {}", "resist".blue(), "handler");
    let proxy_req = event_loop.create_proxy();
    let proxy_ipc = event_loop.create_proxy();
    self
      .with_navigation_handler(move |url| {
        let dst = match url {
          n if n.starts_with("data:") => "raw html".to_string(),
          n if n.starts_with("http://") || n.starts_with("https://") => n,
          _=> "unknown".to_string()
        };
        println!("{} {}", "navigation".green(), dst);
        true
      })
      .with_new_window_req_handler(move |event|{
        let _ = proxy_req.send_event(UserEvent::NewEvent("newWindowReq".to_string(), event)).unwrap();
        false // 後に続く動作を止める
      })
      .with_ipc_handler(move |_window, arg| ipc_handler(&proxy_ipc, arg) )

  }

}

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

  /* setting event_loop */
  println!("{}", "run event_loop".blue());
  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;
    event_handler(&webview, event, control_flow);
  });

}

