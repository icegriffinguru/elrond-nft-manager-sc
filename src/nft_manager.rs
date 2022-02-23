#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::elrond_codec::TopEncode;

const NFT_AMOUNT: u32 = 1;
const ROYALTIES_MAX: u32 = 10_000;

const URI_SLASH: &[u8] = "/".as_bytes();
const HASH_TAG: &[u8] = "#".as_bytes();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct NftAttributes {
    pub creation_timestamp: u64,
}

#[elrond_wasm::contract]
pub trait NftManager {
    #[init]
    fn init(&self, payment_token_id: TokenIdentifier, nft_token_price: BigUint, royalties: u32, base_uri: ManagedBuffer) -> SCResult<()> {
        require!(royalties <= ROYALTIES_MAX, "royalties cannot exceed 100%");
        require!(
            payment_token_id.is_valid_esdt_identifier(),
            "invalid token identifier provided"
        );

        self.payment_token_id().set(&payment_token_id);
        self.nft_token_price().set(&nft_token_price);
        self.royalties().set(royalties);
        self.base_uri().set(&base_uri);

        // set mint_count to 0 for indexing
        self.mint_count().set(0u64);

        Ok(())
    }

    // endpoints - owner-only

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_token(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer) -> AsyncCall {
        require!(self.nft_token_id().is_empty(), "Token already issued");

        // save token name
        self.nft_token_name().set(&token_name);

        let payment_amount = self.call_value().egld_value();
        self.send()
            .esdt_system_sc_proxy()
            .issue_non_fungible(
                payment_amount,
                &token_name,
                &token_ticker,
                NonFungibleTokenProperties {
                    can_freeze: false,
                    can_wipe: false,
                    can_pause: false,
                    can_change_owner: true,
                    can_upgrade: false,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().issue_callback())
    }

    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) -> AsyncCall {
        self.require_token_issued();

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.nft_token_id().get(),
                [EsdtLocalRole::NftCreate][..].iter().cloned(),
            )
            .async_call()
    }

    #[only_owner]
    #[endpoint(pauseMinting)]
    fn pause_minting(&self) -> SCResult<()> {
        self.paused().set(true);

        Ok(())
    }

    #[only_owner]
    #[endpoint(startMinting)]
    fn start_minting(&self) -> SCResult<()> {
        require!(!self.nft_token_id().is_empty(), "token not issued");

        self.paused().clear();

        Ok(())
    }

    

    /// endpoint

    #[payable("*")]
    #[endpoint(mint)]
    fn mint(&self, #[payment_token] payment_token: TokenIdentifier, #[payment_amount] payment_amount: BigUint) {
        self.require_token_issued();

        require!(
            payment_token == self.payment_token_id().get(),
            "not given token identifier"
        );
        require!(
            payment_amount >= self.nft_token_price().get(),
            "not enough tokens"
        );

        let nft_nonce = self._mint();
        let nft_token_id = self.nft_token_id().get();
        let caller = self.blockchain().get_caller();
        self.send().direct(
            &caller,
            &nft_token_id,
            nft_nonce,
            &BigUint::from(NFT_AMOUNT),
            &[],
        );
    }

    /// private

    fn _mint(&self) -> u64 {
        self.require_token_issued();

        let nft_token_id = self.nft_token_id().get();

        let attributes = NftAttributes {
            creation_timestamp: self.blockchain().get_block_timestamp(),
        };
        let mut serialized_attributes = ManagedBuffer::new();
        if let core::result::Result::Err(err) = attributes.top_encode(&mut serialized_attributes) {
            sc_panic!("Attributes encode error: {}", err.message_bytes());
        }

        let attributes_hash: ManagedByteArray<Self::Api, 32> = self.crypto().sha256(&serialized_attributes);

        let mint_count = self.mint_count().get();

        let mut name = ManagedBuffer::new();
        name.append(&self.nft_token_name().get());
        name.append(&ManagedBuffer::new_from_bytes(HASH_TAG));
        name.append(&ManagedBuffer::from(&mint_count.to_ne_bytes()));

        let mut uri = ManagedBuffer::new();
        uri.append(&ManagedBuffer::new_from_bytes(URI_SLASH));
        uri.append(&ManagedBuffer::from(&mint_count.to_ne_bytes()));
        let uris = ManagedVec::from_single_item(uri);

        let nft_nonce = self.send().esdt_nft_create(
            &nft_token_id,
            &BigUint::from(NFT_AMOUNT),
            &name,
            &BigUint::from(self.royalties().get()),
            attributes_hash.as_managed_buffer(),
            &attributes,
            &uris,
        );

        self.mint_count().update(|v| *v += 1);

        nft_nonce
    }

    fn require_token_issued(&self) {
        require!(!self.nft_token_id().is_empty(), "Token not issued");
    }

    // callbacks

    #[callback]
    fn issue_callback(&self, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.nft_token_id().set(&token_id);
            },
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_owner_address();
                let (returned_tokens, token_id) = self.call_value().payment_token_pair();
                if token_id.is_egld() && returned_tokens > 0 {
                    self.send()
                        .direct(&caller, &token_id, 0, &returned_tokens, &[]);
                }
            },
        }
    }

    /// storage

    #[view(getNftTokenId)]
    #[storage_mapper("nft_token_id")]
    fn nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getNftTokenPrice)]
    #[storage_mapper("nft_token_price")]
    fn nft_token_price(&self) -> SingleValueMapper<BigUint>;

    #[view(getPaymentTokenId)]
    #[storage_mapper("payment_token_id")]
    fn payment_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(isPaused)]
    #[storage_mapper("paused")]
    fn paused(&self) -> SingleValueMapper<bool>;

    #[view(getMintCount)]
    #[storage_mapper("mint_count")]
    fn mint_count(&self) -> SingleValueMapper<u64>;

    // base metadatas

    #[view(getNftTokenName)]
    #[storage_mapper("nft_token_name")]
    fn nft_token_name(&self) -> SingleValueMapper<ManagedBuffer>;

    #[view(getRoyalties)]
    #[storage_mapper("royalties")]
    fn royalties(&self) -> SingleValueMapper<u32>;

    #[view(getBaseUri)]
    #[storage_mapper("base_uri")]
    fn base_uri(&self) -> SingleValueMapper<ManagedBuffer>;
}
