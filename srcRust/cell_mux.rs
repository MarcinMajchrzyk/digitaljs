use std::collections::HashMap;

use crate::cell_arith::BigInt;
use crate::operations::ReturnValue;
use crate::vector3vl::Vec3vl;

pub type MuxIdx = fn(sel: &mut Vec3vl) -> Option<String>;

pub fn mux_op(args: &HashMap<String, Vec3vl>, bits: u32, op: &mut MuxIdx) -> Result<ReturnValue, String> {
    let sel = match args.get("sel") {
        Some(s) => &mut s.clone(),
        None => return Err("No selector input found".to_string())
    };

    let idx = op(sel);
    let val = if let Some(i) = idx {
        match args.get(&format!("in{}", i)) {
            Some(a) => a.clone(),
            None => return Err(format!("No input in{}", i))
        }
    } else {
        Vec3vl::xes(bits)
    };
    
    ReturnValue::out(val)
}

pub fn sparse_mux_op(args: &HashMap<String, Vec3vl>, bits: u32, map: &Option<HashMap<String, String>>) -> Result<ReturnValue, String> {
    let sel = match args.get("sel") {
        Some(s) => &mut s.clone(),
        None => return Err("No selector input found".to_string())
    };

    let num = sel.to_hex();

    let selections = match map {
        Some(m ) => m,
        None => return Err("Sparse mutex has no port map".to_string())
    };

    let val = match selections.get(&num) {
        Some(port) => {
            match args.get(port) {
                Some(val) => val.clone(),
                None => return Err(format!("No port named {port}"))
            }
        },
        None => Vec3vl::xes(bits)
    };

    ReturnValue::out(val)
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

