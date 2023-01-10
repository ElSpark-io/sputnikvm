#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
mod misc;

use crate::{ExitError, ExitReason, ExitSucceed, Machine, Opcode};
use core::ops::{BitAnd, BitOr, BitXor};
use elrond_wasm::api::VMApi;
use eltypes::ToEH256;
use primitive_types::{H256, U256};

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Control {
	Continue(usize),
	Exit(ExitReason),
	Jump(usize),
	Trap(Opcode),
}

fn eval_stop<M: VMApi>(_state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	Control::Exit(ExitSucceed::Stopped.into())
}

fn eval_add<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_tuple!(state, overflowing_add)
}

fn eval_mul<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_tuple!(state, overflowing_mul)
}

fn eval_sub<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_tuple!(state, overflowing_sub)
}

fn eval_div<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::arithmetic::div)
}

fn eval_sdiv<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::arithmetic::sdiv)
}

fn eval_mod<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::arithmetic::rem)
}

fn eval_smod<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::arithmetic::srem)
}

fn eval_addmod<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op3_u256_fn!(state, self::arithmetic::addmod)
}

fn eval_mulmod<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op3_u256_fn!(state, self::arithmetic::mulmod)
}

fn eval_exp<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::arithmetic::exp)
}

fn eval_signextend<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::arithmetic::signextend)
}

fn eval_lt<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_bool_ref!(state, lt)
}

fn eval_gt<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_bool_ref!(state, gt)
}

fn eval_slt<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::bitwise::slt)
}

fn eval_sgt<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::bitwise::sgt)
}

fn eval_eq<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_bool_ref!(state, eq)
}

fn eval_iszero<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op1_u256_fn!(state, self::bitwise::iszero)
}

fn eval_and<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256!(state, bitand)
}

fn eval_or<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256!(state, bitor)
}

fn eval_xor<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256!(state, bitxor)
}

fn eval_not<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op1_u256_fn!(state, self::bitwise::not)
}

fn eval_byte<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::bitwise::byte)
}

fn eval_shl<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::bitwise::shl)
}

fn eval_shr<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::bitwise::shr)
}

fn eval_sar<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	op2_u256_fn!(state, self::bitwise::sar)
}

fn eval_codesize<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::codesize(state)
}

fn eval_codecopy<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::codecopy(state)
}

fn eval_calldataload<M: VMApi>(
	state: &mut Machine<M>,
	_opcode: Opcode,
	_position: usize,
) -> Control {
	self::misc::calldataload(state)
}

fn eval_calldatasize<M: VMApi>(
	state: &mut Machine<M>,
	_opcode: Opcode,
	_position: usize,
) -> Control {
	self::misc::calldatasize(state)
}

fn eval_calldatacopy<M: VMApi>(
	state: &mut Machine<M>,
	_opcode: Opcode,
	_position: usize,
) -> Control {
	self::misc::calldatacopy(state)
}

fn eval_pop<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::pop(state)
}

fn eval_mload<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::mload(state)
}

fn eval_mstore<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::mstore(state)
}

fn eval_mstore8<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::mstore8(state)
}

fn eval_jump<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::jump(state)
}

fn eval_jumpi<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::jumpi(state)
}

fn eval_pc<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::pc(state, position)
}

fn eval_msize<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::msize(state)
}

fn eval_jumpdest<M: VMApi>(_state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	Control::Continue(1)
}

fn eval_push1<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 1, position)
}

fn eval_push2<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 2, position)
}

fn eval_push3<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 3, position)
}

fn eval_push4<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 4, position)
}

fn eval_push5<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 5, position)
}

fn eval_push6<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 6, position)
}

fn eval_push7<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 7, position)
}

fn eval_push8<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 8, position)
}

fn eval_push9<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 9, position)
}

fn eval_push10<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 10, position)
}

fn eval_push11<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 11, position)
}

fn eval_push12<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 12, position)
}

fn eval_push13<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 13, position)
}

fn eval_push14<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 14, position)
}

fn eval_push15<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 15, position)
}

fn eval_push16<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 16, position)
}

fn eval_push17<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 17, position)
}

fn eval_push18<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 18, position)
}

fn eval_push19<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 19, position)
}

fn eval_push20<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 20, position)
}

fn eval_push21<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 21, position)
}

fn eval_push22<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 22, position)
}

fn eval_push23<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 23, position)
}

fn eval_push24<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 24, position)
}

fn eval_push25<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 25, position)
}

fn eval_push26<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 26, position)
}

fn eval_push27<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 27, position)
}

fn eval_push28<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 28, position)
}

fn eval_push29<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 29, position)
}

fn eval_push30<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 30, position)
}

fn eval_push31<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 31, position)
}

fn eval_push32<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, position: usize) -> Control {
	self::misc::push(state, 32, position)
}

fn eval_dup1<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 1)
}

fn eval_dup2<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 2)
}

fn eval_dup3<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 3)
}

fn eval_dup4<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 4)
}

fn eval_dup5<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 5)
}

fn eval_dup6<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 6)
}

fn eval_dup7<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 7)
}

fn eval_dup8<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 8)
}

fn eval_dup9<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 9)
}

fn eval_dup10<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 10)
}

fn eval_dup11<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 11)
}

fn eval_dup12<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 12)
}

fn eval_dup13<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 13)
}

fn eval_dup14<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 14)
}

fn eval_dup15<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 15)
}

fn eval_dup16<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::dup(state, 16)
}

fn eval_swap1<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 1)
}

fn eval_swap2<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 2)
}

fn eval_swap3<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 3)
}

fn eval_swap4<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 4)
}

fn eval_swap5<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 5)
}

fn eval_swap6<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 6)
}

fn eval_swap7<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 7)
}

fn eval_swap8<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 8)
}

fn eval_swap9<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 9)
}

fn eval_swap10<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 10)
}

fn eval_swap11<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 11)
}

fn eval_swap12<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 12)
}

fn eval_swap13<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 13)
}

fn eval_swap14<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 14)
}

fn eval_swap15<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 15)
}

fn eval_swap16<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::swap(state, 16)
}

fn eval_return<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::ret(state)
}

fn eval_revert<M: VMApi>(state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	self::misc::revert(state)
}

fn eval_invalid<M: VMApi>(_state: &mut Machine<M>, _opcode: Opcode, _position: usize) -> Control {
	Control::Exit(ExitError::DesignatedInvalid.into())
}

fn eval_external<M: VMApi>(_state: &mut Machine<M>, opcode: Opcode, _position: usize) -> Control {
	Control::Trap(opcode)
}

#[inline]
pub fn eval<M: VMApi>(state: &mut Machine<M>, opcode: Opcode, position: usize) -> Control {
	match opcode {
		Opcode::STOP => eval_stop(state, opcode, position),
        Opcode::ADD => eval_add(state, opcode, position),
        Opcode::MUL => eval_mul(state, opcode, position),
        Opcode::SUB => eval_sub(state, opcode, position),
        Opcode::DIV => eval_div(state, opcode, position),
        Opcode::SDIV => eval_sdiv(state, opcode, position),
        Opcode::MOD => eval_mod(state, opcode, position),
        Opcode::SMOD => eval_smod(state, opcode, position),
        Opcode::ADDMOD => eval_addmod(state, opcode, position),
        Opcode::MULMOD => eval_mulmod(state, opcode, position),
        Opcode::EXP => eval_exp(state, opcode, position),
        Opcode::SIGNEXTEND => eval_signextend(state, opcode, position),
        Opcode::LT => eval_lt(state, opcode, position),
        Opcode::GT => eval_gt(state, opcode, position),
        Opcode::SLT => eval_slt(state, opcode, position),
        Opcode::SGT => eval_sgt(state, opcode, position),
        Opcode::EQ => eval_eq(state, opcode, position),
        Opcode::ISZERO => eval_iszero(state, opcode, position),
        Opcode::AND => eval_and(state, opcode, position),
        Opcode::OR => eval_or(state, opcode, position),
        Opcode::XOR => eval_xor(state, opcode, position),
        Opcode::NOT => eval_not(state, opcode, position),
        Opcode::BYTE => eval_byte(state, opcode, position),
        Opcode::SHL => eval_shl(state, opcode, position),
        Opcode::SHR => eval_shr(state, opcode, position),
        Opcode::SAR => eval_sar(state, opcode, position),
        Opcode::CODESIZE => eval_codesize(state, opcode, position),
        Opcode::CODECOPY => eval_codecopy(state, opcode, position),
        Opcode::CALLDATALOAD => eval_calldataload(state, opcode, position),
        Opcode::CALLDATASIZE => eval_calldatasize(state, opcode, position),
        Opcode::CALLDATACOPY => eval_calldatacopy(state, opcode, position),
        Opcode::POP => eval_pop(state, opcode, position),
        Opcode::MLOAD => eval_mload(state, opcode, position),
        Opcode::MSTORE => eval_mstore(state, opcode, position),
        Opcode::MSTORE8 => eval_mstore8(state, opcode, position),
        Opcode::JUMP => eval_jump(state, opcode, position),
        Opcode::JUMPI => eval_jumpi(state, opcode, position),
        Opcode::PC => eval_pc(state, opcode, position),
        Opcode::MSIZE => eval_msize(state, opcode, position),
        Opcode::JUMPDEST => eval_jumpdest(state, opcode, position),

        Opcode::PUSH1 => eval_push1(state, opcode, position),
        Opcode::PUSH2 => eval_push2(state, opcode, position),
        Opcode::PUSH3 => eval_push3(state, opcode, position),
        Opcode::PUSH4 => eval_push4(state, opcode, position),
        Opcode::PUSH5 => eval_push5(state, opcode, position),
        Opcode::PUSH6 => eval_push6(state, opcode, position),
        Opcode::PUSH7 => eval_push7(state, opcode, position),
        Opcode::PUSH8 => eval_push8(state, opcode, position),
        Opcode::PUSH9 => eval_push9(state, opcode, position),
        Opcode::PUSH10 => eval_push10(state, opcode, position),
        Opcode::PUSH11 => eval_push11(state, opcode, position),
        Opcode::PUSH12 => eval_push12(state, opcode, position),
        Opcode::PUSH13 => eval_push13(state, opcode, position),
        Opcode::PUSH14 => eval_push14(state, opcode, position),
        Opcode::PUSH15 => eval_push15(state, opcode, position),
        Opcode::PUSH16 => eval_push16(state, opcode, position),
        Opcode::PUSH17 => eval_push17(state, opcode, position),
        Opcode::PUSH18 => eval_push18(state, opcode, position),
        Opcode::PUSH19 => eval_push19(state, opcode, position),
        Opcode::PUSH20 => eval_push20(state, opcode, position),
        Opcode::PUSH21 => eval_push21(state, opcode, position),
        Opcode::PUSH22 => eval_push22(state, opcode, position),
        Opcode::PUSH23 => eval_push23(state, opcode, position),
        Opcode::PUSH24 => eval_push24(state, opcode, position),
        Opcode::PUSH25 => eval_push25(state, opcode, position),
        Opcode::PUSH26 => eval_push26(state, opcode, position),
        Opcode::PUSH27 => eval_push27(state, opcode, position),
        Opcode::PUSH28 => eval_push28(state, opcode, position),
        Opcode::PUSH29 => eval_push29(state, opcode, position),
        Opcode::PUSH30 => eval_push30(state, opcode, position),
        Opcode::PUSH31 => eval_push31(state, opcode, position),
        Opcode::PUSH32 => eval_push32(state, opcode, position),

        Opcode::DUP1 => eval_dup1(state, opcode, position),
        Opcode::DUP2 => eval_dup2(state, opcode, position),
        Opcode::DUP3 => eval_dup3(state, opcode, position),
        Opcode::DUP4 => eval_dup4(state, opcode, position),
        Opcode::DUP5 => eval_dup5(state, opcode, position),
        Opcode::DUP6 => eval_dup6(state, opcode, position),
        Opcode::DUP7 => eval_dup7(state, opcode, position),
        Opcode::DUP8 => eval_dup8(state, opcode, position),
        Opcode::DUP9 => eval_dup9(state, opcode, position),
        Opcode::DUP10 => eval_dup10(state, opcode, position),
        Opcode::DUP11 => eval_dup11(state, opcode, position),
        Opcode::DUP12 => eval_dup12(state, opcode, position),
        Opcode::DUP13 => eval_dup13(state, opcode, position),
        Opcode::DUP14 => eval_dup14(state, opcode, position),
        Opcode::DUP15 => eval_dup15(state, opcode, position),
        Opcode::DUP16 => eval_dup16(state, opcode, position),

        Opcode::SWAP1 => eval_swap1(state, opcode, position),
        Opcode::SWAP2 => eval_swap2(state, opcode, position),
        Opcode::SWAP3 => eval_swap3(state, opcode, position),
        Opcode::SWAP4 => eval_swap4(state, opcode, position),
        Opcode::SWAP5 => eval_swap5(state, opcode, position),
        Opcode::SWAP6 => eval_swap6(state, opcode, position),
        Opcode::SWAP7 => eval_swap7(state, opcode, position),
        Opcode::SWAP8 => eval_swap8(state, opcode, position),
        Opcode::SWAP9 => eval_swap9(state, opcode, position),
        Opcode::SWAP10 => eval_swap10(state, opcode, position),
        Opcode::SWAP11 => eval_swap11(state, opcode, position),
        Opcode::SWAP12 => eval_swap12(state, opcode, position),
        Opcode::SWAP13 => eval_swap13(state, opcode, position),
        Opcode::SWAP14 => eval_swap14(state, opcode, position),
        Opcode::SWAP15 => eval_swap15(state, opcode, position),
        Opcode::SWAP16 => eval_swap16(state, opcode, position),

        Opcode::RETURN => eval_return(state, opcode, position),
        Opcode::REVERT => eval_revert(state, opcode, position),
        Opcode::INVALID => eval_invalid(state, opcode, position),

		_ => panic!("Opcode doesn't found!"),
	}
}
