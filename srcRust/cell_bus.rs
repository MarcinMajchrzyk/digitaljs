use std::collections::HashMap;

use crate::gate::SliceOptions;
use crate::operations::ClockHack;
use crate::vector3vl::Vec3vl;

// bit extend
// zero extend
// sign extend

// bus regroup

pub fn bus_slice(args: &HashMap<String, Vec3vl>, options: &Option<SliceOptions>) -> Result<ClockHack, String> {
    let (f, c) = match options {
        Some(o) => (o.first, o.count),
        None => return Err("No slice options provided".to_string())
    };

    let input = match args.get("in") {
        Some(i) => i,
        None => return Err("No input".to_string())
    };

    Ok(ClockHack::Normal(vec![("out".to_string(), 
        if input.is_fully_defined() {
            input.slice(f, f + c)?
        } else {
            Vec3vl::xes(c)
        }
    )]))    
}

pub fn bus_group(args: &HashMap<String, Vec3vl>) -> Result<ClockHack, String> {
    let mut vec = vec![];
    for i in 0..args.len() {
        let val = match args.get(&format!("in{}", i)) {
            Some(v) => v.clone(),
            None => return Err(format!("No input named in{}", i))
        };
        vec.push(val);
    }
    
    Ok(ClockHack::Normal(vec![("out".to_string(), Vec3vl::concat(&mut vec)?)]))
}
