#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {

    use ink_storage::collections::HashMap as StorageHashMap;

    #[ink(storage)]
    pub struct Erc20 {
        /// 总的token
        total_supply: Balance,
        /// 每个account的token
        balances: StorageHashMap<AccountId, Balance>,
        allowances: StorageHashMap<(AccountId,AccountId), Balance>
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InSufficentBalance,
        InsufficientAllowance,
    }

    pub type Result<T> = core::result::Result<T, Error>;

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
        owner: Option<AccountId>,
        #[ink(topic)]
        spender: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        caller: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    impl Erc20 {

        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            /// total_supply  初始化时候直接设置
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, total_supply);
            let instance = Self {
                total_supply: total_supply,
                balances:balances,
                allowances:StorageHashMap::new(),
            };
            instance
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// 公共方法：读或写
        ///读不会修改链上数据
        #[ink(message)]
        pub fn total_supply(&self)->Balance{
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            self.balances.get(owner).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn allowance(&self,  owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval {
               owner: Some(owner),
                spender:Some(spender),
                value,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance)
            }
            self.transfer_from_to(from, to, value)?;
            self.allowances.insert((from, caller), allowance - value);
            Ok(())
        }

        
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let who = Self::env().caller();
            self.transfer_from_to(who, to, value)
        }



        fn transfer_from_to(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let from_balance = self.balance_of_or_zero(&from);
            if from_balance < value {
                return Err(Error::InSufficentBalance);
            }

            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of_or_zero(&to);
            self.balances.insert(to, to_balance + value);

            self.env().emit_event(Transfer{
                from:Some(from),
                to:Some(to),
                value:value,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn burn(&mut self, value: Balance) -> Result<()>{
            let caller = Self::env().caller();
            let caller_balance = self.balance_of_or_zero(&caller);
            if caller_balance < value {
                return Err(Error::InSufficentBalance);
            }
            self.balances.insert(caller, caller_balance - value);
            self.env().emit_event(Burn{
                caller:Some(caller),
                value:value,
            });
            Ok(())
        }

        #[ink(message)]
       pub  fn issue(&mut self, value: Balance) -> Result<()>{
            self.total_supply =  self.total_supply + value;
            Ok(())
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn create_contruct_works(){
            let erc20 =Erc20::new(1000);
            assert_eq!(erc20.total_supply(), 1000);
        }

        fn issue_works(){
            let erc20 =Erc20::new(1000);
            erc20.issue(10);
            assert_eq!(erc20.total_supply(), 1010);
        }
    }
}
