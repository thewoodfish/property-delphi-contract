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

