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
	let TABLE: [fn(state: &mut Machine<M>, opcode: Opcode, position: usize) -> Control; 256] = {
		let mut table = [eval_external as _; 256];
		table[Opcode::STOP.as_usize()] = eval_stop as _;
		table[Opcode::ADD.as_usize()] = eval_add as _;
		table[Opcode::MUL.as_usize()] = eval_mul as _;
		table[Opcode::SUB.as_usize()] = eval_sub as _;
		table[Opcode::DIV.as_usize()] = eval_div as _;
		table[Opcode::SDIV.as_usize()] = eval_sdiv as _;
		table[Opcode::MOD.as_usize()] = eval_mod as _;
		table[Opcode::SMOD.as_usize()] = eval_smod as _;
		table[Opcode::ADDMOD.as_usize()] = eval_addmod as _;
		table[Opcode::MULMOD.as_usize()] = eval_mulmod as _;
		table[Opcode::EXP.as_usize()] = eval_exp as _;
		table[Opcode::SIGNEXTEND.as_usize()] = eval_signextend as _;
		table[Opcode::LT.as_usize()] = eval_lt as _;
		table[Opcode::GT.as_usize()] = eval_gt as _;
		table[Opcode::SLT.as_usize()] = eval_slt as _;
		table[Opcode::SGT.as_usize()] = eval_sgt as _;
		table[Opcode::EQ.as_usize()] = eval_eq as _;
		table[Opcode::ISZERO.as_usize()] = eval_iszero as _;
		table[Opcode::AND.as_usize()] = eval_and as _;
		table[Opcode::OR.as_usize()] = eval_or as _;
		table[Opcode::XOR.as_usize()] = eval_xor as _;
		table[Opcode::NOT.as_usize()] = eval_not as _;
		table[Opcode::BYTE.as_usize()] = eval_byte as _;
		table[Opcode::SHL.as_usize()] = eval_shl as _;
		table[Opcode::SHR.as_usize()] = eval_shr as _;
		table[Opcode::SAR.as_usize()] = eval_sar as _;
		table[Opcode::CODESIZE.as_usize()] = eval_codesize as _;
		table[Opcode::CODECOPY.as_usize()] = eval_codecopy as _;
		table[Opcode::CALLDATALOAD.as_usize()] = eval_calldataload as _;
		table[Opcode::CALLDATASIZE.as_usize()] = eval_calldatasize as _;
		table[Opcode::CALLDATACOPY.as_usize()] = eval_calldatacopy as _;
		table[Opcode::POP.as_usize()] = eval_pop as _;
		table[Opcode::MLOAD.as_usize()] = eval_mload as _;
		table[Opcode::MSTORE.as_usize()] = eval_mstore as _;
		table[Opcode::MSTORE8.as_usize()] = eval_mstore8 as _;
		table[Opcode::JUMP.as_usize()] = eval_jump as _;
		table[Opcode::JUMPI.as_usize()] = eval_jumpi as _;
		table[Opcode::PC.as_usize()] = eval_pc as _;
		table[Opcode::MSIZE.as_usize()] = eval_msize as _;
		table[Opcode::JUMPDEST.as_usize()] = eval_jumpdest as _;

		table[Opcode::PUSH1.as_usize()] = eval_push1 as _;
		table[Opcode::PUSH2.as_usize()] = eval_push2 as _;
		table[Opcode::PUSH3.as_usize()] = eval_push3 as _;
		table[Opcode::PUSH4.as_usize()] = eval_push4 as _;
		table[Opcode::PUSH5.as_usize()] = eval_push5 as _;
		table[Opcode::PUSH6.as_usize()] = eval_push6 as _;
		table[Opcode::PUSH7.as_usize()] = eval_push7 as _;
		table[Opcode::PUSH8.as_usize()] = eval_push8 as _;
		table[Opcode::PUSH9.as_usize()] = eval_push9 as _;
		table[Opcode::PUSH10.as_usize()] = eval_push10 as _;
		table[Opcode::PUSH11.as_usize()] = eval_push11 as _;
		table[Opcode::PUSH12.as_usize()] = eval_push12 as _;
		table[Opcode::PUSH13.as_usize()] = eval_push13 as _;
		table[Opcode::PUSH14.as_usize()] = eval_push14 as _;
		table[Opcode::PUSH15.as_usize()] = eval_push15 as _;
		table[Opcode::PUSH16.as_usize()] = eval_push16 as _;
		table[Opcode::PUSH17.as_usize()] = eval_push17 as _;
		table[Opcode::PUSH18.as_usize()] = eval_push18 as _;
		table[Opcode::PUSH19.as_usize()] = eval_push19 as _;
		table[Opcode::PUSH20.as_usize()] = eval_push20 as _;
		table[Opcode::PUSH21.as_usize()] = eval_push21 as _;
		table[Opcode::PUSH22.as_usize()] = eval_push22 as _;
		table[Opcode::PUSH23.as_usize()] = eval_push23 as _;
		table[Opcode::PUSH24.as_usize()] = eval_push24 as _;
		table[Opcode::PUSH25.as_usize()] = eval_push25 as _;
		table[Opcode::PUSH26.as_usize()] = eval_push26 as _;
		table[Opcode::PUSH27.as_usize()] = eval_push27 as _;
		table[Opcode::PUSH28.as_usize()] = eval_push28 as _;
		table[Opcode::PUSH29.as_usize()] = eval_push29 as _;
		table[Opcode::PUSH30.as_usize()] = eval_push30 as _;
		table[Opcode::PUSH31.as_usize()] = eval_push31 as _;
		table[Opcode::PUSH32.as_usize()] = eval_push32 as _;

		table[Opcode::DUP1.as_usize()] = eval_dup1 as _;
		table[Opcode::DUP2.as_usize()] = eval_dup2 as _;
		table[Opcode::DUP3.as_usize()] = eval_dup3 as _;
		table[Opcode::DUP4.as_usize()] = eval_dup4 as _;
		table[Opcode::DUP5.as_usize()] = eval_dup5 as _;
		table[Opcode::DUP6.as_usize()] = eval_dup6 as _;
		table[Opcode::DUP7.as_usize()] = eval_dup7 as _;
		table[Opcode::DUP8.as_usize()] = eval_dup8 as _;
		table[Opcode::DUP9.as_usize()] = eval_dup9 as _;
		table[Opcode::DUP10.as_usize()] = eval_dup10 as _;
		table[Opcode::DUP11.as_usize()] = eval_dup11 as _;
		table[Opcode::DUP12.as_usize()] = eval_dup12 as _;
		table[Opcode::DUP13.as_usize()] = eval_dup13 as _;
		table[Opcode::DUP14.as_usize()] = eval_dup14 as _;
		table[Opcode::DUP15.as_usize()] = eval_dup15 as _;
		table[Opcode::DUP16.as_usize()] = eval_dup16 as _;

		table[Opcode::SWAP1.as_usize()] = eval_swap1 as _;
		table[Opcode::SWAP2.as_usize()] = eval_swap2 as _;
		table[Opcode::SWAP3.as_usize()] = eval_swap3 as _;
		table[Opcode::SWAP4.as_usize()] = eval_swap4 as _;
		table[Opcode::SWAP5.as_usize()] = eval_swap5 as _;
		table[Opcode::SWAP6.as_usize()] = eval_swap6 as _;
		table[Opcode::SWAP7.as_usize()] = eval_swap7 as _;
		table[Opcode::SWAP8.as_usize()] = eval_swap8 as _;
		table[Opcode::SWAP9.as_usize()] = eval_swap9 as _;
		table[Opcode::SWAP10.as_usize()] = eval_swap10 as _;
		table[Opcode::SWAP11.as_usize()] = eval_swap11 as _;
		table[Opcode::SWAP12.as_usize()] = eval_swap12 as _;
		table[Opcode::SWAP13.as_usize()] = eval_swap13 as _;
		table[Opcode::SWAP14.as_usize()] = eval_swap14 as _;
		table[Opcode::SWAP15.as_usize()] = eval_swap15 as _;
		table[Opcode::SWAP16.as_usize()] = eval_swap16 as _;

		table[Opcode::RETURN.as_usize()] = eval_return as _;
		table[Opcode::REVERT.as_usize()] = eval_revert as _;
		table[Opcode::INVALID.as_usize()] = eval_invalid as _;

		table
	};

	TABLE[opcode.as_usize()](state, opcode, position)
}
