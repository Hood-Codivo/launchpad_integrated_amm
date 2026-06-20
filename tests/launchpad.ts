import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Launchpad } from "../target/types/launchpad";
import { Ammverse } from "../target/types/ammverse";

import {
  createMint,
  createAccount,
  mintTo,
  getAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { PublicKey, Keypair } from "@solana/web3.js";
import { assert } from "chai";

describe("launchpad", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let ammConfigPda: PublicKey;
  let ammPoolPda: PublicKey;
  let ammVaultAKeypair: Keypair;
  let ammVaultBKeypair: Keypair;
  let ammLpMintKeypair: Keypair;
  let lpDestination: PublicKey;

  const program = anchor.workspace.Launchpad as Program<Launchpad>;
  const ammProgram = anchor.workspace.Ammverse as Program<Ammverse>;

  const admin = provider.wallet as anchor.Wallet;
  let feeRecipient: Keypair;

  let globalConfigPda: PublicKey;

  let mint: PublicKey;
  let quoteMint: PublicKey;
  let curvePda: PublicKey;
  let tokenVault: PublicKey;
  let quoteVault: PublicKey;

  let creator: Keypair;
  let creatorTokenAccount: PublicKey;

  let buyer: Keypair;
  let buyerQuoteAccount: PublicKey;
  let buyerTokenAccount: PublicKey;

  let feeRecipientQuoteAccount: PublicKey;

  it("initializes the global config", async () => {
    feeRecipient = Keypair.generate();

    [globalConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("global-config")],
      program.programId,
    );

    await program.methods
      .initializeGlobalConfig(100, 50, new anchor.BN(1))
      .accounts({
        admin: admin.publicKey,
        feeRecipient: feeRecipient.publicKey,
      })
      .rpc();

    const config = await program.account.globalConfig.fetch(globalConfigPda);
    assert.equal(config.admin.toBase58(), admin.publicKey.toBase58());
    assert.equal(config.platformFeeBps, 100);
    assert.equal(config.migrationFeeBps, 50);
    assert.equal(config.migrationMarketCap.toString(), "1");
    assert.equal(config.paused, false);
  });

  it("creates a launch", async () => {
    creator = Keypair.generate();

    const sig = await provider.connection.requestAirdrop(
      creator.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(sig);

    mint = await createMint(
      provider.connection,
      admin.payer,
      creator.publicKey,
      null,
      9,
    );
    quoteMint = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      null,
      9,
    );

    creatorTokenAccount = await createAccount(
      provider.connection,
      creator,
      mint,
      creator.publicKey,
    );
    await mintTo(
      provider.connection,
      admin.payer,
      mint,
      creatorTokenAccount,
      creator,
      800_000_000,
    );

    [curvePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("curve"), mint.toBuffer()],
      program.programId,
    );

    const tokenVaultKeypair = Keypair.generate();
    const quoteVaultKeypair = Keypair.generate();

    await program.methods
      .createLaunch(
        new anchor.BN(800_000_000),
        new anchor.BN(200_000_000),
        new anchor.BN(25_000),
        new anchor.BN(1_000_000_000),
      )
      .accounts({
        creator: creator.publicKey,
        config: globalConfigPda,
        mint,
        quoteMint,
        creatorTokenAccount,
        tokenVault: tokenVaultKeypair.publicKey,
        quoteVault: quoteVaultKeypair.publicKey,
      })
      .signers([creator, tokenVaultKeypair, quoteVaultKeypair])
      .rpc();

    tokenVault = tokenVaultKeypair.publicKey;
    quoteVault = quoteVaultKeypair.publicKey;

    const curve = await program.account.bondingCurve.fetch(curvePda);
    assert.equal(curve.mint.toBase58(), mint.toBase58());
    assert.equal(curve.realTokenReserves.toNumber(), 800_000_000);
    assert.equal(curve.realQuoteReserves.toNumber(), 0);
    assert.equal(curve.migrated, false);
  });

  it("sets up the buyer", async () => {
    buyer = Keypair.generate();

    const sig = await provider.connection.requestAirdrop(
      buyer.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(sig);

    buyerQuoteAccount = await createAccount(
      provider.connection,
      buyer,
      quoteMint,
      buyer.publicKey,
    );
    buyerTokenAccount = await createAccount(
      provider.connection,
      buyer,
      mint,
      buyer.publicKey,
    );
    feeRecipientQuoteAccount = await createAccount(
      provider.connection,
      admin.payer,
      quoteMint,
      feeRecipient.publicKey,
    );

    await mintTo(
      provider.connection,
      admin.payer,
      quoteMint,
      buyerQuoteAccount,
      admin.payer,
      100_000_000,
    );

    const account = await getAccount(provider.connection, buyerQuoteAccount);
    assert.equal(account.amount.toString(), "100000000");
  });

  it("buys tokens off the curve", async () => {
    const curveBefore = await program.account.bondingCurve.fetch(curvePda);

    await program.methods
      .buy(new anchor.BN(1_000), new anchor.BN(1))
      .accounts({
        buyer: buyer.publicKey,
        config: globalConfigPda,
        curve: curvePda,
        buyerQuoteAccount,
        buyerTokenAccount,
        feeRecipientQuoteAccount,
      })
      .signers([buyer])
      .rpc();

    const curveAfter = await program.account.bondingCurve.fetch(curvePda);
    const buyerToken = await getAccount(provider.connection, buyerTokenAccount);

    assert.isTrue(
      curveAfter.realTokenReserves.toNumber() <
        curveBefore.realTokenReserves.toNumber(),
    );
    assert.isTrue(
      curveAfter.realQuoteReserves.toNumber() >
        curveBefore.realQuoteReserves.toNumber(),
    );
    assert.isTrue(buyerToken.amount > BigInt(0));
  });

  it("sells tokens back to the curve", async () => {
    const curveBefore = await program.account.bondingCurve.fetch(curvePda);
    const buyerTokenBefore = await getAccount(
      provider.connection,
      buyerTokenAccount,
    );

    const tokensToSell = new anchor.BN(buyerTokenBefore.amount.toString()).div(
      new anchor.BN(2),
    );

    await program.methods
      .sell(tokensToSell, new anchor.BN(1))
      .accounts({
        seller: buyer.publicKey,
        config: globalConfigPda,
        curve: curvePda,
        sellerTokenAccount: buyerTokenAccount,
        sellerQuoteAccount: buyerQuoteAccount,
        feeRecipientQuoteAccount,
      })
      .signers([buyer])
      .rpc();

    const curveAfter = await program.account.bondingCurve.fetch(curvePda);
    const buyerTokenAfter = await getAccount(
      provider.connection,
      buyerTokenAccount,
    );

    assert.isTrue(
      curveAfter.realTokenReserves.toNumber() >
        curveBefore.realTokenReserves.toNumber(),
    );
    assert.isTrue(
      curveAfter.realQuoteReserves.toNumber() <
        curveBefore.realQuoteReserves.toNumber(),
    );
    assert.equal(
      buyerTokenAfter.amount.toString(),
      (
        BigInt(buyerTokenBefore.amount.toString()) -
        BigInt(tokensToSell.toString())
      ).toString(),
    );
  });

  it("migrates the curve to the amm (creates pool)", async () => {
    [ammConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("amm-config")],
      ammProgram.programId,
    );

    [ammPoolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), mint.toBuffer(), quoteMint.toBuffer()],
      ammProgram.programId,
    );

    ammVaultAKeypair = Keypair.generate();
    ammVaultBKeypair = Keypair.generate();
    ammLpMintKeypair = Keypair.generate();

    await program.methods
      .migrateToAmm(30)
      .accounts({
        migrationPayer: admin.publicKey,
        config: globalConfigPda,
        curve: curvePda,
        mint,
        quoteMint,
        ammConfig: ammConfigPda,
        ammPool: ammPoolPda,
        ammVaultA: ammVaultAKeypair.publicKey,
        ammVaultB: ammVaultBKeypair.publicKey,
        ammLpMint: ammLpMintKeypair.publicKey,
        ammProgram: ammProgram.programId,
      })
      .signers([ammVaultAKeypair, ammVaultBKeypair, ammLpMintKeypair])
      .rpc();

    const curve = await program.account.bondingCurve.fetch(curvePda);
    assert.equal(curve.migrating, true);
    assert.equal(curve.migrated, false);

    const pool = await ammProgram.account.pool.fetch(ammPoolPda);
    assert.equal(pool.mintA.toBase58(), mint.toBase58());
    assert.equal(pool.mintB.toBase58(), quoteMint.toBase58());
  });

  it("finalizes the migration (deposits liquidity)", async () => {
    const curveBefore = await program.account.bondingCurve.fetch(curvePda);
    const lpDestinationKeypair = Keypair.generate();

    await program.methods
      .finalizeMigration()
      .accounts({
        migrationPayer: admin.publicKey,
        curve: curvePda,
        ammPool: ammPoolPda,
        ammVaultA: ammVaultAKeypair.publicKey,
        ammVaultB: ammVaultBKeypair.publicKey,
        ammLpMint: ammLpMintKeypair.publicKey,
        lpDestination: lpDestinationKeypair.publicKey,
        ammProgram: ammProgram.programId,
      })
      .signers([lpDestinationKeypair])
      .rpc();

    lpDestination = lpDestinationKeypair.publicKey;

    const curve = await program.account.bondingCurve.fetch(curvePda);
    assert.equal(curve.migrated, true);
    assert.equal(curve.migrating, false);
    assert.equal(curve.realTokenReserves.toNumber(), 0);
    assert.equal(curve.realQuoteReserves.toNumber(), 0);

    const pool = await ammProgram.account.pool.fetch(ammPoolPda);
    assert.equal(
      pool.reserveA.toNumber(),
      curveBefore.realTokenReserves.toNumber(),
    );
    assert.equal(
      pool.reserveB.toNumber(),
      curveBefore.realQuoteReserves.toNumber(),
    );

    // LP tokens must be locked under the curve PDA, not handed to whoever
    // called this instruction.
    const lpAccount = await getAccount(provider.connection, lpDestination);
    assert.equal(lpAccount.owner.toBase58(), curvePda.toBase58());
    assert.isTrue(lpAccount.amount > BigInt(0));
  });

  it("aborts an in-progress migration and resumes trading", async () => {
    const abortCreator = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      abortCreator.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(sig);

    const abortMint = await createMint(
      provider.connection,
      admin.payer,
      abortCreator.publicKey,
      null,
      9,
    );
    const abortQuoteMint = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      null,
      9,
    );

    const abortCreatorTokenAccount = await createAccount(
      provider.connection,
      abortCreator,
      abortMint,
      abortCreator.publicKey,
    );
    await mintTo(
      provider.connection,
      admin.payer,
      abortMint,
      abortCreatorTokenAccount,
      abortCreator,
      800_000_000,
    );

    const [abortCurvePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("curve"), abortMint.toBuffer()],
      program.programId,
    );

    const abortTokenVaultKeypair = Keypair.generate();
    const abortQuoteVaultKeypair = Keypair.generate();

    await program.methods
      .createLaunch(
        new anchor.BN(800_000_000),
        new anchor.BN(200_000_000),
        new anchor.BN(25_000),
        new anchor.BN(1_000_000_000),
      )
      .accounts({
        creator: abortCreator.publicKey,
        config: globalConfigPda,
        mint: abortMint,
        quoteMint: abortQuoteMint,
        creatorTokenAccount: abortCreatorTokenAccount,
        tokenVault: abortTokenVaultKeypair.publicKey,
        quoteVault: abortQuoteVaultKeypair.publicKey,
      })
      .signers([abortCreator, abortTokenVaultKeypair, abortQuoteVaultKeypair])
      .rpc();

    // Trade a tiny amount so the curve clears the (deliberately low, test-only)
    // migration market cap threshold. Use a separate buyer wallet so its
    // token accounts don't collide with the creator's.
    const abortBuyer = Keypair.generate();
    const abortBuyerSig = await provider.connection.requestAirdrop(
      abortBuyer.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(abortBuyerSig);

    const abortBuyerQuoteAccount = await createAccount(
      provider.connection,
      abortBuyer,
      abortQuoteMint,
      abortBuyer.publicKey,
    );
    await mintTo(
      provider.connection,
      admin.payer,
      abortQuoteMint,
      abortBuyerQuoteAccount,
      admin.payer,
      10_000_000,
    );
    const abortBuyerTokenAccount = await createAccount(
      provider.connection,
      abortBuyer,
      abortMint,
      abortBuyer.publicKey,
    );
    const abortFeeRecipientQuoteAccount = await createAccount(
      provider.connection,
      admin.payer,
      abortQuoteMint,
      feeRecipient.publicKey,
    );

    await program.methods
      .buy(new anchor.BN(1_000), new anchor.BN(1))
      .accounts({
        buyer: abortBuyer.publicKey,
        config: globalConfigPda,
        curve: abortCurvePda,
        buyerQuoteAccount: abortBuyerQuoteAccount,
        buyerTokenAccount: abortBuyerTokenAccount,
        feeRecipientQuoteAccount: abortFeeRecipientQuoteAccount,
      })
      .signers([abortBuyer])
      .rpc();

    const [abortAmmPoolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), abortMint.toBuffer(), abortQuoteMint.toBuffer()],
      ammProgram.programId,
    );
    const abortAmmVaultAKeypair = Keypair.generate();
    const abortAmmVaultBKeypair = Keypair.generate();
    const abortAmmLpMintKeypair = Keypair.generate();

    await program.methods
      .migrateToAmm(30)
      .accounts({
        migrationPayer: admin.publicKey,
        config: globalConfigPda,
        curve: abortCurvePda,
        mint: abortMint,
        quoteMint: abortQuoteMint,
        ammConfig: ammConfigPda,
        ammPool: abortAmmPoolPda,
        ammVaultA: abortAmmVaultAKeypair.publicKey,
        ammVaultB: abortAmmVaultBKeypair.publicKey,
        ammLpMint: abortAmmLpMintKeypair.publicKey,
        ammProgram: ammProgram.programId,
      })
      .signers([
        abortAmmVaultAKeypair,
        abortAmmVaultBKeypair,
        abortAmmLpMintKeypair,
      ])
      .rpc();

    let curve = await program.account.bondingCurve.fetch(abortCurvePda);
    assert.equal(curve.migrating, true);

    await program.methods
      .abortMigration()
      .accounts({
        admin: admin.publicKey,
        config: globalConfigPda,
        curve: abortCurvePda,
      })
      .rpc();

    curve = await program.account.bondingCurve.fetch(abortCurvePda);
    assert.equal(curve.migrating, false);
    assert.equal(curve.migrated, false);
    assert.equal(curve.paused, false);

    // Trading should work again now that the abort cleared `migrating`.
    await program.methods
      .buy(new anchor.BN(1_000), new anchor.BN(1))
      .accounts({
        buyer: abortBuyer.publicKey,
        config: globalConfigPda,
        curve: abortCurvePda,
        buyerQuoteAccount: abortBuyerQuoteAccount,
        buyerTokenAccount: abortBuyerTokenAccount,
        feeRecipientQuoteAccount: abortFeeRecipientQuoteAccount,
      })
      .signers([abortBuyer])
      .rpc();
  });
});
