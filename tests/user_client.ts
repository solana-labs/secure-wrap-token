import {PublicKey, Keypair} from '@solana/web3.js';
import * as splToken from '@solana/spl-token';
import * as anchor from '@coral-xyz/anchor';
import {Program} from '@coral-xyz/anchor';
import {SecureWrapToken} from '../target/types/secure_wrap_token';

import {
  Side,
  findWrapOrderAddress,
  findUnwrapOrderAddress,
} from './order_utils';

import {requestSolAirdrop, findProgramAddress} from './test_utils';

import {
  getMintAndOwner,
  createNewTokenAccount,
  fundTokenAccount,
  getTokenBalance,
} from './mint_utils';

const provider = anchor.AnchorProvider.env();
const connection = provider.connection;

export class UserClient {
  public keypair: Keypair;

  program: Program<SecureWrapToken>;
  mint: PublicKey;
  public publicKey: PublicKey;
  public tokenAccount: PublicKey;
  public swtAccount: PublicKey;

  private constructor() {
    this.keypair = Keypair.generate();
    this.publicKey = this.keypair.publicKey;
    this.program = anchor.workspace.SecureWrapToken as Program<SecureWrapToken>;
  }

  static async createTestUser(
    originalTokenAmount: number,
    mint?: PublicKey,
    mintAuthority?: PublicKey
  ): Promise<UserClient> {
    // If mint is not provided, then default to the global mint from mint_utils.
    if (mint === undefined) {
      [mint, mintAuthority] = await getMintAndOwner();
    }
    const userClient = new UserClient();
    await requestSolAirdrop([userClient.publicKey]);
    await userClient.initialize(mint, mintAuthority, originalTokenAmount);

    return userClient;
  }

  private async initialize(
    mint: PublicKey,
    mintAuthority: Keypair,
    originalTokenAmount: number
  ) {
    this.mint = mint;
    this.tokenAccount = await createNewTokenAccount(this.mint, this.publicKey);
    await fundTokenAccount(
      this.tokenAccount,
      originalTokenAmount,
      mint,
      mintAuthority
    );
    this.swtAccount = await createNewTokenAccount(
      findProgramAddress('secure_wrap_token_mint', [this.mint]).publicKey,
      this.publicKey
    );
  }

  public async getOriginalTokenBalance(): Promise<number> {
    return await getTokenBalance(this.tokenAccount);
  }

  public async getSwtTokenBalance(): Promise<number> {
    return await getTokenBalance(this.swtAccount);
  }

  public async wrap(amount: number): Promise<void> {
    await this.program.methods
      .wrap(new anchor.BN(amount))
      .accounts({
        owner: this.publicKey,
        originalTokenMint: this.mint,
        tokenAccount: this.tokenAccount,
        secureWrapTokenAccount: this.swtAccount,
      })
      .signers([this.keypair])
      .rpc();
  }

  public async unwrap(amount: number): Promise<void> {
    await this.program.methods
      .unwrap(new anchor.BN(amount))
      .accounts({
        owner: this.publicKey,
        originalTokenMint: this.mint,
        tokenAccount: this.tokenAccount,
        secureWrapTokenAccount: this.swtAccount,
      })
      .signers([this.keypair])
      .rpc();
  }

  public async releaseUnwrap(): Promise<void> {
    await this.program.methods
      .releaseUnwrap()
      .accounts({
        owner: this.publicKey,
        originalTokenMint: this.mint,
        tokenAccount: this.tokenAccount,
        secureWrapTokenAccount: this.swtAccount,
      })
      .signers([this.keypair])
      .rpc();
  }

  public async cancelUnwrap(): Promise<void> {
    await this.program.methods
      .cancelUnwrap()
      .accounts({
        owner: this.publicKey,
        originalTokenMint: this.mint,
        secureWrapTokenAccount: this.swtAccount,
      })
      .signers([this.keypair])
      .rpc();
  }

  public async thaw(): Promise<void> {
    await this.program.methods
      .thaw()
      .accounts({
        signer: this.publicKey,
        originalTokenMint: this.mint,
        accountToThaw: this.swtAccount,
      })
      .signers([this.keypair])
      .rpc();
  }

  public async sendSwtTokens({receivingAccount, amount}): Promise<void> {
    const tx = new anchor.web3.Transaction();
    tx.add(
      splToken.createTransferInstruction(
        this.swtAccount,
        receivingAccount,
        this.publicKey,
        amount
      )
    );
    const latestBlockHash = await connection.getLatestBlockhash('confirmed');
    tx.recentBlockhash = await latestBlockHash.blockhash;
    await anchor.web3.sendAndConfirmTransaction(connection, tx, [this.keypair]);
  }

  public async placeOrder({side, amountIn, amountOut}): Promise<void> {
    let sideParam, orderAddress;
    if (side === Side.Wrap) {
      sideParam = {wrap: {}};
      orderAddress = findWrapOrderAddress(this.mint, this.publicKey);
    } else {
      sideParam = {unwrap: {}};
      orderAddress = findUnwrapOrderAddress(this.mint, this.publicKey);
    }
    await this.program.methods
      .placeOrder({
        side: sideParam,
        amountIn: new anchor.BN(amountIn),
        amountOut: new anchor.BN(amountOut),
      })
      .accounts({
        owner: this.publicKey,
        originalTokenMint: this.mint,
        tokenAccount: this.tokenAccount,
        secureWrapTokenAccount: this.swtAccount,
        order: orderAddress,
      })
      .signers([this.keypair])
      .rpc();
  }

  public async cancelOrder({side}): Promise<void> {
    let sideParam, orderAddress;
    if (side === Side.Wrap) {
      sideParam = {wrap: {}};
      orderAddress = findWrapOrderAddress(this.mint, this.publicKey);
    } else {
      sideParam = {unwrap: {}};
      orderAddress = findUnwrapOrderAddress(this.mint, this.publicKey);
    }
    await this.program.methods
      .cancelOrder({side: sideParam})
      .accounts({
        owner: this.publicKey,
        originalTokenMint: this.mint,
        tokenAccount: this.tokenAccount,
        secureWrapTokenAccount: this.swtAccount,
        order: orderAddress,
      })
      .signers([this.keypair])
      .rpc();
  }

  public async fillOrder({side, orderOwner}): Promise<void> {
    let sideParam, orderAddress;
    if (side === Side.Wrap) {
      sideParam = {wrap: {}};
      orderAddress = findWrapOrderAddress(this.mint, orderOwner.publicKey);
    } else {
      sideParam = {unwrap: {}};
      orderAddress = findUnwrapOrderAddress(this.mint, orderOwner.publicKey);
    }
    await this.program.methods
      .fillOrder({side: sideParam})
      .accounts({
        owner: this.publicKey,
        originalTokenMint: this.mint,
        tokenAccount: this.tokenAccount,
        secureWrapTokenAccount: this.swtAccount,
        orderOwner: orderOwner.publicKey,
        orderSecureWrapTokenAccount: orderOwner.swtAccount,
        orderTokenAccount: orderOwner.tokenAccount,
        order: orderAddress,
      })
      .signers([this.keypair])
      .rpc();
  }
}
