use std::collections::hash_map::Iter;
use std::collections::HashMap;

use crate::cell_arith::{arith_binop, arith_comp, arith_const_binop, arith_const_comp, arith_monop, arith_shift, arith_shift_const, Add, ArithBinop, ArithBinopStruct, ArithComp, ArithCompStruct, ArithConstBinop, ArithConstComp, ArithMonop, ArithMonopStruct, ArithShiftStruct, Div, Eq, Ge, Gt, Le, Lt, Mod, Mul, Ne, Negation, Pow, Shl, Shr, Sub, UnaryPlus};
use crate::cell_bus::{bit_extend, bus_group, bus_slice, bus_ungroup, sign_extend, zero_extend, ExtendFn};
use crate::cell_dff::{dff, DffState};
use crate::cell_fsm::{fsm, FsmState};
use crate::cell_io::{clock, constant};
use crate::cell_memory::{memory_op, MemoryState};
use crate::cell_mux::{mux1hot_idx, mux_idx, mux_op, sparse_mux_op, MuxIdx};
use crate::gate::SliceOptions;
use crate::js_types::JsGateParams;
use crate::vector3vl::Vec3vl;

use crate::cell_gates::{gate_11, gate_reduce, gate_x1, Binop, Monop, ReduceFn};

pub enum Operation {
    Arith11(ArithMonop, u32),
    Arith21(ArithBinop, u32, (bool, bool)),
    ArithConst(ArithConstBinop, i32, bool, u32, bool),
    Comp(ArithComp, (bool, bool)),
    CompConst(ArithConstComp, i32, bool, bool),
    BitExtend(ExtendFn, u32),
    BusGroup,
    BusUngroup(Vec<u32>),
    BusSlice(SliceOptions),
    Clock(bool),
    Constant(Vec3vl),
    Dff(DffState),
    Fsm(FsmState),
    Gate11(Monop),
    GateX1(Binop),
    GateReduce(ReduceFn),
    Mux(u32, MuxIdx),
    MuxSparse(u32, HashMap<String, String>),
    Memory(MemoryState),
    None
}

impl Operation {
    pub fn from_name(name: String, gate_params: JsGateParams, graph_id: String, gate_id: String) -> Result<Operation, String> {
        Ok(match name.as_str() {
            "Repeater"  => Operation::Gate11(|v| v.clone()),
            "Not"       => Operation::Gate11(Vec3vl::not),

            "And"       => Operation::GateX1(Vec3vl::and),
            "Or"        => Operation::GateX1(Vec3vl::or),
            "Xor"       => Operation::GateX1(Vec3vl::xor),
            "Nand"      => Operation::GateX1(Vec3vl::nand),
            "Nor"       => Operation::GateX1(Vec3vl::nor),
            "Xnor"      => Operation::GateX1(Vec3vl::xnor),

            "AndReduce"     => Operation::GateReduce(Vec3vl::reduce_and),
            "OrReduce"      => Operation::GateReduce(Vec3vl::reduce_or),
            "XorReduce"     => Operation::GateReduce(Vec3vl::reduce_xor),
            "NandReduce"    => Operation::GateReduce(Vec3vl::reduce_nand),
            "NorReduce"     => Operation::GateReduce(Vec3vl::reduce_nor),
            "XnorReduce"    => Operation::GateReduce(Vec3vl::reduce_xnor),

            "ZeroExtend"    => create_bit_extend(zero_extend, gate_params),
            "SignExtend"    => create_bit_extend(sign_extend, gate_params),

            "BusSlice"      => Operation::BusSlice(SliceOptions::new(gate_params)),
            "BusGroup"      => Operation::BusGroup,
            "BusUngroup"    => Operation::BusUngroup(gate_params.get_groups().unwrap_or(vec![1])),

            "Constant"  => create_constant(gate_params),
            "Clock"     => Operation::Clock(false),
            
            "Dff"       => Operation::Dff(DffState::new(gate_params)),
            "FSM"       => Operation::Fsm(FsmState::new(gate_params, graph_id, gate_id)),

            "Lt"        => create_arith_comp::<Lt>(gate_params),
            "Le"        => create_arith_comp::<Le>(gate_params),
            "Gt"        => create_arith_comp::<Gt>(gate_params),
            "Ge"        => create_arith_comp::<Ge>(gate_params),
            "Eq"        => create_arith_comp::<Eq>(gate_params),
            "Ne"        => create_arith_comp::<Ne>(gate_params),

            "LtConst"   => create_arith_comp_const::<Lt>(gate_params),
            "LeConst"   => create_arith_comp_const::<Le>(gate_params),
            "GtConst"   => create_arith_comp_const::<Gt>(gate_params),
            "GeConst"   => create_arith_comp_const::<Ge>(gate_params),
            "EqConst"   => create_arith_comp_const::<Eq>(gate_params),
            "NeConst"   => create_arith_comp_const::<Ne>(gate_params),

            "Negation"  => create_arith_monop::<Negation>(gate_params),
            "UnaryPlus" => create_arith_monop::<UnaryPlus>(gate_params), 

            "Addition"       => create_arith_binop::<Add>(gate_params),
            "Subtraction"    => create_arith_binop::<Sub>(gate_params),
            "Multiplication" => create_arith_binop::<Mul>(gate_params),
            "Division"       => create_arith_binop::<Div>(gate_params),
            "Modulo"         => create_arith_binop::<Mod>(gate_params),
            "Power"          => create_arith_binop::<Pow>(gate_params),
            "ShiftLeft"      => create_arith_shift::<Shl, Shr>(gate_params),
            "ShiftRight"     => create_arith_shift::<Shr, Shl>(gate_params),

            "AdditionConst"       => create_arith_binop_const::<Add>(gate_params),
            "SubtractionConst"    => create_arith_binop_const::<Sub>(gate_params),
            "MultiplicationConst" => create_arith_binop_const::<Mul>(gate_params),
            "DivisionConst"       => create_arith_binop_const::<Div>(gate_params),
            "ModuloConst"         => create_arith_binop_const::<Mod>(gate_params),
            "PowerConst"          => create_arith_binop_const::<Pow>(gate_params),
            "ShiftLeftConst"      => create_arith_shift_const::<Shl, Shr>(gate_params),
            "ShiftRightConst"     => create_arith_shift_const::<Shr, Shl>(gate_params),

            "Mux"       => create_multiplexer(gate_params, mux_idx),
            "Mux1Hot"   => create_multiplexer(gate_params, mux1hot_idx),
            "MuxSparse" => create_mux_sparse(gate_params),

            "Memory"    => Operation::Memory(MemoryState::new(gate_params, graph_id, gate_id)),
            "NumEntry" | 
            "NumDisplay" | 
            "Button" |
            "Lamp" |
            "Input" |
            "Output" |
            "Subcircuit" => Operation::None,
            _ => return Err(format!("Unknown cell type provided: {name}"))
        })
    }

    pub fn op(&mut self, args: &HashMap<String, Vec3vl>) -> Result<ReturnValue, String> {
        match self {
            Operation::Arith11(op, bits_out) => op(args, *bits_out),
            Operation::Arith21(op, bits, sgn ) => op(args, *bits, *sgn),
            Operation::ArithConst(op, constant, left_op, bits, sgn) => op(args, constant, left_op, *bits, *sgn),
            Operation::Comp(op, sgn) => op(args, *sgn),
            Operation::CompConst(op, constant, left_op, sgn) => op(args, constant, left_op, *sgn),
            Operation::Gate11(op) => gate_11(args, op),
            Operation::GateX1(op) => gate_x1(args, op),
            Operation::GateReduce(op) => gate_reduce(args, op),
            Operation::BitExtend(op, bits) => bit_extend(args, op, *bits),
            Operation::BusSlice(options) => bus_slice(args, options),
            Operation::BusGroup => bus_group(args),
            Operation::BusUngroup(groups) => bus_ungroup(args, groups),
            Operation::Constant(value) => constant(value.clone()),
            Operation::Clock(clock_val) => clock(clock_val),
            Operation::Dff(state) => dff(args, state),
            Operation::Fsm(state) => fsm(args, state),
            Operation::Mux(bits, op) => mux_op(args, *bits, op),
            Operation::MuxSparse(bits, map) => sparse_mux_op(args, *bits, map),
            Operation::Memory(state) => memory_op(args, state),
            Operation::None => ReturnValue::values(None, HashMap::new())
        }
    }

    pub fn get_type(&self) -> String {
        match self {
            Operation::Arith11(_, _)             => "Arith11",
            Operation::Arith21(_, _, _)          => "Arith21",
            Operation::ArithConst(_, _, _, _, _) => "ArithConst",
            Operation::Comp(_, _)                => "Comp",
            Operation::CompConst(_, _, _, _)     => "CompConst",
            Operation::BitExtend(_, _)           => "BitExtend",
            Operation::BusGroup                  => "BusGroup",
            Operation::BusUngroup(_)             => "BusUngroup",
            Operation::BusSlice(_)               => "BusSlice",
            Operation::Clock(_)                  => "Clock",
            Operation::Constant(_)               => "Constant",
            Operation::Dff(_)                    => "DFF",
            Operation::Fsm(_)                    => "FSM",
            Operation::Gate11(_)                 => "Gate11",
            Operation::GateX1(_)                 => "GateX1",
            Operation::GateReduce(_)             => "GateReduce",
            Operation::Mux(_, _)                 => "Mux",
            Operation::MuxSparse(_, _)           => "MuxSparse",
            Operation::Memory(_)                 => "Memory",
            Operation::None                      => "None",
        }.to_string()
    }
}

fn create_bit_extend(func: ExtendFn, gate_params: JsGateParams) -> Operation {
    let (input, output) = match gate_params.get_extend() {
        Some(e) => (e.get_input().unwrap_or(1), e.get_output().unwrap_or(1)),
        None => (1, 1)
    };
    Operation::BitExtend(func, output - input)
}

fn create_constant(gate_params: JsGateParams) -> Operation {
    let value = gate_params.get_constant_str().unwrap_or("0".to_string());
    Operation::Constant(Vec3vl::from_binary(value, None))
}

fn create_arith_monop<T: ArithMonopStruct>(gate_params: JsGateParams) -> Operation {
    let bits_out = match gate_params.get_bits_struct() {
        Some(s) => s.get_bits_out(),
        None => 1
    };
    Operation::Arith11(
        arith_monop::<T>, 
        bits_out
    )
}

fn create_arith_binop<T: ArithBinopStruct>(gate_params: JsGateParams) -> Operation {
    let bits_out = match gate_params.get_bits_struct() {
        Some(s) => s.get_bits_out(),
        None => 1
    };
    let sgn = match gate_params.get_sign() {
        Some(s) => (s.get_in1().unwrap_or(false), s.get_in2().unwrap_or(false)),
        None => (false, false)
    };

    Operation::Arith21(
        arith_binop::<T>, 
        bits_out,
        sgn
    )
}

fn create_arith_shift<L: ArithShiftStruct, R: ArithShiftStruct>(gate_params: JsGateParams) -> Operation {
    let bits_out = match gate_params.get_bits_struct() {
        Some(s) => s.get_bits_out(),
        None => 1
    };
    let sgn = match gate_params.get_sign() {
        Some(s) => (s.get_in1().unwrap_or(false), s.get_in2().unwrap_or(false)),
        None => (false, false)
    };

    Operation::Arith21(
        arith_shift::<L, R>, 
        bits_out,
        sgn
    )
}

fn create_arith_comp<T: ArithCompStruct>(gate_params: JsGateParams) -> Operation {
    let sgn = match gate_params.get_sign() {
        Some(s) => (s.get_in1().unwrap_or(false), s.get_in2().unwrap_or(false)),
        None => (false, false)
    };

    Operation::Comp(
        arith_comp::<T>,
        sgn
    )
}

fn create_arith_binop_const<T: ArithBinopStruct>(gate_params: JsGateParams) -> Operation {
    let bits_out = match gate_params.get_bits_struct() {
        Some(s) => s.get_bits_out(),
        None => 1
    };
    let sgn = match gate_params.get_sign() {
        Some(s) => s.get_in().unwrap_or(false),
        None => false
    };

    Operation::ArithConst(
        arith_const_binop::<T>,
        gate_params.get_constant_num().unwrap_or(0), 
        gate_params.get_left_op().unwrap_or(false),
        bits_out,
        sgn
    )
}

fn create_arith_shift_const<L: ArithShiftStruct, R: ArithShiftStruct>(gate_params: JsGateParams) -> Operation {
    let bits_out = match gate_params.get_bits_struct() {
        Some(s) => s.get_bits_out(),
        None => 1
    };
    let sgn = match gate_params.get_sign() {
        Some(s) => s.get_in().unwrap_or(false),
        None => false
    };

    Operation::ArithConst(
        arith_shift_const::<L, R>,
        gate_params.get_constant_num().unwrap_or(0), 
        gate_params.get_left_op().unwrap_or(false),
        bits_out,
        sgn
    )
}

fn create_arith_comp_const<T: ArithCompStruct>(gate_params: JsGateParams) -> Operation {
    let sgn = match gate_params.get_sign() {
        Some(s) => s.get_in().unwrap_or(false),
        None => false
    };

    Operation::CompConst(
        arith_const_comp::<T>,
        gate_params.get_constant_num().unwrap_or(0), 
        gate_params.get_left_op().unwrap_or(false),
        sgn
    )
}

fn create_multiplexer(gate_params: JsGateParams, operation: MuxIdx) -> Operation {
    let bits = match gate_params.get_bits_struct() {
        Some(s) => s.get_bits_in(),
        None => 1,
    };
    Operation::Mux(bits, operation)
}

fn create_mux_sparse(gate_params: JsGateParams) -> Operation {
    let bits = match gate_params.get_bits_struct() {
        Some(s) => s.get_bits_in(),
        None => 1,
    };
    let inputs = gate_params.get_inputs().map(|v| {
        v.iter().enumerate().map(|(idx, b)| {
            (b.toString(16), format!("in{idx}"))
        }).collect::<HashMap<String, String>>()
    }).unwrap_or_default();
    Operation::MuxSparse(bits, inputs)
}

pub struct ReturnValue {
    out: Option<Vec3vl>,
    others: HashMap<String, Vec3vl>,
    clock: bool
}

impl ReturnValue {
    pub fn out(val: Vec3vl) -> Result<ReturnValue, String> {
        Ok(ReturnValue { 
            out: Some(val), 
            others: HashMap::new(), 
            clock: false 
        })
    }

    pub fn clock(val: Vec3vl) -> Result<ReturnValue, String> {
        Ok(ReturnValue { 
            out: Some(val), 
            others: HashMap::new(), 
            clock: true 
        })
    }

    pub fn values(out: Option<Vec3vl>, others: HashMap<String, Vec3vl>) -> Result<ReturnValue, String> {
        Ok(ReturnValue { 
            out, 
            others, 
            clock: false 
        })
    }

    pub fn get_out(&self) -> Option<Vec3vl> {
        self.out.clone()
    }

    pub fn is_clock(&self) -> bool {
        self.clock
    }

    pub fn signals_iter(&self) -> Iter<'_, String, Vec3vl> {
        self.others.iter()
    }
}