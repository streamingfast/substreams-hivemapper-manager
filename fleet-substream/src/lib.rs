extern crate core;

mod pb;
mod instruction;
mod keyer;
mod option;

use num_bigint::BigInt;
use substreams::log::info;
use substreams::log_info;
use substreams::prelude::*;
use substreams::store::StoreGet;

use crate::instruction::TokenInstruction;

use {
    bs58,
    substreams::{errors::Error, log, store, proto},
    substreams_solana::pb as solpb
};
use crate::instruction::TokenInstruction::InitializeAccount;
use crate::pb::fleet;

#[substreams::handlers::map]
fn map_payouts(blk: solpb::sol::v1::Block) -> Result<pb::fleet::sol::v1::Payouts, Error> {
    log::info!("extracting fleet payouts");
    let mut payouts = vec![] ;
    for trx in blk.transactions {
        if let Some(meta) = trx.meta {

            // Invalid Txn
            if let Some(_) = meta.err {
                continue;
            }
            if meta.inner_instructions_none {
                continue;
            } else {
                if let Some(transaction) = trx.transaction{
                    if let Some(msg) = transaction.message{
                        if let Some(header) = msg.header {

                            let mut contains_program = 0;
                            for log in &meta.log_messages {
                                if log.contains("Program EEjwuvCMVYjgHUeX1BM9qmUog59Pft88c3jbt2ATwcJw invoke") {
                                    contains_program += 1;
                                }
                            }

                            if contains_program >= 2 {
                                log_info!("{}", "payout found");

                                if meta.inner_instructions.len() != 1 as usize{
                                    continue
                                }
                                if meta.inner_instructions[0].instructions.len() != 4 as usize{
                                    continue
                                }
                                let sent_to_first = &meta.inner_instructions[0].instructions[1].accounts[1];
                                let sent_to_second = &meta.inner_instructions[0].instructions[3].accounts[1];

                                // let mut payed_amount = "".to_string();
                                // for postTokenBal in &meta.post_token_balances {
                                //     for preTokenBal in &meta.pre_token_balances {
                                //         if let Some(preTokenAmt) = &preTokenBal.ui_token_amount {
                                //             if let Some(postTokenAmt) = &postTokenBal.ui_token_amount {
                                //                 payed_amount = (postTokenAmt.ui_amount - preTokenAmt.ui_amount).to_string();
                                //             }
                                //         }
                                //     }
                                // }

                                payouts.push(pb::fleet::sol::v1::Payout {
                                    transaction_id: bs58::encode(&transaction.signatures[0]).into_string(),
                                    account_one: Option::from(pb::fleet::sol::v1::PayoutAccount {
                                        spl_account: bs58::encode(&msg.account_keys[*sent_to_first as usize]).into_string(),
                                        payout_address: None,
                                        amount: None,
                                    }),
                                    account_two: Option::from(pb::fleet::sol::v1::PayoutAccount{
                                        spl_account: bs58::encode(&msg.account_keys[*sent_to_second as usize]).into_string(),
                                        payout_address: None,
                                        amount: None,
                                    }),
                                })

                                // for inner_inst in &meta.inner_instructions{
                                //     log_info!("{}", "   inner instruction:");
                                //     for inner_inst_instruction in &inner_inst.instructions {
                                //         log_info!("{}", "       inner instruction instructions:");
                                //         for account in &inner_inst_instruction.accounts {
                                //             log_info!("{}", "           inner inst account:");
                                //             log_info!("{:?}", bs58::encode(&msg.account_keys[*account as usize]).into_string())
                                //         }
                                //     }
                                // }
                            }
                        }
                    }
                }
            }
        }
    }
    return Ok(pb::fleet::sol::v1::Payouts { payouts });
}

#[substreams::handlers::map]
fn map_account_creation(blk: solpb::sol::v1::Block) -> Result<pb::fleet::sol::v1::AccountCreations, Error> {
    log::info!("extracting creations");
    let mut creations = vec![] ;
    for trx in blk.transactions {
        if let Some(meta) = trx.meta {
            if let Some(_) = meta.err {
                continue;
            }
            if let Some(transaction) = trx.transaction {
                if let Some(msg) = transaction.message {
                    for inst in msg.instructions {
                        let program_id = &msg.account_keys[inst.program_id_index as usize];

                        if bs58::encode(program_id).into_string() == "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" {
                            for inner_inst_data in &meta.inner_instructions {

                                for inner_inst in &inner_inst_data.instructions {
                                    // log_info!("{:?}", bs58::encode(&inner_inst).into_string())
                                    if TokenInstruction::unpack(&inner_inst.data).is_err(){
                                        continue
                                    }
                                    let inst_matched = TokenInstruction::unpack(&inner_inst.data)?;
                                    match inst_matched {
                                        TokenInstruction::InitializeAccount2 {
                                            ref owner,
                                        } => {
                                            creations.push(pb::fleet::sol::v1::AccountCreation{
                                                spl_account: "".to_string(),
                                                owner: owner[0].to_string(),
                                            })
                                        }
                                        TokenInstruction::InitializeAccount3 {
                                            ref owner,
                                        } => {
                                            creations.push(pb::fleet::sol::v1::AccountCreation{
                                                spl_account: bs58::encode(&msg.account_keys[inner_inst.accounts[0] as usize]).into_string(),
                                                owner: owner[0].to_string(),
                                            })
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        } else {
                            continue;
                        }
                    }
                }
            }
        }

    }

    return Ok(pb::fleet::sol::v1::AccountCreations { creations });
}

#[substreams::handlers::store]
pub fn store_account_creation(creations: pb::fleet::sol::v1::AccountCreations, output: store::StoreSetString) {
    log::info!("building account creation store");
    for account_creation in creations.creations {
        output.set(
            0,
            account_creation.spl_account,
            &account_creation.owner,
        );
    }
}

#[substreams::handlers::map]
fn map_payouts_with_kv(payouts_pre: pb::fleet::sol::v1::Payouts, store: StoreGetString) -> Result<pb::fleet::sol::v1::Payouts, Error> {
    log::info!("Injecting into payout extractions");
    let mut payouts = vec![] ;
    for payout in payouts_pre.payouts {

        let spl_account_one = payout.account_one.unwrap().spl_account;
        let spl_account_two = payout.account_two.unwrap().spl_account;

        payouts.push(pb::fleet::sol::v1::Payout {
            transaction_id: payout.transaction_id,
            account_one: Option::from(pb::fleet::sol::v1::PayoutAccount {
                spl_account: spl_account_one.clone(),
                payout_address: store.get_last(&spl_account_one),
                amount: None,
            }),
            account_two: Option::from(pb::fleet::sol::v1::PayoutAccount {
                spl_account: spl_account_two.clone(),
                payout_address: store.get_last(&spl_account_two),
                amount: None,
            }),
        })
    }
    return Ok(pb::fleet::sol::v1::Payouts {  payouts });
}

