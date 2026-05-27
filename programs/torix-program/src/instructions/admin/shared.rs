use anchor_lang::prelude::*;
use crate::state::*;


#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct GlobalConfigArgs {
    pub protocol_authority: Pubkey,
    pub end_round_authority: Pubkey,
    pub migration_authority: Pubkey,
    pub winners_per_round: u16,
    pub fee_bps: u16,
    pub round_fee_bps: u16,
    pub round_duration_seconds: i64,
    pub fee_recipient: Pubkey
}

pub fn configure_global<'info>(
    config: &mut Account<'info, GlobalConfig>,
    args: GlobalConfigArgs
) -> () {
    config.protocol_authority = args.protocol_authority;
    config.end_round_authority = args.end_round_authority;
    config.migration_authority = args.migration_authority;

    config.winners_per_round = args.winners_per_round;
    config.fee_bps = args.fee_bps;
    config.round_fee_bps = args.round_fee_bps;
    config.fee_recipient = args.fee_recipient;
    config.round_duration_seconds = args.round_duration_seconds;
}