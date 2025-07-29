use std::cell::RefCell;
use std::collections::{hash_map::Iter, HashMap, HashSet};
use std::rc::Rc;

use wasm_bindgen::JsValue;

use crate::cell_memory::MemoryPortPolarity;
use crate::graph::GraphPtr;
use crate::js_types::{DffPolarityStruct, JsGateParams, PortParams, SliceType};
use crate::link::LinkTarget;
use crate::operations::{ClockHack, Operation};
use crate::vector3vl::Vec3vl;

pub type GatePtr = Rc<RefCell<Gate>>;

pub struct Gate {
    id: String,
    graph: GraphPtr,
    in_vals: HashMap<String, Vec3vl>,
    out_vals: HashMap<String, Vec3vl>,
    links: HashSet<String>,
    linked_to: HashMap<String, Vec<LinkTarget>>,
    params: GateParams,
    subgraph: Option<GraphPtr>,
    subgraph_io_map: Option<HashMap<String, String>>,
    io_dirs: HashMap<String, IoDir>,
    operation: Operation,
    monitors: HashMap<String, Vec<u32>>
}

#[derive(Clone, PartialEq)]
pub enum IoDir {
    In, Out
}

impl Gate {
    pub fn new(graph: GraphPtr, graph_id: String, id: String, gate_params: JsGateParams, port_params: Vec<PortParams>) -> Result<GatePtr, String> {
        let op_type = gate_params.get_type();
        let params = GateParams::new(gate_params, id.clone(), graph_id);
        let op = Operation::from_name(op_type, &params)?;

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

    pub fn do_operation(&mut self, args: HashMap<String, Vec3vl>) -> Result<ClockHack, String> {
        self.operation.op(args)
    }

    pub fn set_subgraph_iomap(&mut self, map: HashMap<String, String>) {
        self.subgraph_io_map = Some(map);
    }

    pub fn get_subgraph_iomap_port(&self, port: String) -> Result<String, String> {
        match &self.subgraph_io_map {
            Some(iomap) => {
                match iomap.get(&port) {
                    Some(i) => Ok(i.clone()),
                    None => Err(format!("Gate {} has no port {}", self.id, port))
                }
            },
            None => Err("Subgraph has no io map".to_string())
        }
    }

    pub fn get_subcir_net(&self) -> Result<String, String> {
        match &self.params.net {
            Some(n) => Ok(n.clone()),
            None => Err("Subcircuit has no net".to_string())
        }
    }

    pub fn iodirs_iter(&self) -> Iter<'_, String, IoDir> {
        self.io_dirs.iter()
    }

    pub fn monitor(&mut self, port: String, monitor_id: u32) {
        match self.monitors.get_mut(&port) {
            Some(vec) => { 
                vec.push(monitor_id);
            },
            None => {
                self.monitors.insert(port, vec![monitor_id]);
            } 
        }
    }

    pub fn unmonitor(&mut self, port: String, monitor_id: u32) {
        if let Some(vec) = self.monitors.get_mut(&port) {
            vec.retain(|v| *v != monitor_id);
        }
    }

    pub fn get_monitors(&self, port: String) -> std::slice::Iter<'_, u32> {
        match self.monitors.get(&port) {
            Some(vec) => vec.iter(),
            None => [].iter()
        }
    }
}

pub struct GateParams {
    pub gate_id:        String,
    pub graph_id:       String,
    pub arst_value:     Option<String>,
    pub bits:           u32,
    pub net:            Option<String>,
    pub numbase:        Option<String>,
    pub propagation:    u32,
    pub gate_type:      String,
    pub slice:          Option<SliceOptions>,
    pub polarity:       PolarityOptions,
    pub left_op:        Option<bool>,
    pub constant_str:   Option<String>,
    pub constant_num:   Option<u32>,
    pub abits:          Option<u32>,
    pub offset:         Option<u32>,
    pub words:          Option<u32>,
    pub memdata:        Option<Vec<Vec3vl>>,
    pub rdports:        Vec<MemoryPortPolarity>,
    pub wrports:        Vec<MemoryPortPolarity>,
    pub inputs:         Option<HashMap<String, String>>,
}

impl GateParams {
    pub fn new(params: JsGateParams, gate_id: String, graph_id: String) -> GateParams {
        let (c_num, c_str) = if params.get_type() == "Constant" {
            (None, params.get_constant_str())
        } else {
            (params.get_constant_num(), None)
        };

        let bits = match params.get_type().as_str() {
            "Mux" | "Mux1Hot" | "MuxSparse" => {
                params.get_bits_mux().get_bits_in()
            },
            _ => {
                params.get_bits()
            }
            
        };

        let rdports = match params.get_rdports() {
            Some(v) => v.iter().map(MemoryPortPolarity::new).collect(),
            None => vec![]
        };

        let wrports = match params.get_wrports() {
            Some(v) => v.iter().map(MemoryPortPolarity::new).collect(),
            None => vec![]
        };

        let inputs = params.get_inputs().map(|v| {
            v.iter().enumerate().map(|(idx, b)| {
                (b.toString(16), format!("in{idx}"))
            }).collect()
        });

        GateParams {
            gate_id,
            graph_id,
            arst_value:     params.get_arst_value(),
            bits,
            net:            params.get_net(),
            numbase:        params.get_numbase(),
            propagation:    params.get_propagation(),
            gate_type:      params.get_type(),
            slice:          SliceOptions::new(params.get_slice()),
            polarity:       PolarityOptions::new(params.get_polarity()),
            left_op:        params.get_left_op(),
            constant_str:   c_str,
            constant_num:   c_num,
            abits:          params.get_abits(),
            offset:         params.get_offset(),
            words:          params.get_words(),
            memdata:        load_memory(params.get_memdata(), bits),
            rdports,
            wrports,
            inputs
        }
    }
}

fn load_memory(memory: Option<Vec<JsValue>>, size: u32) -> Option<Vec<Vec3vl>> {
    memory.map(|v| {
        let mut mem = Vec::new();

        let mut n = 0usize;
        while n < v.len() {
            if let Some(s) = v[n].as_string() {
                mem.push(Vec3vl::from_binary(s, Some(size as usize)));
            } else if let Some(f) = v[n].as_f64() {
                n += 1;
                let val = Vec3vl::from_binary(v[n].as_string().unwrap(), Some(size as usize));
                let count = f as u32;
                for _ in 0..count {
                    mem.push(val.clone());
                }
            }
            n += 1;
        }
        
        mem
    })
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
    pub fn new(options: Option<SliceType>) -> Option<SliceOptions> {
        options.map(|o| SliceOptions { 
            count: o.get_count(), 
            first: o.get_first(), 
            total: o.get_total() 
        })
    }
}