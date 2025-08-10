use std::collections::HashMap;

use crate::gate::GateParams;
use crate::js_types::MemoryPolarityStruct;
use crate::operations::ClockHack;
use crate::triggerMemoryUpdate;
use crate::vector3vl::Vec3vl;

pub struct MemoryState {
  pub gate_id: String,
  pub graph_id: String,
  pub abits: u32,
  pub bits: u32,
  pub memory: Vec<Vec3vl>,
  pub offset: u32,
  pub words: u32,
  pub rdports: Vec<(String, MemoryPortPolarity)>,
  pub wrports: Vec<(String, MemoryPortPolarity)>,
  pub outputs: HashMap<String, Vec3vl>,
  pub last_clk: HashMap<String, i32>
}

pub fn memory_op(args: HashMap<String, Vec3vl>, state: &mut MemoryState) -> Result<ClockHack, String> {
  let pol = |p: bool| -> i32 { if p { 1 } else { -1 }};
  
  let is_enabled = |portname: &String, port: &MemoryPortPolarity| -> Result<bool, String> {
    if let Some(p) = port.enable {
      return match args.get(&format!("{portname}en")).cloned() {
        Some(mut v) => Ok(v.to_array().iter().all(|x| *x == pol(p))),
        None => Err(format!("No port named {portname}en"))
      }
    }
    Ok(true)
  };
  
  let port_active = |portname: &String, port: &MemoryPortPolarity, last_clk: i32| -> Result<bool, String> {
    if let Some(p) = port.clock {
      let clkname = format!("{portname}clk");
      return match args.get(&clkname) {
        Some(v) => {
          Ok(v.lsb() == pol(p) && last_clk == -pol(p)) 
        }
        None => Err(format!("No port named {clkname}"))
      }
    }
    Ok(true)
  };

  let valid_addr = |n: i32| -> bool {
    n >= 0 && n < state.words as i32
  };

  let do_comb_read = |portname: &String, memory: &mut Vec<Vec3vl>, outputs: &mut HashMap<String, Vec3vl>| -> Result<(), String> {
    let port = format!("{portname}addr");
    let sig = match args.get(&port) {
      Some(v) => v,
      None => return Err(format!("No port {port}"))
    };
    let addr = calc_addr(&mut sig.clone(), state.offset)?;

    outputs.insert(format!("{portname}data"), if valid_addr(addr) {
        memory[addr as usize].clone()
      } else {
        Vec3vl::xes(state.bits)
      }
    );

    Ok(())
  };

  let write_value = |portname: &String, port: &MemoryPortPolarity, oldval: Vec3vl, val: Vec3vl| -> Result<Vec3vl, String> {
    match port.enable {
      Some(p) => {
        let mut mask = match args.get(&format!("{portname}en")) {
          Some(v) => v.clone(),
          None => return Err("".to_string())
        };
        if !p { mask = mask.not(); }

        Ok(val.and(mask.clone())?.or(oldval.and(mask.not())?)?)
      },
      None => Ok(val)
    }
  };

  let mut do_read = |portname: &String, port: &MemoryPortPolarity, outputs: &mut HashMap<String, Vec3vl>| -> Result<(), String> {
    do_comb_read(portname, &mut state.memory, outputs)?;

    for (wrportname, wrport) in state.wrports.iter() {
      if let Some(t) = port.transparent {
        if t && port_active(wrportname, wrport, *state.last_clk.get(wrportname).unwrap())? && 
            is_enabled(wrportname, wrport)? && args.get(&format!("{portname}addr")).unwrap() == args.get(&format!("{wrportname}addr")).unwrap() {
          //state.outputs.insert(format!(""), write_value(wrportname, wrport, )?);
        }
      }

      if let Some(c) = port.collision {
        if c && port_active(wrportname, wrport, *state.last_clk.get(wrportname).unwrap())? && 
            is_enabled(wrportname, wrport)? && args.get(&format!("{portname}addr")).unwrap() == args.get(&format!("{wrportname}addr")).unwrap() {
          outputs.insert(format!("{portname}data"), write_value(wrportname, wrport, outputs.get(&format!("{portname}data")).unwrap().clone(), Vec3vl::xes(state.bits))?);
        }
      }
    }

    Ok(())
  };

  let do_write = |portname: &String, port: &MemoryPortPolarity, memory: &mut Vec<Vec3vl>| -> Result<(), String> {
    let addrport = format!("{portname}addr");
    let sig = match args.get(&addrport) {
      Some(v) => v,
      None => return Err(format!("No port {addrport}"))
    };
    let addr = calc_addr(&mut sig.clone(), state.offset)?;

    if valid_addr(addr) {
      let oldval = memory[addr as usize].clone();
      
      let val = match args.get(&format!("{portname}data")) {
        Some(v) => v.clone(),
        None => return Err("".to_string())
      };
      
      let newval = write_value(portname, port, oldval.clone(), val.clone())?;
      let changed = oldval != newval; 
      memory[addr as usize] = newval;

      if changed {
        triggerMemoryUpdate(
          state.graph_id.clone(),
          state.gate_id.clone(),
          addr,
          val.bits,
          val.avec.clone(),
          val.bvec.clone()
        );
      }
    }

    Ok(())
  };

  // do_srst
  // do_arst

  let update_last_clk = |portname: &String, port: &mut MemoryPortPolarity, last_clk: &mut HashMap<String, i32>| {
    if port.clock.is_some() {
      let clkname = format!("{portname}clk");
      last_clk.insert(clkname.clone(), args.get(&clkname).unwrap().lsb());
    }
  };

  for (portname, port) in state.rdports.iter() {
    if port.clock.is_some() && is_enabled(portname, port)? && port_active(portname, port, *state.last_clk.get(&format!("{portname}clk")).unwrap())? {
      do_read(portname, port, &mut state.outputs)?;
    }
  }

  for (portname, port) in state.wrports.iter() {
    if is_enabled(portname, port)? && port_active(portname, port, *state.last_clk.get(&format!("{portname}clk")).unwrap())? {
      do_write(portname, port, &mut state.memory)?;
    }
  }

  for (portname, port) in state.rdports.iter() {
    if port.clock.is_none() && is_enabled(portname, port)? {
      do_comb_read(portname, &mut state.memory, &mut state.outputs)?;
    }
  }

  // do_srst loop
  // do_arst loop

  for (portname, port) in state.rdports.iter_mut() {
    update_last_clk(portname, port, &mut state.last_clk);
  }

  for (portname, port) in state.wrports.iter_mut() {
    update_last_clk(portname, port, &mut state.last_clk);
  }

  Ok(ClockHack::Normal(
    state.outputs.iter().map(|(name, val)| -> (String, Vec3vl) {
      (name.clone(), val.clone())
    }).collect()
  ))
}

fn calc_addr(sig: &mut Vec3vl, offset: u32) -> Result<i32, String> {
  if !sig.is_fully_defined() {
    Ok(-1)
  } else {
    Ok(sig.get_number()? as i32 - offset as i32)
  }
}

impl MemoryState {
  pub fn new(params: &GateParams) -> Result<MemoryState, String> {
    let abits = match params.abits {
      Some(a) => a,
      None => return Err("No abits parameter".to_string())
    };
    
    let memory = match &params.memdata {
      Some(w) => w.clone(),
      None => return Err("No memory parameter".to_string())
    };

    let offset = match params.offset {
      Some(o) => o,
      None => return Err("No offset parameter".to_string())
    };

    let words = match params.words {
      Some(w) => w,
      None => return Err("No words parameter".to_string())
    };

    let rdports: Vec<(String, MemoryPortPolarity)> = params.rdports
      .iter()
      .enumerate()
      .map(|(n, v) | -> (String, MemoryPortPolarity) {
        (format!("rd{n}"), v.clone())
      }).collect();

    let wrports: Vec<(String, MemoryPortPolarity)> = params.wrports
      .iter()
      .enumerate()
      .map(|(n, v)| -> (String, MemoryPortPolarity) {
        (format!("wr{n}"), v.clone())
      }).collect();

    let mut last_clk = HashMap::new();

    for (portname, port) in rdports.iter() {
      if port.clock.is_some() {
        last_clk.insert(format!("{portname}clk"), 0);
      }
    }

    for (portname, port) in wrports.iter() {
      if port.clock.is_some() {
        last_clk.insert(format!("{portname}clk"), 0);
      }
    }

    Ok(MemoryState {
      gate_id: params.gate_id.clone(),
      graph_id: params.graph_id.clone(),
      abits, 
      bits: params.bits_in, 
      memory, 
      offset, 
      words, 
      rdports, 
      wrports, 
      outputs: HashMap::new(),
      last_clk
    })
  }
}

#[derive(Clone)]
pub struct MemoryPortPolarity {
  enable:       Option<bool>,
  clock:        Option<bool>,
  transparent:  Option<bool>,
  collision:    Option<bool>,
  srst:         Option<bool>,
  srst_enable:  Option<bool>,
  srst_value:   Option<String>,
  arst:         Option<bool>,
  arst_value:   Option<String>
}

impl MemoryPortPolarity {
  pub fn new(options: &MemoryPolarityStruct) -> MemoryPortPolarity {
    MemoryPortPolarity { 
      enable:       options.get_enable(), 
      clock:        options.get_clock(),
      transparent:  options.get_transparent(),
      collision:    options.get_collision(),
      srst:         options.get_srst(),
      srst_enable:  options.get_srst_enable(),
      srst_value:   options.get_srst_value(),
      arst:         options.get_arst(),
      arst_value:   options.get_arst_value()
    }
  }
}