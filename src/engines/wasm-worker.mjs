import * as wasm from "../../pkg/digitaljs_wasm_worker.js"

let engine;
let updater = null;
let sender = null;
let queue = [];

self.onmessage = (e) => {
    let msg = e.data;

    if (msg.type == "fetch") {
        wasm.initSync({ module: msg.data });
        engine = wasm.WasmEngine.new();
        self.postMessage({ type: "initFinish" });
    } else if (msg.type == "runStart") {
        queue.forEach((m) => applyMessage(m))
        queue = [];
        sender = setInterval(() => {
            engine._sendUpdates();
        }, 25);
    } else {
        if (engine && queue.length == 0) {
            applyMessage(msg);
        } else {
            queue.push(msg);
        }
    }
}

self.postMessage({ type: "fetchWasm" });

function applyMessage(msg) {
    console.log(msg);

    if (msg.type == 'start') {
        updater = setInterval(() => {
            engine._updateGates();
            //engine._updateMonitors();
        }, engine.getInterval());
    } else if (msg.type == 'addGate') {
        addGateApplied(...msg.args);
    } else if (!(msg.type in engine))
        return;
    else if ('arg' in msg)
        engine[msg.type](msg.arg);
    else if ('args' in msg)
        engine[msg.type].apply(engine, msg.args);
    else
        engine[msg.type]();
}

function addGateApplied(graphId, gateId, gateParams, ports, inputSignals, outputSignals) {
    engine.addGate(
        graphId, 
        gateId, 
        gateParams, 
        ports.map((v) => {
            return {
                id: v.id,
                dir: v.dir,
                bits: v.bits,
                avec: (v.dir == "in") ? inputSignals[v.id]._avec : outputSignals[v.id]._avec,
                bvec: (v.dir == "in") ? inputSignals[v.id]._bvec : outputSignals[v.id]._bvec,
            }
        })
    )
}