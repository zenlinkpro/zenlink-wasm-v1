**This repo is deprecated!**

## Contracts
### erc20

It's a basic ERC20 token contract is a fixed supply token. During contract deployment, all the tokens will be automatically given to the contract creator. It is then up to that user to distribute those tokens to other users as they see fit.

### exchange

Exchange is the core contract of ZenLink Dex Protocol. It implements the following interfaces:

    Initializing token trading pair.
    Token swap.
    Adding/extracting liquidity.
    Defining the liquidity constant function used throughout the protocol.

### factory

The factory contract can be used to create exchange contracts for any ERC20 token that does not already have one. It also functions as a registry of ERC20 tokens that have been added to the system, and the exchange with which they are associated.

## Setup
### Docker
  We recommend using docker to run substrate node and compile the contracts.
    
    docker run -p 9944:9944 zenlinkpro/dex:zenlink-canvas-node

  Compile the contracts using docker.

    cd erc20
    sudo docker run --rm -v "$PWD":/build -w /build zenlinkpro/dex:zenlink_contract_builder cargo +nightly-2020-10-06-x86_64-unknown-linux-gnu contract build
    sudo docker run --rm -v "$PWD":/build -w /build zenlinkpro/dex:zenlink_contract_builder cargo +nightly-2020-10-06-x86_64-unknown-linux-gnu contract generate-metadata
  Then we can find the erc20.wasm and metadata.json in the target folder. 
    Because the factory project depend on the exchange project. So we must run the command in factroy project parent folder.
    
    sudo docker run --rm -v "$PWD":/build -w /build/factory zenlinkpro/dex:zenlink_contract_builder cargo +nightly-2020-10-06-x86_64-unknown-linux-gnu contract build

### Build test environment manually

* Build rust development environment.
* WebAssembly Compilation
```plain
  rustup install nightly-<yyyy-MM-dd>
  rustup target add wasm32-unknown-unknown --toolchain nightly-<yyyy-MM-dd>
  rustup component add rust-src --toolchain nightly-<yyyy-MM-dd>
  rustup target add wasm32-unknown-unknown --toolchain stable
```
* Install the canvan node
```plain
cargo install canvas-node --git https://github.com/paritytech/canvas-node.git --tag v0.1.4 --force --locked
```
ink! CLI
```plain
cargo install cargo-contract --vers 0.7.1 --force --locked
```
#### Complie Contract

we should build 3 contract，erc20, exchange and factory.

```rust
cargo +nightly-<yyyy-MM-dd>-x86_64-unknown-linux-gnu contract build
cargo +nightly-<yyyy-MM-dd>-x86_64-unknown-linux-gnu contract generate-metadata
```
We will see *.wasm and metadata.json in the folder named target.

## Unit test
  We can run unit test in exchange project. 

  Docker

    cd exchange
    sudo docker run --rm -v "$PWD":/build -w /build zenlinkpro/dex:zenlink_contract_builder cargo test
  Manually

    cd exchange
    cargo test
