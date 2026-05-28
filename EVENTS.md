# Events

Torix emits Anchor events for every significant state change. Indexers and UIs should listen for these.

All events are defined in `src/events/curve.rs`.

---

## `LaunchEvent`

Emitted when a new token is launched via `launch`.

```rust
#[event]
pub struct LaunchEvent {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub curve: Pubkey,
    pub timestamp: i64,
    pub token_name: String,
    pub token_symbol: String,
    pub token_uri: String,
}
```

| Field | Description |
|-------|-------------|
| `creator` | The user who called `launch` |
| `mint` | The newly created Token-2022 mint address |
| `curve` | The `CurveState` PDA address |
| `timestamp` | Unix timestamp of the launch |
| `token_name` | Token name (max 32 bytes) |
| `token_symbol` | Token symbol (max 10 bytes) |
| `token_uri` | Off-chain metadata URI (max 200 bytes) |

---

## `BuyEvent`

Emitted on every successful `buy_exact`.

```rust
#[event]
pub struct BuyEvent {
    pub trader: Pubkey,
    pub mint: Pubkey,
    pub curve: Pubkey,
    pub creator: Pubkey,
    pub gross_sol_in: u64,
    pub net_sol_in: u64,
    pub tokens_out: u64,
    pub protocol_fee: u64,
    pub round_fee: u64,
    pub virtual_sol_reserves_after: u64,
    pub virtual_token_reserves_after: u64,
    pub timestamp: i64,
}
```

| Field | Description |
|-------|-------------|
| `trader` | The buyer's wallet address |
| `mint` | Token mint being bought |
| `curve` | The `CurveState` PDA |
| `creator` | The curve's creator (for grouping / leaderboard) |
| `gross_sol_in` | Total SOL sent by the trader (including fees) |
| `net_sol_in` | SOL actually added to the curve reserves (`gross_sol_in - total_fee`) |
| `tokens_out` | Tokens received by the trader |
| `protocol_fee` | SOL sent to `fee_recipient` |
| `round_fee` | SOL sent to `RoundVault` |
| `virtual_sol_reserves_after` | Curve's `virtual_reserves_sol` after the trade |
| `virtual_token_reserves_after` | Curve's `virtual_reserves_tokens` after the trade |
| `timestamp` | Unix timestamp of the trade |

**Fee verification:** `gross_sol_in == net_sol_in + protocol_fee + round_fee`

---

## `SellEvent`

Emitted on every successful `sell_exact`.

```rust
#[event]
pub struct SellEvent {
    pub trader: Pubkey,
    pub mint: Pubkey,
    pub curve: Pubkey,
    pub creator: Pubkey,
    pub tokens_in: u64,
    pub gross_sol_out: u64,
    pub net_sol_out: u64,
    pub protocol_fee: u64,
    pub round_fee: u64,
    pub virtual_sol_reserves_after: u64,
    pub virtual_token_reserves_after: u64,
    pub timestamp: i64,
}
```

| Field | Description |
|-------|-------------|
| `trader` | The seller's wallet address |
| `mint` | Token mint being sold |
| `curve` | The `CurveState` PDA |
| `creator` | The curve's creator |
| `tokens_in` | Tokens sent by the trader to the curve |
| `gross_sol_out` | Total SOL computed by the AMM before fees |
| `net_sol_out` | SOL actually received by the trader (`gross_sol_out - total_fee`) |
| `protocol_fee` | SOL deducted from curve to `fee_recipient` |
| `round_fee` | SOL deducted from curve to `RoundVault` |
| `virtual_sol_reserves_after` | Curve's `virtual_reserves_sol` after the trade |
| `virtual_token_reserves_after` | Curve's `virtual_reserves_tokens` after the trade |
| `timestamp` | Unix timestamp of the trade |

**Fee verification:** `gross_sol_out == net_sol_out + protocol_fee + round_fee`

---

## Indexing Recommendations

### Per-Curve Volume

Sum `gross_sol_in` from `BuyEvent` and `gross_sol_out` from `SellEvent` grouped by `curve`. This matches the on-chain `CurveStats.volume_sol`.

### Leaderboard (Current Round)

1. Listen for `LaunchEvent` to discover new curves in the current round.
2. Accumulate `gross_sol_in + gross_sol_out` per curve from `BuyEvent` and `SellEvent`.
3. Sort descending by volume.
4. Top `winners_per_round` curves are the projected winners.

### Fee Tracking

Track `protocol_fee` and `round_fee` per trade to monitor protocol revenue and prize pool growth.
