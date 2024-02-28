use colored::*;
use anyhow::Context as _;
use serde::{Serialize, Deserialize};
use wry::{
  application::event_loop::EventLoop,
  webview::WebViewBuilder,
   http::{header::CONTENT_TYPE, StatusCode},
};
use include_dir::{include_dir, Dir};


#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Message{
  #[serde(rename = "type")]
  pub type_name: String,
  pub payload: serde_json::Value
}

#[derive(Debug)]
pub enum UserEvent {
  Message(String),
  SubProcess(serde_json::Value),
  NewEvent(String, String)
}

pub trait WuzeiBuilder : Sized {
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
    /*
      custom_protocol使用するなら、cros対策でcustom_protocol通した遷移させた方がいい
    */
    match working_dir {
      None => { },
      Some(n) => { std::env::set_current_dir(&n)?; }
    };
    let path = std::env::current_dir().context("context")?;
    println!("{} {}", "current dir".blue(), path.display());
    println!("{} {:?}", "start_url".blue(), url);
    let dst = match url {
      None => {
        // let html = include_str!("index_vue.html");
        // self.with_html(html)
        self.with_url("http://wuzei.localhost/resource/index_vue.html")
      },
      Some(n) if n.starts_with("https://") || n.starts_with("http://") => {
        self.with_url(n.as_str())
      },
      Some(n) => {
        self.with_url(format!("http://wuzei.localhost/local/{n}").as_str())
        // let content = std::fs::read_to_string(&n).expect("could not read file");
        // self.with_html(content)
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
        // true
      })
      .with_ipc_handler(move |_window, arg| {
        let src: Message = match serde_json::from_str::<Message>(&arg) {
          Ok(n) => n,
          Err(e) => {
            println!("err {e}");
            Message::default()
          }
        };
        match src.type_name.as_str() {
          "message" => { let _ = proxy_ipc.send_event(UserEvent::Message(format!("{:?}",src.payload))).unwrap(); },
          "process" => { let _ = proxy_ipc.send_event(UserEvent::SubProcess(src.payload)).unwrap(); },
          _ => println!("{src:?}")
        };
      } )
      .with_custom_protocol("wuzei".into(), move |request| {
        (match request.uri().path() {
          n if n.starts_with("/resource") => {
            let url = n.trim_start_matches("/resource/");
            println!("custom protocol resource path {:?}", url);
            let content = super::RESOURCE.get_file(url).unwrap().contents_utf8().unwrap();
            wry::http::Response::builder()
              .header(CONTENT_TYPE, "text/html")
              .body(content.to_string().as_bytes().to_vec())
              .unwrap()
          },
          n if n.starts_with("/local") => {
            let url = n.trim_start_matches("/local/");
            println!("custom protocol local path {:?}", url);
            let content = std::fs::read_to_string(&url).expect("could not read file");
            wry::http::Response::builder()
              .header(CONTENT_TYPE, "text/html")
              .body(content.as_bytes().to_vec())
              .unwrap()
          },
          "/data" => {
            println!("custom protocol {:?}", request.uri().path());
            wry::http::Response::builder()
              .header(CONTENT_TYPE, "application/octet-stream")
              .body(vec![0,1,2,3])
              .unwrap()
          },
          _=> {
            println!("custom protocol {:?} {:?}", StatusCode::NOT_FOUND, request.uri().path());
            wry::http::Response::builder()
              .status(StatusCode::NOT_FOUND)
              .body(Vec::new()) 
              .unwrap()
          }   
        }).map(Into::into)

      })
  }

}
