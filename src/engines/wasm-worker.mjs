import * as wasm from "../../pkg/digitaljs_wasm_worker.js"

self.onmessage = (e) => {
  let msg = e.data;

  if (msg.type == "fetch") {
    wasm.initSync({ module: msg.data });
    self.postMessage({ type: "initFinish" });
  } else if (msg.type == "runStart") {
    wasm.call_rust_function();
  }
}

self.postMessage({ type: "fetchWasm" });