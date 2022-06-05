use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, Balance, PanicOnDefault, near_bindgen,Timestamp,BorshStorageKey, log, Promise};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;


// 5 Ⓝ in yoctoNEAR
const PRIZE_AMOUNT: u128 = 5_000_000_000_000_000_000_000_000;
const ONEWEEKINSECONDS:i32 = 604_800;
const NANOSECONDS:i32 = 1_000_000_000;
static ALLTOKENSFORREWARDS:u128 = 100_000_000_000_000_000_000_000_000; // it is 100 Near
// typing
pub type TimestampSec = u32;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct InvestInfo {
    pub amount:u128,
    pub time:i32,
}


#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Accounts,
}
#[near_bindgen]
//#[serde(crate = "near_sdk::serde")]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, )]
pub struct Contract {
    pub accounts: UnorderedMap<AccountId, InvestInfo>, 
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            accounts: UnorderedMap::new(StorageKey::Accounts),
        }
    }
    pub fn make_invest(&mut self, amount:u128) {
        //let accountid:AccountId = env::predecessor_account_id();
        let balance_on_account:u128= near_sdk::env::account_balance();
        if amount as u128 > balance_on_account {
            log!("You doesn't have enough balance for this");
            panic!("You can't use more than {}", balance_on_account)    
        }
        else {
            let time:i32 = near_sdk::env::block_timestamp().try_into().unwrap();
            let account_id = near_sdk::env::predecessor_account_id();
            
            let existing = self.accounts.insert(&account_id, &InvestInfo{amount,time}); 
            assert!(existing.is_none(), "Account already exists")
        }
    }
    pub fn get_info(&self) {
        let account_id = near_sdk::env::predecessor_account_id();
        let get = self.accounts.get(&account_id);
        match get {
            Some(get) => log!("This account have a amount {:?}, expiration_time : {:?}", get.amount, get.time / NANOSECONDS),
            None => log!("Couldn't get account")
        }
    }
    pub fn get_all_tokens(&mut self) -> u128 {
        let values = self.accounts.values();
        let all_tokens:u128 = values.into_iter().map(|all_tokens|all_tokens.amount).sum();
        log!("All tokens = {}", all_tokens);
        all_tokens
    }
    #[payable]
    pub fn get_rewards(&mut self, receiver:AccountId) {
        //let all_tokens = self.get_all_tokens();        
        let get = self.accounts.get(&receiver);
        let (first_invest_time, amount) = match get  {
            Some(get) => (get.time/NANOSECONDS, get.amount),
            None => panic!("You doesn't have a funds!")
        };
        let now:i32 = env::block_timestamp().try_into().unwrap();
        //let balance_on_contract = env::account_balance();
        // we have a time when make funds and how much time pass after that
        let duration = now - first_invest_time;
        let percent_for_rewards = 10;
        if duration > ONEWEEKINSECONDS {
            log!("One second and you got a your rewards");
            let rewards:u128 = ((amount * 100) / percent_for_rewards).try_into().unwrap(); 

            if rewards > ALLTOKENSFORREWARDS {
               Promise::new(receiver).transfer(PRIZE_AMOUNT); 
               let _ = ALLTOKENSFORREWARDS - rewards;
            }
            else {
                panic!("Our founds doesn't have enough Near for you rewards :(")
            }
        } else {
            panic!("You should wait: {} seconds", ONEWEEKINSECONDS - now)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId};
    
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

   #[test]
    fn init_make_invest() {
        let alice = AccountId::new_unchecked("alice.testnet".to_string());
        let context = get_context(alice.clone());
        testing_env!(context.build());
        let mut contract = Contract::new();
        contract.make_invest(22);
        let all_tokens = contract.get_all_tokens();
        assert_eq!(22 as u128, all_tokens);
    }


}
