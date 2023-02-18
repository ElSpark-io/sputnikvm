#![cfg_attr(not(feature = "std"), no_std)]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::heap::String;
use multiversx_sc::api::VMApi;
use primitive_types::{H160, H256, U256};

pub mod events;
pub mod storage;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct ETHAddress<M: ManagedTypeApi>(pub ManagedByteArray<M, 20>);

impl<M: ManagedTypeApi> ETHAddress<M> {
	pub fn new(bytes: ManagedByteArray<M, 20>) -> Self {
		Self(bytes)
	}

	// /// Get raw H160 data
	// pub fn raw(&self) -> H160 {
	//     H160::from_slice(&self.0)
	// }

	/// Encode address to string
	pub fn encode(&self) -> String {
		multiversx_sc::formatter::hex_util::encode_bytes_as_hex(&self.0.to_byte_array())
	}

	// pub fn decode(address: &str) -> SCResult<ETHAddress, error::AddressError> { // TODO
	//     if address.len() != 40 {
	//         return Err(error::AddressError::IncorrectLength);
	//     }

	//     let mut result = multiversx_sc::hex_literal::hex!(address);
	//     Ok(ETHAddress::new(H160(result)))
	// }

	// pub fn try_from_slice(raw_addr: &[u8]) -> SCResult<Self, error::AddressError> {
	//     if raw_addr.len() != 20 {
	//         return Err(error::AddressError::IncorrectLength);
	//     }
	//     Ok(Self::new(H160::from_slice(raw_addr)))
	// }

	pub fn from_array(array: [u8; 20]) -> Self {
		Self(ManagedByteArray::new_from_bytes(&array))
	}
}

pub mod error {
	// use crate::fmt;

	#[derive(Eq, Hash, Clone, Debug, PartialEq)]
	pub enum AddressError {
		FailedDecodeHex,
		IncorrectLength,
	}

	impl AsRef<[u8]> for AddressError {
		fn as_ref(&self) -> &[u8] {
			match self {
				Self::FailedDecodeHex => b"FAILED_DECODE_ETH_ADDRESS",
				Self::IncorrectLength => b"ETH_WRONG_ADDRESS_LENGTH",
			}
		}
	}
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Default)]
pub struct ElsparkH256(pub [u8; 32]);

impl ElsparkH256 {
	pub fn from(h256: H256) -> Self {
		Self(h256.0)
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Default)]
pub struct ElsparkU256(pub [u64; 4]);
impl ElsparkU256 {
	pub fn new(u256: U256) -> Self {
		Self(u256.0)
	}

	pub fn is_zero(&self) -> bool {
		self.0 == U256::zero().0
	}

	pub fn raw(&self) -> U256 {
		U256(self.0)
	}

	pub fn saturating_add(&self, u256: U256) -> ElsparkU256 {
		ElsparkU256(self.raw().saturating_add(u256).0)
	}

	pub fn saturating_add_2(&self, elspark_u256: ElsparkU256) -> ElsparkU256 {
		ElsparkU256(self.raw().saturating_add(elspark_u256.raw()).0)
	}
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Default)]
pub struct Wei(pub [u64; 4]);

impl Wei {
	pub fn from_biguint<T: VMApi>(biguint: BigUint<T>) -> Self {
		Wei(U256::from_big_endian(biguint.to_bytes_be().as_ref()).0)
	}
	pub fn raw(self) -> [u64; 4] {
		self.0
	}

	pub fn is_zero(&self) -> bool {
		self.0 == U256::zero().0
	}

	pub fn as_u256(&self) -> U256 {
		U256(self.0)
	}
}

pub trait ToBigUint {
	fn to_biguint<T: VMApi>(self) -> BigUint<T>;
}

pub trait ToManagedVec {
	fn to_managed_vec<T: VMApi>(self) -> ManagedVec<T, u8>;
}

pub trait ToU256 {
	fn to_u256(self) -> U256;
}

pub trait ToH256 {
	fn to_h256(self) -> H256;
}

impl ToBigUint for U256 {
	fn to_biguint<T: VMApi>(self) -> BigUint<T> {
		let mut bytes = [0u8; 32];
		self.to_big_endian(&mut bytes);
		BigUint::from_bytes_be(&bytes)
	}
}

impl ToManagedVec for H256 {
	fn to_managed_vec<T: VMApi>(self) -> ManagedVec<T, u8> {
		ManagedVec::from(self.as_bytes().to_vec())
	}
}

impl<T: VMApi> ToU256 for BigUint<T> {
	fn to_u256(self) -> U256 {
		U256::from_big_endian(self.to_bytes_be().as_ref())
	}
}

impl<T: VMApi> ToH256 for ManagedVec<T, u8> {
	fn to_h256(self) -> H256 {
		// TODO: Check self.len < 32 condition
		if self.len() < 32 {
			return H256([0u8; 32]);
		}

		let mut result = [0u8; 32];
		for i in 0..32 {
			result[i] = self.get(i);
		}
		H256::from(&result)
	}
}

impl<T: VMApi> ToH256 for ManagedBuffer<T> {
	fn to_h256(self) -> H256 {
		// TODO: Check self.len < 32 condition
		if self.len() < 32 {
			return H256([0u8; 32]);
		}

		let mut result = [0u8; 32];
		self.load_slice(0, &mut result).ok();
		H256::from(&result)
	}
}

// #![cfg_attr(not(feature = "std"), no_std)]
//
// use core::cmp::Ordering;
//
// use multiversx_sc::api::{InvalidSliceError, VMApi};
// use primitive_types::H256;
// use multiversx_sc::types::heap::Vec;
//
// multiversx_sc::imports!();
// multiversx_sc::derive_imports!();
//
// #[derive(
// 	ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Default,
// )]
// pub struct ETHAddress {
// 	pub data: [u8; 20],
// }
//
// impl ETHAddress {
// 	pub fn from(h160: primitive_types::H160) -> Self {
// 		Self { data: h160.0 }
// 	}
//
// 	pub fn as_bytes(&self) -> &[u8] {
// 		&self.data
// 	}
//
// 	pub fn to_h160(&self) -> primitive_types::H160 {
// 		primitive_types::H160(self.data)
// 	}
// }
// #[derive(
// 	TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Default, Clone, Debug, ManagedVecItem,
// )]
// pub struct EH256 {
// 	pub data: [u8; 32],
// }
//
// impl EH256 {
// 	pub fn from(h256: primitive_types::H256) -> Self {
// 		Self { data: h256.0 }
// 	}
//
// 	pub fn as_bytes(&self) -> &[u8] {
// 		&self.data
// 	}
//
// 	pub fn to_h256(&self) -> primitive_types::H256 {
// 		primitive_types::H256(self.data)
// 	}
// }
// impl Eq for EH256 {}
//
// impl PartialEq for EH256 {
// 	fn eq(&self, other: &Self) -> bool {
// 		self.data == other.data
// 	}
// }
//
// pub trait ToEH256 {
// 	fn to_eh256(self) -> EH256;
// }
//
// impl ToEH256 for primitive_types::H256 {
// 	fn to_eh256(self) -> EH256 {
// 		EH256::from(self)
// 	}
// }
//
// pub trait ToH256 {
// 	fn to_h256(self) -> H256;
// }
// impl ToH256 for EH256 {
// 	fn to_h256(self) -> H256 {
// 		H256::from(&self.data)
// 	}
// }
//
// pub trait ManagedVecforEH256<M: VMApi> {
// 	fn to_managed_buffer(&self) -> ManagedBuffer<M>;
// }
//
// impl<M: VMApi> ManagedVecforEH256<M> for EH256 {
// 	fn to_managed_buffer(&self) -> ManagedBuffer<M> {
// 		ManagedBuffer::new_from_bytes(&self.data)
// 	}
// }
//
// pub struct ManagedBufferRefIterator<'a, M: VMApi> {
// 	managed_buffer: &'a ManagedBuffer<M>,
// 	byte_start: usize,
// 	byte_end: usize,
// }
//
// impl<'a, M> ManagedBufferRefIterator<'a, M>
// where
// 	M: VMApi,
// {
// 	pub(crate) fn new(managed_buffer: &'a ManagedBuffer<M>) -> Self {
// 		ManagedBufferRefIterator {
// 			managed_buffer,
// 			byte_start: 0,
// 			byte_end: managed_buffer.len(),
// 		}
// 	}
// }
//
// impl<'a, M> Iterator for ManagedBufferRefIterator<'a, M>
// where
// 	M: VMApi,
// {
// 	type Item = u8;
//
// 	fn next(&mut self) -> Option<Self::Item> {
// 		let next_byte_start = self.byte_start + 1;
// 		if next_byte_start > self.byte_end {
// 			return None;
// 		}
//
// 		let result = unsafe {
// 			u8::from_byte_reader_as_borrow(|dest_slice| {
// 				let _ = self.managed_buffer.load_slice(self.byte_start, dest_slice);
// 			})
// 		};
//
// 		self.byte_start = next_byte_start;
// 		Some(result)
// 	}
// }
//
// pub trait ManagedBufferAccess<M: VMApi> {
// 	fn push(&mut self, byte: u8);
// 	fn get(&self, index: usize) -> u8;
// 	fn try_get(&self, index: usize) -> Option<u8>;
// 	fn set(&mut self, index: usize, data: u8) -> Result<(), InvalidSliceError>;
// 	fn resize(&mut self, size: usize, value: u8);
// 	fn to_vec(&self) -> Vec<u8>;
// 	fn iter(&self) -> ManagedBufferRefIterator<M>;
// }
//
// impl<M: VMApi> ManagedBufferAccess<M> for ManagedBuffer<M> {
// 	fn push(&mut self, byte: u8) {
// 		self.append_bytes(&[byte])
// 	}
//
// 	fn get(&self, index: usize) -> u8 {
// 		match self.try_get(index) {
// 			Some(result) => result,
// 			None => M::error_api_impl().signal_error(b"INDEX_OUT_OF_RANGE_MSG"),
// 		}
// 	}
//
// 	fn try_get(&self, index: usize) -> Option<u8> {
// 		let mut dest_slice = [0u8; 1];
// 		let load_result = self.load_slice(index, &mut dest_slice);
// 		match load_result {
// 			Result::Ok(_) => Some(dest_slice[0]),
// 			Result::Err(_) => None,
// 		}
// 	}
//
// 	fn set(&mut self, index: usize, byte: u8) -> Result<(), InvalidSliceError> {
// 		self.set_slice(index, &[byte])
// 	}
//
// 	fn resize(&mut self, size: usize, byte: u8) {
// 		let len = self.len();
//
// 		if size > len {
// 			// extend
// 			for _ in len..size {
// 				self.append_bytes(&[byte]);
// 			}
// 		} else {
// 			// truncate
// 			self.overwrite(&[]);
// 			for _ in 0..size {
// 				self.append_bytes(&[byte]);
// 			}
// 		}
// 	}
//
// 	// TODO: This needs to be optimized for sure!
// 	fn to_vec(&self) -> Vec<u8> {
// 		self.to_boxed_bytes().into_vec()
// 	}
//
// 	fn iter(&self) -> ManagedBufferRefIterator<M> {
// 		ManagedBufferRefIterator::new(self)
// 	}
// }
//
// pub trait Resizable<M: VMApi, T: ManagedVecItem> {
// 	fn resize(&mut self, size: usize, value: T);
// }
//
// impl<M: VMApi, T: ManagedVecItem + Clone> Resizable<M, T> for ManagedVec<M, T> {
// 	fn resize(&mut self, size: usize, item: T) {
// 		let len = self.len();
//
// 		if size > len {
// 			// extend
// 			for _ in len..size {
// 				self.push(item.clone());
// 			}
// 		} else {
// 			// truncate
// 			self.clear();
// 			for _ in 0..size {
// 				self.push(item.clone());
// 			}
// 		}
// 	}
// }
