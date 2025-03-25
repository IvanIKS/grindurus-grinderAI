use candid::{CandidType, Deserialize, Principal};
use ic_web3::{
    contract::{Contract, Options},
    types::{Address, U256},
    Web3,
};
use std::str::FromStr;

#[derive(CandidType, Deserialize)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub intent_nft_address: String,
    pub pools_nft_address: String,
    pub grinder_ai_address: String,
}

pub struct EthereumInterface {
    web3: Web3,
    intent_nft: Contract,
    pools_nft: Contract,
    grinder_ai: Contract,
}

impl EthereumInterface {
    pub async fn new(config: EthereumConfig) -> Result<Self, String> {
        let web3 = Web3::new(&config.rpc_url)
            .map_err(|e| format!("Failed to create Web3 instance: {}", e))?;

        let intent_nft_address = Address::from_str(&config.intent_nft_address)
            .map_err(|e| format!("Invalid IntentNFT address: {}", e))?;

        let pools_nft_address = Address::from_str(&config.pools_nft_address)
            .map_err(|e| format!("Invalid PoolsNFT address: {}", e))?;

        let grinder_ai_address = Address::from_str(&config.grinder_ai_address)
            .map_err(|e| format!("Invalid GrinderAI address: {}", e))?;

        // Load contract ABIs
        // Note: In production, these would be properly loaded from files
        let intent_nft = Contract::new(web3.eth(), intent_nft_address, include_bytes!("../abis/IntentNFT.json"));
        let pools_nft = Contract::new(web3.eth(), pools_nft_address, include_bytes!("../abis/PoolsNFT.json"));
        let grinder_ai = Contract::new(web3.eth(), grinder_ai_address, include_bytes!("../abis/GrinderAI.json"));

        Ok(Self {
            web3,
            intent_nft,
            pools_nft,
            grinder_ai,
        })
    }

    pub async fn get_intent(&self, account: Address) -> Result<(U256, Vec<U256>), String> {
        self.intent_nft
            .query("getIntent", (account,), None, Options::default(), None)
            .await
            .map_err(|e| format!("Failed to get intent: {}", e))
    }

    pub async fn get_positions(&self, pool_id: U256) -> Result<(Position, Position), String> {
        self.pools_nft
            .query("getPositions", (pool_id,), None, Options::default(), None)
            .await
            .map_err(|e| format!("Failed to get positions: {}", e))
    }

    pub async fn total_supply(&self) -> Result<U256, String> {
        self.intent_nft
            .query("totalSupply", (), None, Options::default(), None)
            .await
            .map_err(|e| format!("Failed to get total supply: {}", e))
    }

    pub async fn owner_of(&self, token_id: U256) -> Result<Address, String> {
        self.intent_nft
            .query("ownerOf", (token_id,), None, Options::default(), None)
            .await
            .map_err(|e| format!("Failed to get owner: {}", e))
    }
}
