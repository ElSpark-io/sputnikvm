use crate::ExitError;
use eltypes::EH256;
use mx_sc::{api::VMApi, contract_base::ContractBase, types::ManagedVec};

/// EVM stack.
#[derive(Clone, Debug)]
pub struct Stack<M: VMApi> {
	data: ManagedVec<M, EH256>,
	limit: usize,
}

impl<M: VMApi> Stack<M> {
	/// Create a new stack with given limit.
	pub fn new(limit: usize) -> Self {
		Self {
			data: ManagedVec::new(),
			limit,
		}
	}

	#[inline]
	/// Stack limit.
	pub fn limit(&self) -> usize {
		self.limit
	}

	#[inline]
	/// Stack length.
	pub fn len(&self) -> usize {
		self.data.len()
	}

	#[inline]
	/// Whether the stack is empty.
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}

	#[inline]
	/// Stack data.
	pub fn data(&self) -> &ManagedVec<M, EH256> {
		&self.data
	}

	#[inline]
	/// Pop a value from the stack. If the stack is already empty, returns the
	/// `StackUnderflow` error.
	pub fn pop(&mut self) -> Result<EH256, ExitError> {
		let last_index = self.data().len() - 1;
		if last_index <= 0 {
			ExitError::StackUnderflow;
		}
		let remove = self.data().get(last_index);
		self.data.remove(last_index);
		Ok(remove)
	}

	#[inline]
	/// Push a new value into the stack. If it will exceed the stack limit,
	/// returns `StackOverflow` error and leaves the stack unchanged.
	pub fn push(&mut self, value: EH256) -> Result<(), ExitError> {
		if self.data.len() + 1 > self.limit {
			return Err(ExitError::StackOverflow);
		}
		self.data.push(value);
		Ok(())
	}

	#[inline]
	/// Peek a value at given index for the stack, where the top of
	/// the stack is at index `0`. If the index is too large,
	/// `StackError::Underflow` is returned.
	pub fn peek(&self, no_from_top: usize) -> Result<EH256, ExitError> {
		if self.data.len() > no_from_top {
			Ok(self.data.get(self.data.len() - no_from_top - 1))
		} else {
			Err(ExitError::StackUnderflow)
		}
	}

	#[inline]
	/// Set a value at given index for the stack, where the top of the
	/// stack is at index `0`. If the index is too large,
	/// `StackError::Underflow` is returned.
	pub fn set(&mut self, no_from_top: usize, val: EH256) -> Result<(), ExitError> {
		if self.data.len() > no_from_top {
			let len = self.data.len();
			self.data.set(len - no_from_top - 1, &val);
			Ok(())
		} else {
			Err(ExitError::StackUnderflow)
		}
	}
}
