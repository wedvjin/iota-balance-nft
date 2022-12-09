use iota_wallet::{
    account_manager::AccountManager,
    iota_client::{constants::SHIMMER_COIN_TYPE, request_funds_from_faucet},
    secret::{stronghold::StrongholdSecretManager, SecretManager},
    ClientOptions, NftOptions, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()
        .name("wallet.log")
        .target_exclusions(&["h2", "hyper", "rustls"])
        .level_filter(log::LevelFilter::Debug);
    let config = fern_logger::LoggerConfig::build()
        .with_output(logger_output_config)
        .finish();
    fern_logger::logger_init(config).unwrap();
    
    let storage_path = "./wallet-db".to_string();
    let password = "shimmertestnetq123456789".to_string();
    let snapshot_path = std::path::Path::new("firefly-backup-testnet.stronghold");

    let stronghold_secret_manager = StrongholdSecretManager::builder()
        .password(&password)
        .build("wallet.stronghold").unwrap();

    let secret_manager = SecretManager::Stronghold(
        stronghold_secret_manager
    );
    
    let account_manager = AccountManager::builder()
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_secret_manager(secret_manager)
        .with_client_options(
            ClientOptions::new()
                .with_node("https://api.testnet.shimmer.network")?
        )
        .with_storage_path(&storage_path)
        .finish()
        .await
        .unwrap();

    if let Err(_err) = account_manager.restore_backup(snapshot_path.to_path_buf(), password).await {
        println!("Already imported");
    }

    let account_handles = &account_manager.get_accounts().await.unwrap();
    let account_handle = account_handles[0].clone();

    account_handle.sync(None).await?;
    println!("{:?}", account_handle.balance().await.unwrap());
    let address = account_handle.addresses().await.unwrap();
    println!("{:?}", &address[0].address().to_bech32());

    // let faucet_response =
    //     request_funds_from_faucet(&"https://faucet.testnet.shimmer.network".to_string(), &address[0].address().to_bech32()).await?;

    // println!("{}", faucet_response);

    //mint to self
    let nft_options = vec![NftOptions {
        address: Some(address[0].address().to_bech32()),
        sender: None,
        metadata: None,
        tag: None::<Vec<u8>>,
        issuer: Some(address[0].address().to_bech32()),
        immutable_metadata: Some(b"some NFT immutable metadata".to_vec()),
    }];
    let transaction = account_handle.mint_nfts(nft_options, None).await.unwrap();
    account_handle
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await
        .unwrap();
    println!("{:?}", transaction);
    
    account_handle.sync(None).await?;
    //mint to self second time
    let nft_options = vec![NftOptions {
        address: Some(address[0].address().to_bech32()),
        sender: None,
        metadata: None,
        tag: None::<Vec<u8>>,
        issuer: Some(address[0].address().to_bech32()),
        immutable_metadata: Some(b"some NFT immutable metadata 2".to_vec()),
    }];
    let transaction = account_handle.mint_nfts(nft_options, None).await.unwrap();
    //error
    account_handle
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await
        .unwrap();
    println!("{:?}", transaction);
    account_handle.sync(None).await?;

    Ok(())
}
