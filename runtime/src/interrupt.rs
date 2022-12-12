use crate::{ExitFatal, Handler, Runtime};
use elrond_wasm::api::VMApi;
/// Interrupt resolution.
pub enum Resolve<'a, 'config, M: VMApi, H: Handler<M>> {
	/// Create interrupt resolution.
	Create(H::CreateInterrupt, ResolveCreate<'a, 'config, M>),
	/// Call interrupt resolution.
	Call(H::CallInterrupt, ResolveCall<'a, 'config, M>),
}

/// Create interrupt resolution.
pub struct ResolveCreate<'a, 'config, M: VMApi> {
	runtime: &'a mut Runtime<'config, M>,
}

impl<'a, 'config, M: VMApi> ResolveCreate<'a, 'config, M> {
	pub(crate) fn new(runtime: &'a mut Runtime<'config, M>) -> Self {
		Self { runtime }
	}
}

impl<'a, 'config, M: VMApi> Drop for ResolveCreate<'a, 'config, M> {
	fn drop(&mut self) {
		self.runtime.status = Err(ExitFatal::UnhandledInterrupt.into());
		self.runtime
			.machine
			.exit(ExitFatal::UnhandledInterrupt.into());
	}
}

/// Call interrupt resolution.
pub struct ResolveCall<'a, 'config, M: VMApi> {
	runtime: &'a mut Runtime<'config, M>,
}

impl<'a, 'config, M: VMApi> ResolveCall<'a, 'config, M> {
	pub(crate) fn new(runtime: &'a mut Runtime<'config, M>) -> Self {
		Self { runtime }
	}
}

impl<'a, 'config, M: VMApi> Drop for ResolveCall<'a, 'config, M> {
	fn drop(&mut self) {
		self.runtime.status = Err(ExitFatal::UnhandledInterrupt.into());
		self.runtime
			.machine
			.exit(ExitFatal::UnhandledInterrupt.into());
	}
}
