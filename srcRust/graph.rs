use std::collections::HashMap;

use crate::gate::{Gate, GatePtr};
use crate::js_types::{GateParams, PortParams};
use crate::link::{Link, LinkTarget};

pub struct Graph {
    id:         String,
    gates:      HashMap<String, GatePtr>,
    links:      HashMap<String, Link>,
    observed:   bool,
}

impl Graph {
    pub fn new(id: String) -> Graph {
        Graph {
            id,
            gates: HashMap::new(),
            links: HashMap::new(),
            observed: false,
        }
    }

    pub fn add_link(&mut self, link_id: String, source: LinkTarget, target: LinkTarget) {
        self.links.insert(link_id.clone(), Link { from: source.clone(), to: target.clone() });
        
        let source_gate = self.gates.get(&source.id).unwrap();
        let target_gate = self.gates.get(&target.id).unwrap();

        source_gate.borrow_mut().add_link_to(source.port, target);
        source_gate.borrow_mut().add_link(link_id.clone());
        target_gate.borrow_mut().add_link(link_id);
    }

    pub fn add_gate(&mut self, gate_id: String, gate_params: GateParams, port_params: Vec<PortParams>) {
        self.gates.insert(
            gate_id.clone(), 
            Gate::new(self.id.clone(), gate_id, gate_params, port_params)
        );
    }

    pub fn get_gate(&self, gate_id: String) -> GatePtr {
        self.gates.get(&gate_id).unwrap().clone()
    }

    pub fn observe(&mut self) {
        self.observed = true;
    }

    pub fn unobserve(&mut self) {
        self.observed = false;
    }

    pub fn observed(&self) -> bool {
        self.observed
    }
}