// Copyright (c) 2024 Algorealm, Inc.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod delphi {
    use ink::storage::Mapping;
    use scale_info::prelude::vec::Vec;
    use scale_info::prelude::vec;

    /// The struct containing more info about our user
    #[derive(scale::Decode, scale::Encode, Default)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct AccountInfo {
        /// Name of user
        name: Vec<u8>,
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
        name: Vec<u8>,
    }

    //// Event to announce the registration of a property type
    #[ink(event)]
    pub struct PropertyTypeRegistered {
        #[ink(topic)]
        account_id: AccountId,
        property_id: PropertyId,
        ptype_ipfs_addr: PropertyRequirementAddr,
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
        #[ink(message, payable)]
        pub fn register_account(&mut self, name: Vec<u8>, timestamp: u64) {
            // Get the contract caller
            let caller = Self::env().caller();

            let new_account = AccountInfo {
                name: name.clone(),
                timestamp,
            };

            // Insert into storage
            self.accounts.insert(&caller, &new_account);

            // Emit event
            self.env().emit_event(AccountCreated {
                account_id: caller,
                name,
            });
        }

        /// Check if an account exists
        /// It also returns the name of the user if it exists
        #[ink(message, payable)]
        pub fn account_exists(&self) -> (bool, Vec<u8>) {
            // get the contract caller
            let caller = Self::env().caller();

            match self.accounts.get(&caller) {
                Some(info) => (true, info.name.clone()),
                None => (false, Vec::new()),
            }
        }

        /// Register a property type
        /// This should only be called by an authority figure (e.g Ministry of Lands)
        #[ink(message, payable)]
        pub fn register_ptype(
            &mut self,
            property_id: PropertyId,
            ptype_ipfs_addr: PropertyRequirementAddr,
        ) {
            // Get the contract caller
            let caller = Self::env().caller();

            // Record the registrar
            // This is important to load all the properties registered by a certain authority
            if let Some(ref mut properties) = self.registrations.get(&caller) {
                // add to the list of registered property types
                properties.push(property_id.clone());
            } else {
                // insert new
                let property_types = vec![property_id.clone()];
                self.registrations.insert(caller, &property_types);
            }

            // Now we will record the property itself and the IPFS address containing its details
            self.properties.insert(&property_id, &ptype_ipfs_addr);

            // Emit event
            self.env().emit_event(PropertyTypeRegistered {
                account_id: caller,
                property_id,
                ptype_ipfs_addr,
            });
        }

        /// Return the IPFS addresses of the property type documents created by a certain authority
        /// They are returned as concatenated bytes separated by the '#' character
        #[ink(message, payable)]
        pub fn ptype_documents(&self, account_id: AccountId) -> Vec<u8> {
            if let Some(property_types) = self.registrations.get(&account_id) {
                property_types
                    .clone()
                    .iter_mut()
                    .flat_map(|vector| {
                        vector.push(b'#'); // add separator
                        vector.clone().into_iter()
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }
    }
}
