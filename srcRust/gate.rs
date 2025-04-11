use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use crate::{graph::GraphPtr, js_types::{JsGateParams, PortParams}};
use crate::link::LinkTarget;
use crate::operations::Operation;
use crate::vector3vl::Vec3vl;

pub type GatePtr = Rc<RefCell<Gate>>;

pub struct Gate {
    id: String,
    graph: GraphPtr,
    in_vals: HashMap<String, Vec3vl>,
    out_vals: HashMap<String, Vec3vl>,
    links: HashSet<String>,
    linked_to: HashMap<String, Vec<LinkTarget>>,
    pub params: GateParams,
    subgraph: Option<GraphPtr>,
    pub subgraph_io_map: Option<HashMap<String, String>>,
    io_dirs: HashMap<String, IoDir>,
    pub operation: Operation,
}

#[derive(Clone)]
pub enum IoDir {
    In, Out
}

impl Gate {
    pub fn new(graph: GraphPtr, id: String, gate_params: JsGateParams, port_params: Vec<PortParams>) -> GatePtr {
        let op_type = gate_params.get_type();
        
        let mut g = Gate {
            id,
            graph,
            in_vals: HashMap::new(),
            out_vals: HashMap::new(),
            links: HashSet::new(),
            linked_to: HashMap::new(),
            params: GateParams::new(gate_params),
            subgraph: None,
            subgraph_io_map: None,
            io_dirs: HashMap::new(),
            operation: Operation::from_name(op_type),
        };

        for p in port_params {
            if p.get_dir() == "in" {
                g.in_vals.insert(p.get_id(), Vec3vl::new(p.get_bits(), p.get_avec(), p.get_bvec()));
                g.io_dirs.insert(p.get_id(), IoDir::In);
            } else {
                g.out_vals.insert(p.get_id(), Vec3vl::new(p.get_bits(), p.get_avec(), p.get_bvec()));
                g.linked_to.insert(p.get_id(), Vec::new());
                g.io_dirs.insert(p.get_id(), IoDir::Out);
            }
        }

        Rc::new(RefCell::new(g))
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn graph_id(&self) -> String {
        self.graph.borrow().get_id()
    }

    pub fn add_link_to(&mut self, port: String, target: LinkTarget) {
        self.linked_to.get_mut(&port).unwrap().push(target);
    }

    pub fn add_link(&mut self, link_id: String) {
        self.links.insert(link_id);
    }

    pub fn get_targets(&self, port: String) -> Vec<LinkTarget> {
        self.linked_to.get(&port).unwrap().clone()
    }

    pub fn get_propagation(&self) -> u32 {
        self.params.propagation
    }

    pub fn get_inputs(&self) -> HashMap<String, Vec3vl> {
        self.in_vals.clone()
    }

    pub fn get_input(&self, port: String) -> Vec3vl {
        self.in_vals.get(&port).unwrap().clone()
    }

    pub fn get_output(&self, port: String) -> Vec3vl {
        self.out_vals.get(&port).unwrap().clone()
    }

    pub fn set_output(&mut self, port: String, sig: Vec3vl) {
        self.out_vals.insert(port, sig);
    }

    pub fn set_input(&mut self, port: String, sig: Vec3vl) {
        self.in_vals.insert(port, sig);
    }

    pub fn get_graph(&self) -> GraphPtr {
        self.graph.clone()
    }

    pub fn set_subgraph(&mut self, subgraph: GraphPtr) {
        self.subgraph = Some(subgraph);
    }

    pub fn get_subgraph(&self) -> GraphPtr {
        match &self.subgraph {
            Some(s ) => s.clone(),
            None => panic!()
        }
    }

    pub fn is_subcircuit(&self) -> bool {
        self.subgraph.is_some()
    }

    pub fn get_port_dir(&self, port: String) -> IoDir {
        self.io_dirs.get(&port).unwrap().clone()
    }

    pub fn is_output(&self) -> bool {
        self.params.gate_type == "Output"
    }

    fn is_special(t: String) -> bool {
        matches!(t.as_str(),
            "Subcircuit" |
            "Input" |
            "Output" |
            "Button" |
            "Lamp" |
            "NumEntry" |
            "NumDisplay"
        )
    }
}

pub struct GateParams {
    pub bits:           u32,
    pub net:            Option<String>,
    pub numbase:        Option<String>,
    pub propagation:    u32,
    pub gate_type:      String,
}

impl GateParams {
    pub fn new(params: JsGateParams) -> GateParams {
        GateParams { 
            bits:           params.get_bits(),
            net:            params.get_net(),
            numbase:        params.get_numbase(),
            propagation:    params.get_propagation(),
            gate_type:      params.get_type()
        }
    }
}