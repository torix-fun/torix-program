# Errors

Reference for every `ErrorCode` variant in the Torix program, including the human-readable message and the exact conditions that trigger it.

All errors are defined in `src/error.rs`.

---

## `SlippageExceeded`

**Message:** `"Slippage exceeded"`

**Triggered by:**
- `buy_exact` — when `tokens_out < min_tokens_out`
- `sell_exact` — when `net_sol_out < min_sol_out`

**Client action:** Increase slippage tolerance or retry with updated reserves.

---

## `WinnerNotWritable`

**Message:** `"Winner account must be writable"`

**Triggered by:**
- `end_round` — when a winner account in `remaining_accounts` is not marked writable.

**Client action:** Ensure winner accounts are passed as writable (`isSigner: false, isWritable: true`).

---

## `RoundNotOver`

**Message:** `"Round is not over"`

**Triggered by:**
- `end_round` — when `Clock::now().unix_timestamp < round.end_timestamp`.

**Client action:** Wait until the round's `end_timestamp` has passed before calling `end_round`.

---

## `InsufficientOutputAmount`

**Message:** `"Insufficient output amount"`

**Triggered by:**
- `calculate_buy_amount_out` — when `tokens_out == 0` (trade too small or reserves imbalanced).
- `calculate_sell_amount_out` — when `gross_sol_out == 0`.

**Client action:** Increase trade size or check curve reserve state.

---

## `NotImplemented`

**Message:** `"Instruction not yet implemented"`

**Triggered by:**
- `start_migration` — always returns this error in v1.
- `migrate_liquidity` — always returns this error in v1.

**Client action:** Do not call these instructions until v2 is deployed.

---

## `ZeroRewardAmount`

**Message:** `"Reward amount must be greater than zero"`

**Triggered by:**
- `end_round` — when any value in `amounts` is `0`.

**Client action:** Ensure all reward amounts are positive integers.

---

## `InsufficientCurveBalance`

**Message:** `"Insufficient curve balance for sell"`

**Triggered by:**
- `sell_exact` — when the curve's spendable SOL (total lamports minus rent exemption) is less than `net_sol_out + protocol_fee + round_fee`.

**Client action:** Reduce sell size or wait for more buy volume to enter the curve.

---

## `ZeroTradeAmount`

**Message:** `"Trade amount must be greater than zero"`

**Triggered by:**
- `buy_exact` — when `sol_in == 0`.
- `sell_exact` — when `tokens_in == 0`.

**Client action:** Provide a positive trade amount.

---

## `MetadataTooLong`

**Message:** `"Token metadata exceeds maximum length"`

**Triggered by:**
- `launch` — when `token_name.len() > 32`, `token_symbol.len() > 10`, or `token_uri.len() > 200`.

**Client action:** Shorten the name, symbol, or URI to fit within the limits.

---

## Implicit Errors

The following are not custom `ErrorCode` variants but are commonly encountered:

### `ProgramError::ArithmeticOverflow`

**Triggered by:**
- Any `checked_add`, `checked_sub`, `checked_mul`, `checked_div` that overflows.
- `u64::try_from(u128)` that fails (value > `u64::MAX`).

**Locations:** `math/bonding_curve.rs`, `math/fees.rs`, `buy_exact.rs`, `sell_exact.rs`, `start_round.rs`, `end_round.rs`.

### `ProgramError::IncorrectAuthority`

**Triggered by:**
- `initialize_global` — when the signer is not the program's upgrade authority.

### Anchor Constraint Errors

**Triggered by:**
- Missing or incorrect PDA bumps.
- Address mismatches (e.g. `fee_recipient != global_config.fee_recipient`).
- Missing required accounts in `remaining_accounts` for `end_round`.
