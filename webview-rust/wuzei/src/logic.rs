use colored::Colorize;
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
  SendData(String),
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
      let _ = proxy.send_event(UserEvent::SendData(src.payload)).unwrap();
    },
    "message" => { let _ = proxy.send_event(UserEvent::Message(src.payload)).unwrap(); },
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
    Event::UserEvent(UserEvent::SendData(e)) => {
      println!("{e:?}");
      let dst = indoc::formatdoc! {r##"
        const raisedEvent = new CustomEvent('propertyChanged', {{
          bubbles: false,
          cancelable: false,
          detail: {{ PropertyName : "a" }}
        }});
        window.chrome.webview.dispatchEvent(raisedEvent);
      "##};
      let _ = webview.evaluate_script(&*format!("console.log('{e}')"));
      // webview.evaluate_script_with_callback(js, callback)
      println!("{dst}");
      let _ = webview.evaluate_script(&*dst);

    }
    Event::WindowEvent {
      event: WindowEvent::CloseRequested,
      ..
    } => *control_flow = ControlFlow::Exit,
    _ => {}
  }
}