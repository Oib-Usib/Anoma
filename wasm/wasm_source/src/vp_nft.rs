use anoma_vm_env::vp_prelude::*;

#[validity_predicate]
fn validate_tx(
    tx_data: Vec<u8>,
    addr: Address,
    keys_changed: HashSet<storage::Key>,
    verifiers: HashSet<Address>,
) -> bool {
    log_string(format!(
        "validate_tx called with token addr: {}, key_changed: {:#?}, \
         verifiers: {:?}",
        addr, keys_changed, verifiers
    ));

    nft::vp(tx_data, &addr, &keys_changed, &verifiers)
}

#[cfg(test)]
mod tests {
    // Use this as `#[test]` annotation to enable logging
    use anoma::types::nft::{self, NftToken};
    use anoma::types::transaction::{CreateNft, MintNft};
    use anoma_tests::log::test;
    use anoma_tests::tx::{self, tx_host_env, TestTxEnv};
    use anoma_tests::vp::*;

    use super::*;

    const VP_ALWAYS_TRUE_WASM: &str =
        "../../wasm_for_tests/vp_always_true.wasm";

    /// Test that no-op transaction (i.e. no storage modifications) accepted.
    #[test]
    fn test_no_op_transaction() {
        let mut tx_env = TestTxEnv::default();
        tx::init_tx_env(&mut tx_env);

        let nft_owner = address::testing::established_address_2();
        tx_env.spawn_accounts([&nft_owner]);

        // just a dummy vp, its not used during testing
        let vp_code =
            std::fs::read(VP_ALWAYS_TRUE_WASM).expect("cannot load wasm");

        let nft_address = tx_host_env::nft::init_nft(CreateNft {
            owner: nft_owner.clone(),
            vp_code: vp_code,
            tokens: vec![],
        });

        tx_env.write_log.commit_tx();

        let vp_env =
            init_vp_env_from_tx(nft_address.clone(), tx_env, |address| {
                // Apply transfer in a transaction
                tx_host_env::insert_verifier(address)
            });

        let tx_data: Vec<u8> = vec![];
        let keys_changed: HashSet<storage::Key> =
            vp_env.all_touched_storage_keys();
        let verifiers: HashSet<Address> = vp_env.get_verifiers();
        assert!(validate_tx(tx_data, nft_address, keys_changed, verifiers));
    }

    /// Test that you can create an nft without tokens
    #[test]
    fn test_mint_no_tokens() {
        let mut tx_env = TestTxEnv::default();
        tx::init_tx_env(&mut tx_env);

        let nft_owner = address::testing::established_address_2();
        tx_env.spawn_accounts([&nft_owner]);

        // just a dummy vp, its not used during testing
        let vp_code =
            std::fs::read(VP_ALWAYS_TRUE_WASM).expect("cannot load wasm");

        let nft_address = tx_host_env::nft::init_nft(CreateNft {
            owner: nft_owner.clone(),
            vp_code: vp_code,
            tokens: vec![],
        });

        tx_env.write_log.commit_tx();

        let vp_env =
            init_vp_env_from_tx(nft_address.clone(), tx_env, |address| {
                // Apply transfer in a transaction
                tx_host_env::nft::mint_tokens(MintNft {
                    address: nft_address.clone(),
                    owner: nft_owner.clone(),
                    tokens: vec![],
                    verifier: nft_owner.clone(),
                });
                tx_host_env::insert_verifier(address)
            });

        let tx_data: Vec<u8> = vec![];
        let keys_changed: HashSet<storage::Key> =
            vp_env.all_touched_storage_keys();
        let verifiers: HashSet<Address> = vp_env.get_verifiers();
        assert!(validate_tx(tx_data, nft_address, keys_changed, verifiers));
    }

    /// Test that you can create an nft with tokens
    #[test]
    fn test_mint_tokens() {
        let mut tx_env = TestTxEnv::default();
        tx::init_tx_env(&mut tx_env);

        let nft_owner = address::testing::established_address_2();
        let nft_token_owner = address::testing::established_address_1();
        tx_env.spawn_accounts([&nft_owner, &nft_token_owner]);

        // just a dummy vp, its not used during testing
        let vp_code =
            std::fs::read(VP_ALWAYS_TRUE_WASM).expect("cannot load wasm");

        let nft_address = tx_host_env::nft::init_nft(CreateNft {
            owner: nft_owner.clone(),
            vp_code: vp_code,
            tokens: vec![],
        });

        tx_env.commit_tx_and_block();

        let vp_env = init_vp_env_from_tx(nft_address.clone(), tx_env, |_| {
            // Apply transfer in a transaction
            tx_host_env::nft::mint_tokens(MintNft {
                address: nft_address.clone(),
                owner: nft_token_owner.clone(),
                tokens: vec![NftToken {
                    id: 1,
                    metadata: "".to_string(),
                    approvals: vec![],
                }],
                verifier: nft_owner.clone(),
            });
        });

        let tx_data: Vec<u8> = vec![];
        let keys_changed: HashSet<storage::Key> =
            vp_env.all_touched_storage_keys();
        let verifiers: HashSet<Address> = vp_env.get_verifiers();

        assert!(validate_tx(tx_data, nft_address, keys_changed, verifiers));
    }

    /// Test that only owner can mint new tokens
    #[test]
    fn test_mint_tokens_wrong_owner() {
        let mut tx_env = TestTxEnv::default();
        tx::init_tx_env(&mut tx_env);

        let nft_owner = address::testing::established_address_2();
        let nft_token_owner = address::testing::established_address_1();
        tx_env.spawn_accounts([&nft_owner, &nft_token_owner]);

        // just a dummy vp, its not used during testing
        let vp_code =
            std::fs::read(VP_ALWAYS_TRUE_WASM).expect("cannot load wasm");

        let nft_address = tx_host_env::nft::init_nft(CreateNft {
            owner: nft_owner.clone(),
            vp_code: vp_code,
            tokens: vec![],
        });

        tx_env.commit_tx_and_block();

        let vp_env = init_vp_env_from_tx(nft_address.clone(), tx_env, |_| {
            // Apply transfer in a transaction
            tx_host_env::nft::mint_tokens(MintNft {
                address: nft_address.clone(),
                owner: nft_token_owner.clone(),
                verifier: nft_token_owner.clone(),
                tokens: vec![NftToken {
                    id: 1,
                    metadata: "".to_string(),
                    approvals: vec![],
                }],
            });
        });

        let tx_data: Vec<u8> = vec![];
        let keys_changed: HashSet<storage::Key> =
            vp_env.all_touched_storage_keys();
        let verifiers: HashSet<Address> = vp_env.get_verifiers();

        assert!(!validate_tx(tx_data, nft_address, keys_changed, verifiers));
    }

    /// Test that an approval can add another approval
    #[test]
    fn test_mint_tokens_with_approvals_authorized() {
        let mut tx_env = TestTxEnv::default();
        tx::init_tx_env(&mut tx_env);

        let nft_owner = address::testing::established_address_2();
        let nft_token_owner = address::testing::established_address_1();
        let nft_token_approval = address::testing::established_address_3();
        let nft_token_approval_2 = address::testing::established_address_4();
        tx_env.spawn_accounts([
            &nft_owner,
            &nft_token_owner,
            &nft_token_approval,
            &nft_token_approval_2,
        ]);

        // just a dummy vp, its not used during testing
        let vp_code =
            std::fs::read(VP_ALWAYS_TRUE_WASM).expect("cannot load wasm");

        // CREATE NFT TRANSACTION

        let nft_address = tx_host_env::nft::init_nft(CreateNft {
            owner: nft_owner.clone(),
            vp_code: vp_code,
            tokens: vec![],
        });

        tx_env.commit_tx_and_block();

        // MINT NEW NFT TOKENS

        tx_host_env::nft::mint_tokens(MintNft {
            address: nft_address.clone(),
            owner: nft_token_owner.clone(),
            verifier: nft_owner.clone(),
            tokens: vec![NftToken {
                id: 1,
                metadata: "".to_string(),
                approvals: vec![nft_token_approval.clone()],
            }],
        });

        tx_env.commit_tx_and_block();

        // TEST ADDING A NEW APPROVAL ADDRESS WITHOUT AUTHORIZATION

        let vp_env = init_vp_env_from_tx(nft_address.clone(), tx_env, |_| {
            let approval_key = nft::get_token_approval_key(
                &nft_address,
                1.to_string().as_ref(),
                &nft_token_approval_2,
            )
            .to_string();
            tx_host_env::write(approval_key, ());
            tx_host_env::insert_verifier(&nft_token_approval);
        });

        let tx_data: Vec<u8> = vec![];
        let keys_changed: HashSet<storage::Key> =
            vp_env.all_touched_storage_keys();
        let verifiers: HashSet<Address> = vp_env.get_verifiers();

        assert!(validate_tx(tx_data, nft_address, keys_changed, verifiers));
    }

    /// Test that an approval can add another approval
    #[test]
    fn test_mint_tokens_with_approvals_not_authorized() {
        let mut tx_env = TestTxEnv::default();
        tx::init_tx_env(&mut tx_env);

        let nft_owner = address::testing::established_address_2();
        let nft_token_owner = address::testing::established_address_1();
        let nft_token_approval = address::testing::established_address_3();
        let nft_token_approval_2 = address::testing::established_address_4();
        tx_env.spawn_accounts([
            &nft_owner,
            &nft_token_owner,
            &nft_token_approval,
            &nft_token_approval_2,
        ]);

        // just a dummy vp, its not used during testing
        let vp_code =
            std::fs::read(VP_ALWAYS_TRUE_WASM).expect("cannot load wasm");

        let nft_address = tx_host_env::nft::init_nft(CreateNft {
            owner: nft_owner.clone(),
            vp_code: vp_code,
            tokens: vec![],
        });

        tx_env.commit_tx_and_block();

        tx_host_env::nft::mint_tokens(MintNft {
            address: nft_address.clone(),
            owner: nft_token_owner.clone(),
            verifier: nft_owner.clone(),
            tokens: vec![NftToken {
                id: 1,
                metadata: "".to_string(),
                approvals: vec![nft_token_approval.clone()],
            }],
        });

        tx_env.commit_tx_and_block();

        let vp_env = init_vp_env_from_tx(nft_address.clone(), tx_env, |_| {
            let approval_key = nft::get_token_approval_key(
                &nft_address,
                1.to_string().as_ref(),
                &nft_token_approval_2,
            )
            .to_string();
            tx_host_env::write(approval_key, ());
            tx_host_env::insert_verifier(&nft_token_approval_2);
        });

        let tx_data: Vec<u8> = vec![];
        let keys_changed: HashSet<storage::Key> =
            vp_env.all_touched_storage_keys();
        let verifiers: HashSet<Address> = vp_env.get_verifiers();

        assert!(!validate_tx(tx_data, nft_address, keys_changed, verifiers));
    }

    /// Test nft address cannot be changed
    #[test]
    fn test_cant_change_owner() {
        let mut tx_env = TestTxEnv::default();
        tx::init_tx_env(&mut tx_env);

        let nft_owner = address::testing::established_address_2();
        let another_address = address::testing::established_address_1();
        tx_env.spawn_accounts([&nft_owner, &another_address]);

        // just a dummy vp, its not used during testing
        let vp_code =
            std::fs::read(VP_ALWAYS_TRUE_WASM).expect("cannot load wasm");

        let nft_address = tx_host_env::nft::init_nft(CreateNft {
            owner: nft_owner.clone(),
            vp_code: vp_code,
            tokens: vec![],
        });

        tx_env.commit_tx_and_block();

        let vp_env = init_vp_env_from_tx(nft_address.clone(), tx_env, |_| {
            let creator_key =
                nft::get_creator_key(&nft_address, &another_address)
                    .to_string();
            tx_host_env::write(creator_key, ());
        });

        let tx_data: Vec<u8> = vec![];
        let keys_changed: HashSet<storage::Key> =
            vp_env.all_touched_storage_keys();
        let verifiers: HashSet<Address> = vp_env.get_verifiers();

        assert!(!validate_tx(tx_data, nft_address, keys_changed, verifiers));
    }
}