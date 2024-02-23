// Copyright (c) 2024 Algorealm, Inc.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod delphi {
    use ink::storage::Mapping;
    use scale_info::prelude::vec;
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
        property: PropertyTypeId,
        /// Time the assertion was created
        timestamp: u64,
    }

    /// The struct describing a property type
    #[derive(scale::Decode, scale::Encode, Default, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct PropertyType {
        /// Id of property type
        id: PropertyTypeId,
        address: PropertyRequirementAddr,
    }

    /// The struct representing a property claim, yet to be verified and attested
    #[derive(scale::Decode, scale::Encode, Default, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct PropertyClaim {
        /// Id of property
        id: PropertyId,
        address: PropertyClaimAddr,
    }

    /// The id of the property
    type PropertyId = Vec<u8>;
    /// The id of the property document type
    type PropertyTypeId = Vec<u8>;
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
        property_type_id: PropertyTypeId,
        ptype_ipfs_addr: PropertyRequirementAddr,
    }

    //// Event to announce the registration of a claim to a property
    #[ink(event)]
    pub struct PropertyClaimRegistered {
        #[ink(topic)]
        claimer: AccountId,
        #[ink(topic)]
        property_type_id: PropertyTypeId,
        property_id: PropertyId,
    }

    #[ink(storage)]
    pub struct Delphi {
        accounts: Mapping<AccountId, AccountInfo>,
        registrations: Mapping<AccountId, Vec<PropertyType>>,
        claims: Mapping<PropertyTypeId, (AccountId, PropertyClaim)>,
        assertions: Mapping<AccountId, IssueInfo>,
    }

    impl Delphi {
        /// Constructor that initializes the default values and memory of the great Delphi
        #[ink(constructor)]
        pub fn new() -> Self {
            Delphi {
                accounts: Default::default(),
                registrations: Default::default(),
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

        /// Check if an account exists.
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

        /// Register a property type.
        /// This should only be called by an authority figure (e.g Ministry of Lands)
        #[ink(message, payable)]
        pub fn register_ptype(
            &mut self,
            property_type_id: PropertyTypeId,
            ptype_ipfs_addr: PropertyRequirementAddr,
        ) {
            // Get the contract caller
            let caller = Self::env().caller();

            // create type
            let property_type = PropertyType {
                id: property_type_id.clone(),
                address: ptype_ipfs_addr.clone(),
            };

            // Record the registrar.
            // This is important to load all the properties registered by a certain authority
            if let Some(ref mut property_types) = self.registrations.get(&caller) {
                // add to the list of registered property types
                property_types.push(property_type.clone());
                self.registrations.insert(caller, property_types);
            } else {
                // insert new
                let property_types = vec![property_type.clone()];
                self.registrations.insert(caller, &property_types);
            }

            // Emit event
            self.env().emit_event(PropertyTypeRegistered {
                account_id: caller,
                property_type_id,
                ptype_ipfs_addr,
            });
        }

        /// Return the info about property type documents created by a certain authority.
        /// They are returned as concatenated bytes separated by the '###' character.
        /// The property id and address are separated by a '~' character
        /// E.g prop_id1~prop_addr1###prop_id2~prop_addr2
        #[ink(message, payable)]
        pub fn ptype_documents(&self, account_id: AccountId) -> Vec<u8> {
            if let Some(property_types) = self.registrations.get(&account_id) {
                property_types
                    .clone()
                    .iter_mut()
                    .flat_map(|ptype| {
                        // make the `id` the collator
                        ptype.id.push(b'~');
                        ptype.id.extend(ptype.address.iter());

                        ptype.id.extend("###".as_bytes()); // add separator
                        ptype.id.clone().into_iter()
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }

        /// Submit a claim to a particular property.
        /// This is the first step, preceeding verification and attestation
        #[ink(message, payable)]
        pub fn register_claim(
            &mut self,
            property_type_id: PropertyTypeId,
            property_id: PropertyId,
            claim_ipfs_addr: PropertyClaimAddr,
        ) {
            // get claimer
            let claimer = Self::env().caller();

            // create a new claim
            let property_claim = PropertyClaim {
                id: property_id.clone(),
                address: claim_ipfs_addr,
            };

            // insert into contract storage
            self.claims
                .insert(&property_type_id, &(claimer.clone(), property_claim));

            // Emit event
            self.env().emit_event(PropertyClaimRegistered {
                claimer,
                property_type_id,
                property_id,
            });
        }
    }
}
