use std::collections::HashMap;

use crate::operations::ClockHack;
use crate::vector3vl::Vec3vl;

pub type Monop = fn(i: Vec3vl) -> Vec3vl;
pub type Binop = fn(l: Vec3vl, r: Vec3vl) -> Result<Vec3vl, String>;
pub type ReduceFn = fn(&Vec3vl) -> Vec3vl;

pub fn gate_11(op: &Monop, args: &HashMap<String, Vec3vl>) -> Result<ClockHack, String> {
    let vec = match args.values().next() {
        Some(v) => v.clone(),
        None => return Err("No input values".to_string())
    };
    
    Ok(ClockHack::Normal(vec![("out".to_string(), op(vec))]))
}

pub fn gate_x1(op: &Binop, args: &HashMap<String, Vec3vl>) -> Result<ClockHack, String> {
    let mut iter = args.values();
    let mut acc = match iter.next() {
        Some(i) => i.clone(),
        None => return Err("No input values".to_string())
    };

    for x in iter {
        acc = op(acc, x.clone())?;
    }

    Ok(ClockHack::Normal(vec![("out".to_string(), acc.clone())]))
}

pub fn gate_reduce(op: &ReduceFn, args: &HashMap<String, Vec3vl>) -> Result<ClockHack, String> {
    let vec = match args.get("in") {
        Some(v) => v,
        None => return Err("".to_string())
    };

    Ok(ClockHack::Normal(vec![("out".to_string(), op(vec))]))
}

pub fn not(o: Vec3vl) -> Vec3vl { o.not() }

pub fn and (l: Vec3vl, r: Vec3vl) -> Result<Vec3vl, String> { l.and(r)  }
pub fn or  (l: Vec3vl, r: Vec3vl) -> Result<Vec3vl, String> { l.or(r)   }
pub fn xor (l: Vec3vl, r: Vec3vl) -> Result<Vec3vl, String> { l.xor(r)  }
pub fn nand(l: Vec3vl, r: Vec3vl) -> Result<Vec3vl, String> { l.nand(r) }
pub fn nor (l: Vec3vl, r: Vec3vl) -> Result<Vec3vl, String> { l.nor(r)  }
pub fn xnor(l: Vec3vl, r: Vec3vl) -> Result<Vec3vl, String> { l.xnor(r) }