use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use crate::{graph::GraphPtr, js_types::PolarityStruct};
use crate::js_types::{JsGateParams, PortParams, SliceType};
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
        let params = GateParams::new(gate_params);
        let op = Operation::from_name(op_type, &params);
        
        let mut g = Gate {
            id,
            graph,
            in_vals: HashMap::new(),
            out_vals: HashMap::new(),
            links: HashSet::new(),
            linked_to: HashMap::new(),
            params,
            subgraph: None,
            subgraph_io_map: None,
            io_dirs: HashMap::new(),
            operation: op,
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

    pub fn add_link_to(&mut self, port: String, target: LinkTarget) -> Result<(), String> {
        match self.linked_to.get_mut(&port) {
            Some(l) => { 
                l.push(target);
                Ok(())
            },
            None => Err(format!("Gate {} has no port {}", self.id, port))
        }
    }

    pub fn add_link(&mut self, link_id: String) {
        self.links.insert(link_id);
    }

    pub fn get_targets(&self, port: String) -> Result<Vec<LinkTarget>, String> {
        match self.linked_to.get(&port) {
            Some(l) => Ok(l.clone()),
            None => Err(format!("Gate {} has no port {}", self.id, port))
        }
    }

    pub fn get_propagation(&self) -> u32 {
        self.params.propagation
    }

    pub fn get_inputs(&self) -> HashMap<String, Vec3vl> {
        self.in_vals.clone()
    }

    pub fn get_input(&self, port: String) -> Result<Vec3vl, String> {
        match self.in_vals.get(&port) {
            Some(i) => Ok(i.clone()),
            None => Err(format!("Gate {} has no port {}", self.id, port)) 
        }
    }

    pub fn get_output(&self, port: String) -> Result<Vec3vl, String> {
        match self.out_vals.get(&port) {
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

    pub fn get_port_dir(&self, port: String) -> Result<IoDir, String> {
        match self.io_dirs.get(&port) {
            Some(i) => Ok(i.clone()),
            None => Err(format!("Gate {} has no port {}", self.id, port))
        }
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
    pub arst_value:   Option<String>,
    pub bits:         u32,
    pub net:          Option<String>,
    pub numbase:      Option<String>,
    pub propagation:  u32,
    pub gate_type:    String,
    pub slice:        Option<SliceOptions>,
    pub polarity:     PolarityOptions,
    pub left_op:      Option<bool>,
    pub constant_str: Option<String>,
    pub constant_num: Option<u32>
}

impl GateParams {
    pub fn new(params: JsGateParams) -> GateParams {
        let (c_num, c_str) = if params.get_type() == "Constant" {
            (None, params.get_constant_str())
        } else {
            (params.get_constant_num(), None)
        };

        GateParams {
            arst_value:     params.get_arst_value(),
            bits:           params.get_bits(),
            net:            params.get_net(),
            numbase:        params.get_numbase(),
            propagation:    params.get_propagation(),
            gate_type:      params.get_type(),
            slice:          SliceOptions::new(params.get_slice()),
            polarity:       PolarityOptions::new(params.get_polarity()),
            left_op:        params.get_left_op(),
            constant_str:   c_str,
            constant_num:   c_num
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
    pub fn new(options: Option<PolarityStruct>) -> PolarityOptions {
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
    pub fn new(options: Option<SliceType>) -> Option<SliceOptions> {
        options.map(|o| SliceOptions { 
            count: o.get_count(), 
            first: o.get_first(), 
            total: o.get_total() 
        })
    }
}