#![cfg_attr(not(feature = "std"), no_std)]

use core::cmp::Ordering;

use elrond_wasm::api::{InvalidSliceError, VMApi};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(
	ManagedVecItem, TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Default,
)]
pub struct ETHAddress {
	pub data: [u8; 20],
}

impl ETHAddress {
	pub fn from(h160: primitive_types::H160) -> Self {
		Self { data: h160.0 }
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.data
	}

	pub fn to_h160(&self) -> primitive_types::H160 {
		primitive_types::H160(self.data)
	}
}
#[derive(
	TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Default, Clone, Debug, ManagedVecItem,
)]
pub struct EH256 {
	pub data: [u8; 32],
}

impl EH256 {
	pub fn from(h256: primitive_types::H256) -> Self {
		Self { data: h256.0 }
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.data
	}

	pub fn to_h256(&self) -> primitive_types::H256 {
		primitive_types::H256(self.data)
	}
}
impl Eq for EH256 {}

impl PartialEq for EH256 {
	fn eq(&self, other: &Self) -> bool {
		self.data == other.data
	}
}

pub trait ToEH256 {
	fn to_eh256(self) -> EH256;
}

impl ToEH256 for primitive_types::H256 {
	fn to_eh256(self) -> EH256 {
		EH256::from(self)
	}
}

pub trait ToH256 {
	fn to_h256(self) -> H256;
}
impl ToH256 for EH256 {
	fn to_h256(self) -> H256 {
		H256::from(&self.data)
	}
}

pub trait ManagedVecforEH256<M: VMApi> {
	fn to_managed_buffer(&self) -> ManagedBuffer<M>;
}

impl<M: VMApi> ManagedVecforEH256<M> for EH256 {
	fn to_managed_buffer(&self) -> ManagedBuffer<M> {
		ManagedBuffer::new_from_bytes(&self.data)
	}
}

pub struct ManagedBufferRefIterator<'a, M: VMApi> {
	managed_buffer: &'a ManagedBuffer<M>,
	byte_start: usize,
	byte_end: usize,
}

impl<'a, M> ManagedBufferRefIterator<'a, M>
where
	M: VMApi,
{
	pub(crate) fn new(managed_buffer: &'a ManagedBuffer<M>) -> Self {
		ManagedBufferRefIterator {
			managed_buffer,
			byte_start: 0,
			byte_end: managed_buffer.len(),
		}
	}
}

impl<'a, M> Iterator for ManagedBufferRefIterator<'a, M>
where
	M: VMApi,
{
	type Item = u8;

	fn next(&mut self) -> Option<Self::Item> {
		let next_byte_start = self.byte_start + 1;
		if next_byte_start > self.byte_end {
			return None;
		}

		let result = unsafe {
			u8::from_byte_reader_as_borrow(|dest_slice| {
				let _ = self.managed_buffer.load_slice(self.byte_start, dest_slice);
			})
		};

		self.byte_start = next_byte_start;
		Some(result)
	}
}

pub trait ManagedBufferAccess<M: VMApi> {
	fn push(&mut self, byte: u8);
	fn get(&self, index: usize) -> u8;
	fn try_get(&self, index: usize) -> Option<u8>;
	fn set(&mut self, index: usize, data: u8) -> Result<(), InvalidSliceError>;
	fn resize(&mut self, size: usize, value: u8);
	// fn as_bytes(&self) -> &[u8];
	fn to_vec(&self) -> Vec<u8>;
	fn iter(&self) -> ManagedBufferRefIterator<M>;
}

impl<M: VMApi> ManagedBufferAccess<M> for ManagedBuffer<M> {
	fn push(&mut self, byte: u8) {
		self.append_bytes(&[byte])
	}

	fn get(&self, index: usize) -> u8 {
		match self.try_get(index) {
			Some(result) => result,
			None => M::error_api_impl().signal_error(b"INDEX_OUT_OF_RANGE_MSG"),
		}
	}

	fn try_get(&self, index: usize) -> Option<u8> {
		let mut dest_slice = [0u8; 1];
		let load_result = self.load_slice(index, &mut dest_slice);
		match load_result {
			Result::Ok(_) => Some(dest_slice[0]),
			Result::Err(_) => None,
		}
	}

	fn set(&mut self, index: usize, byte: u8) -> Result<(), InvalidSliceError> {
		self.set_slice(index, &[byte])
	}

	fn resize(&mut self, size: usize, byte: u8) {
		let len = self.len();

		if size > len {
			// extend
			for _ in len..size {
				self.append_bytes(&[byte]);
			}
		} else {
			// truncate
			self.overwrite(&[]);
			for _ in 0..size {
				self.append_bytes(&[byte]);
			}
		}
	}

	// fn as_bytes(&self) -> &[u8] {
	// 	let mut data = Vec::<u8>::new();
	// 	for i in 0..self.len() {
	// 		let item = self.try_get(i).unwrap();
	// 		data.push(item);
	// 	}
	// 	&data
	// }

	// TODO: This needs to be optimized for sure!
	fn to_vec(&self) -> Vec<u8> {
		let mut data = Vec::<u8>::new();
		for i in 0..self.len() {
			let item = self.try_get(i).unwrap();
			data.push(item);
		}
		data
	}

	fn iter(&self) -> ManagedBufferRefIterator<M> {
		ManagedBufferRefIterator::new(self)
	}
}

pub trait Resizable<M: VMApi, T: ManagedVecItem> {
	fn resize(&mut self, size: usize, value: T);
}

impl<M: VMApi, T: ManagedVecItem + Clone> Resizable<M, T> for ManagedVec<M, T> {
	fn resize(&mut self, size: usize, item: T) {
		let len = self.len();

		if size > len {
			// extend
			for _ in len..size {
				self.push(item.clone());
			}
		} else {
			// truncate
			self.clear();
			for _ in 0..size {
				self.push(item.clone());
			}
		}
	}
}
