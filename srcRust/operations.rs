use std::collections::hash_map::Iter;
use std::collections::HashMap;

use crate::cell_arith::{add, add_c, arith_binop, arith_comp_const_op, arith_comp_op, arith_const_op, arith_monop, div, div_c, equal, equal_c, greater, greater_c, greater_equal, greater_equal_c, less, less_c, less_equal, less_equal_c, modulo, modulo_c, mul, mul_c, negation, negation_c, not_equal, not_equal_c, power, power_c, shift_left, shift_left_c, shift_right, shift_right_c, sub, sub_c, unary_plus, unary_plus_c, ArithBinop, ArithComp, ArithConstBinop, ArithConstComp, ArithConstMonop, ArithMonop};
use crate::cell_bus::{bit_extend, bus_group, bus_slice, sign_extend, zero_extend, ExtendFn};
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
    Arith11(ArithMonop, ArithConstMonop),
    Arith21(ArithBinop, ArithConstBinop),
    ArithConst(ArithConstBinop, u32, bool),
    Comp(ArithComp, ArithConstComp),
    CompConst(ArithConstComp, u32, bool),
    BitExtend(ExtendFn, u32),
    BusGroup,
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

            "Constant"  => create_constant(gate_params),
            "Clock"     => Operation::Clock(false),
            
            "Dff"       => Operation::Dff(DffState::new(gate_params)),
            "FSM"       => Operation::Fsm(FsmState::new(gate_params)),

            "Lt"        => Operation::Comp(less,             less_c),
            "Le"        => Operation::Comp(less_equal,       less_equal_c),
            "Gt"        => Operation::Comp(greater,          greater_c),
            "Ge"        => Operation::Comp(greater_equal,    greater_equal_c),
            "Eq"        => Operation::Comp(equal,            equal_c),
            "Ne"        => Operation::Comp(not_equal,        not_equal_c),

            "LtConst"   => create_constant_comp(gate_params, less_c),
            "LeConst"   => create_constant_comp(gate_params, less_equal_c),
            "GtConst"   => create_constant_comp(gate_params, greater_c),
            "GeConst"   => create_constant_comp(gate_params, greater_equal_c),
            "EqConst"   => create_constant_comp(gate_params, equal_c),
            "NeConst"   => create_constant_comp(gate_params, not_equal_c),

            "Negation"  => Operation::Arith11(negation, negation_c),
            "UnaryPlus" => Operation::Arith11(unary_plus, unary_plus_c), 

            "Addition"       => Operation::Arith21(add,          add_c),
            "Subtraction"    => Operation::Arith21(sub,          sub_c),
            "Multiplication" => Operation::Arith21(mul,          mul_c),
            "Division"       => Operation::Arith21(div,          div_c),
            "Modulo"         => Operation::Arith21(modulo,       modulo_c),
            "Power"          => Operation::Arith21(power,        power_c),
            "ShiftLeft"      => Operation::Arith21(shift_left,   shift_left_c),
            "ShiftRight"     => Operation::Arith21(shift_right,  shift_right_c),

            "AdditionConst"       => create_arith_const(gate_params, add_c),
            "SubtractionConst"    => create_arith_const(gate_params, sub_c),
            "MultiplicationConst" => create_arith_const(gate_params, mul_c),
            "DivisionConst"       => create_arith_const(gate_params, div_c),
            "ModuloConst"         => create_arith_const(gate_params, modulo_c),
            "PowerConst"          => create_arith_const(gate_params, power_c),
            "ShiftLeftConst"      => create_arith_const(gate_params, shift_left_c),
            "ShiftRightConst"     => create_arith_const(gate_params, shift_right_c),

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
            Operation::Arith11(op, op_c) => arith_monop(args, op, op_c),
            Operation::Arith21(op, op_c) => arith_binop(args, op, op_c),
            Operation::ArithConst(op, constant, left_op) => arith_const_op(args, op, constant, left_op),
            Operation::Comp(op, op_c) => arith_comp_op(args, op, op_c),
            Operation::CompConst(op, constant, left_op) => arith_comp_const_op(args, op, constant, left_op),
            Operation::Gate11(op) => gate_11(args, op),
            Operation::GateX1(op) => gate_x1(args, op),
            Operation::GateReduce(op) => gate_reduce(args, op),
            Operation::BitExtend(op, bits) => bit_extend(args, op, *bits),
            Operation::BusSlice(options) => bus_slice(args, options),
            Operation::BusGroup => bus_group(args),
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
            Operation::Arith11(_, _)        => "Arith11",
            Operation::Arith21(_, _)        => "Arith21",
            Operation::ArithConst(_, _, _)  => "ArithConst",
            Operation::Comp(_, _)           => "Comp",
            Operation::CompConst(_, _, _)   => "CompConst",
            Operation::BitExtend(_, _)      => "BitExtend",
            Operation::BusGroup             => "BusGroup",
            Operation::BusSlice(_)          => "BusSlice",
            Operation::Clock(_)             => "Clock",
            Operation::Constant(_)          => "Constant",
            Operation::Dff(_)               => "DFF",
            Operation::Fsm(_)               => "FSM",
            Operation::Gate11(_)            => "Gate11",
            Operation::GateX1(_)            => "GateX1",
            Operation::GateReduce(_)        => "GateReduce",
            Operation::Mux(_, _)            => "Mux",
            Operation::MuxSparse(_, _)      => "MuxSparse",
            Operation::Memory(_)            => "Memory",
            Operation::None                 => "None",
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

fn create_constant_comp(gate_params: JsGateParams, operation: ArithConstComp) -> Operation {
    Operation::CompConst(
        operation, 
        gate_params.get_constant_num().unwrap_or_default(), 
        gate_params.get_left_op().unwrap_or(false)
    )
}

fn create_arith_const(gate_params: JsGateParams, operation: ArithConstBinop) -> Operation {
    Operation::ArithConst(
        operation, 
        gate_params.get_constant_num().unwrap_or_default(), 
        gate_params.get_left_op().unwrap_or(false)
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