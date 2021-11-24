use crate::state::{CentralState, StakeAccount, StakePool, SECONDS_IN_DAY};
use spl_token::state::{Account, Mint};

pub fn get_balance(account: &Account) -> u64 {
    account.amount
}

pub fn get_supply(mint: &Mint) -> u64 {
    mint.supply
}

pub fn calc_reward(
    current_time: u64,
    stake_pool: &StakePool,
    central_state: &CentralState,
    mint: &Mint,
) -> u64 {
    let period = current_time
        .checked_sub(stake_pool.last_crank_time as u64)
        .unwrap()
        .checked_div(SECONDS_IN_DAY)
        .unwrap();

    let amount = stake_pool
        .total_staked
        .checked_mul(central_state.daily_inflation)
        .unwrap()
        .checked_div(mint.supply)
        .unwrap();

    amount.checked_mul(period).unwrap()
}

#[test]
fn test() {
    use solana_program::{program_option::COption, pubkey::Pubkey};
    use spl_token::state::AccountState;

    let decimal_multiplier = 1_000_000;

    let mint = Mint {
        mint_authority: COption::None,
        supply: decimal_multiplier * 1_000_000_000,
        decimals: 6,
        is_initialized: true,
        freeze_authority: COption::None,
    };

    let vault = Account {
        mint: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        amount: decimal_multiplier * 10_000,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let central_state = CentralState {
        signer_nonce: 0,
        daily_inflation: 100_000,
    };

    let stake_account = StakeAccount {
        owner: Pubkey::new_unique().to_bytes(),
        stake_amount: 4_000 * decimal_multiplier,
    };

    let last_crank = 1637639871;
    let current_time = 1637726331;

    let stake_pool = StakePool {
        total_staked: decimal_multiplier * 10_000,
        last_crank_time: last_crank,
        owner: Pubkey::new_unique().to_bytes(),
        rewards_destination: Pubkey::new_unique().to_bytes(),
        nonce: 0,
        name: "Test".to_string(),
    };

    let reward = calc_reward(current_time, &stake_pool, &central_state, &mint);
    println!("Reward {}", reward);
}
