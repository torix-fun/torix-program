use anchor_lang::prelude::*;


#[derive(Accounts)]
pub struct InitializeGlobal {

}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct InitializeGlobalArgs {
    protocol_authority: Pubkey,
    end_round_authority: Pubkey,
    migration_authority: Pubkey,
    winners_per_round: u16
}

pub fn handler(
    ctx: Context<InitializeGlobal>,
    args: InitializeGlobalArgs
) -> Result<()> {
    let InitializeGlobalArgs { 
        protocol_authority, 
        end_round_authority, 
        migration_authority, 
        winners_per_round 
    } = args;

    

    Ok(())
}