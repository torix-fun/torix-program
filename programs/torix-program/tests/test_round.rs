use {
    anchor_lang::{solana_program::{instruction::Instruction, system_program::ID as SYSTEM_PROGRAM_ID}, InstructionData, ToAccountMetas, prelude::*},
    litesvm::LiteSVM,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
    torix_rust_sdk::pda::*
};


#[test]
fn test_start_round() {
    let program_id = torix_program::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/torix_program.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let user = payer.pubkey();
    let round = derive_round();
    let round_vault = derive_round_vault(&round); 

    let instruction = Instruction::new_with_bytes(
        program_id,
        &torix_program::instruction::StartRound {
            duration_seconds: 86400,
        }.data(),
        torix_program::accounts::StartRound {
            user,
            round,
            round_vault,
            system_program: SYSTEM_PROGRAM_ID
        }.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();

    let res = svm.send_transaction(tx);

    match res {
        Ok(r) => println!("{:#?}", r.logs),
        Err(e) => eprintln!("{:#?}", e)
    }
}
