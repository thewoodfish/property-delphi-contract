<img src="https://github.com/thewoodfish/property-delphi/blob/main/public/img/logo.png" style="width: 500px">

Property Delphi is a blockchain solution built with <a target="_blank" href="https://use.ink">ink!</a>, which empowers you to create an indisputable, irrefutable proof of ownership of your various properties. e.g a plot of land.
With Property Delphi, nobody can take what is yours and leave you with nothing.

# The Delphi Contract

This repo contains the code for the delphi smart contract running on a typical substrate contracts node. The property delphi contract is very important because it records critical information about users of the network, the properties being secured or claimed, the claims and right to claim of the properties and so on, serving as a source of truth and decisions. We would examine the contract properly. The property delphi contract is completely built with <a target="_blank" href="https://use.ink">ink!</a> which is the best language for writing smart contracts.

## Examining the `delphi contract`

We will now examine the various constructs that make our solution work, ranging from contract storage,data types,error types to the all important contract function that define the behaviour of the network and make state changes on chain.

### The Data types

- `AccountInfo`:

  ```rust
  pub struct AccountInfo {
      /// Name of user
      name: Vec<u8>,
      /// Time the account was created
      timestamp: TimeString,
  }
  ```

  This represents the account information of an entity participating on the network. It contains the name (or pseudo-name) of the user and the time the account was created.

- `Property`:

  ```rust
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
  ```

  This represents the neccesary information associated with a property. It describes a piece of property, contains the history of the property transfer and acquisition and the all-importqant attestation status of the property.

- `PropertyType`:

  ```rust
      pub struct PropertyType {
      /// Id of property type
      id: PropertyTypeId,
      address: PropertyRequirementAddr,
  }
  ```

  This represents the type of a property. There are many separate information that are inportant to various districts and states that it is important that property documents remain flexible and the authority of the area specify the exact information that in needed on a property document, to prove its validity. Hence, a property type.

### The Error Types

Errors help us handle strange behaviour in our contract and we have defined just two of them:

```rust
    pub enum Error {
        /// Returned when a property owner tries to transfer to himself
        CannotTransferToSelf,
        /// Returned when an unauthorized account tries to sign a property document (attestation)
        UnauthorizedAccount,
    }
```

### Type Aliases

Type aliases helps us have neater and more readable code. Here are the error types defined below:

```rust
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
```

- The `TimeString` type should be a `u64` but we found it hard to decode on the client side, hence we opted for a byte array, which is easily decodable.
- The `AccountIdVec` type is the samething as the standard Polkadot `AccountId`, its a collection on bytes, which makes it easier to work with.

### Event Types

Events are important in smart contracts and blockchains. They help us and external observers know exactly what is happening and state changes being made, all without observing and tracking onchain storage changes explicitly. Here are the events defined by the great delphi:

- The `AccountCreated` Event:

```rust
    /// Event to announce the creation of an account
    #[ink(event)]
    pub struct AccountCreated {
        #[ink(topic)]
        account_id: AccountId,
        name: Vec<u8>,
    }
```

- The `PropertyTypeRegistered` Event:

```rust
    //// Event to announce the registration of a property type
    #[ink(event)]
    pub struct PropertyTypeRegistered {
        #[ink(topic)]
        account_id: AccountId,
        property_type_id: PropertyTypeId,
        ptype_ipfs_addr: PropertyRequirementAddr,
    }
```

- The `PropertyClaimRegistered` Event:

```rust
    //// Event to announce the registration of a claim to a property
    #[ink(event)]
    pub struct PropertyClaimRegistered {
        #[ink(topic)]
        claimer: AccountId,
        #[ink(topic)]
        property_type_id: PropertyTypeId,
        property_id: PropertyId,
    }
```

- The `PropertyTransferred` Event:

```rust
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
```

- The `PropertyDocumentSigned` Event:

```rust
    /// Event to announce the successful attestation of a property
    #[ink(event)]
    pub struct PropertyDocumentSigned {
        #[ink(topic)]
        attester: AccountId,
        #[ink(topic)]
        property_id: PropertyId,
    }
```

### The Delphi Storage

Everything revolves around the storage of a smart contract. It is important that they are wisely thought about nd efficiently built. Here, we used a `Mapping` to store our various important and necessary data. This is the memory of the great delphi and everything she does either involves changing this data or reading and infering from it. Powerful eh? 😃 Let us examing the storage:

```rust
    #[ink(storage)]
    pub struct Delphi {
        accounts: Mapping<AccountId, AccountInfo>,
        registrations: Mapping<AccountId, Vec<PropertyType>>,
        claims: Mapping<PropertyTypeId, Vec<PropertyId>>,
        properties: Mapping<PropertyId, Property>,
        /// This Mapping field is simply unnecessary. But due to the fact that we've found it difficult to
        /// decode an AccountId with Javascript, we will be returning a Vec<u8> instead of an accountId
        account_ids: Mapping<AccountId, AccountIdVec>,
    }
```

- The `accounts Mapping` maps an `AccountId` to its information.
- The `registrations Mapping` maps an authority's (e.g Ministry of land) `AccountId` to the various property types they've registered (represented in a vector).
- The `claims Mapping` maps a property type id to the various properties that fit and are and are subject to provide the information specified in the property type due to geography.
- The `properties Mapping` maps a property identifier to the important details describing the property and its claims.
- The `account_ids Mapping` is a special and simply uneccesary mapping. It maps an account id of bytes to the standard `AccountId`. We added this to improvise and save time on our relentless effort to parse and decode an `AccountId` gotten from the contract. Instead, we simply return a parsable `AccountIdVec`in its stead.

### The Contract Functions

Functions are perhaps the core of a contract. They help interact with onchain storage and make state changes to them. Also, they serve as a mean to peek into contract storage and make inferences and decisions. We will not examing the functions utilized by the great delphi:

- **Initialize contract storage**:
    ```rust
    pub fn new() -> Self { }
    ```
    - Modifies storage: Yes, initializes contract storage.
    - Arguments: None.
    - Return Values: It return the contract storage.
    - Description: The new function is the first function called before the others, at initialization. It initializes a contract storage and prepares it for reading and writing.

- **Register new account**:
    ```rust
    pub fn register_account(
        &mut self,
        account_id: AccountIdVec,
        name: Vec<u8>,
        timestamp: TimeString,
    ) -> Result<()> { ... }
    ```

    - Modifies storage: Yes.
    - Arguments:
        - `account_id`: Account ID vector containing the parsable u8 `AccountId` vector.
        - `name`: The name (or pseudo-name) of the account owner.
        - `timestamp`: The time of creation of the aaccount;
    - Return Values: None.
    - Description: It created a new account on the delphi contract.

- **Check for account existence**:
    ```rust
    pub fn account_exists(&self) -> (bool, Vec<u8>) { ... }
    ```
    - Modifies storage: No
    - Arguments: None.
    - Return Values: It return a boolean to indicate the existence of an account and the name on the account, if any.
    - Description: It is a getter function that checks if an account exists in the contract storage.


- **Register property type**:
    ```rust
    pub fn register_ptype(
        &mut self,
        property_type_id: PropertyTypeId,
        ptype_ipfs_addr: PropertyRequirementAddr,
    ) -> Result<()> { ... }
    ```  
    - Modifies storage: Yes
    - Arguments: 
        - `property_type_id`: The ID describing a particular property type document schema peculiar to a particular region.
        - `ptype_ipfs_addr`: The IPFS CID of the property document schema.
    - Return Values: None.
    - Description: It registers a particular document schema type peculiar to a particular location onchain.

- **Get property claims**:
    ```rust
    pub fn property_claims(&self, property_type_id: PropertyTypeId) -> Vec<u8> { ... }
    ```
    - Modifies storage: No
    - Arguments: 
        - `property_type_id`: The ID describing a particular property type document schema peculiar to a particular region.
    - Return Values: It returns a list of property (claims) ID.
    - Description: It is a getter function that retrieves a list of property (claims) IDs registered according to a particular property type schema.

- **Get property details**:
    ```rust
    pub fn property_detail(&self, property_id: PropertyId) -> Vec<u8> { ... }
    ```
    - Modifies storage: No
    - Arguments: 
        - `property_id`: The ID of a particular property.
    - Return Values: It returns important attributes of the property object stored in contract storage.
    - Description: It is a getter function that returns important attributes relating to a particular property.

- **Transfer property**:
    ```rust
    pub fn transfer_property(
        &mut self,
        property_id: PropertyId,
        recipient: AccountId,
        senders_claim_ipfs_addr: PropertyClaimAddr,
        senders_property_id: PropertyId,
        recipients_claim_ipfs_addr: PropertyClaimAddr,
        recipients_property_id: PropertyId,
        time_of_transfer: PropertyTransferTimestamp,
    ) -> Result<()> { ... }
    ```
    - Modifies storage: Yes
    - Arguments: 
        - `property_id`: The ID of a particular property.
        - `recipient`: The accountId of the user recieving the property.
        - `senders_claim_ipfs_addr`: The IPFS CID of the new property document the sender is entitiled to. This is as a result of the modification and discard of the old document.
        - `senders_property_id`: The property ID of the new property document the sender holds.
        - `recipients_claim_ipfs_addr`: The IPFS CID of the property document of the property being sent to the recipient.
        - `recipients_property_id`: The property ID of the new property document the recipients holds.
        - `time_of_transfer`: The time the transfer operation was dispatched.
    - Return Values: None.
    - Description: It transfers a piece of property from one account to the other, in part or in full.

- **Sign document**:
    ```rust
    pub fn sign_document(
        &mut self,
        property_id: PropertyId,
        property_type_id: PropertyTypeId,
        assertion_timestamp: AssertionTimestamp,
    ) -> Result<()> { ... }
    ```
    - Modifies storage: Yes
    - Arguments: 
        - `property_id`: The ID of a particular property.
        - `property_type_id`: The ID describing a particular property type document schema peculiar to a particular region.
        - `assertion_timestamp`: The time the transfer operation was dispatched.
    - Return Values: None.
    - Description: It confirms and solidifies an accounts claim to a piece of property. This can only be done by the right specific authority.

- **Get attestation status**:
    ```rust
    pub fn attestation_status(&self, property_id: PropertyId) -> Vec<u8> { ... }
    ```
    - Modifies storage: No
    - Arguments: 
        - `property_id`: The ID of a particular property.
    - Return Values: It returns the attestation status of a piece if property and the various previous owners, if any.
    - Description: It is a getter function that returns the attestation status of a piece if property and the various previous owners, if any.


## Running a local node 
- Install the necessary `Rust toolchains` and configure them. Please take a look at <a target="_blank" href="https://docs.substrate.io/install/">this page</a> to guide you appropriately.
- After installation, download a substrate contracts node and start it running.
- After cloning, open the terminal in the root folder and run the command: `cargo contract build --release`.
- After your contract is built, instantiate it on the contracts node.
- To interact with the Property Delphi front end, you have to set it up first. kindly read through the set up at: https://github.com/thewoodfish/property-delphi#how-to-run-or-test-property-delphi. It is very easy.

## Going forward
There a few improvements being considered for the property delphi contract going forward.
- Using the timestamp (`u64`) instead of a timestring
- Returning an `AccountId` instead of a vector of bytes.
- etc.


## Conclusion
Property Delphi helps secure your properties and removes any worries or stress from the thought/incident of losing what belongs to you. So even though your physical documents are duplicated or falsified by scammers, once you've submited your document using property delphi and gotten the right authority to sign it, 😁 YOU MY FRIEND, ARE VERY SAFE, FOREVER! The great Delphi will alwaysspeak in your favour!

Thank you for your time. God bless you! ❤️