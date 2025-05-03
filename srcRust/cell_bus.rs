

use std::collections::HashMap;

use crate::operations::ClockHack;
use crate::vector3vl::Vec3vl;

// bit extend
// zero extend
// sign extend

// bus regroup

pub fn bus_slice(args: &HashMap<String, Vec3vl>, f: u32, c: u32) -> ClockHack {
    let r = args.get("in").unwrap().slice(f, f + c);
    ClockHack::Normal(vec![("out".to_string(), r)])
}

pub fn bus_group(args: &HashMap<String, Vec3vl>) -> ClockHack {
    let mut vec = vec![];
    for i in 0..args.len() {
        vec.push(args.get(&format!("in{}", i)).unwrap().clone());
    }
    ClockHack::Normal(vec![("out".to_string(), Vec3vl::concat(&mut vec))])
}
