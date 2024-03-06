<img src="https://github.com/thewoodfish/property-delphi/blob/main/public/img/logo.png" style="width: 500px">

Property Delphi is a blockchain solution built with <a target="_blank" href="https://use.ink">ink!</a>, which empowers you to create an indisputable, irrefutable proof of ownership of your various properties. e.g a plot of land.
With Property Delphi, nobody can take what is yours and leave you with nothing.

# The Delphi Contract

This repo contains the code for the delphi smart contract running on a typical substrate contracts node. The property oracle contract is very important because it records critical information about users of the network, the properties being secured or claimed, the claims and right to claim of the properties and so on, serving as a source of truth and decisions. We would examine the contract properly. The property delphi contract is completely built with <a target="_blank" href="https://use.ink">ink!</a> which is the best language for writing smart contracts.

## Examining the `delphi contract`

We will now examine the various constructs that make our solution work, ranging from contract storage,data types,error types to the all important contract function that define the behaviour of the network and make state changes on chain.

### The Data types

- `AccountInfo`:

  ```
  pub struct AccountInfo {
      /// Name of user
      name: Vec<u8>,
      /// Time the account was created
      timestamp: TimeString,
  }
  ```

  This represents the account information of an entity participating on the network. It contains the name (or pseudo-name) of the user and the time the account was created.

- `Property`:
    ```
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
    ```
        pub struct PropertyType {
        /// Id of property type
        id: PropertyTypeId,
        address: PropertyRequirementAddr,
    }
    ```

    This represents the type of a property. There are many separate information that are inportant to various districts and states that it is important that property documents remain flexible and the authority of the area specify the exact information that in needed on a property document, to prove its validity. Hence, a property tyep.

- `