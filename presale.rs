use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use solana_program::program::invoke;
use solana_program::system_instruction;

pub mod constant;
use constant::*;

declare_id!("9dKRRg5H1q9ja6GDkxjCUvf9FSAP9xhDjX4uM3jodWS"); // Replace with actual program ID

#[program]
pub mod presale {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        private_price: u64,
        public_price: u64,
        private_sale_duration: i64,
        public_sale_duration: i64,
        regular_referral_rate: u8,
        influencer_referral_rate: u8,
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Assign admin and presale parameters
        presale.admin = ctx.accounts.admin.key();
        presale.private_price = private_price;
        presale.public_price = public_price;
        presale.current_price = private_price;
        presale.presale_start = Clock::get()?.unix_timestamp;
        presale.private_sale_duration = private_sale_duration * 86400;
        presale.public_sale_duration = public_sale_duration * 86400;
        presale.sale_stage = 0; // 0 = Not Started
        presale.total_sold = 0;
        presale.pool_created = false; // Liquidity pool flag

        // Validate and store referral rates
        require!(regular_referral_rate <= 100, PresaleError::InvalidRate);
        require!(influencer_referral_rate <= 100, PresaleError::InvalidRate);
        presale.regular_referral_rate = regular_referral_rate;
        presale.influencer_referral_rate = influencer_referral_rate;

        // Assign storage wallets in Presale state
        presale.presale_wallet = ctx.accounts.presale_wallet.key();
        presale.referral_wallet = ctx.accounts.referral_wallet.key();
        presale.liquidity_wallet = ctx.accounts.liquidity_wallet.key();
        presale.merchant_wallet = ctx.accounts.merchant_wallet.key();

        msg!(
            "Presale contract initialized! Referral: {}%, Influencer: {}%",
            regular_referral_rate,
            influencer_referral_rate
        );

        Ok(())
    }

    pub fn set_stage(ctx: Context<SetStage>) -> Result<()> {
        let presale = &mut ctx.accounts.presale;
        let clock = Clock::get()?; // Get current Solana timestamp

        match presale.sale_stage {
            0 => {
                // Not Started → Start Private Sale
                presale.presale_start = clock.unix_timestamp;
                presale.current_price = presale.private_price;
                presale.sale_stage = 1;
                msg!("Private sale started at {}", presale.presale_start);
            }
            1 => {
                // Private Sale → Public Sale (after 15 days)
                require!(
                    clock.unix_timestamp >= presale.presale_start + presale.private_sale_duration,
                    PresaleError::PrivateSaleNotOver
                );
                presale.current_price = presale.public_price;
                presale.sale_stage = 2;
                msg!("Public sale started at {}", clock.unix_timestamp);
            }
            2 => {
                // Public Sale → Sale Ended (after 60 days)
                require!(
                    clock.unix_timestamp
                        >= presale.presale_start
                            + presale.private_sale_duration
                            + presale.public_sale_duration,
                    PresaleError::PublicSaleNotOver
                );
                presale.sale_stage = 3;
                msg!("Presale ended at {}", clock.unix_timestamp);
            }
            _ => {
                return Err(PresaleError::SaleAlreadyEnded.into());
            }
        }

        Ok(())
    }

    pub fn update_sale_period(
        ctx: Context<UpdateSalePeriod>,
        new_private_sale_duration: i64,
        new_public_sale_duration: i64,
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Ensure the presale has not already ended
        require!(presale.sale_stage < 3, PresaleError::SaleAlreadyEnded);

        // Convert days to seconds
        presale.private_sale_duration = new_private_sale_duration * 86400;
        presale.public_sale_duration = new_public_sale_duration * 86400;

        msg!(
            "Updated sale period: Private Sale = {} days, Public Sale = {} days",
            new_private_sale_duration,
            new_public_sale_duration
        );

        Ok(())
    }

    pub fn buy_tokens(
        ctx: Context<BuyTokens>,
        payment_type: u8, // 0 = Web3, 1 = Web2
        lamports_sent: u64,
        sol_price_in_usd: u64,
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;
        let buyer = &ctx.accounts.buyer;

        // Ensure presale is active (Private Sale or Public Sale)
        require!(
            presale.sale_stage == 1 || presale.sale_stage == 2,
            PresaleError::PresaleNotActive
        );

        let amount_in_usd = (lamports_sent * sol_price_in_usd) / 1_000_000_000;

        // ✅ Ensure SOL price is at least $1
        require!(amount_in_usd >= 1, PresaleError::InvalidPrice);

        let tokens_to_purchase = (amount_in_usd * 1_000_000) / presale.current_price;

        // Ensure enough tokens exist
        let available_tokens = ctx.accounts.presale_wallet.amount;
        let remaining_tokens = available_tokens - presale.total_sold * 1_000_000_000; // ✅ Adjust for sold tokens

        require!(
            remaining_tokens >= tokens_to_purchase * 1_000_000_000, // ✅ Ensure enough tokens remain
            PresaleError::InsufficientTokens
        );

        // If Web3 payment, ensure enough SOL is sent
        if payment_type == 0 {
            require!(
                lamports_sent
                    >= ((tokens_to_purchase * presale.current_price)
                        / (1_000_000 * sol_price_in_usd)),
                PresaleError::InsufficientFunds
            );

            // ✅ Transfer SOL to the stored merchant wallet in `Presale`
            invoke(
                &system_instruction::transfer(
                    &buyer.key(),
                    &presale.merchant_wallet, // ✅ Using stored merchant wallet
                    lamports_sent,
                ),
                &[
                    ctx.accounts.buyer.to_account_info(),           // ✅ Buyer
                    ctx.accounts.merchant_wallet.to_account_info(), // ✅ Merchant wallet
                    ctx.accounts.system_program.to_account_info(),  // ✅ System program (Required)
                ],
            )?;
        }

        // ✅ Update `total_sold`
        presale.total_sold += tokens_to_purchase;

        // ✅ Emit the event
        emit!(BuyTokensEvent {
            buyer: buyer.key(),
            tokens_purchased: tokens_to_purchase,
            sol_spent: lamports_sent,
            sol_price_in_usd,
            payment_type,
        });

        msg!(
            "Buyer {} purchased {} tokens for {} lamports using payment_type: {}",
            buyer.key(),
            tokens_to_purchase,
            lamports_sent,
            payment_type
        );

        Ok(())
    }

    pub fn check_presale_token_balance(ctx: Context<CheckPresaleTokenBalance>) -> Result<u64> {
        let presale = &ctx.accounts.presale;
        let available_tokens = ctx.accounts.presale_wallet.amount;

        // Calculate remaining tokens after sold tokens
        let remaining_tokens = available_tokens - (presale.total_sold * 1_000_000_000); // Adjust for decimals

        msg!("Available presale tokens: {}", remaining_tokens);

        Ok(remaining_tokens)
    }

    pub fn update_sale_price(ctx: Context<UpdateSalePrice>, new_price: u64) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // ✅ Ensure the caller is the admin
        require!(
            presale.admin == ctx.accounts.admin.key(),
            PresaleError::Unauthorized
        );

        // ✅ Ensure the presale is active
        require!(
            presale.sale_stage == 1 || presale.sale_stage == 2,
            PresaleError::PresaleNotActive
        );

        // ✅ Update price based on the current sale stage
        match presale.sale_stage {
            1 => {
                presale.private_price = new_price;
                presale.current_price = new_price; // Update active price if in Private Sale
            }
            2 => {
                presale.public_price = new_price;
                presale.current_price = new_price; // Update active price if in Public Sale
            }
            _ => return Err(PresaleError::PresaleNotActive.into()),
        }

        // ✅ Emit an event to track price changes
        emit!(UpdateSalePriceEvent {
            admin: ctx.accounts.admin.key(),
            new_price,
            sale_stage: presale.sale_stage,
        });

        msg!(
            "Sale price updated to {} for stage {}",
            new_price,
            presale.sale_stage
        );

        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(private_price: u64, public_price: u64, private_sale_duration: i64, public_sale_duration: i64, regular_referral_rate: u8, influencer_referral_rate: u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>, // Admin who deploys the contract

    #[account(
        init,
        payer = admin,
        seeds = [PRESALE_SEED, admin.key().as_ref()],  // Derive Presale PDA
        bump,
        space = 8 +  // Discriminator
               32 +  // Admin pubkey
               8 +   //  Presale start
               8 +   // Private price
               8 +  // Public price
               8 +  // current price
               8 +  // Private sale duration
               8 +  // Public sale duration
               8 +  // Sale stage
               8 +  // Total sold
               1 +  // Pool created flag
               32 + // Presale wallet
               32 + // Referral wallet
               32 + // Liquidity wallet
               32 + // Merchant wallet
               1 +  // Regular referral rate
               1    // Influencer referral rate
    )]
    pub presale: Account<'info, Presale>, // Stores presale details

    pub token_mint: Account<'info, Mint>, // DYAWN token mint

    #[account(init, payer = admin, token::mint = token_mint, token::authority = presale)]
    pub presale_wallet: Account<'info, TokenAccount>,

    #[account(init, payer = admin, token::mint = token_mint, token::authority = presale)]
    pub referral_wallet: Account<'info, TokenAccount>, // Storage for referral rewards

    #[account(mut)]
    pub merchant_wallet: AccountInfo<'info>,

    #[account(mut)]
    pub liquidity_wallet: Account<'info, TokenAccount>, // User-provided token account

    pub system_program: Program<'info, System>, // Required system program
    pub token_program: Program<'info, Token>,   // Required token program
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct SetStage<'info> {
    #[account(mut)]
    pub admin: Signer<'info>, // Only admin can change sale stage

    #[account(
        mut,
        has_one = admin, // Ensures the stored presale.admin matches the Signer
        seeds = [PRESALE_SEED, admin.key().as_ref()],
        bump
    )]
    pub presale: Account<'info, Presale>,
}

#[derive(Accounts)]
pub struct UpdateSalePeriod<'info> {
    #[account(mut)]
    pub admin: Signer<'info>, // Only the admin can update the sale period

    #[account(
        mut,
        has_one = admin, // Ensures only the admin can update the sale period
        seeds = [PRESALE_SEED, admin.key().as_ref()],
        bump
    )]
    pub presale: Account<'info, Presale>,
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>, // The user buying tokens

    #[account(
        mut,
        seeds = [PRESALE_SEED, presale.admin.as_ref()], 
        bump,
    )]
    pub presale: Account<'info, Presale>, // Presale storage PDA

    #[account(mut)]
    pub presale_wallet: Account<'info, TokenAccount>, // Store presale tokens

    #[account(mut, address = presale.merchant_wallet)] // ✅ Ensures correct merchant wallet
    pub merchant_wallet: AccountInfo<'info>,

    pub system_program: Program<'info, System>, // Required for SOL transfer
}

#[derive(Accounts)]
pub struct CheckPresaleTokenBalance<'info> {
    #[account(
        mut,
        seeds = [PRESALE_SEED, presale.admin.as_ref()],
        bump,
    )]
    pub presale: Account<'info, Presale>, // Presale storage PDA

    #[account(mut)]
    pub presale_wallet: Account<'info, TokenAccount>, // Store presale tokens
}

#[derive(Accounts)]
pub struct UpdateSalePrice<'info> {
    #[account(mut)]
    pub admin: Signer<'info>, // Only the admin can update the price

    #[account(
        mut,
        has_one = admin, // Ensures only the admin can update
        seeds = [PRESALE_SEED, admin.key().as_ref()],
        bump
    )]
    pub presale: Account<'info, Presale>,
}

#[account]
pub struct Presale {
    pub admin: Pubkey,              // Admin wallet address
    pub presale_start: i64,         // Presale start timestamp (Unix time)
    pub private_price: u64,         // Token price in private sale
    pub public_price: u64,          // Token price in public sale
    pub current_price: u64,         // Current token price (updates dynamically)
    pub private_sale_duration: i64, // Private sale duration
    pub public_sale_duration: i64,  // Public sale duration
    pub sale_stage: u8,             // Sale stage (0: Not started, 1: Private, 2: Public, 3: Ended)
    pub total_sold: u64,            // Total tokens sold
    pub pool_created: bool,         // Liquidity pool created flag
    pub presale_wallet: Pubkey,     // Token account for presale
    pub referral_wallet: Pubkey,    // Token account for referral rewards
    pub liquidity_wallet: Pubkey,   // Liquidity wallet for unsold tokens
    pub merchant_wallet: Pubkey,
    pub regular_referral_rate: u8, // Referral reward % for regular users
    pub influencer_referral_rate: u8, // Referral reward % for influencers
}

#[event]
pub struct BuyTokensEvent {
    pub buyer: Pubkey,
    pub tokens_purchased: u64,
    pub sol_spent: u64,
    pub sol_price_in_usd: u64,
    pub payment_type: u8,
}

#[event]
pub struct UpdateSalePriceEvent {
    pub admin: Pubkey,
    pub new_price: u64,
    pub sale_stage: u8,
}

#[error_code]
pub enum PresaleError {
    #[msg("Invalid rate: Percentage must be between 0 and 100.")]
    InvalidRate,

    #[msg("Invalid token account provided.")]
    InvalidTokenAccount,

    #[msg("Private sale period is not over yet.")]
    PrivateSaleNotOver,

    #[msg("Public sale period is not over yet.")]
    PublicSaleNotOver,

    #[msg("The presale has already ended.")]
    SaleAlreadyEnded,

    #[msg("Presale is not active.")]
    PresaleNotActive,

    #[msg("Not enough tokens available for purchase.")]
    InsufficientTokens,

    #[msg("Insufficient SOL sent for purchase.")]
    InsufficientFunds,

    #[msg("Invalid payment type. Please choose 1 or 2")]
    InvalidPaymentType,

    #[msg("Invalid price: SOL Amount in Usd must be over than $1.")]
    InvalidPrice, // ✅ New error for minimum SOL price check

    #[msg("Unauthorized: Only the presale admin can perform this action.")]
    // ✅ New admin-only error
    Unauthorized,
}
