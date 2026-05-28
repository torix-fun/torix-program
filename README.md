# Torix.fun Smart Contract

Program ID: `torXFavtJnaJzW7fz2NVrg9f1j824GitYi69zhmJQBK`

Torix.fun is a Solana-based **Launch Wars** protocol. Creators launch memecoins on bonding curves and compete for trading volume within timed rounds. The top-N curves by cumulative volume win SOL rewards from a round prize pool.

## What is Launch Wars?

- **Rounds** last a fixed duration (24 hours in v1).
- **Creators** launch tokens via permissionless `launch`.
- **Traders** buy and sell tokens via `buy_exact` / `sell_exact`.
- **Volume** (buy + sell SOL) is tracked per curve.
- **Winners** — the top `winners_per_round` curves by `volume_sol` — receive rewards from the `RoundVault` when the round ends.

## Core Instructions

| Instruction | Caller | Description |
|-------------|--------|-------------|
| `launch` | Anyone | Create a new bonding curve token |
| `buy_exact` | Anyone | Buy tokens with SOL |
| `sell_exact` | Anyone | Sell tokens for SOL |
| `start_round` | Anyone | Start a new timed round and vault |
| `end_round` | `end_round_authority` | Distribute vault prizes to top-volume creators |
| `initialize_global` | Program upgrade authority | Initialize `GlobalConfig` |
| `update_global` | `protocol_authority` | Update protocol parameters |
| `start_migration` | `migration_authority` | *(v2 — not yet implemented)* |
| `migrate_liquidity` | `migration_authority` | *(v2 — not yet implemented)* |

## Fee Model

Fees are charged **only** on `buy_exact` and `sell_exact`:

- `fee_bps` — total fee in basis points (e.g. 100 = 1%).
- `round_fee_bps` — portion of `fee_bps` that goes to the current round's `RoundVault`.
- The remainder (`fee_bps - round_fee_bps`) goes to `fee_recipient` as the **protocol fee**.

Example: `fee_bps = 100`, `round_fee_bps = 50`.
- Total fee = 1% of trade.
- Round fee = 0.5% → `RoundVault`.
- Protocol fee = 0.5% → `fee_recipient`.

## Authority Model

| Authority | Purpose |
|-----------|---------|
| `protocol_authority` | Update `GlobalConfig` via `update_global` |
| `end_round_authority` | End rounds and distribute prizes via `end_round` |
| `migration_authority` | Execute migration instructions (v2) |

All other instructions are permissionless.

## Documentation

- [`ARCHITECTURE.md`](./ARCHITECTURE.md) — State accounts, PDAs, math, and data flows
- [`SECURITY.md`](./SECURITY.md) — Trust model and security considerations
- [`EVENTS.md`](./EVENTS.md) — On-chain event reference
- [`ERRORS.md`](./ERRORS.md) — Error code reference

## Tech Stack

- Anchor 1.0.2
- Solana (BPF/SBF target)
- spl-token-2022 (Token-2022 program with metadata extensions)
- Rust 2021 edition
