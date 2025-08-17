import * as wasm from "../../pkg/digitaljs_wasm_worker.js"

let engine;
let updater = null;
let sender = null;
let queue = [];

self.onmessage = (e) => {
    let msg = e.data;

    if (msg.type == "fetch") {
        wasm.initSync({ module: msg.data });
        engine = wasm.RustEngine.new();
        queue.forEach((m) => applyMessage(m))
        queue = [];
    } else if (msg.type == "nodeJs") {
        engine = wasm.default.RustEngine.new()
        queue.forEach(m => applyMessage(m));
        queue = [];
    } else {
        if (engine && queue.length == 0) {
            applyMessage(msg);
        } else {
            queue.push(msg);
        }
    }
}

function applyMessage(msg) {
    //console.log(msg);
    if (msg.type == 'start') {
        updater = setInterval(() => {
            engine._updateGates();
            engine._postMonitors();
        }, engine.getInterval());
        sender = setInterval(() => {
            engine._sendUpdates();
        }, 25);
    } else if (msg.type == "updater_stop") {
        _stop();  
    } else if (msg.type == "stop") {
        stop(...msg.args);
    } else if ('arg' in msg)
        engine[msg.type](msg.arg);
    else if ('args' in msg)
        engine[msg.type].apply(engine, msg.args);
    else
        engine[msg.type]();
}

function stop(reqid, sendUpdates) {
    _stop();
    if (sendUpdates) engine._sendUpdates();
    engine._sendAck(reqid);
}

function _stop() {
    if (updater) {
        clearInterval(updater);
        updater = null;
    }
}