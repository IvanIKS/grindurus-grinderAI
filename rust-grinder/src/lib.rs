use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument, HttpMethod, HttpResponse, TransformArgs};
use ic_cdk_macros::*;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable, BoundedStorable};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::borrow::Cow;

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

// Implement stable storage for Principal
impl Storable for Principal {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(self.as_slice().to_vec())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Principal::from_slice(&bytes)
    }
}

impl BoundedStorable for Principal {
    const MAX_SIZE: u32 = 29;
    const IS_FIXED_SIZE: bool = false;
}

// Core types with stable storage implementations
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Intent {
    pub account: Principal,
    pub expire: u64,
    pub pool_ids: Vec<u64>,
}

impl Storable for Intent {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.account.as_slice());
        bytes.extend_from_slice(&self.expire.to_le_bytes());
        bytes.extend_from_slice(&(self.pool_ids.len() as u64).to_le_bytes());
        for id in &self.pool_ids {
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let mut pos = 0;
        let account = Principal::from_slice(&bytes[pos..pos + 29]);
        pos += 29;
        let expire = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap());
        pos += 8;
        let len = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()) as usize;
        pos += 8;
        let mut pool_ids = Vec::with_capacity(len);
        for _ in 0..len {
            let id = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap());
            pool_ids.push(id);
            pos += 8;
        }
        Self {
            account,
            expire,
            pool_ids,
        }
    }
}

impl BoundedStorable for Intent {
    const MAX_SIZE: u32 = 1024; // Adjust this value based on your needs
    const IS_FIXED_SIZE: bool = false;
}

// Position
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub number: u64,
    pub number_max: u64,
    pub price_min: String,
    pub price_max: String,
    pub active_capital: String,
}

// Pool position
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
    let request = CanisterHttpRequestArgument {
        url: "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd".to_string(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers: vec![],
    };

    match ic_cdk::api::management_canister::http_request::http_request(request).await {
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
        Err(e) => Err(format!("Failed to fetch ETH price: {:?}", e))
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
