use std::collections::HashMap;

use crate::cell_arith::BigInt;
use crate::operations::ClockHack;
use crate::vector3vl::Vec3vl;

pub type MuxIdx = fn(sel: &mut Vec3vl) -> Option<String>;

pub fn mux_op(args: HashMap<String, Vec3vl>, op: &mut MuxIdx) -> ClockHack {
    let idx = op(&mut args.get("sel").unwrap().clone());
    ClockHack::Normal(vec![(
        "out".to_string(), 
        if let Some(i) = idx {
            args.get(&format!("in{}", i)).unwrap().clone()
        } else {
            Vec3vl::xes(args.get("in0").unwrap().bits)
        }
    )])
}

pub fn sparse_mux_op(_args: HashMap<String, Vec3vl>) -> ClockHack {
    todo!()
}

pub fn mux_idx(sel: &mut Vec3vl) -> Option<String> {
    if sel.is_fully_defined() {
        Some(BigInt::from_str_radix(&sel.to_hex(), 16).unwrap().to_str_radix(10))
    } else {
        None
    }
}

pub fn mux1hot_idx(sel: &mut Vec3vl) -> Option<String> {    
    if sel.is_fully_defined() {
        let v = BigInt::from_str_radix(&sel.to_hex(), 16).unwrap();
        if v.count_ones() <= 1 {
            Some((BigInt::BITS - v.leading_zeros()).to_string())
        } else {
            None
        }
    } else {
        None
    }
}

