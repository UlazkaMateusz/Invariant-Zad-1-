use thiserror::Error;

use crate::fixed::Fixed;

struct TokenAmount(Fixed);
impl TokenAmount {
    pub fn new(ammount: f64) -> Self {
        Self(Fixed::new(ammount))
    }
}
struct StakedTokenAmount(Fixed);
impl StakedTokenAmount {
    pub fn new(ammount: f64) -> Self {
        Self(Fixed::new(ammount))
    }
}
struct LpTokenAmount(Fixed);
impl LpTokenAmount {
    pub fn new(ammount: f64) -> Self {
        Self(Fixed::new(ammount))
    }
}
struct Price(Fixed);
impl Price {
    pub fn new(ammount: f64) -> Self {
        Self(Fixed::new(ammount))
    }
}
struct Percentage(Fixed);
impl Percentage {
    pub fn new(ammount: f64) -> Self {
        Self(Fixed::new(ammount))
    }
}

struct LpPool {
    price: Price,
    token_amount: TokenAmount,
    st_token_amount: StakedTokenAmount,
    lp_token_amount: LpTokenAmount,
    liquidity_target: TokenAmount,
    min_fee: Percentage,
    max_fee: Percentage,
}

#[derive(Error, Debug)]
enum Errors {
    #[error("TODO")]
    Todo,
}

impl LpPool {
    pub fn init(
        price: Price,
        min_fee: Percentage,
        max_fee: Percentage,
        liquidity_target: TokenAmount,
    ) -> Result<Self, Errors> {
        // TODO: Error handling
        //
        Ok(Self {
            price,
            token_amount: TokenAmount::new(0.0),
            st_token_amount: StakedTokenAmount::new(0.0),
            lp_token_amount: LpTokenAmount::new(0.0),
            liquidity_target,
            min_fee,
            max_fee,
        })
    }

    pub fn add_liquidity(&mut self, token_amount: TokenAmount) -> Result<LpTokenAmount, Errors> {
        self.token_amount.0 = self.token_amount.0 + token_amount.0;
        Ok(LpTokenAmount(token_amount.0))
    }

    pub fn remove_liquidity(
        &mut self,
        lp_token_amount: LpTokenAmount,
    ) -> Result<(TokenAmount, StakedTokenAmount), Errors> {
        todo!()
    }

    pub fn swap(&mut self, staked_token_amount: StakedTokenAmount) -> Result<TokenAmount, Errors> {
        let ammount_after = self.token_amount.0 - staked_token_amount.0;
        let unstake_fee = self.max_fee.0
            - (self.max_fee.0 - self.min_fee.0) * ammount_after / self.liquidity_target.0;
        Ok(TokenAmount(unstake_fee))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    fn create_pool() -> Result<LpPool, Errors> {
        LpPool::init(
            Price::new(1.5),
            Percentage::new(0.1),
            Percentage::new(9.0),
            TokenAmount::new(90.0),
        )
    }

    #[test]
    fn init() -> Result<(), Box<dyn Error>> {
        let pool = create_pool()?;
        Ok(())
    }

    #[test]
    fn add_liquidity() -> Result<(), Box<dyn Error>> {
        let mut pool = create_pool().unwrap();
        let lp_tokens = pool.add_liquidity(TokenAmount::new(100.0))?;

        assert_eq!(lp_tokens.0, Fixed::new(100.0));
        Ok(())
    }

    #[test]
    fn swap() -> Result<(), Box<dyn Error>> {
        let mut pool = create_pool()?;
        let lp_tokens = pool.add_liquidity(TokenAmount::new(100.0))?;

        assert_eq!(lp_tokens.0, Fixed::new(100.0));

        let tokens = pool.swap(StakedTokenAmount::new(6.0))?;

        assert_eq!(tokens.0, Fixed::new(8.991));

        Ok(())
    }
}

//1. LpPool::init(price=1.5, min_fee=0.1%, max_fee9%, liquidity_target=90.0 Token) -> return
//lp_pool
//2. lp_pool.add_liquidity(100.0 Token) -> return 100.0 LpToken
//3. lp_pool.swap(6 StakedToken) -> return 8.991 Token
//4. lp_pool.add_liquidity(10.0 Token) -> 9.9991 LpToken
//5. lp_pool.swap(30.0 StakedToken) -> return 43.44237 Token
//6. lp_pool.remove_liquidity(109.9991) -> return (57.56663 Token, 36 StakedToken
