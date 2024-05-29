#![allow(dead_code)]

use thiserror::Error;

use crate::fixed::Fixed;

#[derive(Debug)]
pub struct TokenAmount(pub u64);

#[derive(Debug)]
pub struct StakedTokenAmount(pub u64);

#[derive(Debug)]
pub struct LpTokenAmount(pub u64);

#[derive(Debug)]
pub struct Price(pub u64);

#[derive(Debug)]
pub struct Percentage(pub u64);

#[derive(Debug)]
pub struct LpPool {
    price: Price,
    token_amount: TokenAmount,
    st_token_amount: StakedTokenAmount,
    lp_token_amount: LpTokenAmount,
    liquidity_target: TokenAmount,
    min_fee: Percentage,
    max_fee: Percentage,
}

#[derive(Error, Debug)]
pub enum Errors {
    #[error("negative value")]
    NegativeValue(String),

    #[error("invalid argument")]
    InvalidArgument(String),

    #[error("zz")]
    InvalidOperation(String),
}

impl LpPool {
    pub fn init(
        price: Price,
        min_fee: Percentage,
        max_fee: Percentage,
        liquidity_target: TokenAmount,
    ) -> Result<Self, Errors> {
        if min_fee.0 > max_fee.0 {
            return Err(Errors::InvalidArgument(
                "min_fee is bigget than max_fee".into(),
            ));
        }

        if liquidity_target.0 == 0 {
            return Err(Errors::InvalidArgument(
                "liquidity_target can not be zero".into(),
            ));
        }

        if price.0 == 0 {
            return Err(Errors::InvalidArgument("price can not be zero".into()));
        }

        Ok(Self {
            price,
            token_amount: TokenAmount(0),
            st_token_amount: StakedTokenAmount(0),
            lp_token_amount: LpTokenAmount(0),
            liquidity_target,
            min_fee,
            max_fee,
        })
    }

    pub fn add_liquidity(&mut self, token_amount: TokenAmount) -> Result<LpTokenAmount, Errors> {
        if self.st_token_amount.0 == 0 {
            // No staked tokens in pool
            self.token_amount.0 = token_amount.0;
            self.lp_token_amount.0 = token_amount.0;
            return Ok(LpTokenAmount(token_amount.0));
        }

        let total_shares = Fixed::from(self.lp_token_amount.0);
        let total_value = Fixed::from(self.token_amount.0)
            + Fixed::from(self.st_token_amount.0) * Fixed::from(self.price.0);

        // shares_for_user = amount * total_shares/total_value
        let tokens_issued = Fixed::from(token_amount.0) * total_shares / total_value;

        if tokens_issued.0 < 0 {
            return Err(Errors::NegativeValue("tokens_issued is negative".into()));
        }

        // SAFETY: we checked that tokens_issued is not negative so cast will always be correct
        let tokens_issued = tokens_issued.0 as u64;

        self.token_amount.0 += token_amount.0;
        self.lp_token_amount.0 += tokens_issued;

        Ok(LpTokenAmount(tokens_issued))
    }

    pub fn remove_liquidity(
        &mut self,
        lp_token_amount: LpTokenAmount,
    ) -> Result<(TokenAmount, StakedTokenAmount), Errors> {
        if lp_token_amount.0 > self.lp_token_amount.0 {
            return Err(Errors::InvalidOperation(
                "tried to remove more tokens than there are in the pool".into(),
            ));
        }

        let proportion = Fixed::from(lp_token_amount.0) / Fixed::from(self.lp_token_amount.0);

        let token_ammount = Fixed::from(self.token_amount.0) * proportion;
        let st_tokens = Fixed::from(self.st_token_amount.0) * proportion;

        if token_ammount.0 < 0 {
            return Err(Errors::NegativeValue("token_ammount is negative".into()));
        }

        if st_tokens.0 < 0 {
            return Err(Errors::NegativeValue("st_tokens  is negative".into()));
        }
        // SAFETY: we checked that token_ammount and st_tokens is not negative so cast will always be correct
        let token_ammount = token_ammount.0 as u64;
        let st_tokens = st_tokens.0 as u64;

        Ok((TokenAmount(token_ammount), StakedTokenAmount(st_tokens)))
    }

    pub fn swap(&mut self, staked_token_amount: StakedTokenAmount) -> Result<TokenAmount, Errors> {
        let user_st_to_tokens = Fixed::from(staked_token_amount.0) * Fixed::from(self.price.0);
        let amount_after = Fixed::from(self.token_amount.0) - user_st_to_tokens;

        let liquidity_target = Fixed::from(self.liquidity_target.0);
        let unstake_fee = if amount_after > liquidity_target {
            Fixed::from(self.min_fee.0)
        } else {
            let max_fee = Fixed::from(self.max_fee.0);
            max_fee - (max_fee - Fixed::from(self.min_fee.0)) * amount_after / liquidity_target
        };

        let fee_tokens = user_st_to_tokens * unstake_fee;

        let tokens = user_st_to_tokens - fee_tokens;

        if tokens.0 < 0 {
            return Err(Errors::NegativeValue("tokens is negative".into()));
        }

        // SAFETY: we checked that tokens is not negative so cast will always be correct
        let tokens = tokens.0 as u64;
        self.st_token_amount.0 += staked_token_amount.0;

        if tokens > self.token_amount.0 {
            return Err(Errors::InvalidOperation(
                "Tried to swap more tokens than there is in a pool".into(),
            ));
        }
        self.token_amount.0 -= tokens;

        Ok(TokenAmount(tokens))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::fixed::FIXED_ONE_U64;

    use super::*;

    fn create_pool() -> Result<LpPool, Errors> {
        LpPool::init(
            Price(15 * FIXED_ONE_U64 / 10),           // 1.5
            Percentage(1 * FIXED_ONE_U64 / 10 / 100), // 0.1% so 0.001
            Percentage(9 * FIXED_ONE_U64 / 100),      // 9.0% so 0.09
            TokenAmount(90 * FIXED_ONE_U64),
        )
    }

    #[test]
    fn example_use_case() -> Result<(), Box<dyn Error>> {
        let mut pool = create_pool()?;
        let lp_tokens = pool.add_liquidity(TokenAmount(100 * FIXED_ONE_U64))?;

        assert_eq!(lp_tokens.0, 100 * FIXED_ONE_U64);

        let tokens = pool.swap(StakedTokenAmount(6 * FIXED_ONE_U64))?;

        assert_eq!(tokens.0, 8991 * FIXED_ONE_U64 / 1000); // 8.991

        let tokens = pool.add_liquidity(TokenAmount(10 * FIXED_ONE_U64))?;

        assert_eq!(tokens.0, 99991 * FIXED_ONE_U64 / 10000); // 9.9991

        let tokens = pool.swap(StakedTokenAmount(30 * FIXED_ONE_U64))?;
        assert_eq!(
            tokens.0,
            434423507580 // 43.4423507580
        ); // Should be 43.44237

        let (tokens, st_tokens) = pool.remove_liquidity(LpTokenAmount(1099991000000))?; //109.9991
        assert_eq!(tokens.0, 575666492420); // Sould be 57.56663 but is 57.5666492420
        assert_eq!(st_tokens.0, 36 * FIXED_ONE_U64);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn fail_to_swap_too_much_tokens() {
        let mut pool = create_pool().unwrap();
        pool.add_liquidity(TokenAmount(100 * FIXED_ONE_U64))
            .unwrap();

        pool.swap(StakedTokenAmount(100 * FIXED_ONE_U64)).unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_remove_too_much_liquidity() {
        let mut pool = create_pool().unwrap();
        pool.add_liquidity(TokenAmount(100 * FIXED_ONE_U64))
            .unwrap();

        pool.remove_liquidity(LpTokenAmount(101 * FIXED_ONE_U64))
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_create_pool_with_wrong_fees() {
        LpPool::init(
            Price(15 * FIXED_ONE_U64 / 10),       // 1.5
            Percentage(10 * FIXED_ONE_U64 / 100), // 10.0% so 0.1
            Percentage(9 * FIXED_ONE_U64 / 100),  // 9.0% so 0.09
            TokenAmount(90 * FIXED_ONE_U64),
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_create_pool_with_zero_as_liquidity_target() {
        LpPool::init(
            Price(15 * FIXED_ONE_U64 / 10),           // 1.5
            Percentage(1 * FIXED_ONE_U64 / 10 / 100), // 0.1% so 0.001
            Percentage(9 * FIXED_ONE_U64 / 100),      // 9.0% so 0.09
            TokenAmount(0),
        )
        .unwrap();
    }
}
