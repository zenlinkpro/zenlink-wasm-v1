#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod factory {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{
        collections::HashMap as StorageHashMap,
    };

    use exchange::Exchange;

    #[ink(event)]
    pub struct NewExchange{
        #[ink(topic)]
        erc20_token_account : AccountId,
        #[ink(topic)]
        exchange_contract_account : AccountId,
    }

    #[ink(storage)]
    pub struct Factory {
        pub exchange_template : Hash,

        pub token_count : u128,

        token_to_exchange : StorageHashMap<AccountId,AccountId>,

        exchange_to_token : StorageHashMap<AccountId,AccountId>,

        id_to_token : StorageHashMap<u128, AccountId>,
    }

    impl Factory {

        #[ink(constructor)]
        pub fn new() -> Self {
            Self{
                exchange_template : Hash::default(),
                token_count : 0,
                token_to_exchange : StorageHashMap::new(),
                exchange_to_token : StorageHashMap::new(),
                id_to_token : StorageHashMap::new(),
                
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        ///Set the Exchange wasm hashcode on the chain. Factory will use it to instantiate Exchange
        /// 
        /// NOTE: A factory has only one exchange_template_address.Once the Settings cannot be changed.
        /// 
        /// #Params
        /// - `exchange_template_address`: Exchange wasm hashcode on the chain
        #[ink(message)]
        pub fn initialize_factory(&mut self, exchange_template_address : Hash){
            assert!(self.exchange_template ==  Hash::default());
            assert!(exchange_template_address != Hash::default());

            self.exchange_template = exchange_template_address;
        }


        /// Create trading pair
        /// 
        /// NOTE: A token account can only create one trading pair.
        /// 
        /// #Params
        /// - `erc20_token_address`: The erc20 token account
        /// - `token_ammount`: Ammount tokens transfer from erc20_token_address to the account of Exchange contract. 
        #[ink(message,payable)]
        pub fn create_exchange(&mut self, erc20_token_account : AccountId, token_ammount : Balance){
            assert!(erc20_token_account != AccountId::default());
            assert!(self.exchange_template != Hash::default());
            assert!(self.token_to_exchange.get(&erc20_token_account) ==  None);
            
            // If Caller don't supply enought dot to instantiated exchange_contract. ExchangeContract will become tombstone.
            let transferred_balance = self.env().transferred_balance();
            assert!(transferred_balance != 0);

            let exchange = Exchange::new(erc20_token_account, self.env().account_id(), self.env().caller(), token_ammount)
                .endowment(transferred_balance)
                .code_hash(self.exchange_template)
                .instantiate()
                .expect("instantiate exchange failed"); 
            
            let exchange_contract_account = exchange.get_address();
            self.token_to_exchange.insert(erc20_token_account.clone(), exchange_contract_account.clone());
            self.exchange_to_token.insert(exchange_contract_account.clone(), erc20_token_account.clone());
    
            let token_id = self.token_count + 1;
            self.token_count = token_id;

            self.id_to_token.insert(token_id, erc20_token_account.clone());
    
            self.env().emit_event( NewExchange {
                erc20_token_account,
                exchange_contract_account,
            });
        }

        /// Get Exchange account by token from the trading pair.
        /// 
        /// #Params
        /// 
        /// - `token`: A token account in a trading pair.
        #[ink(message)]
        pub fn get_exchange(&self, token : AccountId)-> AccountId{
            *self.token_to_exchange.get(&token).unwrap()
        }

        /// Get Token account by token from the trading pair.
        /// 
        /// #Params
        /// 
        /// - `exchange_account`: A Exchange account in a trading pair.
        #[ink(message)]
        pub fn get_token(&self, exchange_account : AccountId)-> AccountId{
            *self.exchange_to_token.get(&exchange_account).unwrap()
        }

        /// Get Token account by index.
        /// 
        /// #Params
        /// 
        /// - `token_id`: The serial number associated with toke, which sort by the order of creation.
        #[ink(message)]
        pub fn get_token_with_id(&self, token_id : u128)-> AccountId{
            *self.id_to_token.get(&token_id).unwrap()
        }
    }
}
