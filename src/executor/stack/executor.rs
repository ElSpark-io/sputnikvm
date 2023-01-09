use crate::backend::Backend;
use crate::gasometer::{self, Gasometer, StorageTarget};
use crate::{
	Capture, Config, Context, CreateScheme, ExitError, ExitReason, ExitSucceed, Handler, Opcode,
	Runtime, Stack, Transfer,
};
use alloc::{
	collections::{BTreeMap, BTreeSet},
	rc::Rc,
	vec::Vec,
};
use elrond_wasm::api::{CryptoApiImpl, VMApi};
use elrond_wasm::types::ManagedType;
use eltypes::{ManagedBufferAccess, EH256};

use core::{cmp::min, convert::Infallible};
use elrond_wasm::{
	contract_base::ContractBase,
	types::{ManagedBuffer, ManagedVec},
};
use evm_core::{ExitFatal, ExitRevert};
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

macro_rules! emit_exit {
	($reason:expr) => {{
		let reason = $reason;
		event!(Exit {
			reason: &reason,
			return_value: &Vec::new(),
		});
		reason
	}};
	($reason:expr, $return_value:expr) => {{
		let reason = $reason;
		let return_value = $return_value;
		event!(Exit {
			reason: &reason,
			return_value: &return_value,
		});
		(reason, return_value)
	}};
}

pub enum StackExitKind {
	Succeeded,
	Reverted,
	Failed,
}

#[derive(Default, Clone, Debug)]
pub struct Accessed {
	pub accessed_addresses: BTreeSet<H160>,
	pub accessed_storage: BTreeSet<(H160, H256)>,
}

impl Accessed {
	pub fn access_address(&mut self, address: H160) {
		self.accessed_addresses.insert(address);
	}

	pub fn access_addresses<I>(&mut self, addresses: I)
	where
		I: Iterator<Item = H160>,
	{
		for address in addresses {
			self.accessed_addresses.insert(address);
		}
	}

	pub fn access_storages<I>(&mut self, storages: I)
	where
		I: Iterator<Item = (H160, H256)>,
	{
		for storage in storages {
			self.accessed_storage.insert((storage.0, storage.1));
		}
	}
}

#[derive(Clone, Debug)]
pub struct StackSubstateMetadata<'config> {
	gasometer: Gasometer<'config>,
	is_static: bool,
	depth: Option<usize>,
	accessed: Option<Accessed>,
}

impl<'config> StackSubstateMetadata<'config> {
	pub fn new(gas_limit: u64, config: &'config Config) -> Self {
		let accessed = if config.increase_state_access_gas {
			Some(Accessed::default())
		} else {
			None
		};
		Self {
			gasometer: Gasometer::new(gas_limit, config),
			is_static: false,
			depth: None,
			accessed,
		}
	}

	pub fn swallow_commit(&mut self, other: Self) -> Result<(), ExitError> {
		self.gasometer.record_stipend(other.gasometer.gas())?;
		self.gasometer
			.record_refund(other.gasometer.refunded_gas())?;

		if let (Some(mut other_accessed), Some(self_accessed)) =
			(other.accessed, self.accessed.as_mut())
		{
			self_accessed
				.accessed_addresses
				.append(&mut other_accessed.accessed_addresses);
			self_accessed
				.accessed_storage
				.append(&mut other_accessed.accessed_storage);
		}

		Ok(())
	}

	pub fn swallow_revert(&mut self, other: Self) -> Result<(), ExitError> {
		self.gasometer.record_stipend(other.gasometer.gas())?;

		Ok(())
	}

	pub fn swallow_discard(&mut self, _other: Self) -> Result<(), ExitError> {
		Ok(())
	}

	pub fn spit_child(&self, gas_limit: u64, is_static: bool) -> Self {
		Self {
			gasometer: Gasometer::new(gas_limit, self.gasometer.config()),
			is_static: is_static || self.is_static,
			depth: match self.depth {
				None => Some(0),
				Some(n) => Some(n + 1),
			},
			accessed: self.accessed.as_ref().map(|_| Accessed::default()),
		}
	}

	pub fn gasometer(&self) -> &Gasometer<'config> {
		&self.gasometer
	}

	pub fn gasometer_mut(&mut self) -> &mut Gasometer<'config> {
		&mut self.gasometer
	}

	pub fn is_static(&self) -> bool {
		self.is_static
	}

	pub fn depth(&self) -> Option<usize> {
		self.depth
	}

	pub fn access_address(&mut self, address: H160) {
		if let Some(accessed) = &mut self.accessed {
			accessed.access_address(address)
		}
	}

	pub fn access_addresses<I>(&mut self, addresses: I)
	where
		I: Iterator<Item = H160>,
	{
		if let Some(accessed) = &mut self.accessed {
			accessed.access_addresses(addresses);
		}
	}

	pub fn access_storage(&mut self, address: H160, key: H256) {
		if let Some(accessed) = &mut self.accessed {
			accessed.accessed_storage.insert((address, key));
		}
	}

	pub fn access_storages<I>(&mut self, storages: I)
	where
		I: Iterator<Item = (H160, H256)>,
	{
		if let Some(accessed) = &mut self.accessed {
			accessed.access_storages(storages);
		}
	}

	pub fn accessed(&self) -> &Option<Accessed> {
		&self.accessed
	}
}

#[auto_impl::auto_impl(&mut, Box)]
pub trait StackState<'config, M: VMApi>: Backend<M> {
	fn metadata(&self) -> &StackSubstateMetadata<'config>;
	fn metadata_mut(&mut self) -> &mut StackSubstateMetadata<'config>;

	fn enter(&mut self, gas_limit: u64, is_static: bool);
	fn exit_commit(&mut self) -> Result<(), ExitError>;
	fn exit_revert(&mut self) -> Result<(), ExitError>;
	fn exit_discard(&mut self) -> Result<(), ExitError>;

	fn is_empty(&self, address: H160) -> bool;
	fn deleted(&self, address: H160) -> bool;
	fn is_cold(&self, address: H160) -> bool;
	fn is_storage_cold(&self, address: H160, key: H256) -> bool;

	fn inc_nonce(&mut self, address: H160);
	fn set_storage(&mut self, address: H160, key: H256, value: H256);
	fn reset_storage(&mut self, address: H160);
	fn log(&mut self, address: H160, topics: ManagedVec<M, EH256>, data: ManagedBuffer<M>);
	fn set_deleted(&mut self, address: H160);
	fn set_code(&mut self, address: H160, code: ManagedBuffer<M>);
	fn transfer(&mut self, transfer: Transfer) -> Result<(), ExitError>;
	fn reset_balance(&mut self, address: H160);
	fn touch(&mut self, address: H160);

	/// Fetch the code size of an address.
	/// Provide a default implementation by fetching the code, but
	/// can be customized to use a more performant approach that don't need to
	/// fetch the code.
	fn code_size(&self, address: H160) -> U256 {
		U256::from(self.code(address).len())
	}

	/// Fetch the code hash of an address.
	/// Provide a default implementation by fetching the code, but
	/// can be customized to use a more performant approach that don't need to
	/// fetch the code.
	fn code_hash(&self, address: H160) -> H256 {
		let ret: ManagedBuffer<M> = ManagedBuffer::new();
		M::crypto_api_impl().keccak256_managed(ret.get_handle(), self.code(address).get_handle());
		H256::from_slice(&ret.to_vec())
	}
}

/// Data returned by a precompile on success.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PrecompileOutput<M: VMApi> {
	pub exit_status: ExitSucceed,
	pub output: ManagedBuffer<M>,
}

/// Data returned by a precompile in case of failure.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PrecompileFailure<M: VMApi> {
	/// Reverts the state changes and consume all the gas.
	Error { exit_status: ExitError },
	/// Reverts the state changes.
	/// Returns the provided error message.
	Revert {
		exit_status: ExitRevert,
		output: ManagedBuffer<M>,
	},
	/// Mark this failure as fatal, and all EVM execution stacks must be exited.
	Fatal { exit_status: ExitFatal },
}

impl<M: VMApi> From<ExitError> for PrecompileFailure<M> {
	fn from(error: ExitError) -> PrecompileFailure<M> {
		PrecompileFailure::Error { exit_status: error }
	}
}

/// Handle provided to a precompile to interact with the EVM.
pub trait PrecompileHandle<M: VMApi> {
	/// Perform subcall in provided context.
	/// Precompile specifies in which context the subcall is executed.
	fn call(
		&mut self,
		to: H160,
		transfer: Option<Transfer>,
		input: ManagedBuffer<M>,
		gas_limit: Option<u64>,
		is_static: bool,
		context: &Context,
	) -> (ExitReason, ManagedBuffer<M>);

	/// Record cost to the Runtime gasometer.
	fn record_cost(&mut self, cost: u64) -> Result<(), ExitError>;

	/// Retreive the remaining gas.
	fn remaining_gas(&self) -> u64;

	/// Record a log.
	fn log(
		&mut self,
		address: H160,
		topics: ManagedVec<M, EH256>,
		data: ManagedBuffer<M>,
	) -> Result<(), ExitError>;

	/// Retreive the code address (what is the address of the precompile being called).
	fn code_address(&self) -> H160;

	/// Retreive the input data the precompile is called with.
	fn input(&self) -> &ManagedBuffer<M>;

	/// Retreive the context in which the precompile is executed.
	fn context(&self) -> &Context;

	/// Is the precompile call is done statically.
	fn is_static(&self) -> bool;

	/// Retreive the gas limit of this call.
	fn gas_limit(&self) -> Option<u64>;
}

/// A precompile result.
pub type PrecompileResult<M> = Result<PrecompileOutput<M>, PrecompileFailure<M>>;

/// A set of precompiles.
/// Checks of the provided address being in the precompile set should be
/// as cheap as possible since it may be called often.
pub trait PrecompileSet<M: VMApi> {
	/// Tries to execute a precompile in the precompile set.
	/// If the provided address is not a precompile, returns None.
	fn execute(&self, handle: &mut impl PrecompileHandle<M>) -> Option<PrecompileResult<M>>;

	/// Check if the given address is a precompile. Should only be called to
	/// perform the check while not executing the precompile afterward, since
	/// `execute` already performs a check internally.
	fn is_precompile(&self, address: H160) -> bool;
}

impl<M: VMApi> PrecompileSet<M> for () {
	fn execute(&self, _: &mut impl PrecompileHandle<M>) -> Option<PrecompileResult<M>> {
		None
	}

	fn is_precompile(&self, _: H160) -> bool {
		false
	}
}

/// Precompiles function signature. Expected input arguments are:
///  * Input
///  * Gas limit
///  * Context
///  * Is static
///
/// In case of success returns the output and the cost.
pub type PrecompileFn<M> = fn(
	&ManagedBuffer<M>,
	Option<u64>,
	&Context,
	bool,
) -> Result<(PrecompileOutput<M>, u64), PrecompileFailure<M>>;

impl<M: VMApi> PrecompileSet<M> for BTreeMap<H160, PrecompileFn<M>> {
	fn execute(&self, handle: &mut impl PrecompileHandle<M>) -> Option<PrecompileResult<M>> {
		let address = handle.code_address();

		self.get(&address).map(|precompile| {
			let input = handle.input();
			let gas_limit = handle.gas_limit();
			let context = handle.context();
			let is_static = handle.is_static();

			match (*precompile)(input, gas_limit, context, is_static) {
				Ok((output, cost)) => {
					handle.record_cost(cost)?;
					Ok(output)
				}
				Err(err) => Err(err),
			}
		})
	}

	/// Check if the given address is a precompile. Should only be called to
	/// perform the check while not executing the precompile afterward, since
	/// `execute` already performs a check internally.
	fn is_precompile(&self, address: H160) -> bool {
		self.contains_key(&address)
	}
}

/// Stack-based executor.
pub struct StackExecutor<'config, 'precompiles, M: VMApi, S, P> {
	config: &'config Config,
	state: S,
	precompile_set: &'precompiles P,
	//TODO: remove vec_test
	vec_test: ManagedBuffer<M>,
}

impl<'config, 'precompiles, S: StackState<'config, M>, P: PrecompileSet<M>, M: VMApi>
	StackExecutor<'config, 'precompiles, M, S, P>
{
	/// Return a reference of the Config.
	pub fn config(&self) -> &'config Config {
		self.config
	}

	/// Return a reference to the precompile set.
	pub fn precompiles(&self) -> &'precompiles P {
		self.precompile_set
	}

	/// Create a new stack-based executor with given precompiles.
	pub fn new_with_precompiles(
		state: S,
		config: &'config Config,
		precompile_set: &'precompiles P,
	) -> Self {
		Self {
			config,
			state,
			precompile_set,
			vec_test: ManagedBuffer::new(),
		}
	}

	pub fn state(&self) -> &S {
		&self.state
	}

	pub fn state_mut(&mut self) -> &mut S {
		&mut self.state
	}

	pub fn into_state(self) -> S {
		self.state
	}

	/// Create a substate executor from the current executor.
	pub fn enter_substate(&mut self, gas_limit: u64, is_static: bool) {
		self.state.enter(gas_limit, is_static);
	}

	/// Exit a substate. Panic if it results an empty substate stack.
	pub fn exit_substate(&mut self, kind: StackExitKind) -> Result<(), ExitError> {
		match kind {
			StackExitKind::Succeeded => self.state.exit_commit(),
			StackExitKind::Reverted => self.state.exit_revert(),
			StackExitKind::Failed => self.state.exit_discard(),
		}
	}

	/// Execute the runtime until it returns.
	pub fn execute(&mut self, runtime: &mut Runtime<'config, M>) -> ExitReason {
		let x = runtime.run(self);
		match x {
			Capture::Exit(s) => s,
			Capture::Trap(_) => unreachable!("Trap is Infallible"),
		}
	}

	/// Get remaining gas.
	pub fn gas(&self) -> u64 {
		self.state.metadata().gasometer.gas()
	}

	fn record_create_transaction_cost(
		&mut self,
		init_code: &ManagedBuffer<M>,
		access_list: &[(H160, Vec<H256>)],
	) -> Result<(), ExitError> {
		let transaction_cost = gasometer::create_transaction_cost(&init_code, access_list);
		let gasometer = &mut self.state.metadata_mut().gasometer;
		gasometer.record_transaction(transaction_cost)
	}

	/// Execute a `CREATE` transaction.
	pub fn transact_create(
		&mut self,
		caller: H160,
		value: U256,
		init_code: ManagedBuffer<M>,
		gas_limit: u64,
		access_list: &[(H160, Vec<H256>)],
	) -> (ExitReason, ManagedBuffer<M>) {
		event!(TransactCreate {
			caller,
			value,
			init_code: &init_code,
			gas_limit,
			address: self.create_address(CreateScheme::Legacy { caller }),
		});

		if let Err(e) = self.record_create_transaction_cost(&init_code, &access_list) {
			return emit_exit!(e.into(), ManagedBuffer::new());
		}
		self.initialize_with_access_list(access_list);

		match self.create_inner(
			caller,
			CreateScheme::Legacy { caller },
			value,
			&init_code,
			Some(gas_limit),
			false,
		) {
			Capture::Exit((s, _, v)) => emit_exit!(s, v),
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Execute a `CREATE2` transaction.
	pub fn transact_create2(
		&mut self,
		caller: H160,
		value: U256,
		init_code: &ManagedBuffer<M>,
		salt: H256,
		gas_limit: u64,
		access_list: &[(H160, Vec<H256>)],
	) -> (ExitReason, ManagedBuffer<M>) {
		let ret: ManagedBuffer<M> = ManagedBuffer::new();
		M::crypto_api_impl().keccak256_managed(ret.get_handle(), init_code.get_handle());
		let code_hash = H256::from_slice(&ret.to_vec());
		event!(TransactCreate2 {
			caller,
			value,
			init_code: &init_code,
			salt,
			gas_limit,
			address: self.create_address(CreateScheme::Create2 {
				caller,
				code_hash,
				salt,
			}),
		});

		if let Err(e) = self.record_create_transaction_cost(&init_code, &access_list) {
			return emit_exit!(e.into(), ManagedBuffer::new());
		}
		self.initialize_with_access_list(access_list);

		match self.create_inner(
			caller,
			CreateScheme::Create2 {
				caller,
				code_hash,
				salt,
			},
			value,
			init_code.into(),
			Some(gas_limit),
			false,
		) {
			Capture::Exit((s, _, v)) => emit_exit!(s, v),
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Execute a `CALL` transaction with a given caller, address, value and
	/// gas limit and data.
	///
	/// Takes in an additional `access_list` parameter for EIP-2930 which was
	/// introduced in the Ethereum Berlin hard fork. If you do not wish to use
	/// this functionality, just pass in an empty vector.
	pub fn transact_call(
		&mut self,
		caller: H160,
		address: H160,
		value: U256,
		data: ManagedBuffer<M>,
		gas_limit: u64,
		access_list: &[(H160, Vec<H256>)],
	) -> (ExitReason, ManagedBuffer<M>) {
		event!(TransactCall {
			caller,
			address,
			value,
			data: &data,
			gas_limit,
		});

		let transaction_cost = gasometer::call_transaction_cost(&data, &access_list);
		let gasometer = &mut self.state.metadata_mut().gasometer;
		match gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return emit_exit!(e.into(), ManagedBuffer::new()),
		}

		// Initialize initial addresses for EIP-2929
		if self.config.increase_state_access_gas {
			let addresses = core::iter::once(caller).chain(core::iter::once(address));
			self.state.metadata_mut().access_addresses(addresses);

			self.initialize_with_access_list(access_list);
		}

		self.state.inc_nonce(caller);

		let context = Context {
			caller,
			address,
			apparent_value: value,
		};

		match self.call_inner(
			address,
			Some(Transfer {
				source: caller,
				target: address,
				value,
			}),
			data,
			Some(gas_limit),
			false,
			false,
			false,
			context,
		) {
			Capture::Exit((s, v)) => emit_exit!(s, v),
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Get used gas for the current executor, given the price.
	pub fn used_gas(&self) -> u64 {
		self.state.metadata().gasometer.total_used_gas()
			- min(
				self.state.metadata().gasometer.total_used_gas() / self.config.max_refund_quotient,
				self.state.metadata().gasometer.refunded_gas() as u64,
			)
	}

	/// Get fee needed for the current executor, given the price.
	pub fn fee(&self, price: U256) -> U256 {
		let used_gas = self.used_gas();
		U256::from(used_gas).saturating_mul(price)
	}

	/// Get account nonce.
	pub fn nonce(&self, address: H160) -> U256 {
		self.state.basic(address).nonce
	}

	/// Get the create address from given scheme.
	pub fn create_address(&self, scheme: CreateScheme) -> H160 {
		match scheme {
			CreateScheme::Create2 {
				caller,
				code_hash,
				salt,
			} => {
				let mut hasher = Keccak256::new();
				hasher.update(&[0xff]);
				hasher.update(&caller[..]);
				hasher.update(&salt[..]);
				hasher.update(&code_hash[..]);
				H256::from_slice(hasher.finalize().as_slice()).into()
			}
			CreateScheme::Legacy { caller } => {
				let nonce = self.nonce(caller);

				let nonce_len = (nonce.bits() as u8) / 8 + 1;

				let mut len = 22 + nonce_len;
				if nonce >= U256::from(128) {
					len += 1;
				}

				let mut data = Vec::<u8>::with_capacity(len as usize);

				data.push(192 + len - 1);
				data.push(148);
				data.append(&mut caller.0.to_vec());

				if nonce < U256::from(128) {
					data.push(nonce.byte(0));
				} else {
					data.push(128 + nonce_len);

					for i in 0..nonce_len as usize {
						let b = nonce.byte(i);
						if b == 0 {
							data.push(128);
						} else {
							data.push(b);
						}
					}
				}
				H256::from_slice(M::crypto_api_impl().keccak256_legacy(&data).as_slice()).into()
			}
			CreateScheme::Fixed(naddress) => naddress,
		}
	}

	pub fn initialize_with_access_list(&mut self, access_list: &[(H160, Vec<H256>)]) {
		let addresses = access_list.iter().map(|a| a.0);
		self.state.metadata_mut().access_addresses(addresses);

		let storage_keys = access_list.into_iter().flat_map(|(address, keys)| {
			keys.into_iter()
				.map(move |key| (address.clone(), key.clone()))
		});
		self.state.metadata_mut().access_storages(storage_keys);
	}

	fn create_inner(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: &ManagedBuffer<M>,
		target_gas: Option<u64>,
		take_l64: bool,
	) -> Capture<(ExitReason, Option<H160>, ManagedBuffer<M>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), None, ManagedBuffer::new())),
				}
			};
		}

		fn check_first_byte<M: VMApi>(
			config: &Config,
			code: &ManagedBuffer<M>,
		) -> Result<(), ExitError> {
			// TODO: Check to be sure
			// if config.disallow_executable_format && Some(&Opcode::EOFMAGIC.as_u8()) == code.get(0)
			if config.disallow_executable_format && Opcode::EOFMAGIC.as_u8() == code.get(0) {
				return Err(ExitError::InvalidCode(Opcode::EOFMAGIC));
			}
			Ok(())
		}

		fn l64(gas: u64) -> u64 {
			gas - gas / 64
		}

		let address = self.create_address(scheme);

		self.state.metadata_mut().access_address(caller);
		self.state.metadata_mut().access_address(address);

		event!(Create {
			caller,
			address,
			scheme,
			value,
			init_code: &init_code,
			target_gas
		});

		if let Some(depth) = self.state.metadata().depth {
			if depth > self.config.call_stack_limit {
				return Capture::Exit((ExitError::CallTooDeep.into(), None, ManagedBuffer::new()));
			}
		}

		if self.balance(caller) < value {
			return Capture::Exit((ExitError::OutOfFund.into(), None, ManagedBuffer::new()));
		}

		let after_gas = if take_l64 && self.config.call_l64_after_gas {
			if self.config.estimate {
				let initial_after_gas = self.state.metadata().gasometer.gas();
				let diff = initial_after_gas - l64(initial_after_gas);
				try_or_fail!(self.state.metadata_mut().gasometer.record_cost(diff));
				self.state.metadata().gasometer.gas()
			} else {
				l64(self.state.metadata().gasometer.gas())
			}
		} else {
			self.state.metadata().gasometer.gas()
		};

		let target_gas = target_gas.unwrap_or(after_gas);

		let gas_limit = min(after_gas, target_gas);
		try_or_fail!(self.state.metadata_mut().gasometer.record_cost(gas_limit));

		self.state.inc_nonce(caller);

		self.enter_substate(gas_limit, false);

		{
			if self.code_size(address) != U256::zero() {
				let _ = self.exit_substate(StackExitKind::Failed);
				return Capture::Exit((
					ExitError::CreateCollision.into(),
					None,
					ManagedBuffer::new(),
				));
			}

			if self.nonce(address) > U256::zero() {
				let _ = self.exit_substate(StackExitKind::Failed);
				return Capture::Exit((
					ExitError::CreateCollision.into(),
					None,
					ManagedBuffer::new(),
				));
			}

			self.state.reset_storage(address);
		}

		let context = Context {
			address,
			caller,
			apparent_value: value,
		};
		let transfer = Transfer {
			source: caller,
			target: address,
			value,
		};
		match self.state.transfer(transfer) {
			Ok(()) => (),
			Err(e) => {
				let _ = self.exit_substate(StackExitKind::Reverted);
				return Capture::Exit((ExitReason::Error(e), None, ManagedBuffer::new()));
			}
		}

		if self.config.create_increase_nonce {
			self.state.inc_nonce(address);
		}

		let mut runtime = Runtime::new(
			Rc::new(init_code.clone()),
			Rc::new(ManagedBuffer::new()),
			context,
			self.config,
		);

		let reason = self.execute(&mut runtime);
		log::debug!(target: "evm", "Create execution using address {}: {:?}", address, reason);

		match reason {
			ExitReason::Succeed(s) => {
				let out = runtime.machine().return_value();

				// As of EIP-3541 code starting with 0xef cannot be deployed
				if let Err(e) = check_first_byte(self.config, &out) {
					self.state.metadata_mut().gasometer.fail();
					let _ = self.exit_substate(StackExitKind::Failed);
					return Capture::Exit((e.into(), None, ManagedBuffer::new()));
				}

				if let Some(limit) = self.config.create_contract_limit {
					if out.len() > limit {
						self.state.metadata_mut().gasometer.fail();
						let _ = self.exit_substate(StackExitKind::Failed);
						return Capture::Exit((
							ExitError::CreateContractLimit.into(),
							None,
							ManagedBuffer::new(),
						));
					}
				}

				match self
					.state
					.metadata_mut()
					.gasometer
					.record_deposit(out.len())
				{
					Ok(()) => {
						let e = self.exit_substate(StackExitKind::Succeeded);
						self.state.set_code(address, out);
						try_or_fail!(e);
						Capture::Exit((ExitReason::Succeed(s), Some(address), ManagedBuffer::new()))
					}
					Err(e) => {
						let _ = self.exit_substate(StackExitKind::Failed);
						Capture::Exit((ExitReason::Error(e), None, ManagedBuffer::new()))
					}
				}
			}
			ExitReason::Error(e) => {
				self.state.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Error(e), None, ManagedBuffer::new()))
			}
			ExitReason::Revert(e) => {
				let _ = self.exit_substate(StackExitKind::Reverted);
				Capture::Exit((
					ExitReason::Revert(e),
					None,
					runtime.machine().return_value(),
				))
			}
			ExitReason::Fatal(e) => {
				self.state.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Fatal(e), None, ManagedBuffer::new()))
			}
		}
		// Capture::Exit((
		// 	ExitReason::Succeed(ExitSucceed::Returned),
		// 	None,
		// 	ManagedBuffer::new(),
		// ))
	}

	#[allow(clippy::too_many_arguments)]
	fn call_inner(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: ManagedBuffer<M>,
		target_gas: Option<u64>,
		is_static: bool,
		take_l64: bool,
		take_stipend: bool,
		context: Context,
	) -> Capture<(ExitReason, ManagedBuffer<M>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), ManagedBuffer::new())),
				}
			};
		}

		fn l64(gas: u64) -> u64 {
			gas - gas / 64
		}

		event!(Call {
			code_address,
			transfer: &transfer,
			input: &input,
			target_gas,
			is_static,
			context: &context,
		});

		let after_gas = if take_l64 && self.config.call_l64_after_gas {
			if self.config.estimate {
				let initial_after_gas = self.state.metadata().gasometer.gas();
				let diff = initial_after_gas - l64(initial_after_gas);
				try_or_fail!(self.state.metadata_mut().gasometer.record_cost(diff));
				self.state.metadata().gasometer.gas()
			} else {
				l64(self.state.metadata().gasometer.gas())
			}
		} else {
			self.state.metadata().gasometer.gas()
		};

		let target_gas = target_gas.unwrap_or(after_gas);
		let mut gas_limit = min(target_gas, after_gas);

		try_or_fail!(self.state.metadata_mut().gasometer.record_cost(gas_limit));

		if let Some(transfer) = transfer.as_ref() {
			if take_stipend && transfer.value != U256::zero() {
				gas_limit = gas_limit.saturating_add(self.config.call_stipend);
			}
		}

		let code = self.code(code_address);

		self.enter_substate(gas_limit, is_static);
		self.state.touch(context.address);

		if let Some(depth) = self.state.metadata().depth {
			if depth > self.config.call_stack_limit {
				let _ = self.exit_substate(StackExitKind::Reverted);
				return Capture::Exit((ExitError::CallTooDeep.into(), ManagedBuffer::new()));
			}
		}

		if let Some(transfer) = transfer {
			match self.state.transfer(transfer) {
				Ok(()) => (),
				Err(e) => {
					let _ = self.exit_substate(StackExitKind::Reverted);
					return Capture::Exit((ExitReason::Error(e), ManagedBuffer::new()));
				}
			}
		}

		// At this point, the state has been modified in enter_substate to
		// reflect both the is_static parameter of this call and the is_static
		// of the caller context.
		let precompile_is_static = self.state.metadata().is_static();
		if let Some(result) = self.precompile_set.execute(&mut StackExecutorHandle {
			executor: self,
			code_address,
			input: &input,
			gas_limit: Some(gas_limit),
			context: &context,
			is_static: precompile_is_static,
		}) {
			return match result {
				Ok(PrecompileOutput {
					exit_status,
					output,
				}) => {
					let _ = self.exit_substate(StackExitKind::Succeeded);
					Capture::Exit((ExitReason::Succeed(exit_status), output))
				}
				Err(PrecompileFailure::Error { exit_status }) => {
					let _ = self.exit_substate(StackExitKind::Failed);
					Capture::Exit((ExitReason::Error(exit_status), ManagedBuffer::new()))
				}
				Err(PrecompileFailure::Revert {
					exit_status,
					output,
				}) => {
					let _ = self.exit_substate(StackExitKind::Reverted);
					Capture::Exit((ExitReason::Revert(exit_status), output))
				}
				Err(PrecompileFailure::Fatal { exit_status }) => {
					self.state.metadata_mut().gasometer.fail();
					let _ = self.exit_substate(StackExitKind::Failed);
					Capture::Exit((ExitReason::Fatal(exit_status), ManagedBuffer::new()))
				}
			};
		}

		let mut runtime = Runtime::new(Rc::new(code), Rc::new(input), context, self.config);

		let reason = self.execute(&mut runtime);
		log::debug!(target: "evm", "Call execution using address {}: {:?}", code_address, reason);

		match reason {
			ExitReason::Succeed(s) => {
				let _ = self.exit_substate(StackExitKind::Succeeded);
				Capture::Exit((ExitReason::Succeed(s), runtime.machine().return_value()))
			}
			ExitReason::Error(e) => {
				let _ = self.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Error(e), ManagedBuffer::new()))
			}
			ExitReason::Revert(e) => {
				let _ = self.exit_substate(StackExitKind::Reverted);
				Capture::Exit((ExitReason::Revert(e), runtime.machine().return_value()))
			}
			ExitReason::Fatal(e) => {
				self.state.metadata_mut().gasometer.fail();
				let _ = self.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Fatal(e), ManagedBuffer::new()))
			}
		}
	}
}

impl<'config, 'precompiles, S: StackState<'config, M>, P: PrecompileSet<M>, M: VMApi> Handler<M>
	for StackExecutor<'config, 'precompiles, M, S, P>
{
	type CreateInterrupt = Infallible;
	type CreateFeedback = Infallible;
	type CallInterrupt = Infallible;
	type CallFeedback = Infallible;

	fn balance(&self, address: H160) -> U256 {
		self.state.basic(address).balance
	}

	fn code_size(&self, address: H160) -> U256 {
		self.state.code_size(address)
	}

	fn code_hash(&self, address: H160) -> H256 {
		if !self.exists(address) {
			return H256::default();
		}

		self.state.code_hash(address)
	}

	fn code(&self, address: H160) -> ManagedBuffer<M> {
		self.state.code(address).into()
	}

	fn storage(&self, address: H160, index: EH256) -> H256 {
		self.state.storage(address, index.to_h256())
	}

	fn original_storage(&self, address: H160, index: H256) -> H256 {
		self.state
			.original_storage(address, index)
			.unwrap_or_default()
	}

	fn exists(&self, address: H160) -> bool {
		if self.config.empty_considered_exists {
			self.state.exists(address)
		} else {
			self.state.exists(address) && !self.state.is_empty(address)
		}
	}

	fn is_cold(&self, address: H160, maybe_index: Option<H256>) -> bool {
		match maybe_index {
			None => !self.precompile_set.is_precompile(address) && self.state.is_cold(address),
			Some(index) => self.state.is_storage_cold(address, index),
		}
	}

	fn gas_left(&self) -> U256 {
		U256::from(self.state.metadata().gasometer.gas())
	}

	fn gas_price(&self) -> U256 {
		self.state.gas_price()
	}
	fn origin(&self) -> H160 {
		self.state.origin()
	}
	fn block_hash(&self, number: U256) -> H256 {
		self.state.block_hash(number)
	}
	fn block_number(&self) -> U256 {
		self.state.block_number()
	}
	fn block_coinbase(&self) -> H160 {
		self.state.block_coinbase()
	}
	fn block_timestamp(&self) -> U256 {
		self.state.block_timestamp()
	}
	fn block_difficulty(&self) -> U256 {
		self.state.block_difficulty()
	}
	fn block_gas_limit(&self) -> U256 {
		self.state.block_gas_limit()
	}
	fn block_base_fee_per_gas(&self) -> U256 {
		self.state.block_base_fee_per_gas()
	}
	fn chain_id(&self) -> U256 {
		self.state.chain_id()
	}

	fn deleted(&self, address: H160) -> bool {
		self.state.deleted(address)
	}

	fn set_storage(&mut self, address: H160, index: EH256, value: EH256) -> Result<(), ExitError> {
		self.state
			.set_storage(address, index.to_h256(), value.to_h256());
		Ok(())
	}

	fn log(
		&mut self,
		address: H160,
		topics: ManagedVec<M, EH256>,
		data: ManagedBuffer<M>,
	) -> Result<(), ExitError> {
		self.state.log(address, topics, data);
		Ok(())
	}

	fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError> {
		let balance = self.balance(address);

		event!(Suicide {
			target,
			address,
			balance,
		});

		self.state.transfer(Transfer {
			source: address,
			target,
			value: balance,
		})?;
		self.state.reset_balance(address);
		self.state.set_deleted(address);

		Ok(())
	}

	#[cfg(not(feature = "tracing"))]
	fn create(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: ManagedBuffer<M>,
		target_gas: Option<u64>,
	) -> Capture<(ExitReason, Option<H160>, ManagedBuffer<M>), Self::CreateInterrupt> {
		self.create_inner(caller, scheme, value, &init_code, target_gas, true)
	}

	#[cfg(feature = "tracing")]
	fn create(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: ManagedBuffer<M>,
		target_gas: Option<u64>,
	) -> Capture<(ExitReason, Option<H160>, ManagedBuffer<M>), Self::CreateInterrupt> {
		let capture = self.create_inner(caller, scheme, value, init_code, target_gas, true);

		if let Capture::Exit((ref reason, _, ref return_value)) = capture {
			emit_exit!(reason, return_value);
		}

		capture
	}

	#[cfg(not(feature = "tracing"))]
	fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: ManagedBuffer<M>,
		target_gas: Option<u64>,
		is_static: bool,
		context: Context,
	) -> Capture<(ExitReason, ManagedBuffer<M>), Self::CallInterrupt> {
		self.call_inner(
			code_address,
			transfer,
			input,
			target_gas,
			is_static,
			true,
			true,
			context,
		)
	}

	#[cfg(feature = "tracing")]
	fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: ManagedBuffer<M>,
		target_gas: Option<u64>,
		is_static: bool,
		context: Context,
	) -> Capture<(ExitReason, ManagedBuffer<M>), Self::CallInterrupt> {
		let capture = self.call_inner(
			code_address,
			transfer,
			input,
			target_gas,
			is_static,
			true,
			true,
			context,
		);

		if let Capture::Exit((ref reason, ref return_value)) = capture {
			emit_exit!(reason, return_value);
		}

		capture
	}

	#[inline]
	fn pre_validate(
		&mut self,
		context: &Context,
		opcode: Opcode,
		stack: &Stack<M>,
	) -> Result<(), ExitError> {
		// log::trace!(target: "evm", "Running opcode: {:?}, Pre gas-left: {:?}", opcode, gasometer.gas());

		if let Some(cost) = gasometer::static_opcode_cost(opcode) {
			self.state.metadata_mut().gasometer.record_cost(cost)?;
		} else {
			let is_static = self.state.metadata().is_static;
			let (gas_cost, target, memory_cost) = gasometer::dynamic_opcode_cost(
				context.address,
				opcode,
				stack,
				is_static,
				self.config,
				self,
			)?;

			let gasometer = &mut self.state.metadata_mut().gasometer;

			gasometer.record_dynamic_cost(gas_cost, memory_cost)?;
			match target {
				StorageTarget::Address(address) => {
					self.state.metadata_mut().access_address(address)
				}
				StorageTarget::Slot(address, key) => {
					self.state.metadata_mut().access_storage(address, key)
				}
				StorageTarget::None => (),
			}
		}

		Ok(())
	}
}

struct StackExecutorHandle<'inner, 'config, 'precompiles, M: VMApi, S, P> {
	executor: &'inner mut StackExecutor<'config, 'precompiles, M, S, P>,
	code_address: H160,
	input: &'inner ManagedBuffer<M>,
	gas_limit: Option<u64>,
	context: &'inner Context,
	is_static: bool,
}

impl<'inner, 'config, 'precompiles, S: StackState<'config, M>, P: PrecompileSet<M>, M: VMApi>
	PrecompileHandle<M> for StackExecutorHandle<'inner, 'config, 'precompiles, M, S, P>
{
	// Perform subcall in provided context.
	/// Precompile specifies in which context the subcall is executed.
	fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: ManagedBuffer<M>,
		gas_limit: Option<u64>,
		is_static: bool,
		context: &Context,
	) -> (ExitReason, ManagedBuffer<M>) {
		// For normal calls the cost is recorded at opcode level.
		// Since we don't go through opcodes we need manually record the call
		// cost. Not doing so will make the code panic as recording the call stipend
		// will do an underflow.
		let gas_cost = crate::gasometer::GasCost::Call {
			value: transfer.clone().map(|x| x.value).unwrap_or_else(U256::zero),
			gas: U256::from(gas_limit.unwrap_or(u64::MAX)),
			target_is_cold: self.executor.is_cold(code_address, None),
			target_exists: self.executor.exists(code_address),
		};

		// We record the length of the input.
		let memory_cost = Some(crate::gasometer::MemoryCost {
			offset: U256::zero(),
			len: input.len().into(),
		});

		if let Err(error) = self
			.executor
			.state
			.metadata_mut()
			.gasometer
			.record_dynamic_cost(gas_cost, memory_cost)
		{
			return (ExitReason::Error(error), ManagedBuffer::new());
		}

		event!(PrecompileSubcall {
			code_address,
			transfer: &transfer,
			input: &input,
			target_gas: gas_limit,
			is_static,
			context
		});

		// Perform the subcall
		match Handler::call(
			self.executor,
			code_address,
			transfer,
			input,
			gas_limit,
			is_static,
			context.clone(),
		) {
			Capture::Exit((s, v)) => (s, v),
			Capture::Trap(_) => unreachable!("Trap is infaillible since StackExecutor is sync"),
		}
	}

	/// Record cost to the Runtime gasometer.
	fn record_cost(&mut self, cost: u64) -> Result<(), ExitError> {
		self.executor
			.state
			.metadata_mut()
			.gasometer
			.record_cost(cost)
	}

	/// Retreive the remaining gas.
	fn remaining_gas(&self) -> u64 {
		self.executor.state.metadata().gasometer.gas()
	}

	/// Record a log.
	fn log(
		&mut self,
		address: H160,
		topics: ManagedVec<M, EH256>,
		data: ManagedBuffer<M>,
	) -> Result<(), ExitError> {
		Handler::log(self.executor, address, topics, data)
	}

	/// Retreive the code address (what is the address of the precompile being called).
	fn code_address(&self) -> H160 {
		self.code_address
	}

	/// Retreive the input data the precompile is called with.
	fn input(&self) -> &ManagedBuffer<M> {
		self.input
	}

	/// Retreive the context in which the precompile is executed.
	fn context(&self) -> &Context {
		self.context
	}

	/// Is the precompile call is done statically.
	fn is_static(&self) -> bool {
		self.is_static
	}

	/// Retreive the gas limit of this call.
	fn gas_limit(&self) -> Option<u64> {
		self.gas_limit
	}
}
