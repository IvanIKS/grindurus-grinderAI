use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk_macros::*;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;

// Operation types matching the original JavaScript enum
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum Operation {
    LongBuy = 0,
    LongSell = 1,
    HedgeSell = 2,
    HedgeRebuy = 3,
    Rebalance = 4,
    Invest = 5,
    Divest = 6,
}

// Core types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Intent {
    pub account: Principal,
    pub expire: u64,
    pub pool_ids: Vec<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub number: u64,
    pub number_max: u64,
    pub price_min: String,
    pub price_max: String,
    pub active_capital: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct PoolPosition {
    pub long: Position,
    pub hedge: Position,
}

// Gas configuration
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct GasConfig {
    pub multiplier_numerator: u64,
    pub multiplier_denominator: u64,
}

// State management
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static INTENTS: RefCell<StableBTreeMap<Principal, Intent, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    static ETH_PRICE: RefCell<f64> = RefCell::new(2700.0);
    static GAS_CONFIG: RefCell<GasConfig> = RefCell::new(GasConfig {
        multiplier_numerator: 14,
        multiplier_denominator: 10,
    });
}

// Constants
const MAX_TX_COST_PERCENT: f64 = 0.0007; // 0.07% from active capital
const MAX_TX_COST_USD: f64 = 0.05; // $0.05

// Helper functions
#[query]
fn get_eth_price() -> f64 {
    ETH_PRICE.with(|price| *price.borrow())
}

#[update]
async fn update_eth_price() -> Result<f64, String> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";
    
    match ic_cdk::api::management_canister::http_request::http_request(
        url.into(),
        "GET".into(),
        None,
        vec![],
    ).await {
        Ok((response,)) => {
            if response.status == 200 {
                if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&response.body) {
                    if let Some(price) = data["ethereum"]["usd"].as_f64() {
                        ETH_PRICE.with(|p| *p.borrow_mut() = price);
                        return Ok(price);
                    }
                }
            }
            Ok(2700.0) // Default fallback price
        }
        Err(e) => Err(format!("Failed to fetch ETH price: {}", e))
    }
}

#[query]
fn get_intent(account: Principal) -> Option<Intent> {
    INTENTS.with(|intents| intents.borrow().get(&account))
}

#[update]
async fn index_all_intent_nft() -> Result<Vec<Intent>, String> {
    // This would need to be implemented using the Ethereum interface
    // For now, we'll return a placeholder
    Ok(Vec::new())
}

// Timer setup for periodic updates
#[post_upgrade]
fn post_upgrade() {
    ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(60), || {
        ic_cdk::spawn(async {
            let _ = update_eth_price().await;
        });
    });
}
