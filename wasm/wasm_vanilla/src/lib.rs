
/* 標準がenvなので、#[link(wasm_import_module = "env")]は省略可 */
#[link(wasm_import_module = "env")]
extern "C" {
  fn date_now() -> f64;
  fn console_log(ptr: *const u16, len: usize);
}

#[no_mangle]
pub fn get_timestamp() -> f64 {
  unsafe { date_now() }
}

#[no_mangle]
pub fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[no_mangle]
pub fn console_write() {
  let utf16: Vec<u16> = String::from("こんにちわ").encode_utf16().collect();
  unsafe { console_log(utf16.as_ptr(), utf16.len()); }
}