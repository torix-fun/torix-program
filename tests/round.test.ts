import { expect } from "chai";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import {
  getFixture, FixtureAccounts,
  deriveCurve,
  deserializeRoundState, deserializeRoundVault,
  launchCurve, buyExact, endRound, airdropSOL,
  ONE_DAY_IN_SECONDS, DEFAULT_FEE_BPS, DEFAULT_ROUND_FEE_BPS,
  INITIAL_VIRTUAL_SOL_RESERVES, INITIAL_VIRTUAL_TOKEN_RESERVES,
} from "./helpers";

const SHORT_ROUND_SECS = 0;

describe("round", () => {
  let fix: FixtureAccounts;

  before(async () => {
    fix = await getFixture();
    console.log("Round vault: ", fix.roundVault.toBase58());
  });

  async function ensureRound(durationSeconds: number = ONE_DAY_IN_SECONDS) {
    await fix.program.methods
      .updateGlobal({
        protocolAuthority: fix.authority.publicKey,
        endRoundAuthority: fix.endRoundAuthority.publicKey,
        migrationAuthority: fix.migrationAuthority.publicKey,
        winnersPerRound: 1,
        feeBps: DEFAULT_FEE_BPS,
        roundFeeBps: DEFAULT_ROUND_FEE_BPS,
        roundDurationSeconds: durationSeconds,
        feeRecipient: fix.feeRecipient.publicKey,
      })
      .accounts({ user: fix.authority.publicKey, globalConfig: fix.globalConfig })
      .signers([fix.authority])
      .rpc();

    await fix.program.methods
      .startRound()
      .accounts({
        user: fix.authority.publicKey,
        round: fix.round,
        roundVault: fix.roundVault,
        globalConfig: fix.globalConfig,
      })
      .signers([fix.authority])
      .rpc();
  }

  describe("startRound", () => {
    it("creates RoundState + RoundVault PDAs with end_timestamp ~ now+86400", async () => {
      await ensureRound(ONE_DAY_IN_SECONDS);
      const roundAcc = deserializeRoundState(
        (await fix.provider.connection.getAccountInfo(fix.round))!.data
      );
      const vaultAcc = deserializeRoundVault(
        (await fix.provider.connection.getAccountInfo(fix.roundVault))!.data
      );

      const now = Math.floor(Date.now() / 1000);
      expect(roundAcc.end_timestamp.toNumber()).to.be.closeTo(now + ONE_DAY_IN_SECONDS, 30);
      expect(roundAcc.vault.toString()).to.equal(fix.roundVault.toString());
      expect(roundAcc.bump).to.be.a("number");
      expect(vaultAcc.bump).to.be.a("number");
    });

    it("re-starts idempotently via init_if_needed", async () => {
      await ensureRound(ONE_DAY_IN_SECONDS);
      const ts1 = deserializeRoundState(
        (await fix.provider.connection.getAccountInfo(fix.round))!.data
      ).end_timestamp.toNumber();

      await new Promise(r => setTimeout(r, 2000));

      await fix.program.methods
        .startRound()
        .accounts({
          user: fix.authority.publicKey,
          round: fix.round,
          roundVault: fix.roundVault,
          globalConfig: fix.globalConfig,
        })
        .signers([fix.authority])
        .rpc();

      const ts2 = deserializeRoundState(
        (await fix.provider.connection.getAccountInfo(fix.round))!.data
      ).end_timestamp.toNumber();

      expect(ts2).to.be.at.least(ts1);
    });
  });

  describe("endRound", () => {
    async function createCurveWithVaultFunds() {
      const mint = Keypair.generate();
      await launchCurve(fix.program, fix.creator, mint);
      const [curve] = deriveCurve(mint.publicKey);

      const solIn = new anchor.BN(1_000_000_000);
      const totalFee = solIn.muln(DEFAULT_FEE_BPS).divn(10000);
      const netSol = solIn.sub(totalFee);
      const k = INITIAL_VIRTUAL_SOL_RESERVES.mul(INITIAL_VIRTUAL_TOKEN_RESERVES);
      const newVSol = INITIAL_VIRTUAL_SOL_RESERVES.add(netSol);
      const newVTok = k.div(newVSol);
      const tokensOut = INITIAL_VIRTUAL_TOKEN_RESERVES.sub(newVTok);
      const minTokens = tokensOut.sub(tokensOut.divn(100));

      await buyExact(fix.program, fix.buyer, curve, mint.publicKey, solIn, minTokens);
      return curve;
    }

    beforeEach(async () => {
      await ensureRound(ONE_DAY_IN_SECONDS);
    });

    it("happy path: winner receives SOL, vault closed, rent to fee_recipient", async () => {
      await ensureRound(SHORT_ROUND_SECS);

      const curve = await createCurveWithVaultFunds();

      const vaultBalance = await fix.provider.connection.getBalance(fix.roundVault);
      const winnerBalBefore = await fix.provider.connection.getBalance(fix.creator.publicKey);
      const feeRecipientBalBefore = await fix.provider.connection.getBalance(fix.feeRecipient.publicKey);
      const rewardAmount = Math.floor(vaultBalance * 0.5);

      await endRound(
        fix.program, fix.endRoundAuthority,
        [rewardAmount],
        [fix.creator.publicKey],
        [curve],
      );

      expect(await fix.provider.connection.getBalance(fix.creator.publicKey))
        .to.equal(winnerBalBefore + rewardAmount);
      expect(await fix.provider.connection.getAccountInfo(fix.roundVault)).to.be.null;
      expect(await fix.provider.connection.getAccountInfo(fix.round)).to.be.null;
      expect(await fix.provider.connection.getBalance(fix.feeRecipient.publicKey))
        .to.be.greaterThan(feeRecipientBalBefore);
    });

    it("rejects RoundNotOver when called before end_timestamp", async () => {
      const curve = await createCurveWithVaultFunds();

      try {
        await endRound(
          fix.program, fix.endRoundAuthority,
          [100_000],
          [fix.creator.publicKey],
          [curve],
        );
        expect.fail("Expected RoundNotOver error");
      } catch (e: any) {
        const code = e instanceof anchor.AnchorError
          ? e.error?.errorCode?.code
          : null;
        const hasLog = e.logs?.some(
          (l: string) => l.includes("RoundNotOver") || l.includes("6002")
        );
        expect(code === "RoundNotOver" || hasLog).to.be.true;
      }
    });

    it("rejects wrong signer (not end_round_authority)", async () => {
      await ensureRound(SHORT_ROUND_SECS);

      const curve = await createCurveWithVaultFunds();

      const impostor = Keypair.generate();
      await airdropSOL(fix.provider.connection, impostor.publicKey);

      try {
        await endRound(
          fix.program, impostor,
          [100_000],
          [fix.creator.publicKey],
          [curve],
        );
        expect.fail("Expected ConstraintAddress error");
      } catch (e: any) {
        const isConstraint = e instanceof anchor.AnchorError
          && e.error?.errorCode?.code === "ConstraintAddress";
        const hasLog = e.logs?.some((l: string) => l.includes("ConstraintAddress"));
        expect(isConstraint || hasLog).to.be.true;
      }
    });

    it("rejects amounts length mismatch vs winners_per_round", async () => {
      await ensureRound(SHORT_ROUND_SECS);

      const curve = await createCurveWithVaultFunds();

      try {
        await endRound(
          fix.program, fix.endRoundAuthority,
          [100_000, 200_000],
          [fix.creator.publicKey],
          [curve],
        );
        expect.fail("Expected mismatch error");
      } catch (e: any) {
        const logs = e.logs ? e.logs.join(" ") : "";
        expect(logs).to.satisfy((s: string) =>
          s.includes("Constraint") || s.includes("mismatch") || s.includes("Error")
        );
      }
    });

    it("rejects insufficient vault balance", async () => {
      await ensureRound(SHORT_ROUND_SECS);

      const curve = await createCurveWithVaultFunds();

      try {
        await endRound(
          fix.program, fix.endRoundAuthority,
          [1_000_000_000],
          [fix.creator.publicKey],
          [curve],
        );
        expect.fail("Expected insufficient balance error");
      } catch (e: any) {
        const code = e instanceof anchor.AnchorError
          ? e.error?.errorCode?.code
          : null;
        const logs = e.logs ? e.logs.join(" ") : "";
        expect(
          code === "insufficientBalance" ||
          code === "RequireGteViolated" ||
          logs.includes("insufficient") ||
          logs.includes("Error")
        ).to.be.true;
      }
    });
  });
});
