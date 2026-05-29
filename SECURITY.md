# Security

This document describes the security model, trust assumptions, and known limitations of the Torix.fun smart contract (v1).

## Trust Model

### Authorities

The protocol recognizes three privileged authorities, all set in `GlobalConfig`:

| Authority | Risk Level | Mitigation |
|-----------|------------|------------|
| `protocol_authority` | High | Can change fees, round duration, winner count, and all authority addresses. Should be a multisig or governance program. |
| `end_round_authority` | Medium | Can decide which curves "win" and how much each receives. The backend computes rankings off-chain; the contract only validates payouts. Should be a backend service key with limited scope. |
| `migration_authority` | Low (currently) | Migration instructions are unimplemented in v1. No risk until v2 is deployed. |

### Permissionless Instructions

`launch`, `buy_exact`, `sell_exact`, and `start_round` are permissionless. This is by design:
- Anyone can create a curve.
- Anyone can start a new round (but there is only one active round PDA at a time).
- Anyone can trade.

## Round Vault Safety

### Source of Funds

The `RoundVault` only receives SOL via `round_fee` deductions on `buy_exact` and `sell_exact`. There are no direct-deposit instructions.

### Distribution Guarantees

`end_round` enforces:
- `now >= round.end_timestamp` — round must actually be over.
- `amounts.len() == winners_per_round` — exactly the configured number of payouts.
- Each `amount > 0` — no zero-reward winners.
- `vault_balance - rent >= sum(amounts)` — vault is solvent for the proposed payouts.
- `winner.is_writable` — winner accounts can receive lamports.
- `curve.owner == torix_program::ID` — curves are real program accounts.
- `curve_state.round == current_round` — curves belong to the round being closed.
- `curve_state.creator == winner` — payout goes to the creator, not an arbitrary address.

After payouts, the vault and round accounts are closed and their rent-exempt lamports sent to `fee_recipient`.

## Arithmetic Safety

### Overflow Protection

All arithmetic uses **checked operations**:
- `checked_add`, `checked_sub`, `checked_mul`, `checked_div`
- `u128` intermediates for reserve multiplications (`virtual_sol * virtual_tokens`)
- `u64::try_from` for narrowing `u128` → `u64` with explicit error mapping

No `as u64` casts are used in math paths.

### Rent-Exemption Safety

In `sell_exact`, the curve's spendable balance is computed as:

```rust
let curve_rent_exemption = Rent::get()?.minimum_balance(discriminator + CurveState::INIT_SPACE);
let curve_lamports = curve.get_lamports().saturating_sub(curve_rent_exemption);
```

This prevents sells from draining the rent-exempt minimum, ensuring the `CurveState` account remains valid.

## Slippage Protection

Both trade instructions accept a slippage parameter:
- `buy_exact(sol_in, min_tokens_out)` — fails if `tokens_out < min_tokens_out`
- `sell_exact(tokens_in, min_sol_out)` — fails if `net_sol_out < min_sol_out`

Clients should set these based on current on-chain state with an acceptable tolerance.

## Account Validation

### Address Constraints

- `fee_recipient` address is constrained to `global_config.fee_recipient` in trade instructions.
- `end_round` signer is constrained to `global_config.end_round_authority`.
- `update_global` signer is constrained to `global_config.protocol_authority`.

### PDA Constraints

All PDAs use Anchor `seeds` + `bump` constraints:
- `CurveState`: `["curve", mint]`
- `RoundState`: `["round"]`
- `RoundVault`: `["round_vault", round]`
- `GlobalConfig`: `["global_config"]`

### Owner Checks

`end_round` explicitly verifies that each curve account's owner is the Torix program ID before deserializing its state.

## Known Limitations (v1)

### Single Round PDA

Only one round can exist at a time (`["round"]`). If a round is started while another is active, the new transaction will fail because the PDA already exists. This is intentional for v1 but limits concurrent rounds.

### Off-Chain Winner Selection

The contract does **not** compute rankings on-chain. The backend must:
1. Index all curves and their `volume_sol`.
2. Sort by volume.
3. Call `end_round` with the correct winner accounts and amounts.

A compromised or buggy backend could select incorrect winners or amounts. The contract validates that amounts sum to ≤ vault balance and that creators match, but it does not validate that the selected curves are actually the top-N by volume.

### Migration Stubs

`start_migration` and `migrate_liquidity` return `ErrorCode::NotImplemented`. These will be implemented in v2. Until then, no curve can reach `Migrated` status.

### No Pause / Emergency Stop

There is no global pause mechanism. If a critical bug is found, the only recourse is to upgrade the program (which requires the upgrade authority) or set extreme fees via `update_global`.

## Fee Configuration Risks

`protocol_authority` can set:
- `fee_bps` up to `10_000` (100%).
- `round_fee_bps` up to `fee_bps`.

There is no hard cap below 100%. A malicious or compromised `protocol_authority` could set fees to 100%, making all trades lose their entire input to fees.

**Recommendation:** Use a multisig or DAO governance for `protocol_authority`.
