import * as anchor from "@coral-xyz/anchor";
import {
  Program, AnchorProvider, BN, BorshCoder, Wallet,
} from "@coral-xyz/anchor";
import {
  Connection, PublicKey, Keypair, LAMPORTS_PER_SOL,
  SystemProgram, Transaction,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import * as fs from "fs";
import * as path from "path";

// ─── Constants ───────────────────────────────────────────────

export const PROGRAM_ID = new PublicKey("torXFavtJnaJzW7fz2NVrg9f1j824GitYi69zhmJQBK");

export const FEE_DENOMINATOR = 10_000;
export const ONE_DAY_IN_SECONDS = 86_400;

export const ROUND_SEED = "round";
export const ROUND_VAULT_SEED = "round_vault";
export const CURVE_SEED = "curve";
export const GLOBAL_CONFIG_SEED = "global_config";
export const MINT_AUTHORITY_SEED = "mint_authority";

export const TOTAL_SUPPLY = new BN("1000000000000000");
export const INITIAL_VIRTUAL_SOL_RESERVES = new BN("20000000000");
export const INITIAL_VIRTUAL_TOKEN_RESERVES = new BN("1073000000000");
export const TOKEN_DECIMALS = 6;

export const DEFAULT_FEE_BPS = 300;
export const DEFAULT_ROUND_FEE_BPS = 100;
export const DEFAULT_WINNERS_PER_ROUND = 1;

// ─── Anchor.toml helpers ─────────────────────────────────────

export interface AnchorTomlConfig {
  clusterUrl: string;
  walletPath: string;
}

export function parseAnchorToml(): AnchorTomlConfig {
  const tomlPath = path.join(__dirname, "..", "Anchor.toml");
  const raw = fs.readFileSync(tomlPath, "utf-8");

  const providerMatch = raw.match(/\[provider\]([^[]+)/);
  if (!providerMatch) throw new Error("No [provider] section in Anchor.toml");

  const section = providerMatch[1];
  const clusterMatch = section.match(/cluster\s*=\s*"([^"]+)"/);
  const walletMatch = section.match(/wallet\s*=\s*"([^"]+)"/);

  if (!clusterMatch) throw new Error("No cluster in [provider]");

  const clusterName = clusterMatch[1];
  const urls: Record<string, string> = {
    localnet: "http://127.0.0.1:8899",
    devnet: "https://api.devnet.solana.com",
    "mainnet-beta": "https://api.mainnet-beta.solana.com",
    testnet: "https://api.testnet.solana.com",
  };

  return {
    clusterUrl: urls[clusterName] || `http://${clusterName}:8899`,
    walletPath: walletMatch?.[1] || "~/.config/solana/id.json",
  };
}

export function loadWalletKeypair(walletPath: string): Keypair {
  const expanded = walletPath.replace(/^~/, process.env.HOME || "");
  const secret = JSON.parse(fs.readFileSync(expanded, "utf-8"));
  return Keypair.fromSecretKey(new Uint8Array(secret));
}

export function createProvider(): AnchorProvider {
  const config = parseAnchorToml();
  const connection = new Connection(config.clusterUrl, "confirmed");
  const wallet = new Wallet(loadWalletKeypair(config.walletPath));
  return new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
}

export function loadProgram(provider?: AnchorProvider): Program {
  const p = provider || createProvider();
  anchor.setProvider(p);
  const idl = JSON.parse(
    fs.readFileSync(
      path.join(__dirname, "..", "target/idl/torix_program.json"),
      "utf-8"
    )
  );
  return new Program(idl, p);
}

// ─── PDA derivations ─────────────────────────────────────────

export function deriveGlobalConfig(): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(GLOBAL_CONFIG_SEED)],
    PROGRAM_ID
  )[0];
}

export function deriveRound(): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(ROUND_SEED)],
    PROGRAM_ID
  )[0];
}

export function deriveRoundVault(round: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(ROUND_VAULT_SEED), round.toBuffer()],
    PROGRAM_ID
  )[0];
}

export function deriveCurve(mint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(CURVE_SEED), mint.toBuffer()],
    PROGRAM_ID
  );
}

export function deriveMintAuthority(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(MINT_AUTHORITY_SEED)],
    PROGRAM_ID
  );
}

export function deriveCurveAta(curve: PublicKey, mint: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(
    mint, curve, true, TOKEN_2022_PROGRAM_ID
  );
}

export function deriveUserAta(user: PublicKey, mint: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(
    mint, user, false, TOKEN_2022_PROGRAM_ID
  );
}

// ─── Account deserializers ───────────────────────────────────

let _coder: BorshCoder | null = null;
function getCoder(): BorshCoder {
  if (!_coder) {
    const idl = JSON.parse(
      fs.readFileSync(
        path.join(__dirname, "..", "target/idl/torix_program.json"),
        "utf-8"
      )
    );
    _coder = new BorshCoder(idl);
  }
  return _coder;
}

export function deserializeGlobalConfig(data: Buffer): any {
  return getCoder().accounts.decode("GlobalConfig", data);
}

export function deserializeRoundState(data: Buffer): any {
  return getCoder().accounts.decode("RoundState", data);
}

export function deserializeRoundVault(data: Buffer): any {
  return getCoder().accounts.decode("RoundVault", data);
}

export function deserializeCurveState(data: Buffer): any {
  return getCoder().accounts.decode("CurveState", data);
}

// ─── Instruction helpers ─────────────────────────────────────

export async function initializeGlobal(
  program: Program,
  authority: Keypair,
  args: {
    protocolAuthority: PublicKey;
    endRoundAuthority: PublicKey;
    migrationAuthority: PublicKey;
    winnersPerRound: number;
    feeBps: number;
    roundFeeBps: number;
    feeRecipient: PublicKey;
  }
): Promise<void> {
  await program.methods
    .initializeGlobal(args)
    .accounts({ user: authority.publicKey })
    .signers([authority])
    .rpc();
}

export async function startRound(
  program: Program,
  user: Keypair,
  durationSeconds: number = ONE_DAY_IN_SECONDS,
): Promise<void> {
  const round = deriveRound();
  const roundVault = deriveRoundVault(round);
  await program.methods
    .startRound(new anchor.BN(durationSeconds))
    .accounts({ user: user.publicKey, round, roundVault })
    .signers([user])
    .rpc();
}

export async function launchCurve(
  program: Program,
  creator: Keypair,
  mint: Keypair,
): Promise<string> {
  const round = deriveRound();
  const globalConfig = deriveGlobalConfig();
  const [curve] = deriveCurve(mint.publicKey);
  const [mintAuthority] = deriveMintAuthority();
  const curveTokenAccount = deriveCurveAta(curve, mint.publicKey);

  return await program.methods
    .launch()
    .accounts({
      user: creator.publicKey,
      mint: mint.publicKey,
      curve,
      globalConfig,
      round,
      mintAuthority,
      curveTokenAccount,
    })
    .signers([creator, mint])
    .rpc();
}

export async function buyExact(
  program: Program,
  buyer: Keypair,
  curvePubkey: PublicKey,
  mint: PublicKey,
  solIn: BN,
  minTokensOut: BN,
): Promise<string> {
  const provider = program.provider as AnchorProvider;
  const round = deriveRound();
  const roundVault = deriveRoundVault(round);
  const globalConfig = deriveGlobalConfig();
  const curveAta = deriveCurveAta(curvePubkey, mint);
  const userAta = deriveUserAta(buyer.publicKey, mint);

  await getOrCreateAssociatedTokenAccount(
    provider.connection, buyer, mint, buyer.publicKey,
    false, undefined, undefined, TOKEN_2022_PROGRAM_ID,
  );

  const globalConfigAcc = deserializeGlobalConfig(
    (await provider.connection.getAccountInfo(globalConfig))!.data
  );

  return await program.methods
    .buyExact(solIn, minTokensOut)
    .accounts({
      user: buyer.publicKey,
      curve: curvePubkey,
      globalConfig,
      round,
      roundVault,
      feeRecipient: globalConfigAcc.fee_recipient,
      mint,
      curveTokenAccount: curveAta,
      userTokenAccount: userAta,
    })
    .signers([buyer])
    .rpc();
}

export async function sellExact(
  program: Program,
  seller: Keypair,
  curvePubkey: PublicKey,
  mint: PublicKey,
  tokensIn: BN,
  minSolOut: BN,
): Promise<string> {
  const provider = program.provider as AnchorProvider;
  const round = deriveRound();
  const roundVault = deriveRoundVault(round);
  const globalConfig = deriveGlobalConfig();
  const curveAta = deriveCurveAta(curvePubkey, mint);
  const userAta = deriveUserAta(seller.publicKey, mint);

  await getOrCreateAssociatedTokenAccount(
    provider.connection, seller, mint, seller.publicKey,
    false, undefined, undefined, TOKEN_2022_PROGRAM_ID,
  );

  const globalConfigAcc = deserializeGlobalConfig(
    (await provider.connection.getAccountInfo(globalConfig))!.data
  );

  return await program.methods
    .sellExact(tokensIn, minSolOut)
    .accounts({
      user: seller.publicKey,
      curve: curvePubkey,
      globalConfig,
      round,
      roundVault,
      feeRecipient: globalConfigAcc.fee_recipient,
      mint,
      curveTokenAccount: curveAta,
      userTokenAccount: userAta,
    })
    .signers([seller])
    .rpc();
}

export async function endRound(
  program: Program,
  user: Keypair,
  amounts: number[],
  winners: PublicKey[],
  curves: PublicKey[],
): Promise<void> {
  const round = deriveRound();
  const roundVault = deriveRoundVault(round);
  const globalConfig = deriveGlobalConfig();
  const globalConfigAcc = deserializeGlobalConfig(
    (await program.provider.connection.getAccountInfo(globalConfig))!.data
  );

  await program.methods
    .endRound(amounts.map(a => new BN(a)))
    .accounts({
      user: user.publicKey,
      round,
      roundVault,
      globalConfig,
      feeRecipient: globalConfigAcc.fee_recipient,
    })
    .remainingAccounts([
      ...winners.map(pk => ({ pubkey: pk, isSigner: false, isWritable: true })),
      ...curves.map(pk => ({ pubkey: pk, isSigner: false, isWritable: false })),
    ])
    .signers([user])
    .rpc();
}

// ─── Utility functions ───────────────────────────────────────

export async function airdropSOL(
  connection: Connection,
  address: PublicKey,
  amount: number = 10 * LAMPORTS_PER_SOL,
): Promise<void> {
  const sig = await connection.requestAirdrop(address, amount);
  await connection.confirmTransaction(sig);
}

export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// ─── Shared fixture ──────────────────────────────────────────

export interface FixtureAccounts {
  program: Program;
  provider: AnchorProvider;
  authority: Keypair;
  endRoundAuthority: Keypair;
  migrationAuthority: Keypair;
  feeRecipient: Keypair;
  creator: Keypair;
  buyer: Keypair;
  globalConfig: PublicKey;
  round: PublicKey;
  roundVault: PublicKey;
}

let _fixture: FixtureAccounts | null = null;

export async function getFixture(): Promise<FixtureAccounts> {
  if (_fixture) return _fixture;

  const provider = createProvider();
  anchor.setProvider(provider);
  const program = loadProgram(provider);

  const authority = Keypair.generate();
  const endRoundAuthority = Keypair.generate();
  const migrationAuthority = Keypair.generate();
  const feeRecipient = Keypair.generate();
  const creator = Keypair.generate();
  const buyer = Keypair.generate();

  const globalConfig = deriveGlobalConfig();
  const round = deriveRound();
  const roundVault = deriveRoundVault(round);

  for (const kp of [authority, endRoundAuthority, migrationAuthority, feeRecipient, creator, buyer]) {
    await airdropSOL(provider.connection, kp.publicKey);
  }

  await initializeGlobal(program, authority, {
    protocolAuthority: authority.publicKey,
    endRoundAuthority: endRoundAuthority.publicKey,
    migrationAuthority: migrationAuthority.publicKey,
    winnersPerRound: DEFAULT_WINNERS_PER_ROUND,
    feeBps: DEFAULT_FEE_BPS,
    roundFeeBps: DEFAULT_ROUND_FEE_BPS,
    feeRecipient: feeRecipient.publicKey,
  });

  await startRound(program, authority);

  _fixture = {
    program, provider, authority, endRoundAuthority,
    migrationAuthority, feeRecipient, creator, buyer,
    globalConfig, round, roundVault,
  };

  return _fixture;
}

export async function getFixtureWithCurve(): Promise<FixtureAccounts & { mint: Keypair; curve: PublicKey }> {
  const fix = await getFixture();
  const mint = Keypair.generate();
  await launchCurve(fix.program, fix.creator, mint);
  const [curve] = deriveCurve(mint.publicKey);
  return { ...fix, mint, curve };
}
