use std::cell::RefCell;
use std::collections::{hash_map::Iter, HashMap, HashSet};
use std::rc::Rc;

use crate::graph::GraphPtr;
use crate::js_types::{DffPolarityStruct, JsGateParams, PortParams};
use crate::link::LinkTarget;
use crate::operations::{Operation, ReturnValue};
use crate::vector3vl::Vec3vl;

pub type GatePtr = Rc<RefCell<Gate>>;

pub struct Gate {
    id: String,
    graph: GraphPtr,
    in_vals: HashMap<String, Vec3vl>,
    out_vals: HashMap<String, Vec3vl>,
    links: HashSet<String>,
    linked_to: HashMap<String, Vec<LinkTarget>>,
    propagation: u32,
    gate_type: String,
    subgraph: Option<GraphPtr>,
    subgraph_io_map: Option<HashMap<String, String>>,
    subgraph_net: Option<String>,
    io_dirs: HashMap<String, IoDir>,
    operation: Operation,
    monitors: HashMap<String, Vec<u32>>
}

#[derive(Clone, PartialEq)]
pub enum IoDir {
    In, Out
}

impl Gate {
    pub fn new(graph: GraphPtr, graph_id: String, gate_id: String, gate_params: JsGateParams, port_params: Vec<PortParams>) -> Result<GatePtr, String> {
        let propagation = gate_params.get_propagation().unwrap_or(0);
        let subgraph_net = gate_params.get_net();

        let op_type = gate_params.get_type();
        let op = Operation::from_name(op_type.clone(), gate_params, graph_id, gate_id.clone())?;

        let mut g = Gate {
            id: gate_id,
            graph,
            in_vals: HashMap::new(),
            out_vals: HashMap::new(),
            links: HashSet::new(),
            linked_to: HashMap::new(),
            propagation,
            gate_type: op_type,
            subgraph: None,
            subgraph_io_map: None,
            subgraph_net,
            io_dirs: HashMap::new(),
            operation: op,
            monitors: HashMap::new()
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

        Ok(Rc::new(RefCell::new(g)))
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn graph_id(&self) -> String {
        self.graph.borrow().get_id()
    }

    pub fn add_link_to(&mut self, port: &String, target: LinkTarget) -> Result<(), String> {
        match self.linked_to.get_mut(port) {
            Some(v) => { 
                v.push(target);
                Ok(())
            },
            None => Err(format!("Gate {} has no port {}", self.id, port))
        }
    }

    pub fn remove_link_to(&mut self, port: &String, target: LinkTarget) -> Result<(), String> {
        match self.linked_to.get_mut(port) {
            Some(v) => {
                v.retain(|t| t.id != target.id);
                Ok(())
            },
            None => Err(format!("Gate {} has no port {}", self.id, port))
        }
    }

    pub fn add_link(&mut self, link_id: String) {
        self.links.insert(link_id);
    }

    pub fn remove_link(&mut self, link_id: &String) {
        self.links.remove(link_id);
    }

    pub fn get_links_iter(&self) -> std::collections::hash_set::Iter<'_, String> {
        self.links.iter()
    }

    pub fn get_targets(&self, port: &String) -> Result<Vec<LinkTarget>, String> {
        match self.linked_to.get(port) {
            Some(l) => Ok(l.clone()),
            None => Err(format!("Gate {} has no port {}", self.id, port))
        }
    }

    pub fn get_propagation(&self) -> u32 {
        self.propagation
    }

    pub fn get_inputs(&self) -> HashMap<String, Vec3vl> {
        self.in_vals.clone()
    }

    pub fn get_input(&self, port: &String) -> Result<Vec3vl, String> {
        match self.in_vals.get(port) {
            Some(i) => Ok(i.clone()),
            None => Err(format!("Gate {} has no port {}", self.id, port)) 
        }
    }

    pub fn get_output(&self, port: &String) -> Result<Vec3vl, String> {
        match self.out_vals.get(port) {
            Some(o ) => Ok(o.clone()),
            None => Err(format!("Gate {} has no port {}", self.id, port))
        }
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

    pub fn get_subgraph(&self) -> Result<GraphPtr, String> {
        match &self.subgraph {
            Some(s ) => Ok(s.clone()),
            None => Err(format!("Gate {} has no subgraph", self.id))
        }
    }

    pub fn is_subcircuit(&self) -> bool {
        self.subgraph.is_some()
    }

    pub fn get_port_dir(&self, port: &String) -> Result<IoDir, String> {
        match self.io_dirs.get(port) {
            Some(i) => Ok(i.clone()),
            None => Err(format!("Gate {} has no port {}", self.id, port))
        }
    }

    pub fn is_output(&self) -> bool {
        self.gate_type == "Output"
    }

    pub fn do_operation(&mut self, args: &HashMap<String, Vec3vl>) -> Result<ReturnValue, String> {
        self.operation.op(args)
    }

    pub fn set_subgraph_iomap(&mut self, map: HashMap<String, String>) {
        self.subgraph_io_map = Some(map);
    }

    pub fn get_subgraph_iomap_port(&self, port: &String) -> Result<String, String> {
        match &self.subgraph_io_map {
            Some(iomap) => {
                match iomap.get(port) {
                    Some(i) => Ok(i.clone()),
                    None => Err(format!("Gate {} has no port {}", self.id, port))
                }
            },
            None => Err("Subgraph has no io map".to_string())
        }
    }

    pub fn get_subcir_net(&self) -> Result<String, String> {
        match &self.subgraph_net {
            Some(n) => Ok(n.clone()),
            None => Err("Subcircuit has no net".to_string())
        }
    }

    pub fn iodirs_iter(&self) -> Iter<'_, String, IoDir> {
        self.io_dirs.iter()
    }

    pub fn monitor(&mut self, port: String, monitor_id: u32) {
        self.monitors.entry(port).or_default().push(monitor_id);
    }

    pub fn unmonitor(&mut self, port: &String, monitor_id: u32) {
        if let Some(vec) = self.monitors.get_mut(port) {
            vec.retain(|v| *v != monitor_id);
        }
    }

    pub fn get_monitors(&self, port: &String) -> std::slice::Iter<'_, u32> {
        match self.monitors.get(port) {
            Some(vec) => vec.iter(),
            None => [].iter()
        }
    }

    pub fn set_memory(&mut self, addr: u32, data: Vec3vl) -> Result<(), String> {
        match &mut self.operation {
            Operation::Memory(memory_state) => {
                memory_state.memory[addr as usize] = data;
                Ok(())
            },
            _ => Err(format!("Attempting to chenge memory in gate id {} type {}", self.id, self.operation.get_type()))
        }

    }
}

#[derive(Clone, Copy)]
pub struct PolarityOptions {
    pub aload:  Option<bool>,
    pub arst:   Option<bool>,
    pub clock:  Option<bool>,
    pub clr:    Option<bool>,
    pub enable: Option<bool>,
    pub set:    Option<bool>,
}

impl PolarityOptions {
    pub fn new(options: Option<DffPolarityStruct>) -> PolarityOptions {
        if let Some(o) = options {
            PolarityOptions { 
                aload:  o.get_aload(), 
                arst:   o.get_arst(), 
                clock:  o.get_clock(), 
                clr:    o.get_clr(), 
                enable: o.get_enable(), 
                set:    o.get_set() 
            }
        } else {
            PolarityOptions {
                aload:  None,
                arst:   None,
                clock:  None, 
                clr:    None, 
                enable: None, 
                set:    None 
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct SliceOptions {
    pub count: u32,
    pub first: u32,
    pub total: u32,
}

impl SliceOptions {
    pub fn new(gate_params: JsGateParams) -> SliceOptions {
        match gate_params.get_slice() {
            Some(o) => SliceOptions { 
                count: o.get_count(), 
                first: o.get_first(), 
                total: o.get_total() 
            },
            None => SliceOptions {
                count: 1,
                first: 0,
                total: 2
            }
            
        }
    }
}