#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::string::String;

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[allow(clippy::cast_possible_truncation)]
pub enum PSP22Error {
    Custom(String),
    InsufficientBalance,
    InsufficientAllowance,
    ZeroRecipientAddress,
    ZeroSenderAddress,
    SafeTransferCheckFailed(String),
}

#[ink::contract]
mod inkerc20 {
    use super::PSP22Error;
    use ink::prelude::string::{ String, ToString };
    use ink::storage::Mapping;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct PspCoin {
        /// Total token supply.
        total_supply: u128,
        /// Mapping from owner to number of owned token.
        balances: Mapping<AccountId, u128>,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        allowances: Mapping<(AccountId, AccountId), u128>,
        /// Token name
        name: Option<String>,
        /// Token symbol
        symbol: Option<String>,
        /// Token decimals
        decimals: u8,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: u128,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: u128,
    }

    impl PspCoin {
        /// Constructor that initializes the contract with initial supply assigned to the caller.
        #[ink(constructor)]
        pub fn new(
            total_supply: u128,
            name: Option<String>,
            symbol: Option<String>,
            decimals: u8
        ) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply,
            });

            Self {
                total_supply,
                balances,
                allowances: Default::default(),
                name,
                symbol,
                decimals,
            }
        }

        /// Simply returns the current value of our bool.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(1000000, Some("MyToken".to_string()), Some("MTK".to_string()), 18)
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }

        /// Returns the account balance for the specified owner.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u128 {
            self.balances.get(owner).unwrap_or_default()
        }

        /// Returns the amount which spender is still allowed to withdraw from owner.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        /// Transfers value amount of tokens from the caller's account to account to.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: u128) -> Result<(), PSP22Error> {
            let from = self.env().caller();
            self._transfer_from_to(&from, &to, value)
        }

        /// Transfers value tokens on the behalf of from to the account to.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u128
        ) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(PSP22Error::InsufficientAllowance);
            }
            self._transfer_from_to(&from, &to, value)?;
            self.allowances.insert((from, caller), &allowance.saturating_sub(value));
            Ok(())
        }

        /// Allows spender to withdraw from the caller's account multiple times, up to
        /// the total amount of value.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), &value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

        /// Increases by delta_value the allowance granted to spender by the caller.
        #[ink(message)]
        pub fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128
        ) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            let allowance = self.allowance(owner, spender);
            self.approve(spender, allowance.saturating_add(delta_value))
        }

        /// Decreases by delta_value the allowance granted to spender by the caller.
        #[ink(message)]
        pub fn decrease_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128
        ) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            let allowance = self.allowance(owner, spender);
            if allowance < delta_value {
                return Err(PSP22Error::InsufficientAllowance);
            }
            self.approve(spender, allowance.saturating_sub(delta_value))
        }

        /// Returns the token name.
        #[ink(message)]
        pub fn token_name(&self) -> Option<String> {
            self.name.clone()
        }

        /// Returns the token symbol.
        #[ink(message)]
        pub fn token_symbol(&self) -> Option<String> {
            self.symbol.clone()
        }

        /// Returns the token decimals.
        #[ink(message)]
        pub fn token_decimals(&self) -> u8 {
            self.decimals
        }

        /// Mints value tokens to the caller's account.
        #[ink(message)]
        pub fn mint(&mut self, value: u128) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            let balance = self.balance_of(caller);
            self.balances.insert(caller, &balance.saturating_add(value));
            self.total_supply = self.total_supply.saturating_add(value);
            self.env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value,
            });
            Ok(())
        }

        /// Burns value tokens from the caller's account.
        #[ink(message)]
        pub fn burn(&mut self, value: u128) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            let balance = self.balance_of(caller);
            if balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }
            self.balances.insert(caller, &balance.saturating_sub(value));
            self.total_supply = self.total_supply.saturating_sub(value);
            self.env().emit_event(Transfer {
                from: Some(caller),
                to: None,
                value,
            });
            Ok(())
        }

        /// Internal function to transfer tokens.
        fn _transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: u128
        ) -> Result<(), PSP22Error> {
            let from_balance = self.balance_of(*from);
            if from_balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            self.balances.insert(from, &from_balance.saturating_sub(value));
            let to_balance = self.balance_of(*to);
            self.balances.insert(to, &to_balance.saturating_add(value));

            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });
            Ok(())
        }
    }
}