use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_timer::Delay;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[wasm_bindgen]
pub async fn sleep_millis(numbers: u16) -> js_sys::Promise {
  let millis: u64 = u64::from(numbers);
  Delay::new(Duration::from_millis(millis)).await.unwrap();

  let promise = js_sys::Promise::resolve(&numbers.into());
  return promise;
}