use std::collections::HashMap;

use crate::gate::SliceOptions;
use crate::operations::ReturnValue;
use crate::vector3vl::Vec3vl;

pub type ExtendFn = fn(&Vec3vl) -> i32;

pub fn bit_extend(args: &HashMap<String, Vec3vl>, op: &ExtendFn, bits: u32) -> Result<ReturnValue, String> {
    let sig = match args.get("in") {
        Some(s) => s,
        None => return Err("Bit extend gate has no input".to_string())
    };

    ReturnValue::out(Vec3vl::concat(&mut vec![sig.clone(), Vec3vl::make_int(bits, op(sig))?])?)
}

pub fn zero_extend(_sig: &Vec3vl) -> i32 {
    -1
}

pub fn sign_extend(sig: &Vec3vl) -> i32 {
    sig.msb()
}

pub fn bus_slice(args: &HashMap<String, Vec3vl>, options: &SliceOptions) -> Result<ReturnValue, String> {
    let f = options.first;
    let c = options.count;

    let input = match args.get("in") {
        Some(i) => i,
        None => return Err("No input".to_string())
    };

    let val = if input.is_fully_defined() {
        input.slice(f, f + c)?
    } else {
        Vec3vl::xes(c)
    };
    ReturnValue::out(val)
}

pub fn bus_group(args: &HashMap<String, Vec3vl>) -> Result<ReturnValue, String> {
    let mut vec = vec![];
    for i in 0..args.len() {
        let val = match args.get(&format!("in{}", i)) {
            Some(v) => v.clone(),
            None => return Err(format!("No input named in{}", i))
        };
        vec.push(val);
    }
    ReturnValue::out(Vec3vl::concat(&mut vec)?)
}
