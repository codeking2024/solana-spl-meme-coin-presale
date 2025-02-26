# Dyawn Token Ecosystem

## Author: John Lee at Digital Heores

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

### Why Manual Token Distribution is Preferred Over Automatic Distribution

In the past, automatic token distribution seemed like a convenient approach for managers to handle allocations efficiently. However, after extensive research and industry experience, it is evident that most projects prefer manual distribution due to its security, flexibility, and control benefits.

#### ❌ Risks of Automatic Token Distribution

##### 1️⃣ Unauthorized or Accidental Transfers

An attacker could manipulate automatic distribution to steal funds.
Manual distribution ensures tokens are only sent to verified wallets.

##### 2️⃣ Incorrect Wallet Addresses for Off-Chain Investors

Many early investors, partners, or advisors may not have provided their correct Solana wallets.
If tokens are automatically sent to an invalid address, they may be permanently lost.
Manual distribution ensures that all wallets are verified before sending tokens.

##### 3️⃣ No Adjustments for Unsold Tokens

Automatic distribution cannot adapt to changes if some presale tokens remain unsold.
Unsold tokens should be reallocated for staking rewards, liquidity pools, or burned.
Manual control ensures that excess tokens are handled properly.

#### ✅ Benefits of Manual Token Distribution

##### 1️⃣ Flexibility for Tokenomics Adjustments

Token allocations may change due to discussions with investors, partners, or the community.
If tokens were automatically distributed, allocations would be locked, preventing any adjustments.
Manual distribution allows real-time modifications before transferring tokens.

##### 2️⃣ Control Over Timing of Distributions

Instead of distributing all tokens at once, manual distribution allows for phased releases.
This helps prevent market dumping and ensures a more stable price action.

##### 3️⃣ Gradual Liquidity Addition

Instead of immediately adding all tokens to DEX liquidity pools, manual distribution allows for:
✅ Strategic liquidity injections to prevent excessive volatility.
✅ Gradual releases to match market demand and maintain price stability.

## 3. Presale contract

This contract facilitates a **token presale** with a **referral system**, allowing users to purchase tokens with **SOL or USDC** during **private and public sale stages**. The contract ensures fair allocation and **secure fund management**, including support for **unsold token transfers** and **referral rewards**.

### 📌 Features

✅ **Two-Stage Presale**: Private sale (15 days) followed by a public sale (60 days).  
✅ **Timed Sales**: Automatically transitions from private to public sale based on time.  
✅ **Supports SOL & USDC Payments**: Buyers can purchase tokens with SOL or USDC.  
✅ **Referral System**: Rewards referrers with tokens (5% for regular users, 10% for influencers).  
✅ **Merchant Wallet**: Funds are collected in a **merchant-provided wallet**.  
✅ **Virtual Wallet Storage**: Purchased tokens are stored in a virtual wallet until withdrawal.  
✅ **Liquidity Management**: Unsold tokens are sent to a liquidity wallet.

### 🛠️ Contract Initialization

The **admin** must initialize the contract before the presale begins.

#### 📍 **Initialization Parameters**

| Parameter                  | Type  | Description                                                                        |
| -------------------------- | ----- | ---------------------------------------------------------------------------------- |
| `private_price`            | `u64` | Price per token in **Private Sale** (USD).                                         |
| `public_price`             | `u64` | Price per token in **Public Sale**.                                                |
| `private_sale_duration`    | `i64` | **Duration of the Private Sale** in seconds (**Default: 15 days**).                |
| `public_sale_duration`     | `i64` | **Duration of the Public Sale** in seconds (**Default: 60 days**, can be updated). |
| `regular_referral_rate`    | `u8`  | **5% reward** for regular referrers                                                |
| `influencer_referral_rate` | `u8`  | **10% reward** for influencers.                                                    |

#### 📥 Required Accounts

The `initialize` function requires the following accounts:

| **Name**                   | **Type**                   | **Mutable?** | **Signer?** | **Description**                                        |
| -------------------------- | -------------------------- | ------------ | ----------- | ------------------------------------------------------ |
| `admin`                    | `Signer`                   | ✅ Yes       | ✅ Yes      | The **admin wallet** that initializes the presale.     |
| `presale`                  | `Account<Presale>`         | ✅ Yes       | ❌ No       | Stores presale details and controls presale state.     |
| `token_mint`               | `Account<Mint>`            | ❌ No        | ❌ No       | The **SPL Token Mint** (e.g., DYAWN token).            |
| `presale_wallet`           | `Account<TokenAccount>`    | ✅ Yes       | ❌ No       | Token account to **store presale tokens**.             |
| `referral_wallet`          | `Account<TokenAccount>`    | ✅ Yes       | ❌ No       | Token account to **store referral rewards**.           |
| `merchant_wallet`          | `SystemAccount`            | ✅ Yes       | ❌ No       | User-provided **merchant wallet** for fund collection. |
| `liquidity_wallet`         | `Account<TokenAccount>`    | ✅ Yes       | ❌ No       | User-provided **token account** for unsold tokens.     |
| `system_program`           | `Program<System>`          | ❌ No        | ❌ No       | Required system program for Solana transactions.       |
| `token_program`            | `Program<Token>`           | ❌ No        | ❌ No       | Solana Token Program to handle token transfers.        |
| `associated_token_program` | `Program<AssociatedToken>` | ❌ No        | ❌ No       | Required to create associated token accounts (ATA).    |

```json
{
  "admin": "BQUHqj6LgS3846f4mTguhN6SRrTLucy1ggGGcefZr9ww",
  "presaleStart": "1740465949",
  "privatePrice": "3500",
  "publicPrice": "7000",
  "currentPrice": "3500",
  "privateSaleDuration": "15",
  "publicSaleDuration": "60",
  "saleStage": 0,
  "totalSold": "0",
  "poolCreated": false,
  "presaleWallet": "4nuqy6We6hCwGkUGcK8tVTKCejkqQP98L6fb8KM3BvTi",
  "referralWallet": "452TFDMzUfTPPnBD7X34LoADYX2fgC1tBgpYTDW79m4x",
  "liquidityWallet": "8LW7ngtPsm85WfiqxYwoqrPxCmaqvDmf4MXVfhWKiW8t",
  "merchantWallet": "Eb1dAwq9f1tLVjVY2TUrAPLous5J4UuftN5ymxE1hTnN",
  "regularReferralRate": 5,
  "influencerReferralRate": 10
}
```

### PDA Security

```rust
require_keys_eq!(ctx.accounts.admin.key(), presale.admin, PresaleError::Unauthorized);
// Compute the correct PDA based on the admin's key
let expected_pda = Pubkey::find_program_address(&[PRESALE_SEED, ctx.accounts.admin.key().as_ref()], &crate::ID).0;

// Ensure the provided presale account matches the expected PDA
require_keys_eq!(ctx.accounts.presale.key(), expected_pda, PresaleError::InvalidPDA);
```

### **Admin manually sends tokens** to:

- `presale_wallet` (**for presale purchases**).
- `referral_wallet` (**for referral rewards**).

### Admin calls `set_stage()`** to start the **private sale\*\*.

#### - Sale Stage Transition Logic

0 - Not Started: Initial state until the admin starts the presale.

1 - Private Sale: Starts when the admin calls start_presale().

2 - Public Sale: Automatically starts after 15 days.

3 - Ended: Automatically ends after 60 days of public sale.

#### - Update sale stage and period

✅ Admin-Only update_sale_period() – Allowing flexible duration updates.

`update_sale_period()`

✅ Secure Sale Stage Transitions – Moving from Not Started → Private → Public → Ended.

`set_stage()`

### Buy Tokens with Sol / USDC

Since purchased tokens cannot be sent directly to the buyer’s wallet and must instead be stored in a virtual wallet (backend-managed), the contract must:

Track purchased tokens per user (without sending them to their real wallet).

Allow users to withdraw their tokens after pool created.

-- Tracking token balance

✅ Allows the admin to track available tokens in real-time.

✅ Users can verify if tokens are available before purchasing or withdrawing rewards.

`check_presale_token_balance()` - Check Available Presale Tokens

`check_reward_token_balance()` - Check Available Referral

-- Update sale price at any time

`update_sale_price()` - Admin Updates Current Sale Price

If in Private Sale (sale_stage == 1), it updates the private sale price.

If in Public Sale (sale_stage == 2), it updates the public sale price.

Fails if called outside an active sale stage.

--Buy tokens by sol and usdc

Since Solana accounts have a max size limit of ~10KB, we can store approximately:

10 buyers → 10 \* 40 = 400 bytes

100 buyers → 100 \* 40 = 4,000 bytes (~4KB)

200 buyers → 200 \* 40 = 8,000 bytes (~8KB)

🔥 Solution if You Expect 1000+ Buyers

If you expect thousands of buyers, storing balances on-chain is inefficient.

Instead, store balances off-chain (e.g., Firebase, PostgreSQL, IPFS) and:

Only store the total sold on-chain.

Let buyers query the backend for their balance.

### web2 payment method

Users send USDT (TRC20, ERC20, BEP20, etc.) to an off-chain payment processor (like Shkeeper.io).

The system automatically converts those tokens into SOL or USDC on Solana.

We do not do automatic conversion and transfer from web2 to SOL/USDC. We do not wait for this to complete.

As soon as payment processor confirms receipt of funds buyTokens() is calling.

Generate a Solana Wallet for Each User in the Backend

Shkeeper.io triggers the Presale Contract’s `buyTokens()` / `buy_tokens_by_usdc` function automatically with generated wallet address.

`buy_tokens(payment_type, lamports_sent, sol_price_in_usd )` - Sol payment

`buy_tokens_by_usdc(payment_type)` - USDC Payment

`payment_type`: web3 / web2

if the type is just web3:

- Users send SOL/USDC directly to the merchant address.

- SOL transfer is required before storing the tokens in the virtual wallet.

web2 / web3:

- Record purchased tokens in a virtual wallet (pubkey, username and balance).

they can just have pubkey and balance , username(if they set in the dashboard)

- Auto-transitions between sale stages.

- Rejects purchases after the presale ends.

### Withdraw Tokens from virtual wallet after liquidity pool created!

Since users cannot withdraw their tokens until the liquidity pool is created, we need to:

- Ensure withdrawals are only allowed after the pool is created.
- Store a flag (pool_created) in the presale contract.
- Let users withdraw their tokens only after this flag is set.
- check recipient address is the same with registered wallet address before withdrwal

`set_pool_created()` - Admin Confirms Liquidity Pool Creation

`withdraw_tokens()` - Users Withdraw After Pool Creation

### 🔹 Implementing the Referral System

### 💰 **Referral System**

- **5% reward** for regular referrers.
- **10% reward** for influencers.
- Referral rewards are stored in the **referral wallet**.

The referral system needs to:

- Track purchases made through referral links.
- Store referral rewards in a separate on-chain balance. (referrer, amount)
- Distinguish between regular users (5%) and influencers (10%).
- Allow users to withdraw their referral rewards after pool created.

`buy_tokens_with_referral()`

-- Referral Reward Calculation

- Accept is_influencer: bool as an argument from the backend.
- Apply a 10% referral reward if is_influencer == true.
- Apply a 5% referral reward if is_influencer == false.

- Referral rewards are stored in a virtual referral wallet.

`withdraw_referral_rewards(regular_rate, influencer_rate)`

`set_referral_rate()`

The admin should be able to change the referral rates at any time.

-- Allow Referrers to Claim Tokens

- Only allowed after the liquidity pool is created.
- Transfers referral rewards from the referral wallet to the referrer’s real wallet.

### About Web2 payment system with privy wallet

1. Backend requests Shkeeper to create an address (Deposit address of holding USDT token).

2. User manually transfers USDT to the Shkeeper-provided address.

3. shkeeper swaps USDT to Solana-compatible assets.

4. Backend calls buy_tokens(web2) to assign tokens to the user’s Privy wallet.

5. User connects wallet & withdraws tokens from contract.

### **After the presale**, unsold tokens are sent to the **liquidity wallet**.

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
