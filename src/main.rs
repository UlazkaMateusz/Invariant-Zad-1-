use std::error::Error;

use crate::{
    fixed::FIXED_ONE_U64,
    lp_pool::{LpPool, LpTokenAmount, Percentage, Price, StakedTokenAmount, TokenAmount},
};

mod fixed;
mod lp_pool;

fn main() -> Result<(), Box<dyn Error>> {
    let price = Price(15 * FIXED_ONE_U64 / 10); // 1.6
    let min_fee = Percentage(1 * FIXED_ONE_U64 / 1000); // 0.1%
    let max_fee = Percentage(5 * FIXED_ONE_U64 / 100); // 5.0%
    let liquidity_target = TokenAmount(1000 * FIXED_ONE_U64);

    let mut pool = LpPool::init(price, min_fee, max_fee, liquidity_target)?;

    let token_amount = TokenAmount(500 * FIXED_ONE_U64);
    let lp_tokens = pool.add_liquidity(token_amount)?;
    println!("Added liquidity: {:?}", lp_tokens);

    let lp_token_amount = LpTokenAmount(200 * FIXED_ONE_U64);
    let (tokens, staked_tokens) = pool.remove_liquidity(lp_token_amount)?;
    println!("Removed liquidity: {:?}, {:?}", tokens, staked_tokens);

    let staked_token_amount = StakedTokenAmount(10 * FIXED_ONE_U64);
    let swapped_tokens = pool.swap(staked_token_amount)?;
    println!("Swapped tokens: {:?}", swapped_tokens);

    Ok(())
}
