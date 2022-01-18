#![cfg(feature = "test-bpf")]
mod utils;

use mpl_token_metadata::state::{UseMethod, Uses};
use solana_program_test::*;
use solana_sdk::{signature::Signer, transaction::Transaction};
use utils::*;

mod uses {
    use mpl_token_metadata::{
        pda::{find_program_as_burner_account, find_use_authority_account},
        state::{Key, UseAuthorityRecord},
    };
    use solana_program::{borsh::try_from_slice_unchecked, program_pack::Pack};
    use solana_sdk::signature::Keypair;
    use spl_token::state::Account;

    use super::*;
    #[tokio::test]
    async fn single_use_success() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        test_metadata
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                Some(Uses {
                    use_method: UseMethod::Single,
                    total: 1,
                    remaining: 1,
                }),
            )
            .await
            .unwrap();

        let ix = mpl_token_metadata::instruction::utilize(
            mpl_token_metadata::id(),
            test_metadata.pubkey.clone(),
            test_metadata.token.pubkey(),
            test_metadata.mint.pubkey(),
            None,
            test_metadata.token.pubkey(),
            context.payer.pubkey(),
            None,
            1,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &test_metadata.token],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let metadata = test_metadata.get_data(&mut context).await;
        let metadata_uses = metadata.uses.unwrap();
        let remaining_uses = metadata_uses.remaining;

        assert_eq!(remaining_uses, 0);
    }

    #[tokio::test]
    async fn success_delegated_and_burn() {
        let mut context = program_test().start_with_context().await;
        let use_authority = Keypair::new();

        let test_meta = Metadata::new();
        test_meta
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                Some(Uses {
                    use_method: UseMethod::Burn,
                    total: 1,
                    remaining: 1,
                }),
            )
            .await
            .unwrap();
        airdrop(&mut context, &use_authority.pubkey(), 10_000_000_000)
            .await
            .unwrap();

        airdrop(&mut context, &test_meta.token.pubkey(), 10_000_000_000)
            .await
            .unwrap();
        let (record, _) =
            find_use_authority_account(&test_meta.mint.pubkey(), &use_authority.pubkey());
        let (burner, _) = find_program_as_burner_account();
        let approveix = mpl_token_metadata::instruction::approve_use_authority(
            mpl_token_metadata::id(),
            record,
            use_authority.pubkey(),
            context.payer.pubkey(),
            context.payer.pubkey(),
            test_meta.token.pubkey(),
            test_meta.pubkey,
            test_meta.mint.pubkey(),
            burner,
            1,
        );
        let approvetx = Transaction::new_signed_with_payer(
            &[approveix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );
        context
            .banks_client
            .process_transaction(approvetx)
            .await
            .unwrap();
        let account = get_account(&mut context, &record).await;
        let record_acct: UseAuthorityRecord = try_from_slice_unchecked(&account.data).unwrap();
        assert_eq!(record_acct.key, Key::UseAuthorityRecord);
        assert_eq!(record_acct.allowed_uses, 1);

        let utilize_ix = mpl_token_metadata::instruction::utilize(
            mpl_token_metadata::id(),
            test_meta.pubkey,
            test_meta.token.pubkey(),
            test_meta.mint.pubkey(),
            Some(record),
            use_authority.pubkey(),
            context.payer.pubkey(),
            Some(burner),
            1,
        );
        let utilize = Transaction::new_signed_with_payer(
            &[utilize_ix],
            Some(&use_authority.pubkey()),
            &[&use_authority],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(utilize)
            .await
            .unwrap();
        let token_account_after_burn = get_account(&mut context, &test_meta.token.pubkey()).await;
        let token_account_after_burn_data: Account =
            Account::unpack_from_slice(token_account_after_burn.data.as_slice()).unwrap();
        assert_eq!(token_account_after_burn_data.amount, 0);
    }

    #[tokio::test]
    async fn success_and_burn() {
        let mut context = program_test().start_with_context().await;

        let test_meta = Metadata::new();
        test_meta
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                Some(Uses {
                    use_method: UseMethod::Burn,
                    total: 1,
                    remaining: 1,
                }),
            )
            .await
            .unwrap();

        airdrop(&mut context, &test_meta.token.pubkey(), 10_000_000_000)
            .await
            .unwrap();

        let utilize_ix = mpl_token_metadata::instruction::utilize(
            mpl_token_metadata::id(),
            test_meta.pubkey,
            test_meta.token.pubkey(),
            test_meta.mint.pubkey(),
            None,
            context.payer.pubkey(),
            context.payer.pubkey(),
            None,
            1,
        );
        let utilize = Transaction::new_signed_with_payer(
            &[utilize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(utilize)
            .await
            .unwrap();
        let token_account_after_burn = get_account(&mut context, &test_meta.token.pubkey()).await;
        let token_account_after_burn_data: Account =
            Account::unpack_from_slice(token_account_after_burn.data.as_slice()).unwrap();
        assert_eq!(token_account_after_burn_data.amount, 0);
    }
}