# Uniswap V2 long-tail MEV strategy

Examples for [https://pawelurbanek.com/uniswap-mev-profit](https://pawelurbanek.com/uniswap-mev-profit)

Usage:

```bash
forge test -vv --fork-url https://eth.merkle.io --fork-block-number 21986784 --via-ir
cargo run --example discover_factories
cargo run --example find_swap_txs
```
