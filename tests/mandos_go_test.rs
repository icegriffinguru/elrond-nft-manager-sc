use elrond_nftmanager::*;
use elrond_wasm::{
    types::{Address, SCResult, ManagedBuffer, OptionalArg, BigUint},
};
use elrond_wasm_debug::{
    rust_biguint, testing_framework::*,
    DebugApi,
};

const WASM_PATH: &'static str = "output/elrond-nftmanager.wasm";

struct NftManagerSetup<NftManagerObjBuilder>
where
    NftManagerObjBuilder: 'static + Copy + Fn() -> elrond_nftmanager::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub em_wrapper: ContractObjWrapper<elrond_nftmanager::ContractObj<DebugApi>, NftManagerObjBuilder>,
}

fn setup_elrond_nftmanager<NftManagerObjBuilder>(
    em_builder: NftManagerObjBuilder,
) -> NftManagerSetup<NftManagerObjBuilder>
where
    NftManagerObjBuilder:
        'static + Copy + Fn() -> elrond_nftmanager::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    
    let em_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        em_builder,
        WASM_PATH,
    );

    blockchain_wrapper.execute_tx(&owner_address, &em_wrapper, &rust_zero, |sc| {
        let payment_token_id_bytes: &[u8] = b"TOKEN-123456";

        let payment_token_id = TokenIdentifier::from(payment_token_id_bytes);
        let nft_token_price = BigUint::from(1000_000_000_000_000_000 as u64);
        let royalties: u32 = 300;
        let base_uri = ManagedBuffer::<DebugApi>::from(b"base_uri");

        let result = sc.init(
            payment_token_id,
            nft_token_price,
            royalties,
            base_uri
        );
        assert_eq!(result, SCResult::Ok(()));

        let token_name = ManagedBuffer::<DebugApi>::from(b"IceWorld");
        let token_ticker = ManagedBuffer::<DebugApi>::from(b"IWC");
        sc.issue_token(token_name, token_ticker);

        StateChange::Commit
    });

    blockchain_wrapper.add_mandos_set_account(em_wrapper.address_ref());

    NftManagerSetup {
        blockchain_wrapper,
        owner_address,
        em_wrapper,
    }
}

// //////////////////////////////////////////////////////////////

#[test]
fn init_test() {
    let em_setup = setup_elrond_nftmanager(elrond_nftmanager::contract_obj);
    em_setup
        .blockchain_wrapper
        .write_mandos_output("_generated_init.scen.json");
}
