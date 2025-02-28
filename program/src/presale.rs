use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer as TokenTransfer},
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

        // Use the correct way to retrieve bump
        let bump = ctx.bumps.presale;

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
        presale.merchant_wallet = ctx.accounts.merchant_wallet.key();

        // Store the bump seed
        presale.bump = bump;

        msg!(
            "Presale contract initialized! Referral: {}%, Influencer: {}%",
            regular_referral_rate,
            influencer_referral_rate
        );

        Ok(())
    }

    pub fn set_stage(ctx: Context<SetStage>) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // ✅ Ensure the caller is the admin
        require!(
            presale.admin == ctx.accounts.admin.key(),
            PresaleError::Unauthorized
        );

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
                // require!(
                //     clock.unix_timestamp >= presale.presale_start + presale.private_sale_duration,
                //     PresaleError::PrivateSaleNotOver
                // );
                presale.current_price = presale.public_price;
                presale.sale_stage = 2;
                msg!("Public sale started at {}", clock.unix_timestamp);
            }
            2 => {
                // Public Sale → Sale Ended (after 60 days)
                // require!(
                //     clock.unix_timestamp
                //         >= presale.presale_start
                //             + presale.private_sale_duration
                //             + presale.public_sale_duration,
                //     PresaleError::PublicSaleNotOver
                // );
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

        // ✅ Ensure the caller is the admin
        require!(
            presale.admin == ctx.accounts.admin.key(),
            PresaleError::Unauthorized
        );

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
        referrer: Pubkey,    // ✅ Optional referrer address
        is_influencer: bool, // ✅ True if referrer is an influencer (backend-provided)
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

        // ✅ Calculate referral reward if referrer exists
        if referrer != Pubkey::default() {
            let referral_percentage = if is_influencer {
                presale.influencer_referral_rate
            } else {
                presale.regular_referral_rate
            };

            let referral_reward = (tokens_to_purchase * referral_percentage as u64) / 100;

            // Ensure enough tokens exist
            let available_rewards = ctx.accounts.referral_wallet.amount;
            let remaining_rewards = available_rewards - presale.referral_charged * 1_000_000_000; // ✅ Adjust for sold tokens

            require!(
                remaining_rewards >= referral_reward * 1_000_000_000, // ✅ Ensure enough reward tokens remain
                PresaleError::InsufficientRewardTokens
            );

            // ✅ Update `referral_charged`
            presale.referral_charged += referral_reward;

            if referral_reward > 0 {
                emit!(ReferralRewardEvent {
                    referrer,
                    referred_buyer: buyer.key(),
                    reward_amount: referral_reward,
                    is_influencer,
                });

                msg!(
                    "Referrer {} received {} tokens from buyer {}",
                    referrer,
                    referral_reward,
                    buyer.key()
                );
            }
        }

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

    pub fn check_reward_token_balance(ctx: Context<CheckRewardTokenBalance>) -> Result<u64> {
        let presale = &ctx.accounts.presale;
        let referral_wallet = &ctx.accounts.referral_wallet;

        // ✅ Fetch the balance of the referral wallet
        let available_rewards = referral_wallet.amount;

        // ✅ Retrieve the total referral rewards charged so far
        let remaining_rewards = available_rewards - presale.referral_charged * 1_000_000_000; // Adjust decimals

        msg!(
            "Referrer Account has {} tokens available in the referral wallet",
            remaining_rewards
        );

        Ok(remaining_rewards)
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

    pub fn buy_tokens_by_stable_coin(
        ctx: Context<BuyTokensByStableCoin>,
        payment_type: u8, // 0 = Web3, 1 = Web2
        stable_coin_amount: u64,
        referrer: Pubkey, // ✅  referrer address 11111111111111111111111111111111
        is_influencer: bool, // ✅ True if referrer is an influencer (backend-provided)
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;
        let buyer = &ctx.accounts.buyer;

        // ✅ Ensure only USDT or USDC is used
        require!(
            ctx.accounts.stable_coin_mint.key() == USDC_ADDRESS,
            PresaleError::InvalidStableToken
        );

        // ✅ Ensure SOL price is at least $1
        require!(stable_coin_amount >= 1, PresaleError::InvalidPrice);

        // ✅ Ensure presale is active (Private Sale or Public Sale)
        require!(
            presale.sale_stage == 1 || presale.sale_stage == 2,
            PresaleError::PresaleNotActive
        );

        // ✅ Convert stable coin amount to token amount
        let tokens_to_purchase = (stable_coin_amount * 1_000_000) / presale.current_price;

        // ✅ Ensure enough tokens exist
        let available_tokens = ctx.accounts.presale_wallet.amount;
        let remaining_tokens = available_tokens - presale.total_sold * 1_000_000_000;

        require!(
            remaining_tokens >= tokens_to_purchase * 1_000_000_000,
            PresaleError::InsufficientTokens
        );

        if payment_type == 0 {
            // ✅ Transfer stable coins to the merchant wallet
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    TokenTransfer {
                        from: ctx.accounts.buyer_stable_coin_account.to_account_info(),
                        to: ctx.accounts.merchant_stable_coin_account.to_account_info(),
                        authority: ctx.accounts.buyer.to_account_info(),
                    },
                ),
                stable_coin_amount * USDC_DECIMALS, // Convert to correct decimal
            )?;
        }

        // ✅ Update `total_sold`
        presale.total_sold += tokens_to_purchase;

        // ✅ Calculate referral reward if referrer exists
        if referrer != Pubkey::default() {
            let referral_percentage = if is_influencer {
                presale.influencer_referral_rate
            } else {
                presale.regular_referral_rate
            };

            let referral_reward = (tokens_to_purchase * referral_percentage as u64) / 100;

            // Ensure enough tokens exist
            let available_rewards = ctx.accounts.referral_wallet.amount;
            let remaining_rewards = available_rewards - presale.referral_charged * 1_000_000_000; // ✅ Adjust for sold tokens

            require!(
                remaining_rewards >= referral_reward * 1_000_000_000, // ✅ Ensure enough reward tokens remain
                PresaleError::InsufficientRewardTokens
            );

            // ✅ Update `referral_charged`
            presale.referral_charged += referral_reward;

            if referral_reward > 0 {
                emit!(ReferralRewardEvent {
                    referrer,
                    referred_buyer: buyer.key(),
                    reward_amount: referral_reward,
                    is_influencer,
                });

                msg!(
                    "Referrer {} received {} tokens from buyer {}",
                    referrer,
                    referral_reward,
                    buyer.key()
                );
            }
        }

        emit!(BuyTokensByStableCoinEvent {
            buyer: buyer.key(),
            tokens_purchased: tokens_to_purchase,
            stable_coin_amount,
            payment_type,
        });

        msg!(
            "Buyer {} purchased {} tokens with {} USDC (Stored for withdrawal)",
            buyer.key(),
            tokens_to_purchase,
            stable_coin_amount
        );

        Ok(())
    }

    pub fn set_referral_rate(
        ctx: Context<SetReferralRate>,
        regular_referral_rate: u8,
        influencer_referral_rate: u8,
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // ✅ Ensure the caller is the admin
        require!(
            presale.admin == ctx.accounts.admin.key(),
            PresaleError::Unauthorized
        );

        // ✅ Ensure referral rates are between 0-100%
        require!(
            regular_referral_rate <= 100 && influencer_referral_rate <= 100,
            PresaleError::InvalidRate
        );

        // ✅ Update referral rates
        presale.regular_referral_rate = regular_referral_rate;
        presale.influencer_referral_rate = influencer_referral_rate;

        // ✅ Emit an event to track changes
        emit!(SetReferralRateEvent {
            admin: ctx.accounts.admin.key(),
            regular_referral_rate,
            influencer_referral_rate,
        });

        msg!(
            "Referral rates updated: Regular = {}%, Influencer = {}%",
            regular_referral_rate,
            influencer_referral_rate
        );

        Ok(())
    }

    pub fn finalize_presale(ctx: Context<FinalizePresale>) -> Result<()> {
        // ✅ 1. Extract the bump first before borrowing presale mutably
        let presale_info = ctx.accounts.presale.to_account_info(); // ✅ Extract AccountInfo before mutable borrow
        let admin_key = ctx.accounts.admin.key(); // ✅ Extract admin key before mutable borrow
        let bump = ctx.bumps.presale;
        let presale = &mut ctx.accounts.presale;

        // ✅ Ensure the caller is the admin
        require!(presale.admin == admin_key, PresaleError::Unauthorized);

        // ✅ 1. Ensure presale has ended
        require!(presale.sale_stage == 3, PresaleError::PresaleActive);

        // ✅ 2. Check if liquidity pool has already been created
        require!(
            !presale.pool_created,
            PresaleError::LiquidityPoolAlreadyCreated
        );

        // ✅ 3. Calculate unsold presale tokens
        let available_presale_tokens = ctx.accounts.presale_wallet.amount;
        let unsold_presale_tokens =
            available_presale_tokens.saturating_sub(presale.total_sold * 1_000_000_000); // Adjust decimals

        // ✅ 4. Calculate unsold reward tokens
        let available_reward_tokens = ctx.accounts.referral_wallet.amount;
        let unsold_reward_tokens =
            available_reward_tokens.saturating_sub(presale.referral_charged * 1_000_000_000); // Adjust decimals

        let seeds: &[&[u8]] = &[
            PRESALE_SEED,
            admin_key.as_ref(), // ✅ Use extracted key instead of ctx.accounts.presale.admin
            &[bump],
        ];

        let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];
        // ✅ Transfer unsold presale tokens to liquidity wallet if any exist
        if unsold_presale_tokens > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    TokenTransfer {
                        from: ctx.accounts.presale_wallet.to_account_info(),
                        to: ctx.accounts.liquidity_wallet.to_account_info(),
                        authority: presale_info.clone(), // ✅ Use the extracted value here
                    },
                    signer_seeds,
                ),
                unsold_presale_tokens,
            )?;
        }

        // ✅ Transfer unsold referral tokens to liquidity wallet if any exist
        if unsold_reward_tokens > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    TokenTransfer {
                        from: ctx.accounts.referral_wallet.to_account_info(),
                        to: ctx.accounts.liquidity_wallet.to_account_info(),
                        authority: presale_info.clone(), // ✅ Use the extracted value here
                    },
                    signer_seeds,
                ),
                unsold_reward_tokens,
            )?;
        }

        // ✅ 5. Mark liquidity pool as created
        presale.pool_created = true;

        // ✅ 6. Emit an event for tracking
        emit!(FinalizePresaleEvent {
            admin: ctx.accounts.admin.key(),
            unsold_presale_tokens,
            unsold_reward_tokens,
        });

        msg!(
            "Presale finalized! {} unsold presale tokens & {} unsold referral tokens moved to liquidity wallet.",
            unsold_presale_tokens,
            unsold_reward_tokens
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
               8 +  // Referral Charged
               1 +  // Pool created flag
               32 + // Presale wallet
               32 + // Referral wallet
               32 + // Liquidity wallet
               32 + // Merchant wallet
               1 +  // Regular referral rate
               1 +   // Influencer referral rate
               1
    )]
    pub presale: Account<'info, Presale>, // Stores presale details

    pub token_mint: Account<'info, Mint>, // DYAWN token mint

    #[account(init, payer = admin, token::mint = token_mint, token::authority = presale)]
    pub presale_wallet: Account<'info, TokenAccount>,

    #[account(init, payer = admin, token::mint = token_mint, token::authority = presale)]
    pub referral_wallet: Account<'info, TokenAccount>, // Storage for referral rewards

    #[account(mut)]
    pub merchant_wallet: AccountInfo<'info>,

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

    #[account(mut)]
    pub referral_wallet: Account<'info, TokenAccount>, // Store Reward tokens

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

#[derive(Accounts)]
pub struct BuyTokensByStableCoin<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>, // The user buying tokens

    #[account(
        mut,
        seeds = [PRESALE_SEED, presale.admin.as_ref()],
        bump,
    )]
    pub presale: Account<'info, Presale>, // Presale storage PDA

    #[account(mut)]
    pub presale_wallet: Account<'info, TokenAccount>, // Presale token storage

    #[account(mut)]
    pub referral_wallet: Account<'info, TokenAccount>, // Store Reward tokens

    #[account(mut)]
    pub buyer_stable_coin_account: Account<'info, TokenAccount>, // Buyer’s USDC account

    #[account(mut)]
    pub merchant_stable_coin_account: Account<'info, TokenAccount>, // Merchant’s stable coin account

    #[account()]
    pub stable_coin_mint: Account<'info, Mint>, // USDC mint

    pub token_program: Program<'info, Token>, // Solana Token Program
}

#[derive(Accounts)]
pub struct SetReferralRate<'info> {
    #[account(mut)]
    pub admin: Signer<'info>, // ✅ Only admin can call this function

    #[account(
        mut,
        has_one = admin, // ✅ Ensures only the admin can update referral rates
        seeds = [PRESALE_SEED, admin.key().as_ref()],
        bump
    )]
    pub presale: Account<'info, Presale>,
}

#[derive(Accounts)]
pub struct CheckRewardTokenBalance<'info> {
    #[account(
        mut,
        seeds = [PRESALE_SEED, presale.admin.as_ref()], 
        bump,
    )]
    pub presale: Account<'info, Presale>, // ✅ Presale contract state

    #[account(mut)]
    pub referral_wallet: Account<'info, TokenAccount>, // ✅ Referral wallet holding reward tokens
}

#[derive(Accounts)]
pub struct FinalizePresale<'info> {
    #[account(mut)]
    pub admin: Signer<'info>, // ✅ Only admin can finalize the presale

    #[account(
        mut,
        has_one = admin, // ✅ Ensures only the presale admin can call this
        seeds = [PRESALE_SEED, admin.key().as_ref()],
        bump
    )]
    pub presale: Account<'info, Presale>, // ✅ Presale account

    #[account(mut)]
    pub presale_wallet: Account<'info, TokenAccount>, // ✅ Source wallet (Presale tokens)

    #[account(mut)]
    pub referral_wallet: Account<'info, TokenAccount>, // ✅ Source wallet (Referral tokens)

    #[account(mut)]
    pub liquidity_wallet: Account<'info, TokenAccount>, // ✅ Destination wallet (Liquidity)

    pub token_program: Program<'info, Token>, // ✅ Solana Token Program
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
    pub referral_charged: u64,      // Total tokens sold
    pub pool_created: bool,         // Liquidity pool created flag
    pub presale_wallet: Pubkey,     // Token account for presale
    pub referral_wallet: Pubkey,    // Token account for referral rewards
    pub merchant_wallet: Pubkey,
    pub regular_referral_rate: u8, // Referral reward % for regular users
    pub influencer_referral_rate: u8, // Referral reward % for influencers
    pub bump: u8,                  // Store bump here
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

#[event]
pub struct BuyTokensByStableCoinEvent {
    pub buyer: Pubkey,
    pub tokens_purchased: u64,
    pub stable_coin_amount: u64,
    pub payment_type: u8, // 0 = Web3, 1 = Web2 (Stored for withdrawal)
}

#[event]
pub struct ReferralRewardEvent {
    pub referrer: Pubkey,       // ✅ The referrer who gets rewarded
    pub referred_buyer: Pubkey, // ✅ The buyer who used the referral
    pub reward_amount: u64,     // ✅ Amount of tokens rewarded
    pub is_influencer: bool,    // ✅ Whether the referrer is an influencer
}

#[event]
pub struct SetReferralRateEvent {
    pub admin: Pubkey,
    pub regular_referral_rate: u8,
    pub influencer_referral_rate: u8,
}

#[event]
pub struct FinalizePresaleEvent {
    pub admin: Pubkey,              // ✅ Admin who finalized presale
    pub unsold_presale_tokens: u64, // ✅ Number of unsold presale tokens moved to liquidity wallet
    pub unsold_reward_tokens: u64, // ✅ Number of unsold referral tokens moved to liquidity wallet
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

    #[msg("Presale is active now.")]
    PresaleActive,

    #[msg("Not enough tokens available for purchase.")]
    InsufficientTokens,

    #[msg("Not enough Reward tokens available for purchase.")]
    InsufficientRewardTokens,

    #[msg("Insufficient SOL sent for purchase.")]
    InsufficientFunds,

    #[msg("Invalid stable token. Only USDC is accepted.")]
    InvalidStableToken,

    #[msg("Not enough USDC available for purchase.")]
    InsufficientUSDC,

    #[msg("Invalid payment type. Please choose 1 or 2")]
    InvalidPaymentType,

    #[msg("Invalid price: SOL Amount in Usd must be over than $1.")]
    InvalidPrice, // ✅ New error for minimum SOL price check

    #[msg("Unauthorized: Only the presale admin can perform this action.")]
    // ✅ New admin-only error
    Unauthorized,

    #[msg("The liquidity pool has not been created yet.")]
    LiquidityPoolNotCreated,

    #[msg("The withdrawal request signature is invalid.")]
    InvalidSignature,

    #[msg("The withdrawal request has expired.")]
    ExpiredSignature,

    #[msg("No unsold tokens available for transfer.")]
    NoUnsoldTokens,

    #[msg("The liquidity pool has already been created.")]
    LiquidityPoolAlreadyCreated,
}
