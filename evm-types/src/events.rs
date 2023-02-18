multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::ETHAddress;

#[derive(TypeAbi, TopEncode)]
pub struct DeployCodeEvent<M: ManagedTypeApi> {
    address: ETHAddress<M>,
}
