import {Keypair, PublicKey} from '@solana/web3.js';
import * as splToken from '@solana/spl-token';
import * as anchor from '@coral-xyz/anchor';

import {requestSolAirdrop} from './test_utils';

let mintOwner: Keypair;
let mint: PublicKey;

const provider = anchor.AnchorProvider.env();
const connection = provider.connection;
const payer = (provider.wallet as NodeWallet).payer;

export async function getTokenBalance(publicKey: PublicKey): Promise<number> {
  const account = await splToken.getAccount(connection, publicKey);
  return Number(account.amount);
}

async function createSplTokenMint(mintOwner: Keypair): Promise<PublicKey> {
  return splToken.createMint(
    connection,
    payer,
    mintOwner.publicKey,
    mintOwner.publicKey,
    9,
    undefined,
    undefined,
    splToken.TOKEN_PROGRAM_ID
  );
}

export async function getMintAndOwner(): Promise<[PublicKey, Keypair]> {
  if (mintOwner === undefined) {
    mintOwner = Keypair.generate();
    await requestSolAirdrop([mintOwner.publicKey]);
    mint = await createSplTokenMint(mintOwner);
  }
  return [mint, mintOwner];
}

export async function mintOwnerFreeze(
  tokenAccount: PublicKey,
  mint?: PublicKey,
  mintOwner?: Keypair
): Promise<void> {
  if (mint === undefined) {
    [mint, mintOwner] = await getMintAndOwner();
  }
  const freeze_tx = new anchor.web3.Transaction();
  freeze_tx.add(
    splToken.createFreezeAccountInstruction(
      tokenAccount,
      mint,
      mintOwner.publicKey
    )
  );
  const latestBlockHash = await connection.getLatestBlockhash('confirmed');
  freeze_tx.recentBlockhash = await latestBlockHash.blockhash;
  await anchor.web3.sendAndConfirmTransaction(connection, freeze_tx, [
    mintOwner,
  ]);
}

export async function createNewTokenAccount(
  mint: PublicKey,
  user: PublicKey
): Promise<PublicKey> {
  const userTokenAccount = await splToken.createAssociatedTokenAccount(
    connection,
    payer,
    mint,
    user
  );
  return userTokenAccount;
}

export async function fundTokenAccount(
  token_account: PublicKey,
  amount: number,
  mint?: PublicKey,
  mintAuthority?: Keypair
) {
  if (mint === undefined) {
    [mint, mintAuthority] = await getMintAndOwner();
  }
  await splToken.mintTo(
    connection,
    mintAuthority,
    mint,
    token_account,
    mintAuthority,
    amount
  );
}
