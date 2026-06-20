import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Ammverse } from "../target/types/ammverse";
import {
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { PublicKey, Keypair } from "@solana/web3.js";
import { assert } from "chai";

describe("ammverse", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Ammverse as Program<Ammverse>;

  const admin = provider.wallet as anchor.Wallet;
  let feeRecipient: Keypair;

  let configPdda: PublicKey;
  let configBump: number;

  let mintA: PublicKey;
  let mintB: PublicKey;
  let poolPda: PublicKey;
  let poolBump: number;
  let vaultA: PublicKey;
  let vaultB: PublicKey;
  let lpMint: PublicKey;

  let depositor: Keypair;
  let depositorTokenA: PublicKey;
  let depositorTokenB: PublicKey;
  let depositorLp: PublicKey;

  it("intializes the amm config", async () => {
    feeRecipient = Keypair.generate();

    [configPdda, configBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("amm-config")],
      program.programId,
    );

    await program.methods
      .initializeAmmConfig(30, 10)
      .accounts({
        admin: admin.publicKey,
        feeRecipient: feeRecipient.publicKey,
      })
      .rpc();

    const config = await program.account.ammConfig.fetch(configPdda);
    assert.equal(config.admin.toBase58(), admin.publicKey.toBase58());
    assert.equal(
      config.feeRecipient.toBase58(),
      feeRecipient.publicKey.toBase58(),
    );
    assert.equal(config.tradeFeeBps, 30);
    assert.equal(config.protocolFeeBps, 10);
    assert.equal(config.paused, false);
  });

  it("creates a pool", async () => {
    mintA = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      null,
      9,
    );
    mintB = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      null,
      9,
    );

    [poolPda, poolBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), mintA.toBuffer(), mintB.toBuffer()],
      program.programId,
    );

    const vaultAKeypair = Keypair.generate();
    const vaultBKeypair = Keypair.generate();
    const lpMintKeypair = Keypair.generate();

    await program.methods
      .createPool(30)
      .accounts({
        payer: admin.publicKey,
        config: configPdda,
        mintA,
        mintB,
        vaultA: vaultAKeypair.publicKey,
        vaultB: vaultBKeypair.publicKey,
        lpMint: lpMintKeypair.publicKey,
      })
      .signers([vaultAKeypair, vaultBKeypair, lpMintKeypair])
      .rpc();

    vaultA = vaultAKeypair.publicKey;
    vaultB = vaultBKeypair.publicKey;
    lpMint = lpMintKeypair.publicKey;

    const pool = await program.account.pool.fetch(poolPda);
    assert.equal(pool.mintA.toBase58(), mintA.toBase58());
    assert.equal(pool.mintB.toBase58(), mintB.toBase58());
    assert.equal(pool.reserveA.toNumber(), 0);
    assert.equal(pool.reserveB.toNumber(), 0);
    assert.equal(pool.tradeFeeBps, 30);
  });

  it("sets up the depositor", async () => {
    depositor = Keypair.generate();

    const sig = await provider.connection.requestAirdrop(
      depositor.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(sig);

    depositorTokenA = await createAccount(
      provider.connection,
      depositor,
      mintA,
      depositor.publicKey,
    );
    depositorTokenB = await createAccount(
      provider.connection,
      depositor,
      mintB,
      depositor.publicKey,
    );
    depositorLp = await createAccount(
      provider.connection,
      depositor,
      lpMint,
      depositor.publicKey,
    );

    await mintTo(
      provider.connection,
      admin.payer,
      mintA,
      depositorTokenA,
      admin.publicKey,
      1_000_000_000,
    );
    await mintTo(
      provider.connection,
      admin.payer,
      mintB,
      depositorTokenB,
      admin.publicKey,
      1_000_000_000,
    );

    const tokenAccount = await getAccount(provider.connection, depositorTokenA);
    assert.equal(tokenAccount.amount.toString(), "1000000000");
  });

  it("initializes pool liquidity", async () => {
    await program.methods
      .initializePoolLiquidity(
        new anchor.BN(100_000_000),
        new anchor.BN(200_000_000),
        new anchor.BN(1),
      )
      .accounts({
        depositor: depositor.publicKey,
        pool: poolPda,
        depositorTokenA,
        depositorTokenB,
        depositorLp,
      })
      .signers([depositor])
      .rpc();

    const pool = await program.account.pool.fetch(poolPda);
    assert.equal(pool.reserveA.toNumber(), 100_000_000);
    assert.equal(pool.reserveB.toNumber(), 200_000_000);

    const lpAccount = await getAccount(provider.connection, depositorLp);
    assert.isTrue(lpAccount.amount > BigInt(0));
  });

  it("adds liquidity proportionally", async () => {
    const poolBefore = await program.account.pool.fetch(poolPda);

    await program.methods
      .addLiquidity(
        new anchor.BN(50_000_000),
        new anchor.BN(100_000_000),
        new anchor.BN(1),
        new anchor.BN(1),
      )
      .accounts({
        depositor: depositor.publicKey,
        pool: poolPda,
        depositorTokenA,
        depositorTokenB,
        depositorLp,
      })
      .signers([depositor])
      .rpc();

    const poolAfter = await program.account.pool.fetch(poolPda);
    assert.equal(
      poolAfter.reserveA.toNumber(),
      poolBefore.reserveA.toNumber() + 50_000_000,
    );
    assert.equal(
      poolAfter.reserveB.toNumber(),
      poolBefore.reserveB.toNumber() + 100_000_000,
    );
  });

  it("swaps exact in (A to B)", async () => {
    const poolBefore = await program.account.pool.fetch(poolPda);
    const userOutputBefore = await getAccount(
      provider.connection,
      depositorTokenB,
    );

    await program.methods
      .swapExactIn(new anchor.BN(10_000_000), new anchor.BN(1))
      .accounts({
        user: depositor.publicKey,
        pool: poolPda,
        userInput: depositorTokenA,
        userOutput: depositorTokenB,
      })
      .signers([depositor])
      .rpc();

    const poolAfter = await program.account.pool.fetch(poolPda);
    const userOutputAfter = await getAccount(
      provider.connection,
      depositorTokenB,
    );

    assert.equal(
      poolAfter.reserveA.toNumber(),
      poolBefore.reserveA.toNumber() + 10_000_000,
    );
    assert.isTrue(
      poolAfter.reserveB.toNumber() < poolBefore.reserveB.toNumber(),
    );
    assert.isTrue(userOutputAfter.amount > userOutputBefore.amount);
  });

  it("removes liquidity", async () => {
    const lpAccountBefore = await getAccount(provider.connection, depositorLp);
    const poolBefore = await program.account.pool.fetch(poolPda);

    const lpToBurn = new anchor.BN(lpAccountBefore.amount.toString()).div(
      new anchor.BN(2),
    );

    await program.methods
      .removeLiquidity(lpToBurn, new anchor.BN(1), new anchor.BN(1))
      .accounts({
        depositor: depositor.publicKey,
        pool: poolPda,
        depositorTokenA,
        depositorTokenB,
        depositorLp,
      })
      .signers([depositor])
      .rpc();

    const poolAfter = await program.account.pool.fetch(poolPda);
    const lpAccountAfter = await getAccount(provider.connection, depositorLp);

    assert.isTrue(
      poolAfter.reserveA.toNumber() < poolBefore.reserveA.toNumber(),
    );
    assert.isTrue(
      poolAfter.reserveB.toNumber() < poolBefore.reserveB.toNumber(),
    );
    assert.equal(
      lpAccountAfter.amount.toString(),
      (
        BigInt(lpAccountBefore.amount.toString()) - BigInt(lpToBurn.toString())
      ).toString(),
    );
  });
});
