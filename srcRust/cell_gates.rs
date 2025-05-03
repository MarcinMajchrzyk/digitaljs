use std::collections::HashMap;

use crate::operations::ClockHack;
use crate::vector3vl::Vec3vl;

pub type Monop = fn(i: Vec3vl) -> Vec3vl;
pub type Binop = fn(l: Vec3vl, r: Vec3vl) -> Vec3vl;

pub fn gate_11(op: &Monop, args: &HashMap<String, Vec3vl>) -> ClockHack {
  ClockHack::Normal(
    vec![
      ("out".to_string(), op(args.values().next().unwrap().clone()))
      ]
    )
}

pub fn gate_x1(op: &Binop, args: &HashMap<String, Vec3vl>) -> ClockHack {
  let mut iter = args.values();
  let init = iter.next().unwrap().clone();

  let r = iter.fold(init, |acc, x| {
    op(acc.clone(), x.clone())
  });

  ClockHack::Normal(vec![("out".to_string(), r.clone())])
}

pub fn not(o: Vec3vl) -> Vec3vl { o.not() }

pub fn and (l: Vec3vl, r: Vec3vl) -> Vec3vl { l.and(r)  }
pub fn or  (l: Vec3vl, r: Vec3vl) -> Vec3vl { l.or(r)   }
pub fn xor (l: Vec3vl, r: Vec3vl) -> Vec3vl { l.xor(r)  }
pub fn nand(l: Vec3vl, r: Vec3vl) -> Vec3vl { l.nand(r) }
pub fn nor (l: Vec3vl, r: Vec3vl) -> Vec3vl { l.nor(r)  }
pub fn xnor(l: Vec3vl, r: Vec3vl) -> Vec3vl { l.xnor(r) }