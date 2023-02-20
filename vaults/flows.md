# Vaults logic flows

## Key features the contract requires


## Creation of a Vault

The creation of a Vault occurs when a Participant deposits a Collateral Asset and receives stablecoins in exchange.

```mermaid
flowchart TD;
    A([User])-.Deposit Collateral.->B[(Vault)];
    B-.Issue Stablecoins.->A;
```

There are multiple rules around creating a vault

```mermaid
flowchart TD;
    A([User])-->B(Calls ''new_vault'' function)
    B-->C{Alreadt has Vault?}
    C-->|Yes| D[Creates companies]
    C-->|No| E([Panic error])
```

