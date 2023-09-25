use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, read_keypair_file, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetContext {
    maxLeverage: Option<i32>,
    name: String,
    onlyIsolated: bool,
    szDecimals: i32,
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
) -> Result<(), Box<dyn std::error::Error>> {
    let rpc_client = RpcClient::new("https://api.devnet.solana.com");
    let program_id = Pubkey::from_str("6pbB1VzzU5VDtmQBkxmQNAcSbPnS9Vyon6kBb2YwgKeo")?;
    let price_data_account = Pubkey::from_str("GqnDxrf8ra4WFD9ZL8vWR5bj7zftBZT8ZJC7wB5w11Xs")?;
    let keypair = read_keypair_file("/Users/akshatsharma/octane/keys/octane.json")?;

    let mut instruction_data = vec![];
    for asset in asset_contexts.iter().filter_map(|asset| asset.maxLeverage) {
        instruction_data.extend_from_slice(&asset.to_le_bytes());
    }

    let instruction = Instruction {
        program_id,
        accounts: vec![solana_sdk::instruction::AccountMeta::new(
            price_data_account,
            false,
        )],
        data: instruction_data,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&keypair.pubkey()));
    let blockhash = rpc_client.get_latest_blockhash()?;
    transaction.try_sign(&[&keypair], blockhash)?;

    let result = rpc_client.send_and_confirm_transaction(&transaction);

    match result {
        Ok(_) => {
            println!("Transaction succeeded");
            Ok(())
        }
        Err(err) => {
            println!("Transaction failed: {:?}", err);
            Err(Box::new(err))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let asset_contexts = fetch_hyperliquid_price().await?;
    let hpos_data = asset_contexts.iter().find(|&asset| asset.name == "HPOS");

    match hpos_data {
        Some(data) => {
            println!("HPOS Data: {:?}", data);
            send_prices_to_solana(vec![data.clone()])?;
        },
        None => {
            println!("HPOS data not found");
        }
    }

    Ok(())
}
