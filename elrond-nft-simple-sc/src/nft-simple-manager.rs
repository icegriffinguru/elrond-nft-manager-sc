#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();


const NFT_AMOUNT: u32 = 1;
const ROYALTIES_MAX: u32 = 10_000;


#[elrond_wasm::module]
pub trait NftModule {
    #[init]
    fn init(&self) {}
}
