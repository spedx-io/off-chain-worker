use reqwest;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::Instruction,
};
use std::str::FromStr;

pub async fn fetch_binance_price(symbol: &str) -> Result<u64, reqwest::Error> {
    let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", symbol);
    let response: serde_json::Value = reqwest::get(&url).await?.json().await?;
    let price: u64 = response["price"].as_str().unwrap().parse().unwrap();
    Ok(price)
}

pub fn send_prices_to_solana(btc_price: u64, eth_price: u64) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Solana client
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com");

    // Your Solana program ID and price data account
    let program_id = Pubkey::from_str("EVkRuHAH76x6GYsx4isftKWCseJTCVtQYvh64MEA6q2U")?;
    let price_data_account = Pubkey::from_str("AStbVxPR31uzdqSaF96cQ9oM1J1UWL31kfA6gYQKjibs")?;

    // Your keypair
    let keypair = Keypair::new();

    // Create instruction data
    let mut instruction_data = vec![];
    instruction_data.extend_from_slice(&btc_price.to_le_bytes());
    instruction_data.extend_from_slice(&eth_price.to_le_bytes());

    // Create the instructions
    let instructions: Vec<Instruction> = vec![
        Instruction {
            program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(price_data_account, false),
            ],
            data: instruction_data,
        },
    ];

// Create the message
    let message = solana_sdk::message::Message::new_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
    );

// Create the transaction
    let mut transaction = Transaction::new_with_payer(
        &message,
        Some(&keypair.pubkey()),
    );
    transaction.try_sign(&[&keypair], rpc_client.get_recent_blockhash()?.0)?;

// Send the transaction
    let result = rpc_client.send_and_confirm_transaction(&transaction);

    match result {
        Ok(_) => {
            println!("Transaction succeeded");
            Ok(())
        },
        Err(err) => {
            println!("Transaction failed: {:?}", err);
            Err(Box::new(err))
        },
    }
}
