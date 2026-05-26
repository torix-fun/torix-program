import { expect } from "chai";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import {
  getFixture, FixtureAccounts,
  deriveCurve, deriveCurveAta, deriveUserAta,
  deserializeCurveState, deserializeGlobalConfig,
  launchCurve, buyExact, sellExact, airdropSOL,
  TOTAL_SUPPLY, INITIAL_VIRTUAL_SOL_RESERVES, INITIAL_VIRTUAL_TOKEN_RESERVES,
  DEFAULT_FEE_BPS, DEFAULT_ROUND_FEE_BPS,
} from "./helpers";

describe("curve", () => {
  let fix: FixtureAccounts;

  before(async () => {
    fix = await getFixture();
  });

  describe("launch", () => {
    it("creates mint, mints total supply, sets CurveState fields correctly", async () => {
      const mint = Keypair.generate();
      const [curve] = deriveCurve(mint.publicKey);
      const curveAta = deriveCurveAta(curve, mint.publicKey);

      await launchCurve(fix.program, fix.creator, mint);

      const state = deserializeCurveState(
        (await fix.provider.connection.getAccountInfo(curve))!.data
      );

      expect(state.status).to.deep.include({ Active: {} });
      expect(state.round.toString()).to.equal(fix.round.toString());
      expect(state.creator.toString()).to.equal(fix.creator.publicKey.toString());
      expect(state.mint.toString()).to.equal(mint.publicKey.toString());
      expect(state.real_reserves_sol.toNumber()).to.equal(0);
      expect(state.real_reserves_tokens.toString()).to.equal(TOTAL_SUPPLY.toString());
      expect(state.virtual_reserves_sol.toString()).to.equal(INITIAL_VIRTUAL_SOL_RESERVES.toString());
      expect(state.virtual_reserves_tokens.toString()).to.equal(INITIAL_VIRTUAL_TOKEN_RESERVES.toString());
      expect(state.stats.volume_sol.toNumber()).to.equal(0);
      expect(state.stats.buy_transactions.toNumber()).to.equal(0);
      expect(state.stats.sell_transactions.toNumber()).to.equal(0);

      const tokenBalance = await fix.provider.connection.getTokenAccountBalance(curveAta, "confirmed");
      expect(tokenBalance.value.uiAmountString).to.equal("1000000000");
    });

    it("rejects duplicate launch (same creator + mint)", async () => {
      const mint = Keypair.generate();
      await launchCurve(fix.program, fix.creator, mint);

      try {
        await launchCurve(fix.program, fix.creator, mint);
        expect.fail("Expected duplicate curve error");
      } catch (e: any) {
        const msg = (e.logs ? e.logs.join(" ") : e.message || e.toString() || "");
        expect(msg.toLowerCase()).to.include("already in use");
      }
    });
  });

  describe("buyExact", () => {
    async function setupBuy() {
      const mint = Keypair.generate();
      const [curve] = deriveCurve(mint.publicKey);
      await launchCurve(fix.program, fix.creator, mint);
      await airdropSOL(fix.provider.connection, fix.buyer.publicKey, 100 * LAMPORTS_PER_SOL);
      return { mint, curve };
    }

    it("updates reserves, distributes fees, increments stats, transfers tokens", async () => {
      const { mint, curve } = await setupBuy();

      const solIn = new anchor.BN(50_000_000);
      const k = INITIAL_VIRTUAL_SOL_RESERVES.mul(INITIAL_VIRTUAL_TOKEN_RESERVES);
      const totalFee = solIn.muln(DEFAULT_FEE_BPS).divn(10000);
      const roundFee = solIn.muln(DEFAULT_ROUND_FEE_BPS).divn(10000);
      const netSol = solIn.sub(totalFee);
      const newVSol = INITIAL_VIRTUAL_SOL_RESERVES.add(netSol);
      const newVTok = k.div(newVSol);
      const tokensOut = INITIAL_VIRTUAL_TOKEN_RESERVES.sub(newVTok);
      const protocolFee = totalFee.sub(roundFee);

      const curveStateBefore = deserializeCurveState(
        (await fix.provider.connection.getAccountInfo(curve))!.data
      );
      const userSolBefore = await fix.provider.connection.getBalance(fix.buyer.publicKey);
      const feeRecipientBefore = await fix.provider.connection.getBalance(fix.feeRecipient.publicKey);
      const vaultBefore = await fix.provider.connection.getBalance(fix.roundVault);

      await buyExact(fix.program, fix.buyer, curve, mint.publicKey, solIn, new anchor.BN(0));

      const userTokenAfter = await fix.provider.connection.getTokenAccountBalance(
        deriveUserAta(fix.buyer.publicKey, mint.publicKey), "confirmed"
      );
      expect(userTokenAfter.value.amount).to.equal(tokensOut.toString());

      const userSolAfter = await fix.provider.connection.getBalance(fix.buyer.publicKey);
      expect(userSolAfter).to.be.at.most(userSolBefore - solIn.toNumber() + totalFee.toNumber());

      const curveStateAfter = deserializeCurveState(
        (await fix.provider.connection.getAccountInfo(curve))!.data
      );
      expect(curveStateAfter.virtual_reserves_sol.toString()).to.equal(newVSol.toString());
      expect(curveStateAfter.virtual_reserves_tokens.toString()).to.equal(newVTok.toString());
      expect(curveStateAfter.stats.volume_sol.toString()).to.equal(solIn.toString());
      expect(curveStateAfter.stats.buy_transactions.toNumber()).to.equal(
        curveStateBefore.stats.buy_transactions.toNumber() + 1
      );

      expect(await fix.provider.connection.getBalance(fix.feeRecipient.publicKey))
        .to.be.at.least(feeRecipientBefore + protocolFee.toNumber());
      expect(await fix.provider.connection.getBalance(fix.roundVault))
        .to.be.at.least(vaultBefore + roundFee.toNumber());
    });

    it("rejects SlippageExceeded (min_tokens_out too high)", async () => {
      const { mint, curve } = await setupBuy();

      try {
        await buyExact(
          fix.program, fix.buyer, curve, mint.publicKey,
          new anchor.BN(50_000_000),
          new anchor.BN(1_000_000_000_000_000),
        );
        expect.fail("Expected SlippageExceeded");
      } catch (e: any) {
        const code = e instanceof anchor.AnchorError
          ? e.error?.errorCode?.code
          : null;
        const hasLog = e.logs?.some((l: string) => l.includes("SlippageExceeded") || l.includes("6000"));
        expect(code === "SlippageExceeded" || hasLog).to.be.true;
      }
    });

    it("rejects InvalidMint", async () => {
      const { mint, curve } = await setupBuy();
      const wrongMint = Keypair.generate();

      try {
        await fix.program.methods
          .buyExact(new anchor.BN(50_000_000), new anchor.BN(0))
          .accounts({
            user: fix.buyer.publicKey,
            curve,
            globalConfig: fix.globalConfig,
            round: fix.round,
            roundVault: fix.roundVault,
            feeRecipient: deserializeGlobalConfig(
              (await fix.provider.connection.getAccountInfo(fix.globalConfig))!.data
            ).fee_recipient,
            mint: wrongMint.publicKey,
            curveTokenAccount: deriveCurveAta(curve, wrongMint.publicKey),
            userTokenAccount: deriveUserAta(fix.buyer.publicKey, wrongMint.publicKey),
          })
          .signers([fix.buyer])
          .rpc();
        expect.fail("Expected InvalidMint");
      } catch (e: any) {
        const code = e instanceof anchor.AnchorError
          ? e.error?.errorCode?.code
          : null;
        const hasLog = e.logs?.some((l: string) => l.includes("InvalidMint") || l.includes("6003"));
        expect(code === "InvalidMint" || hasLog).to.be.true;
      }
    });

    it("rejects InvalidTokenAccount", async () => {
      const { mint, curve } = await setupBuy();

      try {
        await fix.program.methods
          .buyExact(new anchor.BN(50_000_000), new anchor.BN(0))
          .accounts({
            user: fix.buyer.publicKey,
            curve,
            globalConfig: fix.globalConfig,
            round: fix.round,
            roundVault: fix.roundVault,
            feeRecipient: deserializeGlobalConfig(
              (await fix.provider.connection.getAccountInfo(fix.globalConfig))!.data
            ).fee_recipient,
            mint: mint.publicKey,
            curveTokenAccount: Keypair.generate().publicKey,
            userTokenAccount: deriveUserAta(fix.buyer.publicKey, mint.publicKey),
          })
          .signers([fix.buyer])
          .rpc();
        expect.fail("Expected InvalidTokenAccount");
      } catch (e: any) {
        const code = e instanceof anchor.AnchorError
          ? e.error?.errorCode?.code
          : null;
        const hasLog = e.logs?.some((l: string) => l.includes("InvalidTokenAccount") || l.includes("6004"));
        expect(code === "InvalidTokenAccount" || hasLog).to.be.true;
      }
    });

    it("handles zero SOL input", async () => {
      const { mint, curve } = await setupBuy();

      const curveStateBefore = deserializeCurveState(
        (await fix.provider.connection.getAccountInfo(curve))!.data
      );

      await buyExact(fix.program, fix.buyer, curve, mint.publicKey, new anchor.BN(0), new anchor.BN(0));

      const curveStateAfter = deserializeCurveState(
        (await fix.provider.connection.getAccountInfo(curve))!.data
      );
      expect(curveStateAfter.virtual_reserves_sol.toString()).to.equal(curveStateBefore.virtual_reserves_sol.toString());
      expect(curveStateAfter.virtual_reserves_tokens.toString()).to.equal(curveStateBefore.virtual_reserves_tokens.toString());
    });
  });

  describe("sellExact", () => {
    async function setupSell() {
      const mint = Keypair.generate();
      const [curve] = deriveCurve(mint.publicKey);
      await launchCurve(fix.program, fix.creator, mint);
      await airdropSOL(fix.provider.connection, fix.buyer.publicKey, 100 * LAMPORTS_PER_SOL);
      await buyExact(fix.program, fix.buyer, curve, mint.publicKey, new anchor.BN(50_000_000), new anchor.BN(0));
      return { mint, curve };
    }

    it("returns SOL, updates reserves, distributes fees, increments stats", async () => {
      const { mint, curve } = await setupSell();

      const curveStateBefore = deserializeCurveState(
        (await fix.provider.connection.getAccountInfo(curve))!.data
      );
      const userTokenBefore = await fix.provider.connection.getTokenAccountBalance(
        deriveUserAta(fix.buyer.publicKey, mint.publicKey), "confirmed"
      );
      const userSolBefore = await fix.provider.connection.getBalance(fix.buyer.publicKey);
      const feeRecipientBefore = await fix.provider.connection.getBalance(fix.feeRecipient.publicKey);
      const vaultBefore = await fix.provider.connection.getBalance(fix.roundVault);

      const tokensIn = new anchor.BN(userTokenBefore.value.amount).divn(2);
      const vsol = new anchor.BN(curveStateBefore.virtual_reserves_sol);
      const vtok = new anchor.BN(curveStateBefore.virtual_reserves_tokens);
      const k = vsol.mul(vtok);
      const newVTok = vtok.add(tokensIn);
      const newVSol = k.div(newVTok);
      const grossSolOut = vsol.sub(newVSol);
      const totalFee = grossSolOut.muln(DEFAULT_FEE_BPS).divn(10000);
      const roundFee = grossSolOut.muln(DEFAULT_ROUND_FEE_BPS).divn(10000);
      const protocolFee = totalFee.sub(roundFee);
      const netSolOut = grossSolOut.sub(totalFee);
      const minSolOut = netSolOut.sub(netSolOut.divn(100));

      await sellExact(fix.program, fix.buyer, curve, mint.publicKey, tokensIn, minSolOut);

      const userSolAfter = await fix.provider.connection.getBalance(fix.buyer.publicKey);
      expect(userSolAfter - userSolBefore).to.equal(netSolOut.toNumber());

      const userTokenAfter = await fix.provider.connection.getTokenAccountBalance(
        deriveUserAta(fix.buyer.publicKey, mint.publicKey), "confirmed"
      );
      expect(new anchor.BN(userTokenBefore.value.amount).sub(new anchor.BN(userTokenAfter.value.amount)).toString())
        .to.equal(tokensIn.toString());

      const curveStateAfter = deserializeCurveState(
        (await fix.provider.connection.getAccountInfo(curve))!.data
      );
      expect(curveStateAfter.virtual_reserves_sol.toString()).to.equal(newVSol.toString());
      expect(curveStateAfter.virtual_reserves_tokens.toString()).to.equal(newVTok.toString());
      expect(curveStateAfter.stats.sell_transactions.toNumber()).to.equal(
        curveStateBefore.stats.sell_transactions.toNumber() + 1
      );

      expect(await fix.provider.connection.getBalance(fix.feeRecipient.publicKey))
        .to.be.at.least(feeRecipientBefore + protocolFee.toNumber());
      expect(await fix.provider.connection.getBalance(fix.roundVault))
        .to.be.at.least(vaultBefore + roundFee.toNumber());
    });

    it("rejects SlippageExceeded", async () => {
      const { mint, curve } = await setupSell();
      const userTokenBefore = await fix.provider.connection.getTokenAccountBalance(
        deriveUserAta(fix.buyer.publicKey, mint.publicKey), "confirmed"
      );

      try {
        await sellExact(
          fix.program, fix.buyer, curve, mint.publicKey,
          new anchor.BN(userTokenBefore.value.amount),
          new anchor.BN(1_000_000_000_000),
        );
        expect.fail("Expected SlippageExceeded");
      } catch (e: any) {
        const code = e instanceof anchor.AnchorError
          ? e.error?.errorCode?.code
          : null;
        const hasLog = e.logs?.some((l: string) => l.includes("SlippageExceeded") || l.includes("6000"));
        expect(code === "SlippageExceeded" || hasLog).to.be.true;
      }
    });

    it("rejects insufficient token balance", async () => {
      const { mint, curve } = await setupSell();
      const poorUser = Keypair.generate();
      await airdropSOL(fix.provider.connection, poorUser.publicKey);

      try {
        await sellExact(
          fix.program, poorUser, curve, mint.publicKey,
          new anchor.BN(1_000_000),
          new anchor.BN(0),
        );
        expect.fail("Expected transfer error");
      } catch (e: any) {
        const hasLog = e.logs?.some((l: string) =>
          l.includes("insufficient") || l.includes("Program") || l.includes("Error")
        );
        expect(hasLog).to.be.true;
      }
    });

    it("rejects InvalidMint", async () => {
      const { mint, curve } = await setupSell();
      const wrongMint = Keypair.generate();
      const userTokenBefore = await fix.provider.connection.getTokenAccountBalance(
        deriveUserAta(fix.buyer.publicKey, mint.publicKey), "confirmed"
      );

      try {
        await fix.program.methods
          .sellExact(new anchor.BN(userTokenBefore.value.amount), new anchor.BN(0))
          .accounts({
            user: fix.buyer.publicKey,
            curve,
            globalConfig: fix.globalConfig,
            round: fix.round,
            roundVault: fix.roundVault,
            feeRecipient: deserializeGlobalConfig(
              (await fix.provider.connection.getAccountInfo(fix.globalConfig))!.data
            ).fee_recipient,
            mint: wrongMint.publicKey,
            curveTokenAccount: deriveCurveAta(curve, wrongMint.publicKey),
            userTokenAccount: deriveUserAta(fix.buyer.publicKey, wrongMint.publicKey),
          })
          .signers([fix.buyer])
          .rpc();
        expect.fail("Expected InvalidMint");
      } catch (e: any) {
        const code = e instanceof anchor.AnchorError
          ? e.error?.errorCode?.code
          : null;
        const hasLog = e.logs?.some((l: string) => l.includes("InvalidMint") || l.includes("6003"));
        expect(code === "InvalidMint" || hasLog).to.be.true;
      }
    });

    it("rejects InvalidTokenAccount", async () => {
      const { mint, curve } = await setupSell();
      const userTokenBefore = await fix.provider.connection.getTokenAccountBalance(
        deriveUserAta(fix.buyer.publicKey, mint.publicKey), "confirmed"
      );

      try {
        await fix.program.methods
          .sellExact(new anchor.BN(userTokenBefore.value.amount), new anchor.BN(0))
          .accounts({
            user: fix.buyer.publicKey,
            curve,
            globalConfig: fix.globalConfig,
            round: fix.round,
            roundVault: fix.roundVault,
            feeRecipient: deserializeGlobalConfig(
              (await fix.provider.connection.getAccountInfo(fix.globalConfig))!.data
            ).fee_recipient,
            mint: mint.publicKey,
            curveTokenAccount: deriveCurveAta(curve, mint.publicKey),
            userTokenAccount: Keypair.generate().publicKey,
          })
          .signers([fix.buyer])
          .rpc();
        expect.fail("Expected InvalidTokenAccount");
      } catch (e: any) {
        const code = e instanceof anchor.AnchorError
          ? e.error?.errorCode?.code
          : null;
        const hasLog = e.logs?.some((l: string) => l.includes("InvalidTokenAccount") || l.includes("6004"));
        expect(code === "InvalidTokenAccount" || hasLog).to.be.true;
      }
    });
  });
});
