use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("6Ly442xt8FFdhaS5Z8XkZrkbhp9skfBdJdcP1YfjpChe");

#[program]
pub mod agriculture_tokenization {
    use super::*;

    // Initialize an agricultural lot (e.g., crops or livestock)
    pub fn initialize_lot(
        ctx: Context<InitializeLot>,
        lot_name: String,
        yield_estimate: u64,
        harvest_time: i64,
    ) -> Result<()> {
        require!(yield_estimate > 0, AgricultureError::InsufficientYield);
        require!(
            harvest_time > Clock::get()?.unix_timestamp,
            AgricultureError::InvalidHarvestTime
        );

        let lot = &mut ctx.accounts.lot;
        lot.owner = *ctx.accounts.farmer.key;
        lot.lot_name = lot_name;
        lot.yield_estimate = yield_estimate;
        lot.harvest_time = harvest_time;
        lot.token_mint = *ctx.accounts.token_mint.to_account_info().key;

        emit!(LotInitialized {
            lot_name,
            owner: *ctx.accounts.farmer.key,
            yield_estimate,
            harvest_time,
        });

        Ok(())
    }

    // Distribute revenue from sales to token holders
    pub fn distribute_revenue(ctx: Context<DistributeRevenue>, total_revenue: u64) -> Result<()> {
        require!(total_revenue > 0, AgricultureError::InvalidRevenueAmount);
        let total_supply = ctx.accounts.token_mint.supply; // Total token supply

        for holder in ctx.remaining_accounts.iter() {
            let holder_account = Account::<TokenAccount>::try_from(holder)?;
            let holder_share = calculate_share(holder_account.amount, total_supply, total_revenue);

            let cpi_accounts = Transfer {
                from: ctx.accounts.farmer_token_account.to_account_info(),
                to: holder_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),  // Corrected to `owner`
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::transfer(cpi_ctx, holder_share)?;
        }

        emit!(RevenueDistributed {
            lot: ctx.accounts.lot.key(),
            total_revenue,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    // Fetch external weather data (CPI Example)
    pub fn fetch_weather_data(ctx: Context<FetchWeatherData>) -> Result<()> {
        let weather_program = &ctx.accounts.weather_program;
        // CPI call to external weather program (e.g., oracle) to fetch weather data.
        msg!("Fetching weather data from external program...");
        Ok(())
    }
}

// ------------------- HELPER FUNCTIONS -------------------

fn calculate_share(holder_amount: u64, total_supply: u64, total_revenue: u64) -> u64 {
    holder_amount * total_revenue / total_supply
}

// ------------------- EVENTS -------------------

#[event]
pub struct LotInitialized {
    pub lot_name: String,
    pub owner: Pubkey,
    pub yield_estimate: u64,
    pub harvest_time: i64,
}

#[event]
pub struct RevenueDistributed {
    pub lot: Pubkey,
    pub total_revenue: u64,
    pub timestamp: i64,
}

// ------------------- ACCOUNT STRUCTS -------------------

#[account]
pub struct LotAccount {
    pub owner: Pubkey,          // Farmer's address
    pub lot_name: String,       // Name of the agricultural lot
    pub yield_estimate: u64,    
    pub harvest_time: i64,     
    pub token_mint: Pubkey,    
}

#[derive(Accounts)]
pub struct InitializeLot<'info> {
    #[account(
        init, 
        payer = farmer, 
        space = 8 + LotAccount::MAX_SIZE, 
        seeds = [b"lot", farmer.key().as_ref()], 
        bump
    )]
    pub lot: Account<'info, LotAccount>,      // The agricultural lot
    #[account(mut)]
    pub farmer: Signer<'info>,                
    pub token_mint: Account<'info, Mint>,     
    #[account(mut)]
    pub farmer_token_account: Account<'info, TokenAccount>,  
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DistributeRevenue<'info> {
    #[account(mut, has_one = owner @ AgricultureError::InvalidOwner)]
    pub lot: Account<'info, LotAccount>,      
    pub owner: Signer<'info>,                
    #[account(mut)]
    pub farmer_token_account: Account<'info, TokenAccount>,  
    pub token_mint: Account<'info, Mint>,     
    pub token_program: Program<'info, Token>, 
}

#[derive(Accounts)]
pub struct FetchWeatherData<'info> {
    pub weather_program: Program<'info, OracleProgram>,  // Placeholder for external weather program
    #[account(mut)]
    pub farmer: Signer<'info>,                          
}

// ------------------- CUSTOM ERRORS -------------------

#[error_code]
pub enum AgricultureError {
    #[msg("Insufficient yield estimate for the lot.")]
    InsufficientYield,
    #[msg("Harvest time must be in the future.")]
    InvalidHarvestTime,
    #[msg("Revenue must be greater than zero.")]
    InvalidRevenueAmount,
    #[msg("Unauthorized owner for this action.")]
    InvalidOwner,
}

// ------------------- CONSTANTS -------------------

impl LotAccount {
    pub const MAX_SIZE: usize = 8 + 32 + 40 + 8 + 8 + 32; 
}

// Placeholder struct to represent external oracle program. Replace this with the actual program you are using.
pub struct OracleProgram;
