// ------------------------------------------------
//      store_accounts
// ------------------------------------------------
pub fn account_key(
    account_address: &String,
) -> String {
    format!("account:{}", account_address)
}

// ------------------------------------------------
//      store_payouts
// ------------------------------------------------
pub fn payout_key(
    payout_address: &String,
) -> String {
    format!("payout:{}", payout_address)
}
