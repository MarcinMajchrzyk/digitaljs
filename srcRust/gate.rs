use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use crate::{js_types::{GateParams, PortParams}, link::LinkTarget, vector3vl::Vec3vl};

pub type OperationType = fn(args: HashMap<String, Vec3vl>) -> Vec<(String, Vec3vl)>;

pub type GatePtr = Rc<RefCell<Gate>>;

pub struct Gate {
    id: String,
    graph_id: String,
    in_vals: HashMap<String, Vec3vl>,
    out_vals: HashMap<String, Vec3vl>,
    links: HashSet<String>,
    linked_to: HashMap<String, Vec<LinkTarget>>,   // key: output_port_id, vals: connected gates
    pub operation: OperationType,
}

impl Gate {
    pub fn new(graph_id: String, id: String, _gate_params: GateParams, port_params: Vec<PortParams>) -> GatePtr {
        // TODO this is dumb
        let op = if id.chars().nth(0).unwrap() == 'g' {
            |args: HashMap<String, Vec3vl>| {
                let mut iter = args.values();
                let ret = iter.next().unwrap();
                vec![(
                    "out".to_string(), 
                    iter.fold(ret.clone(), |acc, a| {
                        acc.and(a.clone())
                    })
                )]
            }
        } else {
            |_| { vec![] }
        };
        
        let mut g = Gate {
            id,
            graph_id,
            in_vals: HashMap::new(),
            out_vals: HashMap::new(),
            links: HashSet::new(),
            linked_to: HashMap::new(),
            operation: op,
        };

        for p in port_params {
            if p.get_dir() == "in" {
                g.in_vals.insert(p.get_id(), Vec3vl::new(p.get_bits(), p.get_avec(), p.get_bvec()));
            } else {
                g.out_vals.insert(p.get_id(), Vec3vl::new(p.get_bits(), p.get_avec(), p.get_bvec()));
                g.linked_to.insert(p.get_id(), Vec::new());
            }
        }

        Rc::new(RefCell::new(g))
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn graph_id(&self) -> String {
        self.graph_id.clone()
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
        1   // TODO
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