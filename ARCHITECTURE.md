# Architecture

This document describes the internal structure of the Torix.fun smart contract.

## Program ID

`torXFavtJnaJzW7fz2NVrg9f1j824GitYi69zhmJQBK`

## Module Layout

```
src/
  lib.rs                    — Program entrypoint and instruction dispatch
  constants.rs              — Protocol constants, seeds, constraints
  error.rs                  — ErrorCode enum
  state.rs                  — Module re-exports
  state/
    config.rs               — GlobalConfig
    curve.rs                — CurveState, CurveStats, CurveStatus
    round.rs                — RoundState, RoundVault
  math/
    mod.rs                  — Re-exports
    bonding_curve.rs        — Constant-product AMM math
    fees.rs                 — Fee split calculation
  instructions/
    mod.rs                  — Re-exports
    curve/
      launch.rs             — Token creation
      buy_exact.rs          — Buy tokens
      sell_exact.rs         — Sell tokens
    round/
      start_round.rs        — Create round + vault
      end_round.rs          — Distribute prizes
    admin/
      initialize_global.rs  — Init GlobalConfig
      update_global.rs      — Update GlobalConfig
      shared.rs             — GlobalConfigArgs, configure_global helper
    liquidity/
      mod.rs                — Migration stubs (v2)
  events/
    mod.rs                  — Re-exports
    curve.rs                — LaunchEvent, BuyEvent, SellEvent
```

## State Accounts

### GlobalConfig

PDA: `["global_config"]`

| Field | Type | Description |
|-------|------|-------------|
| `protocol_version` | `u8` | Currently `1` |
| `bump` | `u8` | PDA bump |
| `winners_per_round` | `u16` | Number of winning curves per round (e.g. `10`) |
| `fee_bps` | `u16` | Total trade fee in basis points |
| `round_fee_bps` | `u16` | Portion of `fee_bps` directed to the round vault |
| `round_duration_seconds` | `i64` | Duration of each round in seconds (e.g. `86400` = 24h) |
| `protocol_authority` | `Pubkey` | Can update `GlobalConfig` |
| `end_round_authority` | `Pubkey` | Can invoke `end_round` |
| `migration_authority` | `Pubkey` | Can invoke migration instructions (v2) |
| `fee_recipient` | `Pubkey` | Receives protocol fees; also receives closed vault lamports |

### RoundState

PDA: `["round"]`

| Field | Type | Description |
|-------|------|-------------|
| `bump` | `u8` | PDA bump |
| `end_timestamp` | `i64` | Unix timestamp when the round ends |
| `vault` | `Pubkey` | Address of the corresponding `RoundVault` |

There is only **one active round at a time** (single PDA).

### RoundVault

PDA: `["round_vault", round.key()]`

| Field | Type | Description |
|-------|------|-------------|
| `bump` | `u8` | PDA bump |

Holds SOL collected from `round_fee` on trades. Closed at the end of each round, with any residual lamports sent to `fee_recipient`.

### CurveState

PDA: `["curve", mint.key()]`

| Field | Type | Description |
|-------|------|-------------|
| `bump` | `u8` | PDA bump |
| `status` | `CurveStatus` | `Active`, `Migrating`, or `Migrated` |
| `stats` | `CurveStats` | Volume and transaction counters |
| `round` | `Pubkey` | The `RoundState` this curve was launched in |
| `creator` | `Pubkey` | The user who launched the curve |
| `mint` | `Pubkey` | The token mint address |
| `real_reserves_sol` | `u64` | Actual SOL held by the curve PDA |
| `real_reserves_tokens` | `u64` | Actual tokens held in the curve ATA |
| `virtual_reserves_sol` | `u64` | Virtual SOL reserves for AMM pricing |
| `virtual_reserves_tokens` | `u64` | Virtual token reserves for AMM pricing |

### CurveStats

| Field | Type | Description |
|-------|------|-------------|
| `volume_sol` | `u64` | Cumulative SOL volume (buy + sell) |
| `sell_transactions` | `u64` | Number of sell transactions |
| `buy_transactions` | `u64` | Number of buy transactions |

### CurveStatus

- `Active` — Trading is live.
- `Migrating` — Preparing for external AMM migration (v2).
- `Migrated` — Liquidity migrated to external AMM (v2).

## Instruction Details

### `launch`

Creates a Token-2022 mint with metadata extensions, mints the total supply into a curve-associated token account, burns mint authority, and initializes a `CurveState`.

**Accounts:**
- `user` — signer, pays for creation
- `mint` — new mint signer
- `curve` — PDA initialized
- `global_config` — read-only
- `round` — read-only, binds curve to current round
- `mint_authority` — PDA used for minting
- `curve_token_account` — ATA for the curve

**Args:** `LaunchArgs { token_name, token_symbol, token_uri }`

**Constraints:**
- `token_name` ≤ 32 bytes
- `token_symbol` ≤ 10 bytes
- `token_uri` ≤ 200 bytes

### `buy_exact`

Trader sends SOL and receives tokens. Fees are deducted from the gross SOL input.

**Flow:**
1. Calculate fee split from `sol_in`.
2. Compute `net_sol = sol_in - total_fee`.
3. Run constant-product AMM with `net_sol` → `tokens_out`.
4. Validate slippage: `tokens_out >= min_tokens_out`.
5. Transfer `net_sol` from user to curve.
6. Transfer `protocol_fee` to `fee_recipient`.
7. Transfer `round_fee` to `RoundVault`.
8. Transfer `tokens_out` from curve ATA to user ATA.
9. Update `CurveState` reserves and stats.
10. Emit `BuyEvent`.

### `sell_exact`

Trader sends tokens and receives SOL. Fees are deducted from the gross SOL output.

**Flow:**
1. Transfer `tokens_in` from user ATA to curve ATA.
2. Run constant-product AMM with `tokens_in` → `gross_sol_out`.
3. Calculate fee split from `gross_sol_out`.
4. Compute `net_sol_out = gross_sol_out - total_fee`.
5. Validate slippage: `net_sol_out >= min_sol_out`.
6. Validate curve has sufficient spendable SOL (rent-exempt balance excluded).
7. Deduct `net_sol_out` from curve to user.
8. Deduct `protocol_fee` from curve to `fee_recipient`.
9. Deduct `round_fee` from curve to `RoundVault`.
10. Update `CurveState` reserves and stats.
11. Emit `SellEvent`.

### `start_round`

Creates a new `RoundState` and `RoundVault`. Permissionless.

**Flow:**
1. Read `round_duration_seconds` from `GlobalConfig`.
2. `end_timestamp = now + round_duration_seconds`.
3. Initialize `RoundState` and `RoundVault` PDAs.

### `end_round`

Closes the current round and distributes vault SOL to winning creators.

**Caller:** Must be `end_round_authority`.

**Remaining Accounts:** `2 * winners_per_round` accounts, in strict order:
1. `winners_per_round` winner accounts (must be writable, must match `curve.creator`).
2. `winners_per_round` curve accounts (must be owned by the program, must belong to the current round).

**Args:** `amounts: Vec<u64>` — reward per winner, same order as remaining accounts.

**Validations:**
- Round must be over (`now >= end_timestamp`).
- `amounts.len() == winners_per_round`.
- Each `amount > 0`.
- Vault balance ≥ sum of all amounts.
- `curve.creator == winner` for each pair.
- `curve.round == current_round` for each curve.

**Flow:**
1. Iterate pairs, validate, transfer `amount` from vault to winner.
2. Close `RoundVault` → lamports to `fee_recipient`.
3. Close `RoundState` → lamports to `fee_recipient`.

### `initialize_global`

One-time initialization of `GlobalConfig`. Caller must be the program's upgrade authority.

**Validation:** `program_data.upgrade_authority_address == user.key()`

### `update_global`

Updates `GlobalConfig`. Caller must be `protocol_authority`.

**Validation:** `user.key() == global_config.protocol_authority`

**Constraint:** `fee_bps >= round_fee_bps` (protocol fee cannot be negative).

## Bonding Curve Math

The AMM uses a **constant-product market maker** with virtual reserves:

```
k = virtual_sol * virtual_tokens
```

### Buy

```
new_virtual_sol = virtual_sol + net_sol
new_virtual_tokens = k / new_virtual_sol
tokens_out = virtual_tokens - new_virtual_tokens
```

All intermediate math uses `u128`. Results are converted back to `u64` via `try_from` with overflow checks.

### Sell

```
new_virtual_tokens = virtual_tokens + tokens_in
new_virtual_sol = k / new_virtual_tokens
gross_sol_out = virtual_sol - new_virtual_sol
```

Same `u128` safety rules as buy.

## Fee Math

```
total_fee   = amount * fee_bps / 10_000
round_fee   = amount * round_fee_bps / 10_000
protocol_fee = total_fee - round_fee
```

All operations use `checked_mul`, `checked_div`, and `checked_sub`.

## PDA Derivation Summary

| Account | Seeds | Program |
|---------|-------|---------|
| `GlobalConfig` | `["global_config"]` | Torix |
| `RoundState` | `["round"]` | Torix |
| `RoundVault` | `["round_vault", round]` | Torix |
| `CurveState` | `["curve", mint]` | Torix |
| `MintAuthority` | `["mint_authority"]` | Torix |
| `Curve ATA` | `[curve, token_2022_program, mint]` | ATA Program |
