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

  ASSOCIATED_TOKEN_PROGRAM_ID
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

    // Generate a new keypair for the mint account
    const mint = Keypair.generate();

    const mintRent = await getMinimumBalanceForRentExemptMint(connection);

    // START HERE

    // Create the mint account
    const createAccountIx = SystemProgram.createAccount({
      fromPubkey: feePayer.publicKey,
      newAccountPubkey: mint.publicKey,
      space: MINT_SIZE,
      lamports: mintRent,
      programId: TOKEN_PROGRAM_ID,
    });

    // Initialize the mint account
    // Set decimals to 6, and the mint and freeze authorities to the fee payer (you).
    const initializeMintIx = createInitializeMint2Instruction(
      mint.publicKey,
      6, // decimals
      feePayer.publicKey, // mint authority
      feePayer.publicKey, // freeze authority
      TOKEN_PROGRAM_ID
    );

    // Create the associated token account (ATA) for the fee payer
    const associatedTokenAddress = getAssociatedTokenAddressSync(
      mint.publicKey,
      feePayer.publicKey
    );

    const createAssociatedTokenAccountIx = createAssociatedTokenAccountInstruction(
      feePayer.publicKey, // payer
      associatedTokenAddress, // associated token account
      feePayer.publicKey, // owner
      mint.publicKey // mint
    );

    // Mint 21,000,000 tokens to the associated token account
    // decimals = 6 -> multiply by 10^6
    const mintAmount = BigInt(21_000_000) * BigInt(1_000_000);

    const mintToCheckedIx = createMintToCheckedInstruction(
      mint.publicKey, // mint
      associatedTokenAddress, // destination
      feePayer.publicKey, // authority
      mintAmount, // amount (as bigint)
      6 // decimals
    );


    const recentBlockhash = await connection.getLatestBlockhash();

    const transaction = new Transaction({
      feePayer: feePayer.publicKey,
      blockhash: recentBlockhash.blockhash,
      lastValidBlockHeight: recentBlockhash.lastValidBlockHeight
    }).add(
        createAccountIx,
        initializeMintIx,
        createAssociatedTokenAccountIx,
        mintToCheckedIx
    );

    const transactionSignature = await sendAndConfirmTransaction(
      connection,
      transaction,
      [feePayer, mint] // feePayer must sign to pay fees and authority; mint must sign to create account
    );

    console.log("Mint Address:", mint.publicKey.toBase58());
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
