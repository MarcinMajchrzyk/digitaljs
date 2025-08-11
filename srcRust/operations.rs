use std::collections::HashMap;

use crate::cell_arith::{add, add_c, arith_comp_const_op, arith_comp_op, arith_const_op, arith_op, div, div_c, equal, equal_c, greater, greater_c, greater_equal, greater_equal_c, less, less_c, less_equal, less_equal_c, modulo, modulo_c, mul, mul_c, not_equal, not_equal_c, power, power_c, shift_left, shift_left_c, shift_right, shift_right_c, sub, sub_c, ArithBinop, ArithComp, ArithConstBinop, ArithConstComp};
use crate::cell_bus::{bit_extend, bus_group, bus_slice, sign_extend, zero_extend, ExtendFn};
use crate::cell_dff::{dff, DffState};
use crate::cell_fsm::{fsm, FsmState};
use crate::cell_io::{clock, constant};
use crate::cell_memory::{memory_op, MemoryState};
use crate::cell_mux::{mux1hot_idx, mux_idx, mux_op, sparse_mux_op, MuxIdx};
use crate::gate::{GateParams, SliceOptions};
use crate::vector3vl::Vec3vl;

use crate::cell_gates::{and, gate_11, gate_reduce, gate_x1, nand, nor, not, or, xnor, xor, Binop, Monop, ReduceFn};

pub enum Operation {
    //Arith11(Monop),
    Arith21(ArithBinop, ArithConstBinop),
    ArithConst(ArithConstBinop, Option<u32>, Option<bool>),
    Comp(ArithComp, ArithConstComp),
    CompConst(ArithConstComp, Option<u32>, Option<bool>),
    BitExtend(ExtendFn, u32),
    BusGroup,
    BusSlice(Option<SliceOptions>),
    Clock(bool),
    Constant(Option<String>),
    Dff(DffState),
    Fsm(FsmState),
    Gate11(Monop),
    GateX1(Binop),
    GateReduce(ReduceFn),
    Mux(u32, MuxIdx),
    MuxSparse(u32, Option<HashMap<String, String>>),
    Memory(MemoryState),
    None
}

impl Operation {
    pub fn from_name(name: String, gate_params: &GateParams) -> Result<Operation, String> {
        Ok(match name.as_str() {
            "Not"       => Operation::Gate11(not),

            "And"       => Operation::GateX1(and),
            "Or"        => Operation::GateX1(or),
            "Xor"       => Operation::GateX1(xor),
            "Nand"      => Operation::GateX1(nand),
            "Nor"       => Operation::GateX1(nor),
            "Xnor"      => Operation::GateX1(xnor),

            "AndReduce"     => Operation::GateReduce(Vec3vl::reduce_and),
            "OrReduce"      => Operation::GateReduce(Vec3vl::reduce_or),
            "XorReduce"     => Operation::GateReduce(Vec3vl::reduce_xor),
            "NandReduce"    => Operation::GateReduce(Vec3vl::reduce_nand),
            "NorReduce"     => Operation::GateReduce(Vec3vl::reduce_nor),
            "XnorReduce"    => Operation::GateReduce(Vec3vl::reduce_xnor),

            "ZeroExtend"    => Operation::BitExtend(zero_extend, calc_bits_extend(gate_params.bits_in, gate_params.bits_out)?),
            "SignExtend"    => Operation::BitExtend(sign_extend, calc_bits_extend(gate_params.bits_in, gate_params.bits_out)?),
            "BusSlice"      => Operation::BusSlice(gate_params.slice),
            "BusGroup"  => Operation::BusGroup,

            "Constant"  => Operation::Constant(gate_params.constant_str.clone()),
            "Clock"     => Operation::Clock(false),
            
            "Dff"       => Operation::Dff(DffState::new(gate_params)),
            "FSM"       => Operation::Fsm(FsmState::new(gate_params)?),

            "Lt"        => Operation::Comp(less, less_c),
            "Le"        => Operation::Comp(less_equal, less_equal_c),
            "Gt"        => Operation::Comp(greater, greater_c),
            "Ge"        => Operation::Comp(greater_equal, greater_equal_c),
            "Eq"        => Operation::Comp(equal, equal_c),
            "Ne"        => Operation::Comp(not_equal, not_equal_c),

            "LtConst"   => Operation::CompConst(less_c,          gate_params.constant_num, gate_params.left_op),
            "LeConst"   => Operation::CompConst(less_equal_c,    gate_params.constant_num, gate_params.left_op),
            "GtConst"   => Operation::CompConst(greater_c,       gate_params.constant_num, gate_params.left_op),
            "GeConst"   => Operation::CompConst(greater_equal_c, gate_params.constant_num, gate_params.left_op),
            "EqConst"   => Operation::CompConst(equal_c,         gate_params.constant_num, gate_params.left_op),
            "NeConst"   => Operation::CompConst(not_equal_c,     gate_params.constant_num, gate_params.left_op),

            "Addition"       => Operation::Arith21(add, add_c),
            "Subtraction"    => Operation::Arith21(sub, sub_c),
            "Multiplication" => Operation::Arith21(mul, mul_c),
            "Division"       => Operation::Arith21(div, div_c),
            "Modulo"         => Operation::Arith21(modulo, modulo_c),
            "Power"          => Operation::Arith21(power, power_c),
            "ShiftLeft"      => Operation::Arith21(shift_left, shift_left_c),
            "ShiftRight"     => Operation::Arith21(shift_right, shift_right_c),

            "AdditionConst"       => Operation::ArithConst(add_c,         gate_params.constant_num, gate_params.left_op),
            "SubtractionConst"    => Operation::ArithConst(sub_c,         gate_params.constant_num, gate_params.left_op),
            "MultiplicationConst" => Operation::ArithConst(mul_c,         gate_params.constant_num, gate_params.left_op),
            "DivisionConst"       => Operation::ArithConst(div_c,         gate_params.constant_num, gate_params.left_op),
            "ModuloConst"         => Operation::ArithConst(modulo_c,      gate_params.constant_num, gate_params.left_op),
            "PowerConst"          => Operation::ArithConst(power_c,       gate_params.constant_num, gate_params.left_op),
            "ShiftLeftConst"      => Operation::ArithConst(shift_left_c,  gate_params.constant_num, gate_params.left_op),
            "ShiftRightConst"     => Operation::ArithConst(shift_right_c, gate_params.constant_num, gate_params.left_op),

            "Mux"       => Operation::Mux(gate_params.bits_in, mux_idx),
            "Mux1Hot"   => Operation::Mux(gate_params.bits_in, mux1hot_idx),
            "MuxSparse" => Operation::MuxSparse(gate_params.bits_in, gate_params.inputs.clone()),

            "Memory"    => Operation::Memory(MemoryState::new(gate_params)?),
            _           => Operation::None
        })
    }

    pub fn op(&mut self, args: &HashMap<String, Vec3vl>) -> Result<ReturnValue, String> {
        match self {
            Operation::Arith21(op, op_c) => arith_op(args, op, op_c),
            Operation::ArithConst(op, constant, left_op) => arith_const_op(args, op, *constant, *left_op),
            Operation::Comp(op, op_c) => arith_comp_op(args, op, op_c),
            Operation::CompConst(op, constant, left_op) => arith_comp_const_op(args, op, *constant, *left_op),
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

fn calc_bits_extend(input: u32, output: Option<u32>) -> Result<u32, String> {
    match output {
        Some(out) => Ok(out - input),
        None => Err("Bit extend cell without output size".to_string())
    }
}

pub struct ReturnValue {
    pub out: Option<Vec3vl>,
    pub others: HashMap<String, Vec3vl>,
    pub clock: bool
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
}