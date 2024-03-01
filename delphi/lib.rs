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

    /// The struct containing more info about a property
    /// The property might be claimed, attested or neither
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Property {
        /// Id of claimer
        claimer: AccountId,
        /// IPFS location of property claim
        property_claim_addr: PropertyClaimAddr,
        /// Type the property belongs to.
        property_type_id: PropertyTypeId,
        /// List of previous owners and time of transfer
        transfer_history: Vec<(AccountId, PropertyTransferTimestamp)>,
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
    /// The IPFS address (CID) of the document showing the rightful ownership of the property
    type PropertyClaimAddr = Vec<u8>;
    /// The Unix timestamp recording the time a property transfer was made
    type PropertyTransferTimestamp = u64;

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

    /// Event to announce the successful transfer of a property
    #[ink(event)]
    pub struct PropertyTransferred {
        #[ink(topic)]
        sender: AccountId,
        #[ink(topic)]
        recipient: AccountId,
        #[ink(topic)]
        property_id: PropertyId,
    }

    #[ink(storage)]
    pub struct Delphi {
        accounts: Mapping<AccountId, AccountInfo>,
        registrations: Mapping<AccountId, Vec<PropertyType>>,
        claims: Mapping<PropertyTypeId, Vec<PropertyId>>,
        properties: Mapping<PropertyId, Property>,
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
                properties: Default::default(),
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

            // create a new property document
            let property = Property {
                claimer: claimer.clone(),
                property_claim_addr: claim_ipfs_addr,
                property_type_id: property_type_id.clone(),
                transfer_history: Vec::new(),
            };

            // register property under type of claim
            if let Some(mut property_ids) = self.claims.get(&property_type_id) {
                // append to the list if it doesn't contain it already
                if !property_ids.contains(&property_id) {
                    property_ids.push(property_id.clone());
                }

                self.claims.insert(property_type_id.clone(), &property_ids);
            } else {
                // create new class of properties and add the new one to it
                let property_ids = vec![property_id.clone()];

                // insert into contract storage
                self.claims.insert(property_type_id.clone(), &property_ids);
            }

            // register (unattested) property claim onchain
            self.properties.insert(property_id.clone(), &property);

            // Emit event
            self.env().emit_event(PropertyClaimRegistered {
                claimer,
                property_type_id,
                property_id,
            });
        }

        /// Returns a list of property (claims) IDs registered according to a particular property type
        /// The property IDs are separated by the '#' character
        #[ink(message, payable)]
        pub fn property_claims(&self, property_type_id: PropertyTypeId) -> Vec<u8> {
            if let Some(property_ids) = self.claims.get(&property_type_id) {
                property_ids
                    .into_iter()
                    .fold(Vec::new(), |mut ids, inner_vec| {
                        ids.extend(inner_vec);
                        ids.push(b'#');
                        ids
                    })
            } else {
                Default::default()
            }
        }

        /// Return the details of a property
        /// The claimer is returned as the first element of the tuple
        /// The default value of the claimer is the caller. In this scenerio, the length of the vector will be the flag on the client side
        #[ink(message, payable)]
        pub fn property_detail(&self, property_id: PropertyId) -> (AccountId, Vec<u8>) {
            // get claimer
            let claimer = Self::env().caller();

            if let Some(property) = self.properties.get(&property_id) {
                (property.claimer, property.property_claim_addr.clone())
            } else {
                (claimer, Default::default())
            }
        }

        /// Transfer a property (or parts of it) from one user to the other
        /// If a part of the property is transferred, the new properties automatically becomes unattested and have to be signed afresh
        #[ink(message, payable)]
        pub fn transfer_property(
            &mut self,
            property_id: PropertyId,
            recipient: AccountId,
            senders_claim_ipfs_addr: PropertyClaimAddr,
            senders_property_id: PropertyId,
            recipients_claim_ipfs_addr: PropertyClaimAddr,
            recipients_property_id: PropertyId,
            time_of_transfer: PropertyTransferTimestamp,
        ) {
            // get caller (which is the account making the transfer)
            let caller = Self::env().caller();

            // get the property
            if let Some(mut property) = self.properties.get(&property_id) {
                // check if the property is being transferred as a whole
                if recipients_claim_ipfs_addr.len() != 0 {
                    // it wasn't
                    // delete the claims IPFS address because it is invalid now
                    if let Some(ids) = self.claims.get(&property.property_type_id) {
                        let filtered_ids = ids
                            .iter()
                            .filter(|&id| id != &property_id)
                            .cloned()
                            .collect::<Vec<PropertyId>>();
                        
                        self.claims
                            .insert(&property.property_type_id, &filtered_ids);
                    }

                    // now delete the (old whole) property record
                    self.properties.remove(&property_id);

                    // register new property under type of claim
                    if let Some(mut property_ids) = self.claims.get(&property.property_type_id) {
                        // append to the list if it doesn't contain it already
                        if !property_ids.contains(&senders_property_id) {
                            property_ids.push(senders_property_id.clone());
                        }

                        if !property_ids.contains(&recipients_property_id) {
                            property_ids.push(recipients_property_id.clone());
                        }

                        // insert the two new property IDs into storage
                        self.claims
                            .insert(property.property_type_id.clone(), &property_ids);
                    } else {
                        // create new class of properties and add the new one to it
                        let property_ids =
                            vec![senders_property_id.clone(), recipients_property_id.clone()];

                        // insert into contract storage
                        self.claims
                            .insert(property.property_type_id.clone(), &property_ids);
                    }

                    // create a new property document for the sender
                    let senders_property = Property {
                        claimer: caller.clone(),
                        property_claim_addr: senders_claim_ipfs_addr,
                        property_type_id: property.property_type_id.clone(),
                        transfer_history: vec![(caller.clone(), time_of_transfer)],
                    };

                    // create a new property document for the recipients
                    let recipients_property = Property {
                        claimer: recipient.clone(),
                        property_claim_addr: recipients_claim_ipfs_addr,
                        property_type_id: property.property_type_id.clone(),
                        transfer_history: vec![(caller.clone(), time_of_transfer)],
                    };

                    // register the both (unattested) property claims onchain
                    self.properties
                        .insert(senders_property_id.clone(), &senders_property);
                    self.properties
                        .insert(recipients_property_id.clone(), &recipients_property);
                } else {
                    // The property was tranferred as a whole
                    // Here we need not do much, just change the property claimer
                    // Then we add the time of transfer and the id of the previous owner
                    property.claimer = recipient;
                    property.property_claim_addr = senders_claim_ipfs_addr;
                    property.transfer_history.push((caller, time_of_transfer));

                    // save to contract storage
                    self.properties.insert(property_id.clone(), &property);
                }

                // emit event
                self.env().emit_event(PropertyTransferred {
                    sender: caller,
                    recipient,
                    property_id,
                });
            }
        }
    }
}
