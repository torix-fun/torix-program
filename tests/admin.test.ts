import { expect } from "chai";
import { Keypair } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import {
  getFixture, FixtureAccounts, initializeGlobal,
  deserializeGlobalConfig, airdropSOL,
  DEFAULT_FEE_BPS, DEFAULT_ROUND_FEE_BPS, DEFAULT_WINNERS_PER_ROUND,
  ONE_DAY_IN_SECONDS,
} from "./helpers";

describe("admin", () => {
  let fix: FixtureAccounts;

  before(async () => {
    fix = await getFixture();
    console.log("Fee recipient: ", fix.feeRecipient.publicKey.toBase58());
  });

  describe("initializeGlobal", () => {
    it("happy path: initializes with all 7 args and checks on-chain state", async () => {
      const state = deserializeGlobalConfig(
        (await fix.provider.connection.getAccountInfo(fix.globalConfig))!.data
      );

      expect(state.protocol_version).to.equal(1);
      expect(state.winners_per_round).to.equal(DEFAULT_WINNERS_PER_ROUND);
      expect(state.fee_bps).to.equal(DEFAULT_FEE_BPS);
      expect(state.round_fee_bps).to.equal(DEFAULT_ROUND_FEE_BPS);
      expect(state.protocol_authority.toString()).to.equal(fix.authority.publicKey.toString());
      expect(state.end_round_authority.toString()).to.equal(fix.endRoundAuthority.publicKey.toString());
      expect(state.migration_authority.toString()).to.equal(fix.migrationAuthority.publicKey.toString());
      expect(state.fee_recipient.toString()).to.equal(fix.feeRecipient.publicKey.toString());
    });

    it("rejects re-initialization (account already in use)", async () => {
      try {
        await initializeGlobal(fix.program, fix.authority, {
          protocolAuthority: fix.authority.publicKey,
          endRoundAuthority: fix.authority.publicKey,
          migrationAuthority: fix.authority.publicKey,
          winnersPerRound: 1,
          feeBps: DEFAULT_FEE_BPS,
          roundFeeBps: DEFAULT_ROUND_FEE_BPS,
          roundDurationSeconds: ONE_DAY_IN_SECONDS,
          feeRecipient: Keypair.generate().publicKey,
        });
        expect.fail("Expected re-initialization to fail");
      } catch (e: any) {
        const msg = e.logs ? e.logs.join(" ") : e.message || "";
        expect(msg).to.include("already in use");
      }
    });

    it("accepts fee_bps > 10000 (no on-chain validation)", async () => {
      await fix.program.methods
        .updateGlobal({
          protocolAuthority: fix.authority.publicKey,
          endRoundAuthority: fix.endRoundAuthority.publicKey,
          migrationAuthority: fix.migrationAuthority.publicKey,
          winnersPerRound: DEFAULT_WINNERS_PER_ROUND,
          feeBps: 15000,
          roundFeeBps: DEFAULT_ROUND_FEE_BPS,
          roundDurationSeconds: ONE_DAY_IN_SECONDS,
          feeRecipient: fix.feeRecipient.publicKey,
        })
        .accounts({ user: fix.authority.publicKey, globalConfig: fix.globalConfig })
        .signers([fix.authority])
        .rpc();

      const state = deserializeGlobalConfig(
        (await fix.provider.connection.getAccountInfo(fix.globalConfig))!.data
      );
      expect(state.fee_bps).to.equal(15000);

      // Reset
      await fix.program.methods
        .updateGlobal({
          protocolAuthority: fix.authority.publicKey,
          endRoundAuthority: fix.endRoundAuthority.publicKey,
          migrationAuthority: fix.migrationAuthority.publicKey,
          winnersPerRound: DEFAULT_WINNERS_PER_ROUND,
          feeBps: DEFAULT_FEE_BPS,
          roundFeeBps: DEFAULT_ROUND_FEE_BPS,
          roundDurationSeconds: ONE_DAY_IN_SECONDS,
          feeRecipient: fix.feeRecipient.publicKey,
        })
        .accounts({ user: fix.authority.publicKey, globalConfig: fix.globalConfig })
        .signers([fix.authority])
        .rpc();
    });

    it("accepts winners_per_round = 0 (no on-chain validation)", async () => {
      await fix.program.methods
        .updateGlobal({
          protocolAuthority: fix.authority.publicKey,
          endRoundAuthority: fix.endRoundAuthority.publicKey,
          migrationAuthority: fix.migrationAuthority.publicKey,
          winnersPerRound: 0,
          feeBps: DEFAULT_FEE_BPS,
          roundFeeBps: DEFAULT_ROUND_FEE_BPS,
          roundDurationSeconds: ONE_DAY_IN_SECONDS,
          feeRecipient: fix.feeRecipient.publicKey,
        })
        .accounts({ user: fix.authority.publicKey, globalConfig: fix.globalConfig })
        .signers([fix.authority])
        .rpc();

      const state = deserializeGlobalConfig(
        (await fix.provider.connection.getAccountInfo(fix.globalConfig))!.data
      );
      expect(state.winners_per_round).to.equal(0);

      // Reset
      await fix.program.methods
        .updateGlobal({
          protocolAuthority: fix.authority.publicKey,
          endRoundAuthority: fix.endRoundAuthority.publicKey,
          migrationAuthority: fix.migrationAuthority.publicKey,
          winnersPerRound: DEFAULT_WINNERS_PER_ROUND,
          feeBps: DEFAULT_FEE_BPS,
          roundFeeBps: DEFAULT_ROUND_FEE_BPS,
          roundDurationSeconds: ONE_DAY_IN_SECONDS,
          feeRecipient: fix.feeRecipient.publicKey,
        })
        .accounts({ user: fix.authority.publicKey, globalConfig: fix.globalConfig })
        .signers([fix.authority])
        .rpc();
    });
  });

  describe("updateGlobal", () => {
    it("protocol authority updates all fields", async () => {
      const newEndRound = Keypair.generate().publicKey;
      const newMigration = Keypair.generate().publicKey;
      const newFeeRecipient = Keypair.generate().publicKey;

      await fix.program.methods
        .updateGlobal({
          protocolAuthority: fix.authority.publicKey,
          endRoundAuthority: newEndRound,
          migrationAuthority: newMigration,
          winnersPerRound: 5,
          feeBps: 500,
          roundFeeBps: 200,
          roundDurationSeconds: ONE_DAY_IN_SECONDS,
          feeRecipient: newFeeRecipient,
        })
        .accounts({ user: fix.authority.publicKey, globalConfig: fix.globalConfig })
        .signers([fix.authority])
        .rpc();

      const state = deserializeGlobalConfig(
        (await fix.provider.connection.getAccountInfo(fix.globalConfig))!.data
      );

      expect(state.winners_per_round).to.equal(5);
      expect(state.fee_bps).to.equal(500);
      expect(state.round_fee_bps).to.equal(200);
      expect(state.end_round_authority.toString()).to.equal(newEndRound.toString());
      expect(state.migration_authority.toString()).to.equal(newMigration.toString());
      expect(state.fee_recipient.toString()).to.equal(newFeeRecipient.toString());

      // Reset
      await fix.program.methods
        .updateGlobal({
          protocolAuthority: fix.authority.publicKey,
          endRoundAuthority: fix.endRoundAuthority.publicKey,
          migrationAuthority: fix.migrationAuthority.publicKey,
          winnersPerRound: DEFAULT_WINNERS_PER_ROUND,
          feeBps: DEFAULT_FEE_BPS,
          roundFeeBps: DEFAULT_ROUND_FEE_BPS,
          roundDurationSeconds: ONE_DAY_IN_SECONDS,
          feeRecipient: fix.feeRecipient.publicKey,
        })
        .accounts({ user: fix.authority.publicKey, globalConfig: fix.globalConfig })
        .signers([fix.authority])
        .rpc();
    });

    it("fails with wrong signer (not protocol_authority)", async () => {
      const impostor = Keypair.generate();
      await airdropSOL(fix.provider.connection, impostor.publicKey);

      try {
        await fix.program.methods
          .updateGlobal({
            protocolAuthority: fix.authority.publicKey,
            endRoundAuthority: Keypair.generate().publicKey,
            migrationAuthority: Keypair.generate().publicKey,
            winnersPerRound: 1,
            feeBps: DEFAULT_FEE_BPS,
            roundFeeBps: DEFAULT_ROUND_FEE_BPS,
            roundDurationSeconds: ONE_DAY_IN_SECONDS,
            feeRecipient: Keypair.generate().publicKey,
          })
          .accounts({ user: impostor.publicKey, globalConfig: fix.globalConfig })
          .signers([impostor])
          .rpc();
        expect.fail("Expected constraint error");
      } catch (e: any) {
        const isConstraint = e instanceof anchor.AnchorError
          && e.error?.errorCode?.code === "ConstraintAddress";
        const hasLog = e.logs?.some((l: string) => l.includes("ConstraintAddress"));
        expect(isConstraint || hasLog).to.be.true;
      }
    });
  });
});
