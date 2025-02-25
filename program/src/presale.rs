use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer as TokenTransfer};
use solana_program::program::invoke;
use solana_program::system_instruction;

pub mod constant;
use constant::*;

declare_id!("36bPWu6A8f6zxBwvBm2MbpHjrUViKAeQoreYovDUM3ot");

#[program]
mod dyawn_presale {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        total_supply: u64,
        referral_treasury_amount: u64,
        start_price: u64,
        public_sale_price: u64,
        presale_start: i64,
        private_sale_period: i64,
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        let (presale_authority, presale_bump) =
            Pubkey::find_program_address(&[b"dddd"], ctx.program_id);

        // Initialize presale state
        presale.token_mint = ctx.accounts.presale_token_mint.key();
        presale.treasury_account = ctx.accounts.referral_treasury_account.key();
        presale.merchant_account = ctx.accounts.merchant_account.key();
        presale.admin = presale_authority;
        presale.bump = presale_bump;
        presale.total_supply = total_supply;
        presale.referral_treasury_amount = referral_treasury_amount;
        presale.start_price = start_price;
        presale.current_price = start_price;
        presale.public_sale_price = public_sale_price;
        presale.presale_start = presale_start;
        presale.private_sale_period = private_sale_period * 86400;
        presale.price_increase_time = presale_start + private_sale_period * 86400;

        let current_time = Clock::get()?.unix_timestamp;
        presale.stage = if current_time < presale_start {
            SaleStage::PreLaunch
        } else {
            SaleStage::PrivateSale
        };

        // Call helper function to mint and transfer tokens
        mint_and_transfer_tokens(&ctx, total_supply, referral_treasury_amount)?;

        Ok(())
    }

    pub fn buy_tokens(
        ctx: Context<BuyTokens>,
        lamports_sent: u64,
        sol_price_in_usd: u64,
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Calculate the amount of tokens to send based on current price
        let amount_in_usd = (lamports_sent * sol_price_in_usd) / 1_000_000_000;
        let tokens_to_transfer = (amount_in_usd * 1_000_000) / presale.current_price; // USD to token conversion

        // Check if enough tokens are available
        let available_tokens = ctx.accounts.presale_token_account.amount;

        require!(
            available_tokens >= tokens_to_transfer * 1_000_000_000,
            PresaleError::InsufficientTokens
        );

        // Check if SOL sent is enough to cover token cost
        require!(
            lamports_sent
                >= ((tokens_to_transfer * presale.current_price) / (1_000_000 * sol_price_in_usd)),
            PresaleError::InsufficientFunds
        );

        // Transfer SOL to merchant account
        invoke(
            &system_instruction::transfer(
                &ctx.accounts.buyer.key(),
                &presale.merchant_account,
                lamports_sent,
            ),
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.merchant_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Transfer tokens from presale token account to buyer's token account using PDA as authority
        let seeds: &[&[u8]] = &[b"dddd", &[presale.bump]];
        let signer_seeds = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TokenTransfer {
                    from: ctx.accounts.presale_token_account.to_account_info(),
                    to: ctx.accounts.buyer_token_account.to_account_info(),
                    authority: ctx.accounts.presale.to_account_info(),
                },
                signer_seeds,
            ),
            tokens_to_transfer * 1_000_000_000,
        )?;

        msg!(
            "Bought {} tokens for {} lamports",
            tokens_to_transfer,
            lamports_sent
        );

        Ok(())
    }

    pub fn buy_tokens_by_stable_coin(
        ctx: Context<BuyTokensByStableCoin>,
        stable_coin_amount: u64,
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Check if the stable coin is USDT or USDC
        require!(
            ctx.accounts.stable_coin_mint.key() == USDC_ADDRESS
                || ctx.accounts.stable_coin_mint.key() == USDT_ADDRESS,
            PresaleError::InvalidStableToken
        );

        // Calculate token amount from stable_coin_amount and current price
        let mut tokens_to_transfer = stable_coin_amount * 1_000_000 / presale.current_price;

        let mut actual_stable_coin_amount = stable_coin_amount;

        // Check token availability
        let available_tokens = ctx.accounts.presale_token_account.amount;
        if available_tokens < tokens_to_transfer * 1_000_000_000 {
            tokens_to_transfer = available_tokens / 1_000_000_000;
            actual_stable_coin_amount = presale.current_price * tokens_to_transfer / 1_000_000;
        }

        require!(tokens_to_transfer > 0, PresaleError::InsufficientTokens);

        // Update presale state
        presale.total_supply -= tokens_to_transfer;

        // Transfer stable coin from buyer to merchant
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TokenTransfer {
                    from: ctx.accounts.buyer_stable_coin_account.to_account_info(),
                    to: ctx.accounts.merchant_stable_coin_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            actual_stable_coin_amount * 1_000_000,
        )?;

        // Transfer tokens from presale to buyer using PDA
        let seeds: &[&[u8]] = &[b"dddd", &[presale.bump]];
        let signer_seeds = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TokenTransfer {
                    from: ctx.accounts.presale_token_account.to_account_info(),
                    to: ctx.accounts.buyer_token_account.to_account_info(),
                    authority: ctx.accounts.presale.to_account_info(),
                },
                signer_seeds,
            ),
            tokens_to_transfer * 1_000_000_000,
        )?;

        msg!(
            "Bought {} tokens with {} stable coins",
            tokens_to_transfer,
            stable_coin_amount
        );

        Ok(())
    }

    /// Automatically updates the sale stage based on time
    pub fn update_stage(ctx: Context<UpdateStage>) -> Result<()> {
        let presale = &mut ctx.accounts.presale;
        let current_time = Clock::get()?.unix_timestamp;

        if current_time < presale.presale_start {
            presale.stage = SaleStage::PreLaunch; // ✅ Before presale starts
        } else if current_time < presale.price_increase_time {
            presale.stage = SaleStage::PrivateSale;
            presale.current_price = presale.start_price; // ✅ Ensure private sale price
        } else if current_time < presale.price_increase_time + (60 * 86400) {
            presale.stage = SaleStage::PublicSale;
            presale.current_price = presale.public_sale_price; // ✅ Update price to $0.007
        } else {
            presale.stage = SaleStage::Market;
        }

        msg!(
            "Sale stage updated to: {:?}, Current Price: {}",
            presale.stage,
            presale.current_price
        );

        Ok(())
    }

    pub fn set_stage(ctx: Context<SetStage>, new_stage: SaleStage) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        let seeds: &[&[u8]] = &[b"dddd", &[presale.bump]];

        // Ensure the PDA (admin) is the authority
        let admin_pubkey = Pubkey::create_program_address(seeds, ctx.program_id)
            .map_err(|_| PresaleError::Unauthorized)?;

        require_keys_eq!(admin_pubkey, presale.admin, PresaleError::Unauthorized);

        presale.stage = new_stage.clone();
        msg!("Sale stage manually set to: {:?}", new_stage);

        Ok(())
    }

    /// Allows the admin to manually update the token price
    pub fn set_price(ctx: Context<SetPrice>, new_price: u64) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        let seeds: &[&[u8]] = &[b"dddd", &[presale.bump]];

        // Ensure the PDA (admin) is the authority
        let admin_pubkey = Pubkey::create_program_address(seeds, ctx.program_id)
            .map_err(|_| PresaleError::Unauthorized)?;

        require_keys_eq!(admin_pubkey, presale.admin, PresaleError::Unauthorized);

        presale.current_price = new_price.clone();
        msg!("Admin manually updated the token price to: {}", new_price);

        Ok(())
    }

    pub fn withdraw_rewards(
        ctx: Context<WithdrawRewards>,
        reward_amount: u64, // Verified reward amount from IPFS
        referrer: Pubkey,   // Referrer wallet address
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Verify that the connected wallet matches the referrer address
        require_keys_eq!(
            ctx.accounts.user.key(),
            referrer,
            PresaleError::UnauthorizedReferrer
        );

        // Check if the referral treasury has enough tokens
        let treasury_balance = ctx.accounts.referral_treasury_account.amount;
        require!(
            treasury_balance >= reward_amount * 1_000_000_000,
            PresaleError::InsufficientTokens
        );

        // Transfer tokens from Referral Treasury to referrer
        let seeds: &[&[u8]] = &[b"dddd", &[presale.bump]];
        let signer_seeds = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TokenTransfer {
                    from: ctx.accounts.referral_treasury_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.presale.to_account_info(),
                },
                signer_seeds,
            ),
            reward_amount * 1_000_000_000, // Token amount with decimals
        )?;

        msg!(
            "Referral rewards of {} tokens withdrawn by {}",
            reward_amount,
            referrer
        );

        Ok(())
    }

    pub fn refill_treasury(
        ctx: Context<RefillTreasury>,
        amount: u64, // Amount of tokens to add to the Referral Treasury
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Check token mint balance
        let mint_balance = ctx.accounts.presale_token_mint.supply;
        require!(
            mint_balance >= amount * 1_000_000_000,
            PresaleError::InsufficientMintBalance
        );

        // Ensure only the admin can refill
        let seeds: &[&[u8]] = &[b"dddd", &[presale.bump]];
        let signer_seeds = &[&seeds[..]];

        // Ensure the PDA (admin) is the authority
        let admin_pubkey = Pubkey::create_program_address(seeds, ctx.program_id)
            .map_err(|_| PresaleError::Unauthorized)?;

        require_keys_eq!(admin_pubkey, presale.admin, PresaleError::Unauthorized);

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.presale_token_mint.to_account_info(),
                    to: ctx.accounts.referral_treasury_account.to_account_info(),
                    authority: ctx.accounts.presale.to_account_info(),
                },
                signer_seeds,
            ),
            amount * 1_000_000_000, // Token amount with decimals
        )?;

        msg!("Referral Treasury refilled with {} tokens by admin", amount);

        Ok(())
    }

}

// Helper function to mint tokens and transfer to referral treasury
pub fn mint_and_transfer_tokens<'info>(
    ctx: &Context<Initialize>,
    total_supply: u64,
    referral_treasury_amount: u64,
) -> Result<()> {
    let cpi_program = ctx.accounts.token_program.to_account_info();

    // Update the seed to only use b"presale" and the bump
    let seeds: &[&[u8]] = &[b"dddd", &[ctx.accounts.presale.bump]];
    let signer_seeds = &[&seeds[..]];

    // Mint tokens to presale account using PDA as the signer
    token::mint_to(
        CpiContext::new_with_signer(
            cpi_program.clone(),
            MintTo {
                mint: ctx.accounts.presale_token_mint.to_account_info(),
                to: ctx.accounts.presale_token_account.to_account_info(),
                authority: ctx.accounts.presale.to_account_info(),
            },
            signer_seeds,
        ),
        total_supply * 1_000_000_000,
    )?;

    // Transfer tokens to referral treasury using PDA as the signer
    token::transfer(
        CpiContext::new_with_signer(
            cpi_program,
            TokenTransfer {
                from: ctx.accounts.presale_token_account.to_account_info(),
                to: ctx.accounts.referral_treasury_account.to_account_info(),
                authority: ctx.accounts.presale.to_account_info(),
            },
            signer_seeds,
        ),
        referral_treasury_amount * 1_000_000_000,
    )?;

    msg!(
        "Total Supply Minted: {} tokens, Referral Treasury Funded: {} tokens",
        total_supply,
        referral_treasury_amount
    );

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + 256, seeds = [b"dddd"], bump)]
    pub presale: Account<'info, Presale>,

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub merchant_account: AccountInfo<'info>,

    #[account(init, mint::decimals = 9, mint::authority = presale, payer = admin)]
    pub presale_token_mint: Account<'info, Mint>,

    #[account(init, payer = admin, token::mint = presale_token_mint, token::authority = presale)]
    pub presale_token_account: Account<'info, TokenAccount>,

    #[account(init, payer = admin, token::mint = presale_token_mint, token::authority = presale)]
    // pub referral_treasury_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut, seeds = [b"dddd"], bump = presale.bump)]
    pub presale: Account<'info, Presale>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut, token::mint = presale.token_mint, token::authority = presale)]
    pub presale_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub merchant_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BuyTokensByStableCoin<'info> {
    #[account(mut, seeds = [b"dddd"], bump = presale.bump)]
    pub presale: Account<'info, Presale>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub merchant: AccountInfo<'info>,

    #[account(mut)]
    pub stable_coin_mint: Account<'info, Mint>,

    #[account(mut, token::mint = presale.token_mint, token::authority = presale)]
    pub presale_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,

    #[account(
    mut,
    associated_token::mint = stable_coin_mint,
    associated_token::authority = buyer
)]
    pub buyer_stable_coin_account: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = stable_coin_mint, associated_token::authority = merchant)]
    pub merchant_stable_coin_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateStage<'info> {
    #[account(mut, seeds = [b"dddd"], bump = presale.bump)]
    pub presale: Account<'info, Presale>,
}

#[derive(Accounts)]
pub struct SetStage<'info> {
    #[account(mut, seeds = [b"dddd"], bump = presale.bump)]
    pub presale: Account<'info, Presale>,
}

#[derive(Accounts)]
pub struct SetPrice<'info> {
    #[account(mut, seeds = [b"dddd"], bump = presale.bump)]
    pub presale: Account<'info, Presale>,
}

#[derive(Accounts)]
pub struct WithdrawRewards<'info> {
    #[account(mut, seeds = [b"dddd"], bump = presale.bump)]
    pub presale: Account<'info, Presale>,

    #[account(mut)]
    pub user: Signer<'info>, // Referrer’s wallet

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>, // Referrer’s token account

    #[account(mut, token::mint = presale.token_mint, token::authority = presale)]
    pub referral_treasury_account: Account<'info, TokenAccount>, // Referral treasury

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RefillTreasury<'info> {
    #[account(mut, seeds = [b"dddd"], bump = presale.bump)]
    pub presale: Account<'info, Presale>,

    #[account(mut)]
    pub presale_token_mint: Account<'info, Mint>,

    #[account(mut, token::mint = presale.token_mint, token::authority = presale)]
    pub referral_treasury_account: Account<'info, TokenAccount>, // Referral treasury account

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Presale {
    pub admin: Pubkey,
    pub bump: u8,
    pub token_mint: Pubkey,
    pub treasury_account: Pubkey,
    pub merchant_account: Pubkey,
    pub start_price: u64,
    pub current_price: u64,
    pub public_sale_price: u64,
    pub price_increase_time: i64,
    pub presale_start: i64,
    pub private_sale_period: i64,
    pub total_supply: u64,
    pub referral_treasury_amount: u64,
    pub stage: SaleStage,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum SaleStage {
    PreLaunch,
    PrivateSale,
    PublicSale,
    Market,
}

// Custom error
#[error_code]
pub enum PresaleError {
    #[msg("Not enough tokens available for the requested purchase.")]
    InsufficientTokens,
    #[msg("Insufficient SOL sent for the requested token purchase.")]
    InsufficientFunds,
    #[msg("Please buy our tokens using USDT or USDC.")]
    InvalidStableToken,
    #[msg("You are not admin for this contract.")]
    Unauthorized,
    #[msg("You are not actual refferrer for this reward.")]
    UnauthorizedReferrer,
    #[msg("Not enough tokens available for the reward refilling.")]
    InsufficientMintBalance,
    #[msg("Minting this amount would exceed the supply cap.")]
    SupplyCapExceeded,
}
