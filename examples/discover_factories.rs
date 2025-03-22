use std::sync::Arc;

use alloy::{
    eips::BlockNumberOrTag,
    primitives::{map::HashSet, U256},
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::Filter,
    sol,
    sol_types::SolEvent,
};
use eyre::Result;
use futures_util::StreamExt;

sol! {
  event Sync(uint112 indexed current, uint112 indexed delta);

  #[sol(rpc)]
  interface IUniV2Pair {
    function factory() returns(address);
  }

  #[sol(rpc)]
  interface IUniV2Factory {
    function allPairsLength() returns(uint);
    function allPairs(uint) returns(address);
  }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut known_factories = HashSet::new();
    println!("Discovering factories...");
    let rpc_url = "wss://ethereum-rpc.publicnode.com";

    let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;
    let provider = Arc::new(provider);

    let filter = Filter::new()
        .event(Sync::SIGNATURE)
        .from_block(BlockNumberOrTag::Latest);

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    while let Some(log) = stream.next().await {
        let ipair = IUniV2Pair::new(log.address(), provider.clone());

        let factory = match ipair.factory().call().await {
            Ok(factory) => factory._0,
            Err(_e) => {
                continue;
            }
        };

        let ifactory = IUniV2Factory::new(factory, provider.clone());
        let all_pairs_length = match ifactory.allPairsLength().call().await {
            Ok(all_pairs_length) => all_pairs_length._0,
            Err(_e) => {
                continue;
            }
        };

        if all_pairs_length == U256::ZERO {
            continue;
        }

        if known_factories.contains(&factory) {
            continue;
        }

        println!("Found new UniV2 factory: {:?}", factory);
        known_factories.insert(factory);
    }

    Ok(())
}
