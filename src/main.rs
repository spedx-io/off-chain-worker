extern crate switchboard_solana;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use switchboard_solana::{FunctionRunner, Cluster};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetContext {
    max_leverage: Option<i32>,
    name: String,
    only_isolated: bool,
    sz_decimals: i32,
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

pub fn send_prices_to_solana(
    asset_contexts: Vec<AssetContext>,
) -> Result<Vec<Instruction>, Box<dyn std::error::Error>> {
    let program_id = Pubkey::from_str("6pbB1VzzU5VDtmQBkxmQNAcSbPnS9Vyon6kBb2YwgKeo")?;
    let price_data_account = Pubkey::from_str("GqnDxrf8ra4WFD9ZL8vWR5bj7zftBZT8ZJC7wB5w11Xs")?;

    let mut instruction_data = vec![];
    for asset in asset_contexts.iter() {
        if ["FTT", "HPOS", "WLD", "LDO", "GMX", "LINK", "dYdX"].contains(&asset.name.as_str()) {
            if let Some(max_leverage) = asset.max_leverage {
                instruction_data.extend_from_slice(&max_leverage.to_le_bytes());
            }
        }
    }
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
    let relevant_data: Vec<AssetContext> = asset_contexts
        .iter()
        .filter(|&asset| ["FTT", "HPOS", "WLD", "LDO", "GMX", "LINK", "dYdX"].contains(&asset.name.as_str()))
        .cloned()
        .collect();

    // Generate Solana instructions based on fetched data
    let instructions = if !relevant_data.is_empty() {
        println!("Relevant Data: {:?}", relevant_data);
        send_prices_to_solana(relevant_data).unwrap()
    } else {
        println!("Relevant data not found");
        vec![]
    };

    // Emit the instructions to the Switchboard function
    runner.emit(instructions).await.unwrap();
}
