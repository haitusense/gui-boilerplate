// CLIを表示しない（アタッチされないので標準出力は出ない）
#![windows_subsystem = "windows"]

use wry::{
  application::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
  },
  webview::WebViewBuilder,
};

fn main() -> wry::Result<()> {
  
  let html = include_str!("index.html");

  let event_loop = EventLoop::new();
  let window = WindowBuilder::new()
  .with_title("test")
  .build(&event_loop)?;

  let _webview = WebViewBuilder::new(window)?
    .with_html(html)?
    // .with_url("http://tauri.app")?
    .with_ipc_handler(move |_webview, arg| {
      let dst: Vec<&str> = arg.split(",").collect();
      println!("{dst:?}");
    })
    .with_devtools(true)
    .build()?;

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    if let Event::WindowEvent {
      event: WindowEvent::CloseRequested,
      ..
    } = event
    {
      *control_flow = ControlFlow::Exit
    }
  });
}
