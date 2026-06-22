// Creates a fresh SPL token mint on devnet and mints an initial supply to a
// recipient wallet, so it can be used as the `mint` in launchpad's create_launch.
//
// Usage:
//   node scripts/create-launch-token.mjs <recipientPubkey> [decimals] [totalSupplyHuman]
//
// Defaults: decimals=6, totalSupplyHuman=1000000000 (1 billion tokens)
//
// Pays fees and acts as mint authority using ./wallet/id.json. Mints the full
// total supply to the recipient's associated token account (creating it if
// needed) so the recipient can satisfy create_launch's requirement of holding
// at least `real_token_reserves` of the mint.

import fs from "fs";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";

const RPC_ENDPOINT = process.env.RPC_ENDPOINT ?? "https://api.devnet.solana.com";

const [recipientArg, decimalsArg, totalSupplyArg] = process.argv.slice(2);
if (!recipientArg) {
  console.error("Usage: node scripts/create-launch-token.mjs <recipientPubkey> [decimals] [totalSupplyHuman]");
  process.exit(1);
}

const recipient = new PublicKey(recipientArg);
const decimals = decimalsArg ? Number(decimalsArg) : 6;
const totalSupplyHuman = totalSupplyArg ? BigInt(totalSupplyArg) : 1_000_000_000n;
const totalSupplyRaw = totalSupplyHuman * 10n ** BigInt(decimals);

const secret = JSON.parse(fs.readFileSync(new URL("../wallet/id.json", import.meta.url)));
const payer = Keypair.fromSecretKey(Uint8Array.from(secret));

const connection = new Connection(RPC_ENDPOINT, "confirmed");

console.log("Payer / mint authority:", payer.publicKey.toBase58());
console.log("Recipient:", recipient.toBase58());
console.log("Decimals:", decimals);
console.log("Total supply (human):", totalSupplyHuman.toString());
console.log("Total supply (raw):", totalSupplyRaw.toString());

const mint = await createMint(connection, payer, payer.publicKey, null, decimals);
console.log("\nMint created:", mint.toBase58());

const recipientAta = await getOrCreateAssociatedTokenAccount(connection, payer, mint, recipient);
console.log("Recipient ATA:", recipientAta.address.toBase58());

await mintTo(connection, payer, mint, recipientAta.address, payer, totalSupplyRaw);
console.log("Minted", totalSupplyRaw.toString(), "raw units to recipient.");

console.log("\nUse these in Create Launch:");
console.log("  Mint address:", mint.toBase58());
console.log("  Total supply (raw):", totalSupplyRaw.toString());
console.log("  Real token reserves (raw, 80% example):", ((totalSupplyRaw * 80n) / 100n).toString());
