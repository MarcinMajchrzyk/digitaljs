use std::collections::HashMap;

use crate::cell_arith::BigInt;
use crate::operations::ClockHack;
use crate::vector3vl::Vec3vl;

pub type MuxIdx = fn(sel: &mut Vec3vl) -> Option<String>;

pub fn mux_op(args: HashMap<String, Vec3vl>, op: &mut MuxIdx) -> Result<ClockHack, String> {
    let sel = match args.get("sel") {
        Some(s) => &mut s.clone(),
        None => return Err("No selector input found".to_string())
    };

    let idx = op(sel);
    Ok(ClockHack::Normal(vec![(
        "out".to_string(), 
        if let Some(i) = idx {
            match args.get(&format!("in{}", i)) {
                Some(a) => a.clone(),
                None => return Err(format!("No input in{}", i))
            }
        } else {
            Vec3vl::xes(args.get("in0").unwrap().bits)
        }
    )]))
}

pub fn sparse_mux_op(_args: HashMap<String, Vec3vl>) -> Result<ClockHack, String> {
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

