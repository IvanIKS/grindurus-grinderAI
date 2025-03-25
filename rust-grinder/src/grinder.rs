use crate::{ethereum::EthereumInterface, Intent, Operation, Position};
use candid::Principal;
use ic_web3::types::{U256, Address};
use std::str::FromStr;

pub struct Grinder {
    eth: EthereumInterface,
    max_tx_cost_percent: f64,
    max_tx_cost: f64,
}

impl Grinder {
    pub fn new(eth: EthereumInterface) -> Self {
        Self {
            eth,
            max_tx_cost_percent: 0.0007, // 0.07%
            max_tx_cost: 0.05,           // $0.05
        }
    }

    pub async fn iterate_pool(&self, pool_id: U256) -> Result<Vec<Operation>, String> {
        let (long, hedge) = self.eth.get_positions(pool_id).await?;
        let mut operations = Vec::new();

        // Check if rebalancing is needed
        if self.needs_rebalance(&long, &hedge) {
            operations.push(Operation::Rebalance);
        }

        // Check long position
        if self.should_long_buy(&long) {
            operations.push(Operation::LongBuy);
        } else if self.should_long_sell(&long) {
            operations.push(Operation::LongSell);
        }

        // Check hedge position
        if self.should_hedge_sell(&hedge) {
            operations.push(Operation::HedgeSell);
        } else if self.should_hedge_rebuy(&hedge) {
            operations.push(Operation::HedgeRebuy);
        }

        Ok(operations)
    }

    fn needs_rebalance(&self, long: &Position, hedge: &Position) -> bool {
        // Implement rebalancing logic
        let long_capital = U256::from_str(&long.active_capital).unwrap_or_default();
        let hedge_capital = U256::from_str(&hedge.active_capital).unwrap_or_default();
        
        // Simple example: check if difference is more than 10%
        if long_capital > hedge_capital {
            long_capital.saturating_sub(hedge_capital) > (long_capital / 10)
        } else {
            hedge_capital.saturating_sub(long_capital) > (hedge_capital / 10)
        }
    }

    fn should_long_buy(&self, position: &Position) -> bool {
        let current_number = position.number;
        let max_number = position.number_max;
        let price_min = U256::from_str(&position.price_min).unwrap_or_default();
        
        current_number < max_number && self.get_current_price() <= price_min
    }

    fn should_long_sell(&self, position: &Position) -> bool {
        let price_max = U256::from_str(&position.price_max).unwrap_or_default();
        
        position.number > 0 && self.get_current_price() >= price_max
    }

    fn should_hedge_sell(&self, position: &Position) -> bool {
        let price_max = U256::from_str(&position.price_max).unwrap_or_default();
        
        position.number < position.number_max && self.get_current_price() >= price_max
    }

    fn should_hedge_rebuy(&self, position: &Position) -> bool {
        let price_min = U256::from_str(&position.price_min).unwrap_or_default();
        
        position.number > 0 && self.get_current_price() <= price_min
    }

    fn get_current_price(&self) -> U256 {
        // This should be implemented to get the current market price
        // For now, returning a placeholder
        U256::from(0)
    }

    pub fn verify_tx_cost(&self, gas_estimate: U256, gas_price: U256, eth_price: f64, active_capital: U256) -> bool {
        let gas_cost = gas_estimate.saturating_mul(gas_price);
        let tx_cost_eth = gas_cost.as_u128() as f64 / 1e18;
        let tx_cost_usd = tx_cost_eth * eth_price;
        
        let max_cost_from_capital = active_capital.as_u128() as f64 * self.max_tx_cost_percent;
        
        tx_cost_usd <= self.max_tx_cost && tx_cost_usd <= max_cost_from_capital
    }
}
