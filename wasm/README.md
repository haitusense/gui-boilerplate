# WASM

## wasm_vanilla

### build

```ps1
ps> rustup target list
# wasm32-wasi : 標準ライブラリに統合スタンドアロンバイナリ
# wasm32-unknown-unknown : 標準ライブラリの想定無し(unknown)
# wasm32-unknown-emscripten : ウェブブラウザ向け
ps> rustup target add wasm32-wasi
ps> cargo build --target wasm32-wasi --release
```

### targetの違いに関して

#### wasm32-unknown-unknown

importされるobjectの内容が

```json
{
  "[[Exports]]" : [
    { "name" : "memory", "kind" : "memory" },
    { "name" : "get_timestamp", "kind" : "function" },
    { "name" : "add", "kind" : "function" },
    { "name" : "console_write", "kind" : "function" },
    { "name" : "__data_end", "kind" : "global" },
    { "name" : "__heap_base", "kind" : "global" },
  ],
  "[[Imports]]" : [
    { "module" : "env", "name" : "data_now" },
    { "module" : "env", "name" : "console_log" }
  ]
}
```

#### wasm32-wasi

そのままwasm_import_moduleするとImport #0 module="wasi_snapshot_preview1" error  
dummy入れておけば動かないことはない

```json
{
  "[[Exports]]" : [
    { "name" : "memory", "kind" : "memory" },
    { "name" : "get_timestamp", "kind" : "function" },
    { "name" : "add", "kind" : "function" },
    { "name" : "console_write", "kind" : "function" },
  ],
  "[[Imports]]" : [
    { "module" : "env", "name" : "data_now" },
    { "module" : "env", "name" : "console_log" },
    { "module" : "wasi_snapshot_preview1", "name" : "fd_write" }
    { "module" : "wasi_snapshot_preview1", "name" : "environ_get" }
    { "module" : "wasi_snapshot_preview1", "name" : "environ_sizes_get" }
    { "module" : "wasi_snapshot_preview1", "name" : "proc_exit" }
  ]
}
```

```javascript
const importObject = {
  env: {
    console_log: () => { console.log("Hello WebAssembly!"); },
  },
  wasi_snapshot_preview1: {
    fd_write: (i32a, i32b, i32c, i32d) => { return 0 },
    environ_get: (i32a, i32b) => { return 0 },
    environ_sizes_get: (i32a, i32b) => { return 0 },
    proc_exit: (i32) => console.log(i32),
  },
};
```

### WebAssembly.Memory

```javascript
const importObject = {
  env: {
    console_log: (ptr, len) => {
      const chars = new Uint16Array(instance_2.exports.memory.buffer, ptr, len);
      console.log(String.fromCharCode(...chars));
    },
  },
};
```

```javascript
const memory = new WebAssembly.Memory({
  initial: 10,
  maximum: 100
});
const instance = await WebAssembly.instantiateStreaming(fetch("memory.wasm"), { js: { mem: memory } })
const summands = new Uint32Array(memory.buffer);
for (let i = 0; i < 10; i++) {
  summands[i] = i;
}
sum = instance.exports.accumulate(0, 10);
console.log(sum);
```

## wasm_vanilla

### build

```ps1
ps> cargo install wasm-pack
ps> wasm-pack build --target web
```

## Wasmer

making

```javascript
  <script type="module">
    import { init, Wasmer } from "https://unpkg.com/@wasmer/sdk@latest?module";
    // await init();
    // const pkg = await Wasmer.getImports(module);
    console.log(Wasmer);
    // const a = Wasmer.fromFile(fetch('/target/wasm32-unknown-unknown/release/wasm_vanilla.wasm'))
    // console.log(a);
  </script>
```

[Rust で WebAssembly から console.log](https://zenn.dev/a24k/articles/20221012-wasmple-simple-console)
[WebAssembly と JavaScript との間で自在にデータをやりとり](https://zenn.dev/a24k/articles/20221107-wasmple-passing-buffer)

## .NET WebAssembly Browser app

```ps1
ps> dotnet workload install wasm-experimental
ps> dotnet new wasmbrowser -o myApp
```

### Build

You can build the app from Visual Studio or from the command-line:

```ps1
ps> dotnet build -c Debug
ps> dotnet build -c Release
```

After building the app, the result is in the `bin/$(Configuration)/net7.0/browser-wasm/AppBundle` directory.

### Run

You can build the app from Visual Studio or the command-line:

```ps1
ps> dotnet run -c Debug/Release
```

Or you can start any static file server from the AppBundle directory:

```ps1
ps> dotnet tool install dotnet-serve
ps> dotnet serve -d:bin/$(Configuration)/net7.0/browser-wasm/AppBundle
```

## reference

[暗黙の型変換](https://wasm-dev-book.netlify.app/hello-wasm.html#%E6%9A%97%E9%BB%99%E3%81%AE%E5%9E%8B%E5%A4%89%E6%8F%9B)
[linear-memory](https://rustwasm.github.io/docs/book/what-is-webassembly.html#linear-memory)