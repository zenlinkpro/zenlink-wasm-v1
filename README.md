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
    sudo docker run --rm -v "$PWD":/build -w /build zenlink_contract_builder cargo +nightly-2020-10-06-x86_64-unknown-linux-gnu contract build
    sudo docker run --rm -v "$PWD":/build -w /build zenlink_contract_builder cargo +nightly-2020-10-06-x86_64-unknown-linux-gnu contract generate-metadata
  Then we can find the erc20.wasm and metadata.json in the target folder. 

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

## Test

* **connect the canvas node with web.**

[https://paritytech.github.io/canvas-ui/#/upload](https://paritytech.github.io/canvas-ui/#/upload)

* **upload contract**

![图片](https://uploader.shimo.im/f/ZhbssN1w5IL9nRvo.png!thumbnail)

When we upload all the three contract. We will deploy them. Click the "deploy" label. We can see the contracts on the chain.

![图片](https://uploader.shimo.im/f/yuokMk6PVsQUKN5c.png!thumbnail)

* **Deploy Erc20 contract**

Click the "Deploy" label .

![图片](https://uploader.shimo.im/f/y54Gb3DZMFdtVlIx.png!thumbnail)

We can give this erc20 contract a name like "token_a". And we can deploy the second erc20 token named "token_b".

* **Deploy factory contract**

We deploy the factory contract just by click the "deploy" lable.

* **Execute factory contract to creating trading pair**

![图片](https://uploader.shimo.im/f/QvxksWguJTVkRryz.png!thumbnail)

Call the **initialize_factory** interface. "exchangeTemplateAddress" represent the hash code of the exchange contract on the chain. We can copy it from the deploy panel.

![图片](https://uploader.shimo.im/f/71Q0XxOZl8oM8hB8.png!thumbnail)

Then we we can call the "create_exchange" interface.

![图片](https://uploader.shimo.im/f/f4ZVkDys7OCXBIIt.png!thumbnail)

In this picture, We create a token_A-Dot trading pair. Add we privide 10unit Dot and 10Uint token_a to the liquidity pool.

After create the trading pair successfully. We Call the**get_exchange**interface to get the exchange contract accountId.

![图片](https://uploader.shimo.im/f/ZiZo2m2esJp0OLq7.png!thumbnail)

So the exchange pair has been instantiated. We can add it to web browser.

![图片](https://uploader.shimo.im/f/QbHlpdpLJ8gwvSjD.png!thumbnail)

* **Trade on token_a-Dot pair**

We can invoke some transfer in this pair. Like add_liquidity/ remove_liquidity, dot-token_a swap and so on.

![图片](https://uploader.shimo.im/f/cc5hQctxPXOAAQCb.png!thumbnail)

* **Trade between on tokens.**

Now, we deploy a erc20 named token_c. Then we can create the token_c-Dot trading pair. The steps are same as creating token_a.

After that, we can trade between in tokens.

![图片](https://uploader.shimo.im/f/t7RlwIQmauhehIHI.png!thumbnail)

After trading between on tokens successfully, We can call the **balance_of**  interface of erc20 contract to check the balance of token which we get in this transaction。

![图片](https://uploader.shimo.im/f/uKLuCZ1BugVKhVrI.png!thumbnail)

Please attention to the "Call results" of "balance_of". It my be a hexadecimal number or just a bug. After copy it to a txt editor, It wll show nornally.
