extern crate switchboard_solana;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::env;
use switchboard_solana::{FunctionRunner, Cluster};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetContext {
    max_leverage: Option<i32>,
    name: String,
    only_isolated: bool,
    sz_decimals: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BirdeyePrice {
    price: f64,
}

pub async fn fetch_hyperliquid_price() -> Result<Vec<AssetContext>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.hyperliquid.xyz/info")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "type": "metaAndAssetCtxs"
        }))
        .send()
        .await?;

    let api_response: Vec<serde_json::Value> = res.json().await?;
    for item in &api_response {
        if let Some(universe) = item.get("universe") {
            let asset_contexts: Vec<AssetContext> = serde_json::from_value(universe.clone())?;
            return Ok(asset_contexts);
        }
    }
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Universe not found")))
}

pub async fn fetch_birdeye_price(address: &str, api_key: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .get(&format!("https://public-api.birdeye.so/public/price?address={}", address))
        .header("X-API-KEY", api_key)
        .header("x-chain", "solana")
        .send()
        .await?;

    let api_response: BirdeyePrice = res.json().await?;
    Ok(api_response.price)
}

pub fn send_prices_to_solana(
    asset_contexts: Vec<AssetContext>,
    ethonsol_price: f64,
    blze_price: f64,
) -> Result<Vec<Instruction>, Box<dyn std::error::Error>> {
    let program_id = Pubkey::from_str("7VwEKCGjDEH9hdYX1mYVwRLeQA1DeFti6qk5bto3QEqL")?;
    let price_data_account = Pubkey::from_str("GqnDxrf8ra4WFD9ZL8vWR5bj7zftBZT8ZJC7wB5w11Xs")?;

    let mut instruction_data = vec![];
    for asset in asset_contexts.iter().filter_map(|asset| asset.max_leverage) {
        instruction_data.extend_from_slice(&asset.to_le_bytes());
    }

    // Add additional prices from Birdeye
    let ethonsol_price_u64 = (ethonsol_price * 1e6) as u64; // Convert to appropriate unit
    let blze_price_u64 = (blze_price * 1e6) as u64; // Convert to appropriate unit

    instruction_data.extend_from_slice(&ethonsol_price_u64.to_le_bytes());
    instruction_data.extend_from_slice(&blze_price_u64.to_le_bytes());

    let instruction = Instruction {
        program_id,
        accounts: vec![solana_sdk::instruction::AccountMeta::new(
            price_data_account,
            false,
        )],
        data: instruction_data,
    };

    Ok(vec![instruction])
}

#[tokio::main(worker_threads = 12)]
async fn main() {
    // Initialize the FunctionRunner
    let runner = match FunctionRunner::new_from_cluster(Cluster::Devnet, None) {
        Ok(runner) => runner,
        Err(e) => {
            eprintln!("Failed to initialize FunctionRunner: {:?}", e);
            return;
        }
    };

    // Fetch asset contexts from HyperLiquid
    let asset_contexts = fetch_hyperliquid_price().await.unwrap();
    let hpos_data = asset_contexts.iter().find(|&asset| asset.name == "HPOS");

    // Fetch asset context from Birdeye
    let api_key = env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY must be set");
    let ethonsol_price = fetch_birdeye_price("4EqmCRdEqcv8YPvQ77NuhuFQufHaBFM6XHGxPuachgLW", &api_key).await.unwrap();
    let blze_price = fetch_birdeye_price("BLZEEuZUBVqFhj8adcCFPJvPVCiCyVmh3hkJMrU8KuJA", &api_key).await.unwrap();


    // Generate Solana instructions based on fetched data
    let instructions = send_prices_to_solana(asset_contexts, ethonsol_price, blze_price).unwrap();

    // Emit the instructions to the Switchboard function
    runner.emit(instructions).await.unwrap();
}
