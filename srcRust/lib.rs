#![allow(dead_code)]
use std::collections::{BTreeSet, HashMap, HashSet};

use gate::{GatePtr, IoDir};
use graph::{Graph, GraphPtr};
use js_types::{JsGateParams, IOmap, PortParams, SigParams, TargetParams};
use link::LinkTarget;
use operations::ClockHack;
use vector3vl::Vec3vl;
use wasm_bindgen::prelude::*;

mod graph;
mod gate;
mod link;
mod js_types;
mod vector3vl;
mod operations;

mod cell_arith;
mod cell_bus;
mod cell_dff;
mod cell_gates;
mod cell_io;
mod cell_mux;

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
    graphs: HashMap<String, GraphPtr>,
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
    pub fn add_gate(&mut self, graph_id: String, gate_id: String, gate_params: JsGateParams, port_params: Vec<PortParams>) {
        let graph = self.graphs.get(&graph_id).unwrap();
        graph.borrow_mut().add_gate(graph.clone(), gate_id.clone(), gate_params, port_params);
        self.enqueue(self.graphs.get(&graph_id).unwrap().clone().borrow().get_gate(gate_id));
    }

    #[wasm_bindgen(js_name = addLink)]
    pub fn add_link(&mut self, graph_id: String, link_id: String, from: TargetParams, to: TargetParams) {
        let graph = self.graphs.get_mut(&graph_id).unwrap();
        let source_target = LinkTarget { id: from.get_id(), port: from.get_port(), magnet: from.get_port() };
        let target_target = LinkTarget { id: to.get_id(), port: to.get_port(), magnet: to.get_magnet() };

        graph.borrow_mut().add_link(link_id.clone(), source_target.clone(), target_target.clone());

        let source_gate = graph.borrow().get_gate(source_target.id);
        let target_gate = graph.borrow().get_gate(target_target.id);

        let sig = source_gate.borrow().get_output(source_target.port);

        self.set_gate_input_signal_priv(target_gate, target_target.port, sig);
    }

    fn set_gate_input_signal_priv(&mut self, target_gate: GatePtr, port: String, sig: Vec3vl) {
        let old_sig = target_gate.borrow().get_input(port.clone());
        if old_sig == sig { return; }
        target_gate.borrow_mut().set_input(port.clone(), sig.clone());

        if target_gate.borrow().is_subcircuit() {
            let subgraph = target_gate.borrow().get_subgraph();
            
            match &target_gate.borrow().subgraph_io_map {
                Some(iomap) => {
                    let gate = subgraph.borrow().get_gate(iomap.get(&port).unwrap().clone());
                    self.set_gate_output_signal_priv(&gate, "out".to_string(), sig);
                },
                None => panic!()
            };
        } else if target_gate.borrow().is_output() {
            let subgraph = target_gate.borrow().get_graph();
            match subgraph.borrow().subcircuit() {
                Some(subcir) => {
                    let subcir_port = target_gate.borrow().params.net.clone().unwrap();
                    self.set_gate_output_signal_priv(&subcir, subcir_port, sig);
                },
                None => {  }
            };
        } else {
            self.enqueue(target_gate);
        }
    }

    fn set_gate_output_signals_priv(&mut self, gate: GatePtr, sigs: Vec<(String, Vec3vl)>) {
        for (port, sig) in sigs {
            self.set_gate_output_signal_priv(&gate, port, sig);
        }
    } 

    fn set_gate_output_signal_priv(&mut self, gate: &GatePtr, port: String, sig: Vec3vl) {
        let old_sig = gate.borrow().get_output(port.clone());
        if old_sig == sig.clone() { return; }
        gate.borrow_mut().set_output(port.clone(), sig.clone());

        self.mark_update_priv(gate.clone(), port.clone());
        for target in gate.borrow().get_targets(port) {
            let target_gate = self.graphs.get(&gate.borrow().graph_id()).unwrap().borrow().get_gate(target.id.clone());
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
        let name = gate.borrow().get_graph().borrow().get_id() + &gate.borrow().get_id();
        sq.insert(name, (gate.clone(), gate.borrow().get_inputs().clone()));
    }

    #[wasm_bindgen(js_name = _updateGates)]
    pub fn update_gates_priv(&mut self) {
        match self.pq.first() {
            Some(k) => {
                if *k == self.tick {
                    self.update_gates_next_priv();
                } else {
                    self.tick = self.tick.wrapping_add(1);
                }
            },
            None => {
                self.tick = self.tick.wrapping_add(1);
            }
        }
    }

    fn update_gates_next_priv(&mut self) {
        let k = self.pq.pop_first().unwrap();
        self.tick = k;

        while let Some(q) = self.queue.remove(&k) {
            for (gate, sigs) in q.values() {
                let result = gate.borrow_mut().operation.op(sigs.clone());
                let new_sigs = match result {
                    ClockHack::Clock(v) =>  {
                        self.enqueue(gate.clone());
                        v 
                    },
                    ClockHack::Normal(v) => v
                };
                self.set_gate_output_signals_priv(gate.clone(), new_sigs);
            }

            if self.queue.get(&k).is_some() {
                self.pq.pop_first();
            }
        }
        
        self.tick = self.tick.wrapping_add(1);
    }

    fn mark_update_priv(&mut self, gate: GatePtr, port: String) {
        // TODO check if graph is observed
        let name = gate.borrow().get_graph().borrow().get_id() + &gate.borrow().get_id();
        let s = match self.to_update.get_mut(&name) {
            Some(v) => v,
            None => {
                let v = HashSet::new();
                self.to_update.insert(name.clone(), (gate.clone(), v));
                self.to_update.get_mut(&name).unwrap()
            }
        };

        s.1.insert(port);
    }

    #[wasm_bindgen(js_name = _sendUpdates)]
    pub fn send_updates_priv(&mut self) {
        let mut updates= Vec::new();

        for (_gate_id, (gate, ports)) in self.to_update.iter() {
            let mut signals = Vec::new();

            for port in ports {
                let sig = gate.borrow().get_output(port.clone());
                signals.push(PortUpdate { port: port.clone(), bits: sig.bits, avec: sig.avec, bvec: sig.bvec });
            }

            updates.push(UpdateStruct {
                graph_id: gate.borrow().graph_id(),
                gate_id: gate.borrow().get_id(),
                val: signals
            });
        }
        self.to_update = HashMap::new(); 
        sendUpdates(self.tick, false, updates);
    }

    #[wasm_bindgen(js_name = changeInput)]
    pub fn change_input(&mut self, graph_id: String, gate_id: String, sig: SigParams) {
        let gate = self.graphs.get(&graph_id).unwrap().borrow().get_gate(gate_id);
        self.set_gate_output_signal_priv(
            &gate, 
            String::from("out"), 
            Vec3vl::new(sig.get_bits(), sig.get_avec(), sig.get_bvec())
        );
    }

    #[wasm_bindgen(js_name = addSubcircuit)]
    pub fn add_subcircuit(&mut self, graph_id: String, gate_id: String, subgraph_id: String, io_map: Vec<IOmap>) {
        let graph = self.graphs.get(&graph_id).unwrap();
        let gate = graph.borrow().get_gate(gate_id);
        let subgraph = self.graphs.get(&subgraph_id).unwrap().clone();

        gate.borrow_mut().set_subgraph(subgraph.clone());

        subgraph.borrow_mut().set_subcircuit(gate.clone());

        let mut map = HashMap::new();
        for i in io_map {
            let port = i.get_port();
            let io_id = i.get_io_id();

            map.insert(port.clone(), io_id.clone());
            let io = subgraph.borrow().get_gate(io_id);

            match gate.borrow().get_port_dir(port.clone()) {
                IoDir::In => self.set_gate_output_signal_priv(&io, "out".to_string(), gate.borrow().get_input(port)),
                IoDir::Out => self.set_gate_output_signal_priv(&gate, port, io.borrow().get_input("in".to_string())),
            };  
        }

        gate.borrow_mut().subgraph_io_map = Some(map);
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