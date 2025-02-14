# solana-spl-meme-coin-presale

This is anchor (Rust) project for presale contract on solana

# SPL MEME Token Presale Contract

## 1. Mint Token by owner

### 1.1 Mint token by running this command.

<code>spl-token create-token</code>

### 1.2 Create ATA from mint address

<code>spl-token create-account mint_token_pub</code>

#### This command checks for an SPL Token Account.

<code>spl-token account-info </code>Your-Associated-Token-Account.

<code>spl-token mint mint_token_pub 5000</code>

The Debug trait allows Rust to format the enum for printing.
The msg!() macro can now print SaleStage without errors.

## Running a Cron Job to Update update_stage

If you want this to run automatically at set time intervals, you can use cron job or backend service.
