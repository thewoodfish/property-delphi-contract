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
        timestamp: TimeString,
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
        /// The time and the account that made the assertion
        assertion: (AssertionTimestamp, AccountId),
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

    /// Delphi's error type.
    #[derive(scale::Decode, scale::Encode, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Error {
        /// Returned when a property owner tries to transfer to himself
        CannotTransferToSelf,
        /// Returned when an unauthorized account tries to sign a property document (attestation)
        UnauthorizedAccount,
    }

    /// Delphi's result type.
    pub type Result<T> = core::result::Result<T, Error>;
    /// The id of the property
    type PropertyId = Vec<u8>;
    /// Timestamp in words (used because of issues returning and parsing a u64)
    type TimeString = Vec<u8>;
    /// The id of the property document type
    type PropertyTypeId = Vec<u8>;
    /// The IPFS address (CID) of the requirements of the property
    type PropertyRequirementAddr = Vec<u8>;
    /// The IPFS address (CID) of the document showing the rightful ownership of the property
    type PropertyClaimAddr = Vec<u8>;
    /// The Unix timestamp recording the time a property transfer was made
    type PropertyTransferTimestamp = TimeString;
    /// The time the assertion was made by the right authority after verifying that the property belongs to the account
    type AssertionTimestamp = Vec<u8>;
    /// The (JS) parsable AccountId in vector form
    type AccountIdVec = Vec<u8>;

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

    /// Event to announce the successful attestation of a property
    #[ink(event)]
    pub struct PropertyDocumentSigned {
        #[ink(topic)]
        attester: AccountId,
        #[ink(topic)]
        property_id: PropertyId,
    }

    #[ink(storage)]
    pub struct Delphi {
        accounts: Mapping<AccountId, AccountInfo>,
        registrations: Mapping<AccountId, Vec<PropertyType>>,
        claims: Mapping<PropertyTypeId, Vec<PropertyId>>,
        properties: Mapping<PropertyId, Property>,
        /// This Mapping field is simply unnecessary. But due to the fact that we've found it difficult to
        /// decode an AccountId with Javascript, we will be returning a vec instead of an accountId
        account_ids: Mapping<AccountId, AccountIdVec>,
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
                account_ids: Default::default(),
            }
        }

        /// Register an account
        #[ink(message, payable)]
        pub fn register_account(
            &mut self,
            account_id: AccountIdVec,
            name: Vec<u8>,
            timestamp: TimeString,
        ) -> Result<()> {
            // Get the contract caller
            let caller = Self::env().caller();

            let new_account = AccountInfo {
                name: name.clone(),
                timestamp,
            };

            // Insert into storage
            self.accounts.insert(&caller, &new_account);

            // Save the mapping of AccountId(real) -> AccountId(Vec)
            self.account_ids.insert(caller.clone(), &account_id);

            // Emit event
            self.env().emit_event(AccountCreated {
                account_id: caller,
                name,
            });

            Ok(())
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
        ) -> Result<()> {
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

            Ok(())
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
        ) -> Result<()> {
            // get claimer
            let claimer = Self::env().caller();

            // create a new property document
            let property = Property {
                claimer: claimer.clone(),
                property_claim_addr: claim_ipfs_addr,
                property_type_id: property_type_id.clone(),
                transfer_history: Vec::new(),
                // the claimer's address is the default value for the id of the asserting authority
                // this is not a bug as the assertion flag will be the timestamp of the signing of the document
                assertion: (Default::default(), claimer.clone()),
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

            Ok(())
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
        /// The default value of the claimer is the caller.
        /// The vector is the claimers parsable account id + the claim's IPFS address + the property type ID separated by a '$' character
        #[ink(message, payable)]
        pub fn property_detail(&self, property_id: PropertyId) -> Vec<u8> {
            let mut return_vec = Vec::new();

            if let Some(property) = self.properties.get(&property_id) {
                // get parsable account ID mapping to the claimers ID
                if let Some(account_id) = self.account_ids.get(&property.claimer) {
                    return_vec.extend(account_id.iter());
                    return_vec.push(b'$');
                }

                return_vec.extend(property.property_claim_addr.clone());
                return_vec.push(b'$');
                return_vec.extend(property.property_type_id.clone());
            }

            return_vec
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
        ) -> Result<()> {
            // get caller (which is the account making the transfer)
            let caller = Self::env().caller();

            // check to prevent transfer to self
            if recipient == caller {
                return Err(Error::CannotTransferToSelf);
            }

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
                        transfer_history: vec![(caller.clone(), time_of_transfer.clone())],
                        assertion: (Default::default(), caller.clone()),
                    };

                    // create a new property document for the recipients
                    let recipients_property = Property {
                        claimer: recipient.clone(),
                        property_claim_addr: recipients_claim_ipfs_addr,
                        property_type_id: property.property_type_id.clone(),
                        transfer_history: vec![(caller.clone(), time_of_transfer)],
                        assertion: (Default::default(), recipient.clone()),
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

            Ok(())
        }

        /// Sign a property document and cement the owner as the undisputed rightful owner of the property.
        /// It returns an error if the attested is unauthorized to attest ownership.
        /// Authorization is gotten by checking for equality between the account that created the property type and the attesting account
        #[ink(message, payable)]
        pub fn sign_document(
            &mut self,
            property_id: PropertyId,
            property_type_id: PropertyTypeId,
            assertion_timestamp: AssertionTimestamp,
        ) -> Result<()> {
            // get caller (which is the account making the attestation)
            let caller = Self::env().caller();

            // check that only the authorized account can sign.
            if let Some(property_types) = self.registrations.get(&caller) {
                if !property_types
                    .iter()
                    .any(|ptype| ptype.id == property_type_id)
                {
                    // error! unauthorized
                    return Err(Error::UnauthorizedAccount);
                }
            }

            // now sign document
            if let Some(mut property) = self.properties.get(&property_id) {
                property.assertion = (assertion_timestamp, caller.clone());

                // update property
                self.properties.insert(&property_id, &property);

                // emit event
                self.env().emit_event(PropertyDocumentSigned {
                    attester: caller,
                    property_id,
                });
            }

            Ok(())
        }

        /// Return the verification status of a property.
        /// This verification status includes: 1. AccountIds showing transfer History 2. AssertionTimestamp
        /// The accountId's showing transfer history are separated with a '$' character.
        /// The history is separated from the timestamp by a '@' character
        #[ink(message, payable)]
        pub fn attestation_status(&self, property_id: PropertyId) -> Vec<u8> {
            // the vector we are returning, containing all the accountIds that have had possession of the property
            let mut transfer_history = Vec::new();
        
            if let Some(property) = self.properties.get(&property_id) {
                // we need to return AccountIdVec, hence we need to make the conversion
                for (account_id, _) in &property.transfer_history {
                    transfer_history.push(self.convert_accountid_to_vec(account_id));
                }
        
                // Flatten and concatenate the vectors in transfer_history
                let mut flattened_history = Vec::new();
                for inner_vec in &transfer_history {
                    flattened_history.extend(inner_vec.iter().copied());
                    flattened_history.push(b'$');
                }
        
                // append the assertion timestamp to it
                flattened_history.push(b'@');
                flattened_history.extend(property.assertion.0.iter());
                flattened_history
            } else {
                // 0 is the flag to indicate that the property has not been attested
                Default::default()
            }
        }

        /// Helper function to convert an AccountId into an AccountIdvec.
        /// It uses the account_ids mapping property of our contract storage
        pub fn convert_accountid_to_vec(&self, account_id: &AccountId) -> AccountIdVec {
            if let Some(account_id_vec) = self.account_ids.get(account_id) {
                account_id_vec
            } else {
                Default::default()
            }
        }
    }
}
