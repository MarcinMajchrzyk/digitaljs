use std::collections::HashMap;

use crate::vector3vl::Vec3vl;

pub type Monop = fn(i: Vec3vl) -> Vec3vl;
pub type Binop = fn(l: Vec3vl, r: Vec3vl) -> Vec3vl;

pub enum Operation {
    Gate11(Monop),
    GateX1(Binop),
    None
}

impl Operation {
    pub fn from_name(name: String) -> Operation {
        match name.as_str() {
            "Not"   => Operation::Gate11(not),
            "And"   => Operation::GateX1(and),
            "Or"    => Operation::GateX1(or),
            "Xor"   => Operation::GateX1(xor),
            "Nand"  => Operation::GateX1(nand),
            "Nor"   => Operation::GateX1(nor),
            "Xnor"  => Operation::GateX1(xnor), 
            _       => Operation::None
        }
    }

    pub fn op(&mut self, args: HashMap<String, Vec3vl>) -> Vec<(String, Vec3vl)> {
        match self {
            Operation::Gate11(op) => {
                vec![("out".to_string(), op(args.values().next().unwrap().clone()))]
            },
            Operation::GateX1(op) => {
                let mut iter = args.values();
                let init = iter.next().unwrap().clone();

                let r = iter.fold(init, |acc, x| {
                    op(acc.clone(), x.clone())
                });

                vec![("out".to_string(), r.clone())]
                //todo!()
            },
            Operation::None => vec![]
        }
    }
}

fn not(o: Vec3vl) -> Vec3vl { o.not() }

fn and (l: Vec3vl, r: Vec3vl) -> Vec3vl { l.and(r)  }
fn or  (l: Vec3vl, r: Vec3vl) -> Vec3vl { l.or(r)   }
fn xor (l: Vec3vl, r: Vec3vl) -> Vec3vl { l.xor(r)  }
fn nand(l: Vec3vl, r: Vec3vl) -> Vec3vl { l.nand(r) }
fn nor (l: Vec3vl, r: Vec3vl) -> Vec3vl { l.nor(r)  }
fn xnor(l: Vec3vl, r: Vec3vl) -> Vec3vl { l.xnor(r) }
