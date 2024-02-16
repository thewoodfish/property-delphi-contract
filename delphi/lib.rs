#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod delphi {
    use ink::storage::Mapping;
    use scale_info::prelude::vec::Vec;

    /// The struct containing more info about our user
    #[derive(scale::Decode, scale::Encode, Default)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct AccountInfo {
        /// Name of user
        name: Vec<u8>,
        /// This is to know if the user is an authority figure (e.g MInistry of land and works).
        is_authority: bool,
        /// Time the account was created
        timestamp: u64,
    }

    /// The struct containing info about the assertions of the land made by an authority
    #[derive(scale::Decode, scale::Encode, Default)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct IssueInfo {
        /// Id of property
        property: PropertyId,
        /// Time the assertion was created
        timestamp: u64,
    }

    /// The id of the property
    type PropertyId = Vec<u8>;
    /// The IPFS address (CID) of the requirements of the property
    type PropertyRequirementAddr = Vec<u8>;
    /// The IPFS address (CID) of the document showing the reghtful ownership of the property
    type PropertyClaimAddr = Vec<u8>;

    //// Event to announce the creation of an account
    #[ink(event)]
    pub struct AccountCreated {
        #[ink(topic)]
        account_id: AccountId,
        #[ink(topic)]
        is_authority: bool,
        name: Vec<u8>,
    }

    #[ink(storage)]
    pub struct Delphi {
        accounts: Mapping<AccountId, AccountInfo>,
        registrations: Mapping<AccountId, Vec<PropertyId>>,
        properties: Mapping<PropertyId, PropertyRequirementAddr>,
        claims: Mapping<AccountId, PropertyClaimAddr>,
        assertions: Mapping<AccountId, IssueInfo>,
    }

    impl Delphi {
        /// Constructor that initializes the default values and memory of the great Delphi
        #[ink(constructor)]
        pub fn new() -> Self {
            Delphi {
                accounts: Default::default(),
                registrations: Default::default(),
                properties: Default::default(),
                claims: Default::default(),
                assertions: Default::default(),
            }
        }

        /// Register an account
        /// The account can be an individual (property owner) account or an authority's account
        #[ink(message)]
        pub fn register_account(&mut self, name: Vec<u8>, is_authority: bool, timestamp: u64) {
            // get the contract caller
            let caller = Self::env().caller();

            let new_account = AccountInfo {
                name: name.clone(),
                is_authority,
                timestamp,
            };

            // insert into storage
            self.accounts.insert(&caller, &new_account);

            // emit event
            self.env().emit_event(AccountCreated {
                account_id: caller,
                is_authority,
                name,
            });
        }

        /// Check if an account exists and whether it is an authority figure
        /// It returns a tuple (account_exists?, account_is_authority?)
        #[ink(message)]
        pub fn account_exists(&self, account_id: AccountId) -> (bool, bool) {
            match self.accounts.get(&account_id) {
                Some(info) => (true, if info.is_authority { true } else { false }),
                None => (false, false),
            }
        }
    }
}
