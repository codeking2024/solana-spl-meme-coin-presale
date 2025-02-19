# Dyawn Presale Smart Contract

## Author: John Lee at Digital Heores

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

# Dyawn Token Ecosystem

## 1. Token Mint

### Mint Contract (One-Time User)

`Mints` 8.2 billion tokens once and `transfers` them to the `admin wallet`.
Send `30% of tokens` directly to `Liquidity Wallets`. ( liquidity Wallet address required)
`Disables minting` permanently to prevent further creation of tokens.
🔹 Lifecycle:
Used only once during token creation.
Once tokens are minted and distributed, this contract is no longer needed.

Solana Playground (https://beta.solpg.io/)

🔹 Step 1: Create a New Token Mint

`spl-token create-token --decimals 9`

This will return a MINT_ADDRESS. Copy it!

🔹 Step 2: Create the Admin Wallet’s Token Account

`spl-token create-account <MINT_ADDRESS> --owner <ADMIN_WALLET_ADDRESS>`

This creates a token account (ADMIN_ATA) for the admin.

🔹 Step 3: Mint 8.2 Billion Tokens to the Admin Wallet

`spl-token mint <MINT_ADDRESS> 8200000000 <ADMIN_ATA>`

Tokens are automatically sent to the admin’s associated token account.

🔹 Step 4: Add MetaData

```js
import * as web3 from "@solana/web3.js";
import {
  createCreateMetadataAccountV3Instruction,
  PROGRAM_ID as TOKEN_METADATA_PROGRAM_ID,
} from "@metaplex-foundation/mpl-token-metadata";

// ✅ Replace with your Mint Address (already created token)
const MINT_ADDRESS = new web3.PublicKey(
  "GUAU4kKNmJJeuKWaVsQbFVuAW9wb5swLPq3TB3PxjiHE"
);

// ✅ Replace with your Metadata Details
const TOKEN_NAME = "Dyawn2025";
const TOKEN_SYMBOL = "DYAWN";
const IMAGE_URL =
  "https://gateway.pinata.cloud/ipfs/bafkreifxxkkywltqbyn3djxi7j3rbdzlf6tpylj3gdtvqdcg4w42w3za4y"; // Upload JSON to IPFS/Arweave
const DESCRIPTION = "This is my Solana token with metadata!";

const main = async () => {
  // ✅ Use Solana Playground's wallet and connection
  const connection = pg.connection;
  const wallet = pg.wallet;

  console.log("Using Wallet:", wallet.publicKey.toBase58());

  // ✅ Get Metadata PDA (Program Derived Address)
  const [metadataPDA] = web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      MINT_ADDRESS.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );

  console.log("Metadata PDA:", metadataPDA.toBase58());

  // ✅ Create Metadata Instruction
  const metadataInstruction = createCreateMetadataAccountV3Instruction(
    {
      metadata: metadataPDA,
      mint: MINT_ADDRESS,
      mintAuthority: wallet.publicKey,
      payer: wallet.publicKey,
      updateAuthority: wallet.publicKey,
    },
    {
      createMetadataAccountArgsV3: {
        data: {
          name: TOKEN_NAME,
          symbol: TOKEN_SYMBOL,
          uri: IMAGE_URL, // JSON metadata link
          sellerFeeBasisPoints: 0, // 0% fee
          creators: null,
          collection: null,
          uses: null,
        },
        isMutable: true,
        collectionDetails: null,
      },
    }
  );

  // ✅ Create and Sign the Transaction
  const transaction = new web3.Transaction().add(metadataInstruction);
  transaction.feePayer = wallet.publicKey;

  // ✅ Get Latest Blockhash
  const { blockhash } = await connection.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;

  // ✅ Sign and Send Transaction using pg.wallet
  const signedTransaction = await wallet.signTransaction(transaction);
  const signature = await connection.sendRawTransaction(
    signedTransaction.serialize()
  );

  console.log("✅ Metadata Added Successfully!");
  console.log("Transaction Signature:", signature);
};

// Run the script
main().catch(console.error);
```

🔹 Step 5: Disable Minting (Revoke Authority)

`spl-token authorize <MINT_ADDRESS> mint --disable`

### Main Token Contract (Permanent Core Logic)

Defines the DYAWN token and handles all future interactions.
Responsible for token `transfers`, `burning`, and `ownership management`.

🔹 Lifecycle:
This contract runs forever as long as DYAWN exists.

🔹 Key Features:
✅ Handle token transfers between users.
✅ Implements the burn function (users or admin can destroy tokens).
✅ Maintains total supply and ensures compliance with Solana SPL standards.
✅ Holds ownership controls (admin can execute limited actions like enabling/disabling features).

- The Main Token Contract can be designed to allow governance proposals for community-driven upgrades.
- The Main Token Contract can integrate staking logic, where users can:
  Lock their DYAWN tokens for staking rewards.
  Automatically receive staking rewards in DYAWN or another token.
- The Main Token Contract can introduce features like:
  Automatic token burns based on transactions.
  Dynamic transaction fees for liquidity, staking, or referrals.
  `setBurnRate and Burn Mechanism`

## 2. Distribution of tokens according to tokenomics

Tokens will be distributed into multiple wallets based on tokenomics allocation `manually`.

Presale: 40% (3.28B tokens)

Liquidity (DEX & CEX) 30% 2.46B => Already transffered while minting.

• DEX Liquidity Provision (15%)

• CEX Liquidity Reserves (15%) During minting, allocate the full liquidity amount (30% total). Immediately distribute: 15% to DEX liquidity wallet (to be used for Raydium, Orca, Serum). 15% to CEX liquidity wallet (reserved for exchange listings). Cold wallets remain untouched until liquidity deployment begins.

Marketing & Partnerships: 10% (820M tokens)

Referral Program & Incentives 10% 820M

Airdrop & Staking Rewards: 5% 410M

Development & Team 5% 410M

Wallet addresses will be publicly shared for transparency.

## 1. Why Manual Token Distribution is Preferred Over Automatic Distribution

In the past, automatic token distribution seemed like a convenient approach for managers to handle allocations efficiently. However, after extensive research and industry experience, it is evident that most projects prefer manual distribution due to its security, flexibility, and control benefits.

### ❌ Risks of Automatic Token Distribution

#### 1️⃣ Unauthorized or Accidental Transfers

An attacker could manipulate automatic distribution to steal funds.
Manual distribution ensures tokens are only sent to verified wallets.

#### 2️⃣ Incorrect Wallet Addresses for Off-Chain Investors

Many early investors, partners, or advisors may not have provided their correct Solana wallets.
If tokens are automatically sent to an invalid address, they may be permanently lost.
Manual distribution ensures that all wallets are verified before sending tokens.

#### 3️⃣ No Adjustments for Unsold Tokens

Automatic distribution cannot adapt to changes if some presale tokens remain unsold.
Unsold tokens should be reallocated for staking rewards, liquidity pools, or burned.
Manual control ensures that excess tokens are handled properly.

### ✅ Benefits of Manual Token Distribution

#### 1️⃣ Flexibility for Tokenomics Adjustments

Token allocations may change due to discussions with investors, partners, or the community.
If tokens were automatically distributed, allocations would be locked, preventing any adjustments.
Manual distribution allows real-time modifications before transferring tokens.

#### 2️⃣ Control Over Timing of Distributions

Instead of distributing all tokens at once, manual distribution allows for phased releases.
This helps prevent market dumping and ensures a more stable price action.

#### 3️⃣ Gradual Liquidity Addition

Instead of immediately adding all tokens to DEX liquidity pools, manual distribution allows for:
✅ Strategic liquidity injections to prevent excessive volatility.
✅ Gradual releases to match market demand and maintain price stability.

## 2. Why Do We Need a Separate "Main Token Contract" Instead of Just Using SPL Token Program?

Your are correct that Solana’s SPL Token Program already includes essential token functions, such as:

#### `InitializeMint → Creates the token`.

#### `Burn → Allows token burning`.

#### `Transfer → Moves tokens between accounts`.

#### `Approve/Revoke → Manages spending allowances`.

#### `SetAuthority → Changes token ownership`.

#### `FreezeAccount/ThawAccount → Controls token freezing`.

#### `CloseAccount → Closes token accounts`.

So Why Do We Need a Separate "Main Token Contract"?\
The SPL Token Program only provides basic token operations. However,\
 it does not handle custom business logic such as:

#### ✅ Custom Tokenomics Management ( for the future update)

SPL tokens cannot enforce burn mechanisms based on trading volume or automated tax collection.
The Main Token Contract can introduce features like:
Automatic token burns based on transactions.
Dynamic transaction fees for liquidity, staking, or referrals.

#### ✅ Advanced Token Control Features

SPL does not have built-in access control for admin-level functions.
A custom contract allows role-based access, ensuring that only authorized accounts can:
Adjust burn rates dynamically.
Distribute marketing and staking rewards securely.

#### ✅ Staking & Rewards System (for the future update)

SPL does not support staking directly.
The Main Token Contract can integrate staking logic, where users can:
Lock their DYAWN tokens for staking rewards.
Automatically receive staking rewards in DYAWN or another token.

#### ✅ Custom Governance & Upgradeability

SPL tokens are not upgradable.
The Main Token Contract can be designed to allow governance proposals for community-driven upgrades.

# **DYAWN Token Allocation Proposal**

## **Overview**

To ensure a balanced and sustainable token economy, we have revised the allocation strategy based on client feedback. The updated model improves liquidity, enhances referral incentives, and ensures staking rewards without relying solely on future fees.

---

## 3. Updated Token Distribution

| **Category**                      | **Allocation (%)** | **Amount (Billion DYAWN)** | **Purpose**                                                                      |
| --------------------------------- | ------------------ | -------------------------- | -------------------------------------------------------------------------------- |
| **Sales (Private & Public)**      | 40%                | 3.28B                      | Tokens allocated for private and public sales, funding the project.              |
| **Liquidity (DEX & CEX)**         | 30%                | 2.46B                      | Increased liquidity to support smooth trading and reduce volatility.             |
| **Marketing & Partnerships**      | 10%                | 820M                       | Funds allocated for promotional campaigns, partnerships, and brand growth.       |
| **Referral Program & Incentives** | 10%                | 820M                       | Supports referral rewards and incentive programs to encourage participation.     |
| **Airdrop & Staking Rewards**     | 5%                 | 410M                       | Ensures early staking rewards and airdrop distribution for community engagement. |
| **Development & Team**            | 5%                 | 410M                       | Reduced team allocation with a structured vesting period to maintain stability.  |

---

## 4. Response to Liquidity Wallet Funding Feedback

What Does "Manual Funding" Mean?\
Yes, manually funding a wallet means sending tokens manually from the admin wallet to a specific user or contract wallet instead of having an automated distribution system within a smart contract.

### Presale Wallet (Smart Contract):

Manually send tokens to presale account to prevent vulnerabilities and limit token exposure.
Holds only the tokens necessary for the active presale.

### Liquidity Wallets (Cold Storage for DEX & CEX):

You are absolutely right!
Liquidity Pools Need Immediate Allocation for Stability
Should be funded directly after or during minting, not manually like the presale wallet.

Adjusted Approach for Liquidity Wallet Funding
🔹 Instead of manually sending tokens to liquidity wallets later, the required tokens should be allocated during the minting process and sent immediately to cold wallets for:

- DEX Liquidity Provision (15%)
- CEX Liquidity Reserves (15%)
  During minting, allocate the full liquidity amount (30% total).
  Immediately distribute:
  15% to DEX liquidity wallet (to be used for Raydium, Orca, Serum).
  15% to CEX liquidity wallet (reserved for exchange listings).
  Cold wallets remain untouched until liquidity deployment begins.

## Liquidity Allocation Strategy

Instead of manually funding liquidity pools from the **Presale Smart Contract**, follow this structured plan:

| **Stage**                | **Liquidity Allocation Action**                                                          |
| ------------------------ | ---------------------------------------------------------------------------------------- |
| **During Minting**       | ✅ **Send 30% (or defined %) of tokens directly to Liquidity Wallets.**                  |
| **Before Public Sale**   | ✅ **Lock part of the liquidity in a smart contract or multi-sig wallet.**               |
| **At Listing Time**      | ✅ **Release funds to CEXs & DEX pools as needed (not manually from presale contract).** |
| **After Trading Starts** | ✅ **Gradually inject liquidity into pools to stabilize price action.**                  |

#### How Can We Send Tokens During Minting to DEX or CEX Wallets?

✅ Yes, DEX and CEX wallets function the same as any other Solana wallet.
✅ You can create and manage these wallets manually or through scripts.

📌 How to Create and Fund Liquidity Wallets During Minting
To ensure liquidity is available before trading starts, we need to:

Create dedicated liquidity wallets for DEX & CEX allocations.
Send tokens to these wallets during the minting process.

##### 🔹 Step 1: Create DEX & CEX Liquidity Wallets

These wallets are standard Solana wallets that hold liquidity tokens.

✅ Option 1: Create Liquidity Wallets Manually
Use Phantom, Solflare, or Solana CLI to generate two wallets:
One for DEX liquidity (Raydium, Orca, Serum).
One for CEX liquidity (for future exchange listings).
These wallets will be used to hold liquidity before listing on exchanges.
✅ Option 2: Automatically Create Wallets in Smart Contract
You can also generate and assign liquidity wallets in a smart contract:

✅ This ensures that liquidity wallets are generated automatically when minting begins.

##### 🔹 Step 2: Send Liquidity Tokens During Minting

Once liquidity wallets are created, they need to receive the allocated liquidity tokens.

✅ Minting & Funding Liquidity Wallets During Token Creation
When minting DYAWN tokens, you can immediately send a portion to DEX & CEX liquidity wallets.

## 5. Feedback on setBurnRate and Burn Mechanism

You are right!
The SPL Token Program includes a standard Burn function, which allows a wallet to destroy a specific amount of tokens from its own balance.
This is a manual, admin-triggered process—it does not happen automatically on transactions or under specific conditions.

### Why is setBurnRate Needed?

The standard SPL Burn only allows manual burning, whereas setBurnRate introduces automatic burning based on token activity.

### Differences Between SPL Standard Burn and Custom `setBurnRate` Mechanism

| **Feature**          | **SPL Standard Burn**                | **Custom `setBurnRate` Mechanism**               |
| -------------------- | ------------------------------------ | ------------------------------------------------ |
| **How Burn Works**   | Admin manually calls `Burn` function | Automatically burns a percentage of transactions |
| **When Tokens Burn** | Only when the admin executes it      | Happens during every transaction (buy/sell)      |
| **Use Case**         | One-time or periodic burns           | Continuous deflationary supply reduction         |
| **Flexibility**      | Requires admin action each time      | Adjustable burn rate via governance              |
| **Decentralization** | Centralized control                  | Can be managed by DAO/governance                 |

For now, We need Keeping a manual burn function for now and introducing setBurnRate later if they want.

## 6. Response to Feedback on setReferralBonus

### Pre-Sale Referral System Needs Flexibility

The referral bonus may need adjustments based on market conditions or early buyer demand.

Instead of hardcoding fixed values, setReferralBonus allows updates without redeploying the contract.

Example: If initially set to 5%, but interest is low, it can be increased to 10% to attract more buyers.

## 7. Feedback on allocateLiquidity & Liquidity Management Strategy

✅ Liquidity should be managed custodially for security & simplicity – Instead of automating it, liquidity will be deposited manually based on market conditions.

✅ Batch convert Web2 funds to Solana before sending to liquidity pools.

I will Remove `allocateLiquidity` from the Tokenomics contract.

## 8. About buyTokens and Internal Server Wallet

### Does buyTokens Send Tokens to an Internal Server Wallet?

✅ Yes, for Web2 purchases, tokens will be credited to an internal server wallet first.

Web2 users do not receive DYAWN immediately because they may not have a Solana wallet at the time of purchase.

The backend stores token balances in an internal database until users withdraw.

Once a Web2 user creates or connects a Solana wallet, the backend transfers tokens from the internal server wallet to their personal wallet.

✅ For Web3 purchases (Solana), tokens will be sent directly to the user's wallet.

## 9. Response to Feedback on Web2 to Web3 Conversion & Detection

- Web3 Transactions (SOL/USDC) Do Not Need Detection

When a user purchases DYAWN with SOL/USDC via buyToken, the smart contract instantly processes the transaction.
There is no need for backend detection because the transaction is executed on-chain in real time.

- Web2 Payments Require a More Detailed Conversion Process

### Web2 Payment Process

| **Step** | **Process**                                                          | **Handled By**       |
| -------- | -------------------------------------------------------------------- | -------------------- |
| **1**    | User pays in Web2 currency (USDT ERC20, TRC20, BTC, etc.)            | Web2 Payment Gateway |
| **2**    | Web2 payment is confirmed                                            | Web2 Payment Gateway |
| **3**    | Conversion process starts (manual or automated swap to SOL/USDC)     | Payment Provider     |
| **4**    | Converted funds (SOL/USDC) arrive in the treasury wallet             | Blockchain           |
| **5**    | Backend verifies receipt of SOL/USDC and triggers token distribution | Backend              |
| **6**    | DYAWN tokens are sent to the user’s Solana wallet                    | Smart Contract       |

---

✅ **Step 3 (conversion) must be acknowledged as a time-consuming process.**  
✅ **Step 4 (detecting SOL/USDC) happens after conversion is completed, not before.**  
✅ **Step 5 ensures DYAWN tokens are only distributed once the funds are received.**

## 10. Handling Web2 Users & Privy.io Wallets

Web2 users who log in via social networks and email will use **Privy.io**, which provides an **embedded Solana wallet**. However, since these wallets **start with zero balance**, they require special handling for gas fees and withdrawals.

---

### Issues & Solutions for Web2 Users

| **Issue**                                            | **Solution**                                                          | **Implementation**                                                                      |
| ---------------------------------------------------- | --------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| **Privy Wallets Start Empty (No SOL for Gas Fees)**  | ✅ **Sponsor gas fees using a backend-controlled fee payer wallet.**  | The backend automatically covers transaction fees when Web2 users interact with Solana. |
| **Users May Want to Withdraw to an External Wallet** | ✅ **Allow users to link and withdraw to a Phantom/Solflare wallet.** | Users can choose to withdraw DYAWN tokens to a manually entered Solana address.         |
| **Privy Wallets Do Not Sync with External Wallets**  | ✅ **Provide an option for manual transfers.**                        | Users can transfer tokens from their Privy wallet to another wallet within the app.     |

---

### How Web2 Users Can Withdraw DYAWN Tokens

- **User logs in with a Web2 account (Google, Twitter, Email) via Privy.io.**
- **User purchases DYAWN using a Web2 payment method (USDT ERC20, TRC20, BTC, etc.).**
- **DYAWN is credited to their Privy wallet (but they have no SOL for gas fees).**
- **When the user requests a withdrawal, the backend sponsors the transaction fee.**
- **The user chooses to withdraw to either:**

✅ **Their Privy wallet** (if they want to keep using it).

✅ **An external Phantom/Solflare wallet** (if they prefer full control).

- **Tokens are transferred, and the user can now manage them in their chosen wallet.**
