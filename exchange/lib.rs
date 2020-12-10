#![cfg_attr(not(feature = "std"), no_std)]

pub use self::exchange::Exchange;

use ink_lang as ink;

#[ink::contract]
mod exchange {

    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{
        collections::HashMap as StorageHashMap,
    };

    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_env::{
         call,
         DefaultEnvironment,
         call::{ build_call, ExecutionInput, utils::ReturnType,}
    };

    #[ink(event)]
    pub struct CreateExchange {
        #[ink(topic)]
        exchange       : AccountId,
    } 

    #[ink(event)]
    pub struct TokenPurchase {
        #[ink(topic)]
        buyer         : AccountId,
        #[ink(topic)]
        dot_sold     : Balance,
        #[ink(topic)]
        tokens_bought : Balance,
    }

    #[ink(event)]
    pub struct DotPurchase{
        #[ink(topic)]
        buyer       : AccountId,
        #[ink(topic)]
        tokens_sold : Balance,
        #[ink(topic)]
        dot_bought : Balance,
    }

    #[ink(event)]
    pub struct AddLiquidity{
        #[ink(topic)]
        provider      : AccountId,
        #[ink(topic)]
        dot_ammount  : Balance,
        #[ink(topic)]
        token_ammount : Balance,
    }

    #[ink(event)]
    pub struct RemoveLiquidity{
        #[ink(topic)]
        provider      : AccountId,
        #[ink(topic)]
        dot_ammount  : Balance,
        #[ink(topic)]
        token_ammount : Balance,
    }

    #[ink(event)]
    pub struct Transfer{
        #[ink(topic)]
        from   : AccountId,
        #[ink(topic)]
        to     : AccountId,
        #[ink(topic)]
        value  : Balance,
    }

    #[ink(event)]
    pub struct Approval{
        #[ink(topic)]
        owner : AccountId,
        #[ink(topic)]
        spender:AccountId,
        #[ink(topic)]
        value : Balance,
    }
    
    #[ink(storage)]
    pub struct Exchange {
        pub name : Hash,
        pub symbol : Hash,
        pub decimals : u32,
        //total liquidity
        pub total_supply : Balance,

        pub balances : StorageHashMap<AccountId, Balance>, 
        allowances: StorageHashMap<(AccountId, AccountId), Balance>,
        //address of the ERC20 token traded on this contract
        pub token : AccountId,
        //the address of factory contract.
        factory : AccountId,
        gas_limit :u64,
        exchange_account_id: AccountId, 
    }

    impl Exchange {

        /// Constructor of the Exchange contract
        /// 
        /// 
        /// NOTE: Exchange contract show onoly be instantiated by Factory.
        /// 
        /// #Params
        /// 
        /// - `token_account_id`: AccountId of a Erc20 token which trade on this  contract
        /// - `factory_account_id`: AccountId of the Factory which instantiate this  contract
        /// - `deployer`: Account deploy this contract and provide initial liquidity
        /// - `token_ammount`: Ammount of token the deployer will transfer from token_account_id to this contract account
        #[ink(constructor)]
        pub fn new(token_account_id: AccountId, factory_account_id : AccountId, deployer : AccountId,token_ammount : Balance) -> Self {
            let mut instance = Self{
                name : Hash::default(), 
                symbol : Hash::default(),
                decimals : 18,
                total_supply : Self::env().balance(),
                balances :StorageHashMap::new(),
                allowances : StorageHashMap::new(),
                token : token_account_id,  
                factory : factory_account_id,         
                gas_limit : 507085500000,
                exchange_account_id : Self::env().account_id(),
            };

            //The contract deployer show transfer some token to this contract. if success, the contract deployer become first liquidity provider.
            if token_ammount > 0{
                instance.token_transfer_from(deployer, instance.exchange_account_id, token_ammount);
            }
            instance.balances.insert(deployer, instance.total_supply);

            instance
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default(), AccountId::default(), AccountId::default(), Balance::default())
        }

        /// Deposit Dot and Tokens (self.token) at current ratio to mint lp tokens.
        /// 
        /// Return The amount of lp minted 
        /// 
        /// #Params
        /// 
        /// - `min_liquidity`:  Minimum number of lp sender will mint if total lp supply is greater than 0.
        /// - `max_tokens`: Maximum number of tokens deposited. Deposits max amount if total lp supply is 0.
        /// - `deadline`: Time after which this transaction can no longer be executed.
        #[ink(message, payable,selector = "0xDEADBEEF")]
        pub fn add_liquidity(&mut self, min_liquidity: u128, max_tokens: u128, deadline: Timestamp) ->u128{
            let transfferred_value = self.env().transferred_balance();
            assert!(deadline >= self.env().block_timestamp() && max_tokens > 0 && transfferred_value > 0);
            let total_liquidity = self.total_supply;
            let caller = self.env().caller();
            if total_liquidity > 0{
                assert!(min_liquidity > 0);
                let dot_reserve = self.env().balance() - transfferred_value;
                let token_reserve = self.token_balance();
                let token_ammount = transfferred_value * token_reserve / dot_reserve + 1;
                let liquidity_minted = transfferred_value * total_liquidity / dot_reserve;

                assert!(max_tokens >= token_ammount && liquidity_minted >= min_liquidity);

                let caller_luquidity = self.balances.get(&caller).unwrap_or(&0u128).clone();
                self.balances.insert(caller, caller_luquidity + liquidity_minted);

                self.total_supply = total_liquidity + liquidity_minted;
                assert!(self.token_transfer_from(caller, self.exchange_account_id, token_ammount));

                self.env().emit_event( AddLiquidity {
                    provider : caller,
                    dot_ammount : transfferred_value,
                    token_ammount: token_ammount,
                });

                self.env().emit_event( Transfer {
                    from : AccountId::default(),
                    to : caller,
                    value : liquidity_minted,
                });

                liquidity_minted
            }else{
                let token_ammount = max_tokens;

                let initial_liquidity = self.env().balance();
                self.total_supply = initial_liquidity;
                self.balances.insert(caller, initial_liquidity);

                assert!(self.token_transfer_from(caller, self.exchange_account_id, token_ammount));

                self.env().emit_event( AddLiquidity {
                    provider : caller,
                    dot_ammount : transfferred_value,
                    token_ammount: token_ammount,
                });

                self.env().emit_event( Transfer {
                    from : AccountId::default(),
                    to : caller,
                    value : initial_liquidity,
                });

                initial_liquidity 
            }
        }

        /// Burn lp tokens to withdraw Dot and Tokens at current ratio.
        /// 
        /// Return The amount of Dot and Tokens withdrawn.
        /// 
        /// #Params
        /// 
        /// - `ammount`: Amount of lp burned.
        /// - `min_dot`: Minimum Dot withdrawn.
        /// - `min_tokens`: Minimum Tokens withdrawn
        /// - `deadline`: Time after which this transaction can no longer be executed.
        #[ink(message)]
        pub fn remove_liquidity(&mut self, ammount : Balance,min_dot : Balance, min_token : Balance, deadline : Timestamp ) ->(Balance, Balance){
            assert!((ammount > 0 && deadline >= self.env().block_timestamp()) && (min_dot > 0  && min_token > 0));
            let caller = self.env().caller();
            let total_liquidity = self.total_supply;

            assert!(total_liquidity > 0);

            let token_reserve = self.token_balance();
            let dot_ammount  = ammount * self.env().balance() / total_liquidity;
            let token_ammount = ammount * token_reserve / total_liquidity;

            assert!((dot_ammount > min_dot) && (token_ammount > min_token));
            let caller_luquidity = *self.balances.get(&caller).unwrap();
            assert!(caller_luquidity >= ammount);

            self.balances.insert(caller, caller_luquidity - ammount);
            self.total_supply = total_liquidity - ammount;

            self.env().transfer(caller, dot_ammount).expect("transfer error");
            self.token_transfer(caller, token_ammount);

            self.env().emit_event( RemoveLiquidity {
                provider : caller,
                dot_ammount,
                token_ammount,
            });

            self.env().emit_event( Transfer {
                from :  caller,
                to : AccountId::default(),
                value : ammount,
            });

            (dot_ammount, token_ammount)
        }   

        pub fn input_price(&self, input_amount : Balance, input_reserve : Balance, output_reserve : Balance) -> Balance{
            assert!(input_reserve > 0 && output_reserve > 0);
            let input_ammount_with_fee = input_amount * 997u128;
            let numerator = input_ammount_with_fee * output_reserve;
            let denominator = (input_reserve * 1000u128) + input_ammount_with_fee;
            numerator / denominator
        }

        pub fn output_price(&self, output_ammount: Balance, input_reserve : Balance, output_reserve : Balance) -> Balance{
            assert!(input_reserve > 0 && output_reserve > 0);
            let numerator = input_reserve * output_ammount * 1000u128;
            let denomiator = (output_reserve - output_ammount) *997u128;
            numerator / denomiator + 1
        }
  
        fn dot_to_token_input(&mut self, dot_sold: Balance, min_tokens: Balance, deadline: Timestamp, 
                                buyer: AccountId, recipient: AccountId) -> Balance{
            assert!(deadline >= self.env().block_timestamp() && (dot_sold > 0) && (min_tokens > 0));
            let token_reserve = self.token_balance();
            let tokens_bought = self.input_price(dot_sold, self.env().balance() - dot_sold, token_reserve);
            assert!(tokens_bought >= min_tokens);
            assert!(self.token_transfer(recipient, tokens_bought));
            self.env().emit_event( TokenPurchase {
                buyer,
                dot_sold,
                tokens_bought,
            });
            tokens_bought
        }

        /// Convert Dot to Tokens.
        /// 
        /// Return bought token
        /// 
        /// # Params
        /// 
        /// - `min_token`: Minimum Tokens bought
        /// - `deadline ` : Time after which this transaction can no longer be executed
        #[ink(message, payable)]
        pub fn dot_to_token_swap_input(&mut self, min_tokens : Balance,deadline :Timestamp) ->Balance{
            let caller = self.env().caller();
            let transferred_balance = self.env().transferred_balance();
            self.dot_to_token_input(transferred_balance, min_tokens, deadline, caller, caller)
        }

        /// Convert DOT to Tokens and transfer the token to a specified account
        /// 
        /// Return bought token
        /// 
        /// # Params
        /// 
        /// - `min_token`: Minimum Tokens bought
        /// - `deadline ` : Time after which this transaction can no longer be executed
        /// - 'recipient' : AcccountId will get the transferred token
        #[ink(message, payable, selector = "0xa0a8e619")]
        pub fn dot_to_token_transfer_input(&mut self,min_tokens : Balance, deadline : Timestamp, recipient: AccountId) ->Balance{
            assert!(recipient != self.exchange_account_id && recipient != AccountId::default());
            let transferred_balance = self.env().transferred_balance();
            self.dot_to_token_input(transferred_balance, min_tokens, deadline, self.env().caller(), recipient)
        }
        
        fn dot_to_token_output(&mut self, tokens_bought : Balance, max_dot :Balance, deadline : Timestamp, buyer: AccountId, recipient: AccountId) -> Balance{
            assert!(deadline >= self.env().block_timestamp() && tokens_bought >0 && max_dot > 0);

            let tokens_reserve = self.token_balance();
            let dot_sold = self.output_price(tokens_bought, self.env().balance() - max_dot, tokens_reserve);
            let dot_refund = max_dot - dot_sold;
            if dot_refund > 0 {
                self.env().transfer(buyer, dot_refund).expect("transfer error");
            }
            assert!(self.token_transfer(recipient, tokens_bought));
            self.env().emit_event( TokenPurchase {
                buyer,
                dot_sold,
                tokens_bought,
            });
            dot_sold
        }

        /// Convert Dot to Tokens
        /// 
        /// NOTE: User specifies maximum input(dot) and exact output.
        /// 
        /// #Params
        /// 
        /// - `tokens_bought`: Amount of tokens bought.
        /// - `deadline ` : Time after which this transaction can no longer be executed
        #[ink(message, payable)]
        pub fn dot_to_token_swap_output(&mut self, tokens_bought : Balance, deadline : Timestamp) -> Balance{
            let caller = self.env().caller(); 
            let transferred_balance = self.env().transferred_balance();
            self.dot_to_token_output(tokens_bought,transferred_balance, deadline, caller, caller)
        }

        /// Convert Dot to Tokens
        /// 
        /// NOTE: User specifies maximum input and exact output.
        /// 
        /// #Params
        /// 
        /// - `tokens_bought`: Amount of tokens bought.
        /// - `deadline ` : Time after which this transaction can no longer be executed
        /// - `recipient`: AcccountId will get the transferred token
        #[ink(message, payable, selector="0x0783f403")]
        pub fn dot_to_token_transfer_output(&mut self, tokens_bought : Balance, deadline : Timestamp, recipient: AccountId) ->Balance{
            assert!(recipient != self.exchange_account_id && recipient != AccountId::default());
            let transferred_balance = self.env().transferred_balance();
            self.dot_to_token_output(tokens_bought, transferred_balance, deadline,  self.env().caller(), recipient)
        }

        fn token_to_dot_input(&mut self, tokens_sold : Balance, min_dot : Balance, deadline : Timestamp, buyer: AccountId, recipient: AccountId)->Balance{
            assert!(deadline >= self.env().block_timestamp() && tokens_sold > 0 && min_dot > 0);
            let token_reserve = self.token_balance();
            let dot_bought = self.input_price(tokens_sold, token_reserve, self.env().balance());
            assert!(dot_bought > min_dot);

            self.env().transfer(recipient, dot_bought).expect("transfer error");

            assert!(self.token_transfer_from(buyer, self.exchange_account_id, tokens_sold));

            self.env().emit_event( DotPurchase {
                buyer,
                tokens_sold,
                dot_bought,
            });
            dot_bought
        }

        /// Convert Tokens to Dot.
        /// 
        /// #params:
        /// - `tokens_sold`:Amount of Tokens sold.
        /// - `min_dot`: Minimum Dot purchased.
        /// - `deadline`: Time after which this transaction can no longer be executed
        #[ink(message)]
        pub fn token_to_dot_swap_input(&mut self, tokens_sold : Balance, min_dot : Balance, deadline : Timestamp) -> Balance{
            let caller = self.env().caller();
            self.token_to_dot_input(tokens_sold, min_dot, deadline, caller, caller)
        }

        /// Convert Tokens to Dot.
        /// 
        /// #params:
        /// 
        /// - `tokens_sold`:Amount of Tokens sold.
        /// - `min_dot`: Minimum Dot purchased.
        /// - `deadline`: Time after which this transaction can no longer be executed.
        /// - `recipient`: AcccountId will get the transferred dot.
        #[ink(message)]
        pub fn token_to_dot_transfer_input(&mut self, tokens_sold : Balance, min_dot : Balance, deadline : Timestamp, recipient: AccountId) ->Balance{
            assert!(self.exchange_account_id != recipient);
            self.token_to_dot_input(tokens_sold, min_dot, deadline, self.env().caller(), recipient)
        }

        fn token_to_dot_output(&mut self, dot_bought : Balance, max_tokens : Balance, deadline : Timestamp, buyer: AccountId, recipient: AccountId)->Balance{
            assert!(deadline >= self.env().block_timestamp() && dot_bought > 0);
            let token_reserve = self.token_balance();
            let tokens_sold = self.output_price(dot_bought, token_reserve, self.env().balance());
            assert!(max_tokens >= tokens_sold);

            self.env().transfer(recipient, dot_bought).expect("transfer error");

            assert!(self.token_transfer_from(buyer, self.exchange_account_id, tokens_sold));

            self.env().emit_event( DotPurchase {
                buyer,
                tokens_sold,
                dot_bought,
            });
            tokens_sold
        }

        /// Convert Tokens to Dot.
        /// 
        /// Return Amount of Tokens sold.
        /// 
        /// #Params:
        /// 
        /// - `dot_bought`: Amount of Dot purchased.
        /// - `max_tokens`: Maximum Tokens sold.
        /// - `deadline`: Time after which this transaction can no longer be executed.
        #[ink(message)]
        pub fn token_to_dot_swap_output(&mut self, dot_bought : Balance, max_tokens : Balance, deadline : Timestamp)->Balance{
            let caller = self.env().caller();
            self.token_to_dot_output(dot_bought, max_tokens, deadline, caller, caller)
        }

        /// Convert Tokens to Dot.
        /// 
        /// Return Amount of Tokens sold.
        /// 
        /// #Params:
        /// 
        /// - `dot_bought`: Amount of Dot purchased.
        /// - `max_tokens`: Maximum Tokens sold.
        /// - `deadline`: Time after which this transaction can no longer be executed.
        /// - `recipient`: AcccountId will get the transferred dot.
        #[ink(message)]
        pub fn token_to_dot_transfer_output(&mut self, dot_bought : Balance, max_tokens : Balance, deadline : Timestamp, recipient: AccountId) ->Balance{
            assert!(recipient != self.exchange_account_id && recipient != AccountId::default());
            let caller = self.env().caller();
            self.token_to_dot_output(dot_bought, max_tokens, deadline, caller, recipient)
        }

        fn token_to_token_input(&mut self, tokens_sold : Balance, min_tokens_bought : Balance, min_dot_bought : Balance,
             deadline : Timestamp, buyer : AccountId, recipient :AccountId, exchange_addr : AccountId) ->Balance{
            assert!(deadline >= self.env().block_timestamp() && tokens_sold > 0 && (min_dot_bought > 0 && min_tokens_bought > 0));

            let token_reserve =self.token_balance();
            let dot_bought =  self.input_price(tokens_sold, token_reserve, self.env().balance());
            
            assert!(dot_bought >= min_dot_bought);
            assert!(self.token_transfer_from(buyer, self.exchange_account_id, tokens_sold));
            
            //Exchange(exchange_addr) call dot_to_token_transfer_input function //0xa0a8e619
            let selector_balance_of = call::Selector::new([0xa0, 0xa8,0xe6, 0x19]);
            let  tokens_bought =  build_call::<DefaultEnvironment>()
                .callee(exchange_addr)
                .gas_limit(self.gas_limit)
                .transferred_value(dot_bought)
                .exec_input(
                    ExecutionInput::new(selector_balance_of.into()).push_arg(min_tokens_bought).push_arg(deadline).push_arg(recipient),
                ).returns::<ReturnType<Balance>>().fire().unwrap();
            
            self.env().emit_event( DotPurchase {
                buyer,
                tokens_sold,
                dot_bought,
            });

            tokens_bought
        }
        
        /// Convert Tokens (self.token) to Tokens (token_addr).
        /// 
        /// Return the Ammount of Token(token_addr) bought
        /// 
        /// #Params
        /// - `tokens_sold`: Amount of Tokens sold
        /// - `min_tokens_bought`: Minimum Tokens (token_addr) purchased.
        /// - `min_dot_bought`: Minimum Dot purchased as intermediary.
        /// - `deadline`: Time after which this transaction can no longer be executed.
        /// - `token_addr`: token_addr The address of the token being purchased.
        #[ink(message)]
        pub fn token_to_token_swap_input(&mut self,tokens_sold: Balance, min_tokens_bought : Balance, 
            min_dot_bought : Balance, deadline : Timestamp, token_addr : AccountId) ->Balance{
                let exchange_addr = self.exchange_from_factory(token_addr);
                let caller = self.env().caller();
                self.token_to_token_input(tokens_sold, min_tokens_bought, min_dot_bought, deadline,
                    caller, caller, exchange_addr)
        }

        /// Convert Tokens (self.token) to Tokens (token_addr).
        /// 
        /// Return the Ammount of Token(token_addr) bought
        /// 
        /// #Params
        /// - `tokens_sold`: Amount of Tokens sold
        /// - `min_tokens_bought`: Minimum Tokens (token_addr) purchased.
        /// - `min_dot_bought`: Minimum Dot purchased as intermediary.
        /// - `deadline`: Time after which this transaction can no longer be executed.
        /// - `token_addr`: token_addr The address of the token being purchased.
        /// - `recipient`: AcccountId will get the transferred dot.
        #[ink(message)]
        pub fn token_to_token_transfer_input(&mut self,tokens_sold: Balance, min_tokens_bought : Balance, 
            min_dot_bought : Balance, deadline : Timestamp, recipient : AccountId,token_addr : AccountId) ->Balance{
                let exchange_addr = self.exchange_from_factory(token_addr);
                self.token_to_token_input(tokens_sold, min_tokens_bought, min_dot_bought, deadline,
                    self.env().caller(), recipient, exchange_addr)
        }

        fn token_to_token_output(&mut self, tokens_bought : Balance, max_tokens_sold : Balance, max_dot_sold : Balance,
            deadline : Timestamp, buyer : AccountId, recipient : AccountId, exchange_addr : AccountId) -> Balance{
            assert!(exchange_addr != self.exchange_account_id && exchange_addr != AccountId::default());
            //call dot_to_token_output_price
            let selector_dot_to_token_output_price = call::Selector::new([0x69, 0xde,0xb0, 0x15]);
            let  dot_bought = build_call::<DefaultEnvironment>()
                .callee(exchange_addr)
                .gas_limit(self.gas_limit)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(selector_dot_to_token_output_price.into()).push_arg(tokens_bought),
                ).returns::<ReturnType<Balance>>().fire().unwrap();
            
            let token_reserve = self.token_balance();
            let tokens_sold = self.output_price(dot_bought, token_reserve, self.env().balance());
            // tokens sold is always > 0
            assert!( max_tokens_sold >= tokens_sold && max_dot_sold >= dot_bought);
            assert!(self.token_transfer_from(buyer, self.exchange_account_id, tokens_sold));
            //call dot_to_token_transfer_output
            let selector_dot_to_token_transfer_output= call::Selector::new([0x07, 0x83,0xf4, 0x03]);
            build_call::<DefaultEnvironment>()
                .callee(exchange_addr)
                .gas_limit(self.gas_limit)
                .transferred_value(dot_bought)
                .exec_input(
                    ExecutionInput::new(selector_dot_to_token_transfer_output.into()).push_arg(tokens_bought).push_arg(deadline).push_arg(recipient),
                ).returns::<ReturnType<Balance>>().fire().unwrap();

            self.env().emit_event( DotPurchase {
                buyer,
                tokens_sold,
                dot_bought,
            });
            
            tokens_sold
        }

        /// Convert Tokens (self.token) to Tokens (token_addr).
        /// 
        /// Return the Ammount of Token(token_addr) sold
        /// 
        /// #Params
        /// 
        /// - `tokens_bought`: Amount of Tokens (token_addr) bought
        /// - `max_tokens_sold`: Maximum Tokens (self.token) sold.
        /// - `max_dot_sold`: Minimum Dot purchased as intermediary.
        /// - `deadline`: Time after which this transaction can no longer be executed.
        /// - `token_addr`: token_addr The address of the token being purchased.
        #[ink(message)]
        pub fn token_to_token_swap_output(&mut self, tokens_bought : Balance, max_tokens_sold : Balance,
            max_dot_sold: Balance,  deadline : Timestamp, token_addr : AccountId) -> Balance{
            let exchange_addr = self.exchange_from_factory(token_addr);
            let caller = self.env().caller();
            self.token_to_token_output(tokens_bought, max_tokens_sold, max_dot_sold, deadline, caller, caller, exchange_addr)
        }

        /// Convert Tokens (self.token) to Tokens (token_addr).
        /// 
        /// Return the Ammount of Token(token_addr) sold
        /// 
        /// #Params
        /// 
        /// - `tokens_bought`: Amount of Tokens (token_addr) bought
        /// - `max_tokens_sold`: Maximum Tokens (self.token) sold.
        /// - `max_dot_sold`: Minimum Dot purchased as intermediary.
        /// - `deadline`: Time after which this transaction can no longer be executed.
        /// - `token_addr`: token_addr The address of the token being purchased.
        /// - `recipient`: AcccountId will get the transferred tokens.
        #[ink(message)]
        pub fn token_to_token_transfer_output(&mut self, tokens_bought : Balance, max_tokens_sold : Balance,
            max_dot_sold: Balance,  deadline : Timestamp,recipient : AccountId,  token_addr : AccountId) -> Balance{
            let exchange_addr = self.exchange_from_factory(token_addr);
            let caller = self.env().caller();
            self.token_to_token_output(tokens_bought, max_tokens_sold, 
                max_dot_sold, deadline, caller, recipient, exchange_addr)
        }

        /// Convert Tokens (self.token) to Tokens (exchange_addr.token).
        /// 
        /// Return Amount of Tokens (exchange_addr.token) bought.
        /// 
        /// NOTE: Allows trades through contracts that were not deployed from the same factory.call
        /// 
        /// Params:
        ///     
        /// - `tokens_bought` Amount of Tokens bought.
        /// - `min_tokens_bought` Minimum Tokens (token_addr) purchased.
        /// - `min_dot_bought` Minimum dot purchased as intermediary.
        /// - `deadline` Time after which this transaction can no longer be executed.
        /// - `exchange_addr` The address of the exchange for the token being purchased.
        #[ink(message)]
        pub fn token_to_exchange_swap_input(&mut self, tokens_bought : Balance, max_tokens_sold : Balance,
            max_dot_sold: Balance,  deadline : Timestamp, exchange_addr : AccountId)->Balance{
            let caller = self.env().caller();
            self.token_to_token_input( tokens_bought, max_tokens_sold, max_dot_sold, deadline, caller, caller, exchange_addr)
        }

        /// Convert Tokens (self.token) to Tokens (exchange_addr.token).
        /// 
        /// Return Amount of Tokens (exchange_addr.token) bought.
        /// 
        /// NOTE: Allows trades through contracts that were not deployed from the same factory.call
        /// 
        /// Params:call
        ///     
        /// - `tokens_bought`: Amount of Tokens bought.
        /// - `min_tokens_bought`: Minimum Tokens (token_addr) purchased.
        /// - `min_dot_bought`: Minimum Dot purchased as intermediary.
        /// - `deadline`: Time after which this transaction can no longer be executed.
        /// - `recipient`: The address that receives output Dot.
        /// - `exchange_addr` The address of the exchange for the token being purchased.
        #[ink(message)]
        pub fn token_to_exchange_transfer_input(&mut self, tokens_bought : Balance, max_tokens_sold : Balance,
            max_dot_sold: Balance,  deadline : Timestamp, recipient : AccountId,exchange_addr : AccountId)->Balance{
            let caller = self.env().caller();
            self.token_to_token_input(tokens_bought, max_tokens_sold, max_dot_sold, deadline, caller, recipient, exchange_addr)
        }

        /// Convert Tokens (self.token) to Tokens (exchange_addr.token).
        /// 
        /// Return Amount of Tokens (exchange_addr.token) bought.
        /// 
        /// NOTE: Allows trades through contracts that were not deployed from the same factory.call
        /// 
        /// Params:call
        ///     
        /// - `tokens_bought`: Amount of Tokens (token_addr) bought
        /// - `max_tokens_sold`: Maximum Tokens (self.token) sold.
        /// - `max_dot_sold`: Maximum Dot purchased as intermediary.
        /// - `deadline`: Time after which this transaction can no longer be executed.
        /// - `exchange_addr`: The address of the exchange for the token being purchased.
        #[ink(message)]
        pub fn token_to_exchange_swap_output(&mut self, tokens_bought : Balance, max_tokens_sold : Balance,
            max_dot_sold: Balance,  deadline : Timestamp, exchange_addr : AccountId)->Balance{
            let caller = self.env().caller();
            self.token_to_token_output(tokens_bought, max_tokens_sold,max_dot_sold, deadline, caller, caller, exchange_addr)
        }

        /// Convert Tokens (self.token) to Tokens (exchange_addr.token).
        /// 
        /// Return Amount of Tokens (exchange_addr.token) bought.
        /// 
        /// NOTE: Allows trades through contracts that were not deployed from the same factory.call
        /// 
        /// Params:call
        ///     
        /// - `tokens_bought`: Amount of Tokens (token_addr) bought
        /// - `max_tokens_sold`: Maximum Tokens (self.token) sold.
        /// - `max_dot_sold`: Maximum Dot purchased as intermediary.
        /// - `deadline`: Time after which this transaction can no longer be executed.
        /// - `recipient`: The address that receives output Dot.
        /// - `exchange_addr`: The address of the exchange for the token being purchased.
        #[ink(message)]
        pub fn token_to_exchange_transfer_output(&mut self, tokens_bought : Balance, max_tokens_sold : Balance,
            max_dot_sold: Balance,  deadline : Timestamp, recipient : AccountId,exchange_addr : AccountId)->Balance{
            let caller = self.env().caller();
            self.token_to_token_output( tokens_bought, max_tokens_sold, 
                max_dot_sold, deadline, caller, recipient, exchange_addr)
        }

        /// Calculate how many token can be exchanged for a certain number of dot.
        /// 
        /// Return amount of token bought
        /// 
        /// #Params
        /// 
        /// - `dot_sold`: Amount of dot sold.
        #[ink(message)]
        pub fn dot_to_token_input_price(&mut self, dot_sold : Balance)->Balance{
            assert!(dot_sold > 0);

            let token_reserve = self.token_balance();
            self.input_price(dot_sold, self.env().balance(), token_reserve)
        }

        /// Calculate how many dot it need to buy a certain amount of token
        /// 
        /// Return ammount of dot need in this exchange.
        /// 
        /// #Params
        /// 
        /// - `tokens_bought`: Amount of token bought.
        #[ink(message, selector="0x69deb015")]
        pub fn dot_to_token_output_price(&mut self, tokens_bought : Balance)->Balance{
            assert!(tokens_bought > 0);
            
            let token_reserve = self.token_balance();
            self.output_price(tokens_bought, self.env().balance(), token_reserve )
        }

        /// Calculate how many dot can be exchanged for a certain number of tokens
        /// 
        /// Return ammount of dot sell.
        /// 
        /// #Params
        /// 
        /// - `tokens_bought`: Amount of token bought.
        #[ink(message)]
        pub fn token_to_dot_input_price(&mut self, tokens_sold : Balance)->Balance{
            assert!(tokens_sold > 0);

            let token_reserve = self.token_balance();
            self.input_price(tokens_sold, token_reserve, self.env().balance())
        }

        /// Calculate how many token it need to buy a certain amount of dot
        /// 
        /// Return ammount of token need in this exchange.
        /// 
        /// #Params
        /// 
        /// - `dot_bought`: Amount of dot bought.
        #[ink(message)]
        pub fn token_to_dot_output_price(&mut self, dot_bought : Balance)->Balance{
            assert!(dot_bought > 0);

            let token_reserve = self.token_balance();
            let dot_bought = self.output_price(dot_bought, token_reserve, self.env().balance());
            dot_bought
        }

        /// Return the total liqudity in this trading pair.
        #[ink(message)]
        pub fn total_supply(&self) ->Balance{
            self.total_supply
        }

        /// Return the liquidity of the owner
        /// 
        /// #params
        /// 
        /// -`owner`: The account of a liquidity provider 
        #[ink(message)]
        pub fn balance_of(&mut self, owner : AccountId) -> Balance{
            self.balances.get(&owner).unwrap_or(&0u128).clone()
        }

        fn transfer_from_to(&mut self, from: AccountId, to: AccountId, value: Balance) -> bool {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return false
            }
            
            // Update the sender's balance.
            self.balances.insert(from, from_balance - value);

            // Update the receiver's balance.
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value);

            self.env().emit_event(Transfer {
                from,
                to,
                value,
            });

            true
        }

        /// The Contract caller transfer some liquidity to another account
        /// 
        /// #Params:
        /// 
        /// - `to`: An account receive the transferred liquidity
        /// - `value`: Amount of liquidity will be transferred.
        #[ink(message, selector = "0xfae3a09d" )]
        pub fn transfer(&mut self, to : AccountId, value : Balance) -> bool{
            let caller = self.env().caller();
            self.transfer_from_to(caller, to, value)
        }

        /// The Contract caller transfer some liquidity from an account to another account 
        /// 
        /// #Params:
        /// 
        /// - `from`: An account pay transferred liquidity.
        /// - `to`: An account receive the transferred liquidity
        /// - `value`: Amount of liquidity will be transferred.
        #[ink(message, selector = "0xfcfb2ccd")]
        pub fn transfer_from(&mut self,from : AccountId, to : AccountId, value : Balance) -> bool{
            let caller = self.env().caller();
            let allowance = self.allowances.get(&(from, caller)).unwrap_or(&0u128).clone();
            if allowance < value {
                 return false
            }
            self.allowances.insert((from, caller), allowance - value);

            self.transfer_from_to(from, to, value)
        }
        
        /// Approve spender can transfer liquidity from the caller account
        ///
        /// #Params 
        /// - `spender`: The account can transfer liquidity.
        /// - `value`: The amount liquidity can be transferred.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            true
        }

        ///Return the token amount in liqudity pool
        #[ink(message)]
        pub fn token_balance(&mut self) -> Balance{                                                                          
            let selector_balance_of = call::Selector::new([0x56, 0xe9,0x29, 0xb2]);
            build_call::<DefaultEnvironment>()
                .callee(self.token)
                .gas_limit(self.gas_limit / 2)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(selector_balance_of.into()).push_arg(&self.exchange_account_id),
                ).returns::<ReturnType<Balance>>().fire().unwrap()
        }

        ///#[ink(message)]
        fn token_transfer(&mut self, to : AccountId, value : Balance)->bool{
            //transfer function seletor from metadata.json 0xfae3a09d
            let selector_transfer  = call::Selector::new([0xfa, 0xe3,0xa0, 0x9d]);
            build_call::<DefaultEnvironment>()
                .callee(self.token)
                .gas_limit(self.gas_limit / 2)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(selector_transfer.into())
                    .push_arg(&to)
                    .push_arg(value),
                ).returns::<ReturnType<bool>>().fire().unwrap()
        }  

        ///Transfer token 
        //#[ink(message)]
        fn token_transfer_from(&mut self, from : AccountId, to : AccountId, value : u128) ->bool{
            //selector transfer_from in erc20 metadata.json 0xfcfb2ccd
            let selector_transfer_from = call::Selector::new([0xfc, 0xfb,0x2c, 0xcd]);
            build_call::<DefaultEnvironment>()
                .callee(self.token)
                .gas_limit(self.gas_limit / 2)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(selector_transfer_from.into())
                    .push_arg(&from)
                    .push_arg(&to)
                    .push_arg(value),
                ).returns::<ReturnType<bool>>().fire().unwrap()
        }

        ///Get exchange account in tradint by token account
        /// 
        /// #Params
        /// 
        /// -`token account`: The token account in the trading pair.
        #[ink(message)]
        pub fn exchange_from_factory(&mut self, token_account: AccountId) -> AccountId{
            //get_exchange seletor from metadata.json 0xce34755e
            let selector_transfer_from = call::Selector::new([0xce, 0x34,0x75, 0x5e]);
            build_call::<DefaultEnvironment>()
                .callee(self.factory)
                .gas_limit(self.gas_limit / 2)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(selector_transfer_from.into())
                    .push_arg(&token_account),
                ).returns::<ReturnType<AccountId>>().fire().unwrap()
        }

        ///Get token account in tradint by Exchange account
        /// 
        /// #Params
        /// 
        /// -`exchange account`: The Exchange account in the trading pair.
        #[ink(message)]
        pub fn token_from_factory(&mut self, exchange_addr: AccountId) ->AccountId{
            let selector = call::Selector::new([0x97, 0x38,0x04, 0x08]);
            build_call::<DefaultEnvironment>()
                .callee(self.factory)
                .gas_limit(self.gas_limit / 2)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(selector.into())
                    .push_arg(&exchange_addr),
                ).returns::<ReturnType<AccountId>>().fire().unwrap()
        }
        
        ///Return the Exchange self account id.
        #[ink(message)]
        pub fn get_address(&self) ->AccountId{
            self.exchange_account_id
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ink_env::{
        AccountId,
    };
    use ink_lang as ink;

    #[ink::test]
    fn test_transfer_liquidity_should_work(){
        let token_account_id = AccountId::from([0x01; 32]);
        let factory_account_id = AccountId::from([0x02; 32]);
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().unwrap();
        let liquidity_amount = 50000u128;

        let mut contract = Exchange::new(token_account_id, factory_account_id, accounts.alice, 0u128);
        contract.balances.insert(accounts.alice, liquidity_amount);

        assert_eq!(contract.balance_of(accounts.alice), liquidity_amount);
        assert!(contract.transfer(accounts.bob, 200));
        assert_eq!(contract.balance_of(accounts.alice), liquidity_amount - 200);
        assert_eq!(contract.balance_of(accounts.bob), 200);
    }

    #[ink::test]
    fn test_transfer_from_liquidity_should_work(){
        let token_account_id = AccountId::from([0x01; 32]);
        let factory_account_id = AccountId::from([0x02; 32]);
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().unwrap();
        let mut contract = Exchange::new(token_account_id, factory_account_id, accounts.alice, 0u128);

        let liquidity_amount = 50000u128;
        contract.balances.insert(accounts.alice, liquidity_amount);

        contract.approve(accounts.alice, 500);
        assert!(contract.transfer_from(accounts.alice, accounts.bob, 200));
        assert_eq!(contract.balance_of(accounts.bob), 200);
    }

    #[ink::test]
    fn test_output_price(){
        let token_account_id = AccountId::from([0x01; 32]);
        let factory_account_id = AccountId::from([0x02; 32]);
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().unwrap();
        let contract = Exchange::new(token_account_id, factory_account_id, accounts.alice, 0u128);

        let output_amount = 40;
        let input_reserve = 200000;
        let output_reserve = 200000;
        let sell_amount = contract.output_price(output_amount, input_reserve, output_reserve);
        println!("bought amount:{} sell amount:{}", output_amount, sell_amount);

        let output_amount_2 = 40;
        let input_reserve_2 = 200;
        let output_reserve_2 = 200;
        let sell_amount_2 = contract.output_price(output_amount_2, input_reserve_2, output_reserve_2);
        println!("bought amount:{} sell amount:{}", output_amount_2, sell_amount_2);

        assert!(sell_amount_2 > sell_amount);
    }

    #[ink::test]
    fn test_input_price(){
        let token_account_id = AccountId::from([0x01; 32]);
        let factory_account_id = AccountId::from([0x02; 32]);
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().unwrap();
        let contract = Exchange::new(token_account_id, factory_account_id, accounts.alice, 0u128);

        let intput_amount = 40;
        let input_reserve = 200000;
        let output_reserve = 200000;
        let bought_amount = contract.input_price(intput_amount, input_reserve, output_reserve);
        println!("sell amount:{} bought amount:{}", intput_amount, bought_amount);

        let input_amount_2 = 40;
        let input_reserve_2 = 200;
        let output_reserve_2 = 200;
        let bought_amount_2 = contract.input_price(input_amount_2, input_reserve_2, output_reserve_2);
        println!("sell amount:{} bought amount:{}", input_amount_2, bought_amount_2);

        assert!(bought_amount > bought_amount_2);
    }
}