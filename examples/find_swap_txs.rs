use eyre::Result;

use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    providers::{ext::DebugApi, Provider, ProviderBuilder, WsConnect},
    rpc::types::trace::geth::{
        CallFrame, GethDebugTracingCallOptions, GethDebugTracingOptions, GethTrace,
    },
    sol,
    sol_types::SolEvent,
};
use futures_util::StreamExt;

sol! {
  event Swap(address,uint256,uint256,uint256,uint256,address);
}

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = std::env::var("RPC_WS").expect("RPC_WS is not set");

    let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;

    let mut tx_stream = provider
        .subscribe_pending_transactions()
        .await?
        .into_stream()
        .fuse();

    let conf = "{\"tracer\": \"callTracer\", \"tracerConfig\": { \"withLog\": true }}";

    while let Some(tx_hash) = tx_stream.next().await {
        let Ok(Some(tx)) = provider.get_transaction_by_hash(tx_hash).await else {
            continue;
        };
        let tx = tx.into_request();

        let tracer_opts = serde_json::from_str::<GethDebugTracingOptions>(conf).unwrap();
        let tracing_opts = GethDebugTracingCallOptions::default();
        let tracing_opts = tracing_opts.with_tracing_options(tracer_opts);

        let trace = match provider
            .debug_trace_call(
                tx.clone(),
                BlockId::Number(BlockNumberOrTag::Latest),
                tracing_opts,
            )
            .await
        {
            Ok(trace) => trace,
            Err(_e) => {
                // println!("Error tracing tx: {}", e);
                continue;
            }
        };

        let trace = match trace {
            GethTrace::CallTracer(frame) => frame,
            _ => continue,
        };

        let mut all_calls = Vec::new();
        collect_calls(&trace, &mut all_calls);
        for trace in all_calls {
            let logs = trace.logs;

            for log in logs {
                let Some(topics) = log.topics else {
                    continue;
                };

                if topics.is_empty() {
                    continue;
                }
                let first_topic = topics[0];
                if first_topic == Swap::SIGNATURE_HASH {
                    println!("Found swap tx {}", &tx_hash);
                }
            }
        }
    }

    Ok(())
}

fn collect_calls(frame: &CallFrame, result: &mut Vec<CallFrame>) {
    result.push(frame.clone());

    for call in &frame.calls {
        collect_calls(call, result);
    }
}
