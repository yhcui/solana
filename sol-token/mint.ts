/** Challenge: Mint an SPL Token
 *
 * In this challenge, you will create an SPL token!
 *
 * Goal:
 *   Mint an SPL token in a single transaction using Web3.js and the SPL Token library.
 *
 * Objectives:
 *   1. Create an SPL mint account.
 *   2. Initialize the mint with 6 decimals and your public key (feePayer) as the mint and freeze authorities.
 *   3. Create an associated token account for your public key (feePayer) to hold the minted tokens.
 *   4. Mint 21,000,000 tokens to your associated token account.
 *   5. Sign and send the transaction.
 */

import {
  Keypair,
  Connection,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";

import {
  createAssociatedTokenAccountInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  createMintToCheckedInstruction,
  MINT_SIZE,
  getMinimumBalanceForRentExemptMint,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";

import bs58 from "bs58";

//Create a connection to the RPC endpoint (fallback to local test-validator)
const rpcEndpoint = process.env.RPC_ENDPOINT ?? "http://127.0.0.1:8899";
const connection = new Connection(rpcEndpoint, "confirmed");

// Entry point of your TypeScript code (we will call this)
async function main() {
  try {

    // Prepare fee payer (from SECRET or generate + airdrop for local testing)
    let feePayer: Keypair;
    if (process.env.SECRET) {
      feePayer = Keypair.fromSecretKey(bs58.decode(process.env.SECRET as string));
    } else {
      feePayer = Keypair.generate();
      // If using local validator or devnet, airdrop some SOL for fees
      const sig = await connection.requestAirdrop(feePayer.publicKey, LAMPORTS_PER_SOL);
      await connection.confirmTransaction(sig, "confirmed");
    }

    // Create the mint using spl-token helper (abstracted)
    const mintPubkey = await createMint(
      connection,
      feePayer, // payer (Signer)
      feePayer.publicKey, // mint authority
      feePayer.publicKey, // freeze authority
      6 // decimals
    );

    // Get or create associated token account for the fee payer
    const ata = await getOrCreateAssociatedTokenAccount(
      connection,
      feePayer, // payer
      mintPubkey, // mint
      feePayer.publicKey // owner
    );

    // Mint 21,000,000 tokens (6 decimals)
    const mintAmount = BigInt(21_000_000) * BigInt(1_000_000);

    const mintToIx = createMintToCheckedInstruction(
      mintPubkey,
      ata.address,
      feePayer.publicKey,
      mintAmount,
      6
    );


    const recentBlockhash = await connection.getLatestBlockhash();

    const transaction = new Transaction({
      feePayer: feePayer.publicKey,
      blockhash: recentBlockhash.blockhash,
      lastValidBlockHeight: recentBlockhash.lastValidBlockHeight,
    }).add(mintToIx);

    const transactionSignature = await sendAndConfirmTransaction(connection, transaction, [feePayer]);

    console.log("Mint Address:", mintPubkey.toBase58());
    console.log("Transaction Signature:", transactionSignature);
  } catch (error) {
    console.error(`Oops, something went wrong: ${error}`);
  }
}

// Run the script
main().catch((err) => {
  console.error(err);
  process.exit(1);
});
