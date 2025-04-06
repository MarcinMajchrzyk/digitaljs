#![allow(dead_code)]
use std::collections::{BTreeSet, HashMap, HashSet};

use gate::GatePtr;
use graph::Graph;
use js_types::{GateParams, PortParams, SigParams, TargetParams};
use link::LinkTarget;
use vector3vl::Vec3vl;
use wasm_bindgen::prelude::*;

mod graph;
mod gate;
mod link;
mod js_types;
mod vector3vl;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(module = "/src/engines/wasm-js-functions.mjs")]
extern "C" {
    fn sendUpdates(tick: u32, pendingEvents: bool, updates: Vec<UpdateStruct>);
}

#[wasm_bindgen]
struct WasmEngine {
    interval: u32,
    tick: u32,
    graphs: HashMap<String, Graph>,
    //monitors: ???,
    //monitorChecks: ???,
    //alarms: ???,
    queue: HashMap<u32, HashMap<String, (GatePtr, HashMap<String, Vec3vl>)>>,
    pq: BTreeSet<u32>,
    to_update: HashMap<String, (GatePtr, HashSet<String>)>,
}

#[wasm_bindgen]
impl WasmEngine {
    pub fn new() -> WasmEngine {
        WasmEngine { 
            interval: 10,
            tick: 0,
            graphs: HashMap::new(),
            queue: HashMap::new(),
            pq: BTreeSet::new(),
            to_update: HashMap::new()
        }
    }

    pub fn interval(&mut self, ms: u32) {
        self.interval = ms;
    }

    #[wasm_bindgen(js_name = getInterval)]
    pub fn get_interval(&self) -> u32 {
        self.interval
    }

    #[wasm_bindgen(js_name = addGraph)]
    pub fn add_graph(&mut self, id: String) {
        self.graphs.insert(id.clone(), Graph::new(id));
    }

    #[wasm_bindgen(js_name = updateGatesNext)]
    pub fn update_gates_next() {
        todo!()
    }

    #[wasm_bindgen(js_name = addGate)]
    pub fn add_gate(&mut self, graph_id: String, gate_id: String, gate_params: GateParams, port_params: Vec<PortParams>) {
        self.graphs.get_mut(&graph_id).unwrap().add_gate(gate_id.clone(), gate_params, port_params);
        self.enqueue(self.graphs.get(&graph_id).unwrap().get_gate(gate_id));
    }

    #[wasm_bindgen(js_name = addLink)]
    pub fn add_link(&mut self, graph_id: String, link_id: String, from: TargetParams, to: TargetParams) {
        let graph = self.graphs.get_mut(&graph_id).unwrap();
        let source_target = LinkTarget { id: from.get_id(), port: from.get_port(), magnet: from.get_port() };
        let target_target = LinkTarget { id: to.get_id(), port: to.get_port(), magnet: to.get_magnet() };

        graph.add_link(link_id.clone(), source_target.clone(), target_target.clone());

        let source_gate = graph.get_gate(source_target.id);
        let target_gate = graph.get_gate(target_target.id);

        let sig = source_gate.borrow().get_output(source_target.port);

        self.set_gate_input_signal_priv(target_gate, target_target.port, sig);
    }

    fn set_gate_input_signal_priv(&mut self, target_gate: GatePtr, port: String, sig: Vec3vl) {
        let old_sig = target_gate.borrow().get_input(port.clone());
        if old_sig == sig { return; }

        target_gate.borrow_mut().set_input(port, sig);

        self.enqueue(target_gate);
    }

    fn set_gate_output_signals_priv(&mut self, gate: GatePtr, sigs: Vec<(String, Vec3vl)>) {
        for (port, sig) in sigs {
            self.set_gate_output_signal_priv(&gate, port, sig);
        }
    } 

    fn set_gate_output_signal_priv(&mut self, gate: &GatePtr, port: String, sig: Vec3vl) {
        let old_sig = gate.borrow().get_output(port.clone());
        if old_sig == sig { return; }
        gate.borrow_mut().set_output(port.clone(), sig.clone());

        self.mark_update_priv(gate.clone(), port.clone());

        for target in gate.borrow().get_targets(port) {
            let target_gate = self.graphs.get(&gate.borrow().graph_id()).unwrap().get_gate(target.id);
            self.set_gate_input_signal_priv(target_gate, target.port, sig.clone());
        }

        // TODO monitors
    }

    fn enqueue(&mut self, gate: GatePtr) {
        let k = self.tick.wrapping_add(gate.borrow().get_propagation());

        let sq = match self.queue.get_mut(&k) {
            Some(q) => q,
            None => {
                let q = HashMap::new();
                self.queue.insert(k, q);
                self.pq.insert(k);
                self.queue.get_mut(&k).unwrap()
            }
        };

        sq.insert(gate.borrow().get_id().clone(), (gate.clone(), gate.borrow().get_inputs().clone()));
    }

    #[wasm_bindgen(js_name = _updateGates)]
    pub fn update_gates_priv(&mut self) -> u32 {
        if let Some(t) = self.pq.first() {
            if *t == self.tick {
                return self.update_gates_next_priv();
            }
        }
        self.tick = self.tick.wrapping_add(1);
        0
    }

    fn update_gates_next_priv(&mut self) -> u32 {
        let k = self.pq.pop_first().unwrap();
        self.tick = k;

        let q = self.queue.remove(&k).unwrap();
        let mut count = 0;

        for (_gate_id, (gate, sigs)) in q.iter() {
            let new_sig = (gate.borrow().operation)(sigs.clone());
            self.set_gate_output_signals_priv(gate.clone(), new_sig);
            count += 1;
        }
        
        self.tick = self.tick.wrapping_add(1);
        count
    }

    fn mark_update_priv(&mut self, gate: GatePtr, port: String) {
        // TODO check if graph is observed

        let s = match self.to_update.get_mut(&gate.borrow().get_id()) {
            Some(v) => v,
            None => {
                let v = HashSet::new();
                self.to_update.insert(gate.borrow().get_id(), (gate.clone(), v));
                self.to_update.get_mut(&gate.borrow().get_id()).unwrap()
            }
        };

        s.1.insert(port);
    }

    #[wasm_bindgen(js_name = _sendUpdates)]
    pub fn send_updates_priv(&mut self) {
        let mut updates= Vec::new();

        for (gate_id, (gate, ports)) in self.to_update.iter() {
            let mut signals = Vec::new();

            for port in ports {
                let sig = gate.borrow().get_output(port.clone());
                signals.push(PortUpdate { port: port.clone(), bits: sig.bits, avec: sig.avec, bvec: sig.bvec });
            }

            updates.push(UpdateStruct {
                graph_id: gate.borrow().graph_id(),
                gate_id: gate_id.to_string(),
                val: signals
            });
        }
        self.to_update = HashMap::new();
    
        sendUpdates(self.tick, false, updates);
    }

    #[wasm_bindgen(js_name = changeInput)]
    pub fn change_input(&mut self, graph_id: String, gate_id: String, sig: SigParams) {
        let gate = self.graphs.get(&graph_id).unwrap().get_gate(gate_id);
        self.set_gate_output_signal_priv(
            &gate, 
            String::from("out"), 
            Vec3vl { 
                bits: sig.get_bits(), 
                avec: sig.get_avec(), 
                bvec: sig.get_bvec() 
            }
        );
    }
}

#[wasm_bindgen(getter_with_clone)]
struct UpdateStruct {
    pub graph_id: String,
    pub gate_id: String,
    pub val: Vec<PortUpdate>
}

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
struct PortUpdate {
    pub port: String,
    pub bits: u32,
    pub avec: Vec<u32>,
    pub bvec: Vec<u32>
}