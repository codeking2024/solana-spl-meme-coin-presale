# Dyawn Presale Smart Contract
## Author: John Lee from Digital Heores

This is a Solana-based presale smart contract that allows users to purchase tokens using SOL, USDT, or USDC, manage sale stages, implement referral rewards, and more.
It supports secure token sales with a PDA-based token authority and off-chain storage for influencer and referral management.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Accounts in Contract](#Accounts-in-Contract)
- [Instruction Methods](#instruction-methods)
- [Accounts Explained](#accounts-explained)
- [Error Codes](#error-codes)
- [Integration Guide](#integration-guide)
- [Security Considerations](#security-considerations)
- [License](#license)

## Overview

The Dyawn Presale Smart Contract facilitates a token presale on the Solana blockchain, supporting multiple cryptocurrencies for payments, dynamic sale stages, and a referral system.
Built with the Anchor framework, this contract ensures secure and efficient token transactions using Solana's Program Derived Addresses (PDAs).

## Features

- Token purchases with **SOL**, **USDT**, or **USDC**.
- **Referral system** rewarding referrers with tokens from a dedicated treasury.
- **Dynamic sale stages**: PreLaunch, Private Sale, Public Sale, and Market.
- **PDA-based authority** for secure token operations.
- **Admin controls** for manually updating prices, stages, and refilling the referral treasury.
- Off-chain storage integration for influencers and referral rewards verification.

## Accounts in Contract

### 1. **Presale Account (`presale`):**

- **Purpose:** Main state account that stores all presale information such as token mint, admin authority (PDA), total supply, referral treasury, prices, sale stages, and timing data.
- **Seed:** `[b"xxxx"]` with a bump, ensuring a unique Program Derived Address (PDA).

### 2. **Admin Account (`admin`):**

- **Purpose:** The admin who initializes the contract and pays for the initialization. This account is used only during the initialization phase.

### 3. **Merchant Account (`merchant_account`):**

- **Purpose:** The wallet that receives SOL and stable coin payments from buyers.
  Set during initialization and used in both SOL and stable coin purchases.

### 4. **Token Mint Account (`presale_token_mint`):**

- **Purpose:** The mint account for the presale token, initialized with 9 decimals and owned by the presale PDA authority.

### 5. **Presale Token Account (`presale_token_account`):**

- **Purpose:** The token account holding the supply of tokens for sale, owned by the presale PDA.

### 6. **Referral Treasury Account (`referral_treasury_account`):**

- **Purpose:** Holds tokens reserved for referral rewards, managed by the presale PDA authority.

### 7. **Buyer Account (`buyer`):** **(User Wallet Connected to the Solana web3)**

- **Purpose:** Represents the user purchasing tokens. Required for signing transactions during token purchases.

### 8. **Buyer Token Account (`buyer_token_account`):**

- **Purpose:** The token account where purchased tokens are transferred after a successful purchase.
- **Generated from** Wallet address using solana web3

### 9. **Stable Coin Mint (`stable_coin_mint`):**

- **Purpose:** Represents the mint of the stable coin used in `buy_tokens_by_stable_coin` (must be USDT or USDC).
- **It is the stable coin mint address** like USDT or USDC

### 10. **Buyer Stable Coin Account (`buyer_stable_coin_account`):**

- **Purpose:** The buyer’s token account holding the stable coins (USDT/USDC) used for purchasing tokens. -**Generated from** wallet using solana web3

### 11. **Merchant Stable Coin Account (`merchant_stable_coin_account`):**

- **Type:** `Account<'info, TokenAccount>`
- **Purpose:** Merchant’s token account for receiving stable coin payments.

## Instruction Methods

### initialize

Initializes the presale with total supply, prices, sale timings, and mints tokens.

#### Initialization Parameters

When deploying the Dyawn Presale Smart Contract, Web3 developers must provide the following initialization parameters:

| Parameter                | Type | Description                                                                                       |
| ------------------------ | ---- | ------------------------------------------------------------------------------------------------- |
| `totalSupply`            | u64  | Total supply of tokens to be minted (e.g., `1000000000` for 1 billion tokens).                    |
| `referralTreasuryAmount` | u64  | Amount of tokens allocated to the Referral Treasury (e.g., `500000` tokens).                      |
| `startPrice`             | u64  | Initial price per token in USD (multiplied by 100,000 for precision, e.g., `3500` means $0.0035). |
| `publicSalePrice`        | u64  | Price per token in the public sale phase (e.g., `7000` means $0.007).                             |
| `presaleStart`           | i64  | UNIX timestamp for when the presale starts (e.g., `1739454094` for 2025-03-15 00:01:34 UTC).      |
| `privateSalePeriod`      | i64  | Duration of the private sale period in days (e.g., `15` days).                                    |

### Accounts for Initialization

When calling the `initialize` function of the Dyawn Presale Smart Contract, Web3 developers must provide the following accounts:

| Account                   | Type     | Mutable | Signer | Description                                                                                                                   |
| ------------------------- | -------- | ------- | ------ | ----------------------------------------------------------------------------------------------------------------------------- |
| `presale`                 | `pubkey` | ✅      |        | PDA account for presale state, derived from the seed and bump. pls use `6TohdpqajjZ4rLUZMqkj27AAq44zLm4HGt7o4qaM5diC` for now |
| `admin`                   | `pubkey` | ✅      | ✅     | Admin wallet initializing the contract and paying for the transaction.                                                        |
| `merchantAccount`         | `pubkey` | ✅      |        | Merchant’s wallet to receive SOL and stable coin payments.                                                                    |
| `presaleTokenMint`        | `pubkey` | ✅      | ✅     | Token mint account for the presale tokens, created during initialization.                                                     |
| `presaleTokenAccount`     | `pubkey` | ✅      | ✅     | Token account holding presale tokens, associated with the mint and owned by the presale PDA.                                  |
| `referralTreasuryAccount` | `pubkey` | ✅      | ✅     | Token account holding referral reward tokens, associated with the mint and owned by the presale PDA.                          |
| `systemProgram`           | `pubkey` |         |        | Solana System Program (`11111111111111111111111111111111`) for system-level operations.                                       |
| `tokenProgram`            | `pubkey` |         |        | Solana Token Program (`TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`) for SPL token operations.                                |

Web3 devs should generate these three accounts before initializing...

```js
import { Keypair, PublicKey, Connection, clusterApiUrl } from "@solana/web3.js";

// Create a new connection to the Solana cluster
const connection = new Connection(clusterApiUrl("devnet"), "confirmed");

// 1. Generate a random keypair for the token mint
const presaleTokenMint = Keypair.generate();
console.log("Presale Token Mint:", presaleTokenMint.publicKey.toBase58());

// 2. Generate a random keypair for the presale token account
const presaleTokenAccount = Keypair.generate();
console.log("Presale Token Account:", presaleTokenAccount.publicKey.toBase58());

// 3. Generate a random keypair for the referral treasury account
const referralTreasuryAccount = Keypair.generate();
console.log(
  "Referral Treasury Account:",
  referralTreasuryAccount.publicKey.toBase58()
);

// These generated accounts can now be provided as parameters when calling the `initialize` function.
```

### buy_tokens

Allows users to buy tokens with SOL, verifying sufficient funds and transferring tokens securely.

#### Accounts for `buy_tokens` Function

When calling the `buy_tokens` function of the Dyawn Presale Smart Contract, Web3 developers must provide the following accounts:

| Account               | Type     | Mutable | Signer | Description                                                                                                                                    |
| --------------------- | -------- | ------- | ------ | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `presale`             | `pubkey` | ✅      |        | It is from the initialize function.                                                                                                            |
| `buyer`               | `pubkey` | ✅      | ✅     | Wallet address of the buyer purchasing the tokens.                                                                                             |
| `presaleTokenAccount` | `pubkey` | ✅      |        | Token account holding presale tokens, associated with the mint and owned by the presale PDA. it will be provided from the initialize function. |
| `buyerTokenAccount`   | `pubkey` | ✅      |        | Token account of the buyer where purchased tokens will be transferred. it should be generated from user wallet using solana web3.              |
| `merchantAccount`     | `pubkey` | ✅      |        | Merchant’s wallet to receive SOL payments.                                                                                                     |
| `systemProgram`       | `pubkey` |         |        | Solana System Program (`11111111111111111111111111111111`) for SOL transfers.                                                                  |
| `tokenProgram`        | `pubkey` |         |        | Solana Token Program (`TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`) for SPL token transfers.                                                  |

#### How to get ATA from User Wallet using web3?

```js
import { PublicKey } from "@solana/web3.js";
import { getAssociatedTokenAddress } from "@solana/spl-token";

// Function to get the buyer's associated token account
async function getBuyerTokenAccount(walletAddress, mintAddress) {
  const walletPubkey = new PublicKey(walletAddress);
  const mintPubkey = new PublicKey(mintAddress);

  const buyerTokenAccount = await getAssociatedTokenAddress(
    mintPubkey, // Token mint address
    walletPubkey // Owner wallet address
  );

  console.log("Buyer Token Account:", buyerTokenAccount.toBase58());
  return buyerTokenAccount;
}

// Example usage
const walletAddress = "BQUHqj6LgS38464fmTguhN6SRrTLucy1ggGGcefZrX...";
const mintAddress = "ECxown6bSKDmM3TD6PbUHKzY6CYfzXgrx8UCA4...";

getBuyerTokenAccount(walletAddress, mintAddress);
```

### buy_tokens_by_stable_coin

Enables token purchases using USDT/USDC, ensuring stable coin validity and transferring tokens accordingly.

### update_stage

Automatically updates the sale stage based on the current time.

### set_stage

Allows the admin to manually set the sale stage using the PDA authority.

### set_price

Enables the admin to manually update the token price.

### withdraw_rewards

Permits referrers to withdraw their earned tokens from the Referral Treasury after verification.

### refill_treasury

Lets the admin add more tokens to the Referral Treasury within the supply cap limit.

```

```
