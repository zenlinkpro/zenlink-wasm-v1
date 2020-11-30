
#![cfg_attr(not(feature = "std"), no_std)]

pub use self::erc20::Erc20;

use ink_lang as ink;

#[ink::contract]
pub mod erc20 {
    //#[cfg(not(feature = "ink-as-dependency"))]
    #[ink(storage)]
    pub struct Erc20 {
        /// The total supply.
        total_supply: Balance,
        /// The balance of each user.
        balances: ink_storage::collections::HashMap<AccountId, Balance>,
        /// Approval spender on behalf of the message's sender.
        allowances: ink_storage::collections::HashMap<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = ink_storage::collections::HashMap::new();
            balances.insert(caller, initial_supply);

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });

            Self {
                total_supply: initial_supply,
                balances,
                allowances: ink_storage::collections::HashMap::new()
            }
        }

        #[ink(message, payable)]
        pub fn balance(&self)->Balance{
            self.env().transferred_balance()
            //self.env().balance()
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval{
                owner: owner,
                spender: spender,
                value
            });

            true
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_of_or_zero(&owner, &spender)
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> bool {
            //let caller = self.env().caller();
            // let allowance = self.allowance_of_or_zero(&from, &caller);
            // if allowance < value {
            //     return false
            // }
            // self.allowances.insert((from, caller), allowance - value);
            self.transfer_from_to(from, to, value)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> bool {
            //let blance = self.env().transferred_balance();
            self.transfer_from_to(self.env().caller(), to, value)
        }

        fn transfer_from_to(&mut self, from: AccountId, to: AccountId, value: Balance) -> bool {
            let from_balance = self.balance_of_or_zero(&from);
            if from_balance < value {
                return false
            }
            
            // Update the sender's balance.
            self.balances.insert(from, from_balance - value);

            // Update the receiver's balance.
            let to_balance = self.balance_of_or_zero(&to);
            self.balances.insert(to, to_balance + value);

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });

            true
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        fn allowance_of_or_zero(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            *self.allowances.get(&(*owner, *spender)).unwrap_or(&0)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_env::{hash::{
            Blake2x256,
            CryptoHash,
            HashOutput,
        }, Clear, AccountId};

        use ink_lang as ink;

        type Event = <Erc20 as ::ink_lang::BaseEvent>::Type;

        fn assert_transfer_event(
            event: &ink_env::test::EmittedEvent,
            expected_from: Option<AccountId>,
            expected_to: Option<AccountId>,
            expected_value: Balance,
        ) {
            let decode_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::Transfer(Transfer { from, to, value }) = decode_event {
                assert_eq!(from, expected_from, "encountered invalid Transfer.from");
                assert_eq!(to, expected_to, "encountered invalid Transfer.to");
                assert_eq!(value, expected_value, "encountered invalid Trasfer.value");
            } else {
                panic!("encountered unexpected event kind: expected a Transfer event")
            }


            fn encoded_into_hash<T>(entity: &T) -> Hash
            where
                T: scale::Encode
            {
                {
                    let mut result = Hash::clear();
                    let len_result = result.as_ref().len();
                    let encoded = entity.encode();
                    let len_encoded = encoded.len();
                    if len_encoded <= len_result {
                        result.as_mut()[..len_encoded].copy_from_slice(&encoded);
                        return result
                    }
                    let mut hash_output =
                        <<Blake2x256 as HashOutput>::Type as Default>::default();
                    <Blake2x256 as CryptoHash>::hash(&encoded, &mut hash_output);
                    let copy_len = core::cmp::min(hash_output.len(), len_result);
                    result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
                    result
                }
            }

            let expected_topics = vec![
                encoded_into_hash(b"Erc20::Transfer"),
                encoded_into_hash(&expected_from),
                encoded_into_hash(&expected_to),
                encoded_into_hash(&expected_value),
            ];
            for (n, (actual_topic, expected_topic)) in
                event.topics.iter().zip(expected_topics).enumerate()
            {
                let topic = actual_topic
                    .decode::<Hash>()
                    .expect("encountered invalid topic encoding");
                assert_eq!(topic, expected_topic, "encountered invalid topic at {}", n)
            }
        }

        #[ink::test]
        fn new_works() {
            let contract = Erc20::new(777);
            assert_eq!(contract.total_supply(), 777);

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                777,
            );
        }

        #[ink::test]
        fn balance_works() {
            let contract = Erc20::new(100);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x1; 32])),
                100,
            );
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert!(contract.transfer(AccountId::from([0x0; 32]), 10));
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
            assert!(!contract.transfer(AccountId::from([0x0; 32]), 100));

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(2, emitted_events.len());
            assert_transfer_event(
                &emitted_events[1],
                Some(AccountId::from([0x1; 32])),
                Some(AccountId::from([0x0; 32])),
                10,
            );
        }

        #[ink::test]
        fn transfer_from_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract.approve(AccountId::from([0x1; 32]), 20);
            contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 10);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(3, emitted_events.len());
            assert_transfer_event(
                &emitted_events[2],
                Some(AccountId::from([0x1; 32])),
                Some(AccountId::from([0x0; 32])),
                10,
            );
        }
    }
}
