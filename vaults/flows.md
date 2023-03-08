# Vaults logic flows


## New Vault (new_vault)
```mermaid
flowchart TD
    A([Start]) --> CallsNewVault(Calls 'new_vault' function)
    CallsNewVault --> IsPanicModeON{Is Panic Mode Enabled?}
    IsPanicModeON --> |Yes| ThrowError
    IsPanicModeON --> |No| IsUserApproved{Is Authorized Caller?}
    IsUserApproved --> |No| ThrowError([Throw Error])
    IsUserApproved --> |Yes| AllValuesPositive{All values positive?}
    AllValuesPositive --> |No| ThrowError
    AllValuesPositive --> |Yes| VaultAlreadyCreated{Is Vault already created?}
    VaultAlreadyCreated --> |Yes| ThrowError
    VaultAlreadyCreated --> |No| CheckPriceUpdated{Updated in the last 15min?}
    CheckPriceUpdated --> |No| EnablePanicMode[Enable Panic Mode] --> ThrowError
    CheckPriceUpdated --> |Yes| CheckDepositRate{Is Initial debt valid?}
    CheckDepositRate --> |No| ThrowError
    CheckDepositRate --> |Yes| DepositCollateral[Deposit Collateral]
    DepositCollateral --> TakeFee[Take deposit fee]
    TakeFee --> WithdrawStable[Issue stable coin]
    WithdrawStable --> UpdateProtocolStats[Update protocol's stats]
    UpdateProtocolStats --> B([END])
```


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

