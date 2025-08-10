use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  pub type JsGateParams;

  #[wasm_bindgen(method, structural, getter = arst_value)]
  pub fn get_arst_value(this: &JsGateParams) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = bits)]
  pub fn get_bits(this: &JsGateParams) -> u32;

  #[wasm_bindgen(method, structural, getter = bits)]
  pub fn get_bits_struct(this: &JsGateParams) -> JsStructBits;

  #[wasm_bindgen(method, structural, getter = inputs)]
  pub fn get_inputs(this: &JsGateParams) -> Option<Vec<JsBigInt>>;

  #[wasm_bindgen(method, structural, getter = label)]
  pub fn get_label(this: &JsGateParams) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = net)]
  pub fn get_net(this: &JsGateParams) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = numbase)]
  pub fn get_numbase(this: &JsGateParams) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = propagation)]
  pub fn get_propagation(this: &JsGateParams) -> u32;

  #[wasm_bindgen(method, structural, getter = type)]
  pub fn get_type(this: &JsGateParams) -> String;

  #[wasm_bindgen(method, structural, getter = slice)]
  pub fn get_slice(this: &JsGateParams) -> Option<SliceType>;

  #[wasm_bindgen(method, structural, getter = polarity)]
  pub fn get_polarity(this: &JsGateParams) -> Option<DffPolarityStruct>;

  #[wasm_bindgen(method, structural, getter = leftOp)]
  pub fn get_left_op(this: &JsGateParams) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = constant)]
  pub fn get_constant_num(this: &JsGateParams) -> Option<u32>;

  #[wasm_bindgen(method, structural, getter = constant)]
  pub fn get_constant_str(this: &JsGateParams) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = abits)]
  pub fn get_abits(this: &JsGateParams) -> Option<u32>;

  #[wasm_bindgen(method, structural, getter = offset)]
  pub fn get_offset(this: &JsGateParams) -> Option<u32>;

  #[wasm_bindgen(method, structural, getter = words)]
  pub fn get_words(this: &JsGateParams) -> Option<u32>;

  #[wasm_bindgen(method, structural, getter = memdata)]
  pub fn get_memdata(this: &JsGateParams) -> Option<Vec<JsValue>>;

  #[wasm_bindgen(method, structural, getter = rdports)]
  pub fn get_rdports(this: &JsGateParams) -> Option<Vec<MemoryPolarityStruct>>;

  #[wasm_bindgen(method, structural, getter = wrports)]
  pub fn get_wrports(this: &JsGateParams) -> Option<Vec<MemoryPolarityStruct>>;

  #[wasm_bindgen(method, structural, getter = init_state)]
  pub fn get_init_state(this: &JsGateParams) -> Option<u32>;

  #[wasm_bindgen(method, structural, getter = states)]
  pub fn get_states(this: &JsGateParams) -> Option<u32>;

  #[wasm_bindgen(method, structural, getter = trans_table)]
  pub fn get_trans_table(this: &JsGateParams) -> Option<Vec<JsFsmState>>;
}

#[wasm_bindgen]
extern "C" {
  pub type PortParams;

  #[wasm_bindgen(method, structural, getter = id)]
  pub fn get_id(this: &PortParams) -> String;

  #[wasm_bindgen(method, structural, getter = dir)]
  pub fn get_dir(this: &PortParams) -> String;

  #[wasm_bindgen(method, structural, getter = bits)]
  pub fn get_bits(this: &PortParams) -> u32;

  #[wasm_bindgen(method, structural, getter = avec)]
  pub fn get_avec(this: &PortParams) -> Vec<u32>;

  #[wasm_bindgen(method, structural, getter = bvec)]
  pub fn get_bvec(this: &PortParams) -> Vec<u32>;
}

#[wasm_bindgen]
extern "C" {
  pub type TargetParams;

  #[wasm_bindgen(method, structural, getter = id)]
  pub fn get_id(this: &TargetParams) -> String;

  #[wasm_bindgen(method, structural, getter = port)]
  pub fn get_port(this: &TargetParams) -> String;

  #[wasm_bindgen(method, structural, getter = magnet)]
  pub fn get_magnet(this: &TargetParams) -> String;
}

#[wasm_bindgen]
extern "C" {
  pub type IOmap;

  #[wasm_bindgen(method, structural, getter = port)]
  pub fn get_port(this: &IOmap) -> String;

  #[wasm_bindgen(method, structural, getter = io_id)]
  pub fn get_io_id(this: &IOmap) -> String;
}

#[wasm_bindgen]
extern "C" {
  pub type SliceType;

  #[wasm_bindgen(method, structural, getter = count)]
  pub fn get_count(this: &SliceType) -> u32;

  #[wasm_bindgen(method, structural, getter = first)]
  pub fn get_first(this: &SliceType) -> u32;

  #[wasm_bindgen(method, structural, getter = total)]
  pub fn get_total(this: &SliceType) -> u32;
}

#[wasm_bindgen]
extern "C" {
  pub type DffPolarityStruct;

  #[wasm_bindgen(method, structural, getter = aload)]
  pub fn get_aload(this: &DffPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = arst)]
  pub fn get_arst(this: &DffPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = clock)]
  pub fn get_clock(this: &DffPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = clr)]
  pub fn get_clr(this: &DffPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = enable)]
  pub fn get_enable(this: &DffPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = set)]
  pub fn get_set(this: &DffPolarityStruct) -> Option<bool>;
}

#[wasm_bindgen]
extern "C" {
  pub type MemoryPolarityStruct;

  #[wasm_bindgen(method, structural, getter = enable_polarity)]
  pub fn get_enable(this: &MemoryPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = clock_polarity)]
  pub fn get_clock(this: &MemoryPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = transparent)]
  pub fn get_transparent(this: &MemoryPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = collision)]
  pub fn get_collision(this: &MemoryPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = srst_polarity)]
  pub fn get_srst(this: &MemoryPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = enable_srst)]
  pub fn get_srst_enable(this: &MemoryPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = srst_value)]
  pub fn get_srst_value(this: &MemoryPolarityStruct) -> Option<String>;

  #[wasm_bindgen(method, structural, getter = arst_polarity)]
  pub fn get_arst(this: &MemoryPolarityStruct) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = arst_value)]
  pub fn get_arst_value(this: &MemoryPolarityStruct) -> Option<String>;
}

#[wasm_bindgen]
extern "C" {
  pub type JsBigInt;

  #[wasm_bindgen(method, structural)]
  pub fn toString(this: &JsBigInt, arg: u32) -> String;
}

#[wasm_bindgen]
extern "C" {
  pub type JsStructBits;

  #[wasm_bindgen(method, structural, getter = in)]
  pub fn get_bits_in(this: &JsStructBits) -> u32;

  #[wasm_bindgen(method, structural, getter = sel)]
  pub fn get_bits_sel(this: &JsStructBits) -> u32;

  #[wasm_bindgen(method, structural, getter = out)]
  pub fn get_bits_out(this: &JsStructBits) -> u32;
}

#[wasm_bindgen]
extern "C" {
  pub type JsMonitorParams;

  #[wasm_bindgen(method, structural, getter = triggerValues)]
  pub fn get_trigger_values(this: &JsMonitorParams) -> Option<Vec<JsVec3vl>>;

  #[wasm_bindgen(method, structural, getter = stopOnTrigger)]
  pub fn get_stop_on_trigger(this: &JsMonitorParams) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = oneShot)]
  pub fn get_one_shot(this: &JsMonitorParams) -> Option<bool>;

  #[wasm_bindgen(method, structural, getter = synchronous)]
  pub fn get_synchronous(this: &JsMonitorParams) -> Option<bool>;
}

#[wasm_bindgen]
extern "C" {
  pub type JsVec3vl;

  #[wasm_bindgen(method, structural, getter = _bits)]
  pub fn get_bits(this: &JsVec3vl) -> u32;

  #[wasm_bindgen(method, structural, getter = _avec)]
  pub fn get_avec(this: &JsVec3vl) -> Vec<u32>;

  #[wasm_bindgen(method, structural, getter = _bvec)]
  pub fn get_bvec(this: &JsVec3vl) -> Vec<u32>;
}

#[wasm_bindgen]
extern "C" {
  pub type JsAlarmStruct;

  #[wasm_bindgen(method, structural, getter = stopOnAlarm)]
  pub fn get_stop_on_alarm(this: &JsAlarmStruct) -> bool;

  #[wasm_bindgen(method, structural, getter = synchronous)]
  pub fn get_synchronous(this: &JsAlarmStruct) -> bool;
}

#[wasm_bindgen]
extern "C" {
  pub type JsFsmState;

  #[wasm_bindgen(method, structural, getter = ctrl_in)]
  pub fn get_ctrl_in(this: &JsFsmState) -> String;

  #[wasm_bindgen(method, structural, getter = ctrl_out)]
  pub fn get_ctrl_out(this: &JsFsmState) -> String;

  #[wasm_bindgen(method, structural, getter = state_in)]
  pub fn get_state_in(this: &JsFsmState) -> u32;

  #[wasm_bindgen(method, structural, getter = state_out)]
  pub fn get_state_out(this: &JsFsmState) -> u32;
}