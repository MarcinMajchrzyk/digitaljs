use std::cell::RefCell;
use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::rc::Rc;

use crate::gate::{Gate, GatePtr};
use crate::js_types::{JsGateParams, PortParams};
use crate::link::{Link, LinkTarget};

pub type GraphPtr = Rc<RefCell<Graph>>;

pub struct Graph {
    id:         String,
    gates:      HashMap<String, GatePtr>,
    links:      HashMap<String, Link>,
    subcircuit: Option<GatePtr>,
    observed:   bool,
}

impl Graph {
    pub fn new(id: String) -> GraphPtr {
        Rc::new(RefCell::new(Graph {
            id,
            gates:      HashMap::new(),
            links:      HashMap::new(),
            subcircuit: None,
            observed:   false,
        }))
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn add_link(&mut self, link_id: String, source: LinkTarget, target: LinkTarget) -> Result<(), String> {
        self.links.insert(link_id.clone(), Link { from: source.clone(), to: target.clone() });
        
        let source_gate = self.get_gate(source.id.clone())?;
        let target_gate = self.get_gate(target.id.clone())?;

        source_gate.borrow_mut().add_link_to(source.port, target)?;
        source_gate.borrow_mut().add_link(link_id.clone());
        target_gate.borrow_mut().add_link(link_id);
        Ok(())
    }

    pub fn add_gate(&mut self, graph: GraphPtr, gate_id: String, gate_params: JsGateParams, port_params: Vec<PortParams>) -> Result<(), String> {
        self.gates.insert(
            gate_id.clone(), 
            Gate::new(graph, self.id.clone(), gate_id, gate_params, port_params)?
        );
        Ok(())
    }

    pub fn get_gate(&self, gate_id: String) -> Result<GatePtr, String> {
        match self.gates.get(&gate_id) {
            Some(g) => Ok(g.clone()),
            None => Err(format!("Graph {} has no gate {}", self.id, gate_id))
        }
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

    pub fn set_subcircuit(&mut self, gate: GatePtr) {
        self.subcircuit = Some(gate);
    }

    pub fn subcircuit(&self) -> Option<GatePtr> {
        self.subcircuit.clone()
    }

    pub fn gate_iter(&self) -> Values<'_, String, GatePtr> {
        self.gates.values()
    }
}