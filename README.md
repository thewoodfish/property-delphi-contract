<img src="https://github.com/thewoodfish/property-delphi/blob/main/public/img/logo.png" style="width: 500px">

Property Delphi is a blockchain solution built with <a target="_blank" href="https://use.ink">ink!</a>, which empowers you to create an indisputable, irrefutable proof of ownership of your various properties. e.g a plot of land.
With Property Delphi, nobody can take what is yours and leave you with nothing.

# The Delphi Contract

This repo contains the code for the delphi smart contract running on a typical substrate contracts node. The property oracle contract is very important because it records critical information about users of the network, the properties being secured or claimed, the claims and right to claim of the properties and so on, serving as a source of truth and decisions. We would examine the contract properly. The property delphi contract is completely built with <a target="_blank" href="https://use.ink">ink!</a> which is the best language for writing smart contracts.

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

Everything revolves around the storage of a smart contract. It is important that they are wisely thought about nd efficiently built. Here, we used qa `Mapping` to store our various important and necessary data. This is the memory of the great delphi and everything she does either involves changing this data or reading and infering from it. Powerful eh? 😃 Let us examing the storage:

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

- The `new` function:
    - Signature: 
        ```rust 
        pub fn new() -> Self { }
        ```
    - Make state changes: Yes, initializes contract storage.
    - Arguments: None.
    - Return Values: None.
    - Desc: The new function is the first function called before the others, at initialization. It initializes a contract storage and prepares it for reading and writing.

- 