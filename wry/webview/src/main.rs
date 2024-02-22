use wry::{
  application::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
  },
  webview::WebViewBuilder,
};
use anyhow::{Context, Result};

fn main() -> anyhow::Result<()> {
  let event_loop = EventLoop::new();
  let window = WindowBuilder::new()
    .with_title("はろー")
    .build(&event_loop);
  let _webview = WebViewBuilder::new(window).context("")
    .with_url("https://tauri.app/")?
    .build()?;

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::NewEvents(StartCause::Init) => println!("起動完了"),
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => *control_flow = ControlFlow::Exit, // アプリを終了させる。ExitWithCode(n)で終了ステータスを設定できる
      _ => {}
    }
  });
}