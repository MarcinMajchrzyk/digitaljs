use std::collections::HashMap;
use std::vec;

use crate::gate::PolarityOptions;
use crate::js_types::JsGateParams;
use crate::operations::ReturnValue;
use crate::vector3vl::Vec3vl;

pub struct FsmState {
  bits_out: u32,
  polarity: PolarityOptions,
  transitions: HashMap<u32, Vec<FsmTransition>>,
  init_state: u32,
  current_state: u32,
  last_clk: i32
}

#[derive(Clone)]
pub struct FsmTransition {
  pub id:       String,
  pub ctrl_in:  Vec3vl,
  pub ctrl_out: Vec3vl,
  pub state_out: u32
}

pub struct FsmTempStruct {
  pub ctrl_in: Vec3vl,
  pub ctrl_out: Vec3vl,
  pub state_in: u32,
  pub state_out: u32
}

pub fn fsm(args: &HashMap<String, Vec3vl>, state: &mut FsmState) -> Result<ReturnValue, String> {
  let next_trans = |current_state: u32, data_in: &Vec3vl, transitions: &HashMap<u32, Vec<FsmTransition>>| -> Result<Option<FsmTransition>, String> {
    let links = match transitions.get(&current_state) {
      Some(l) => l,
      None => &vec![]
    };
    
    for trans in links {
      let ctrl_in = &trans.ctrl_in;
      let xmask = ctrl_in.xmask();
      if data_in.or(&xmask)? == ctrl_in.or(&xmask)? {
        return Ok(Some(trans.clone()))
      }
    }
    Ok(None)
  };

  let next_output = |current_state: u32, data_in: Vec3vl, bits_out: u32, transitions: &HashMap<u32, Vec<FsmTransition>>| -> Result<Vec3vl, String> {
    let links = match transitions.get(&current_state) {
      Some(l) => l,
      None => &vec![]
    };

    let ixmask = data_in.xmask();
    let mut results = vec![];

    for trans in links {
      let ctrl_in = &trans.ctrl_in;
      let xmask = ctrl_in.xmask().or(&ixmask)?;
      
      if data_in.or(&xmask)? == ctrl_in.or(&xmask)? {
        results.push(trans.ctrl_out.clone());
      }
    }

    let xes = Vec3vl::xes(bits_out);
    if results.is_empty() { 
      Ok(xes)
    } else {
      let iter = results.iter();
      let acc = results.first().unwrap().clone();

      Ok(iter.skip(1).try_fold(acc, |other, results: &Vec3vl| -> Result<Vec3vl, String> {
        let eqs = results.xnor(&other)?.or(&xes)?;
        results.and(&eqs)?.or(&xes.and(&eqs.xmask())?)
      })?)
    }
  };

  let pol = |what: bool| -> i32 {
    if what { 1 } else { -1 }
  };

  let arst = match args.get("arst") {
    Some(arst) => arst,
    None => return Err("FSM cell has no async reset signal".to_string())
  };

  let clk = match args.get("clk") {
    Some(clk) => clk,
    None => return Err("FSM cell has no clock signal".to_string())
  };

  let clk_pol = match state.polarity.clock {
    Some(c) => c,
    None => return Err("FSM cell has no clock polarity".to_string())
  };

  let arst_pol = match state.polarity.arst {
    Some(a) => a,
    None => return Err("FSM cell has no async reset polarity".to_string())
  };
  
  let data_in = match args.get("in") {
    Some(v) => v.clone(),
    None => return Err("FSM cell has no input signal".to_string())
  };

  if arst.lsb() == pol(arst_pol) {
    state.current_state = state.init_state;
  } else {
    let last_clk = state.last_clk;
    if clk.lsb() == pol(clk_pol) && last_clk == -pol(clk_pol) {
      state.current_state = if let Some(trans) = next_trans(state.current_state, &data_in, &state.transitions)? {
        trans.state_out
      } else {
        state.init_state
      };
    }
  }

  state.last_clk = clk.lsb();

  ReturnValue::out(next_output(state.current_state, data_in, state.bits_out, &state.transitions)?)
}

impl FsmState {
  pub fn new(params: JsGateParams) -> FsmState {
    let (bits_in, bits_out) = match params.get_bits_struct() {
      Some(b) => (b.get_bits_in(), b.get_bits_out()),
      None => (1, 1)
    };

    let mut transitions: HashMap<u32, Vec<FsmTransition>> = HashMap::new();
    let vec = params.get_trans_table().map(|v| {
      v.iter().map(|o| {
        FsmTempStruct {
          ctrl_in: Vec3vl::from_binary(o.get_ctrl_in(), Some(bits_in as usize)),
          ctrl_out: Vec3vl::from_binary(o.get_ctrl_out(), Some(bits_out as usize)),
          state_in: o.get_state_in(),
          state_out: o.get_state_out(),
        }
      }).collect::<Vec<FsmTempStruct>>()
    }).unwrap_or_default();

    for (id, t) in vec.iter().enumerate() {
      let transition = FsmTransition {
        id: format!("tr{id}"),
        ctrl_in: t.ctrl_in.clone(),
        ctrl_out: t.ctrl_out.clone(),
        state_out: t.state_out,
      };
      
      transitions.entry(t.state_in).or_default().push(transition);
    }

    let init_state = params.get_init_state().unwrap_or(0);
    FsmState { 
      bits_out, 
      polarity: PolarityOptions::new(params.get_polarity()),
      transitions,
      init_state,
      current_state: init_state,
      last_clk: 0,
    }
  }
}