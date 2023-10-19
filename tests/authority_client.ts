import {PublicKey, Keypair} from '@solana/web3.js';
import * as anchor from '@coral-xyz/anchor';
import {Program} from '@coral-xyz/anchor';
import {SecureWrapToken} from '../target/types/secure_wrap_token';
import {getTokenBalance} from './mint_utils';
import {findProgramAddress} from './test_utils';

import {
  Side,
  findWrapOrderAddress,
  findUnwrapOrderAddress,
} from './order_utils';

export class AuthorityClient {
  public keypair: Keypair;
  public mint: PublicKey;

  program: Program<SecureWrapToken>;
  secureWrapTokenProgramData: PublicKey;

  constructor() {
    this.keypair = (anchor.AnchorProvider.env().wallet as NodeWallet).payer;
    this.program = anchor.workspace.SecureWrapToken as Program<SecureWrapToken>;
    this.secureWrapTokenProgramData = PublicKey.findProgramAddressSync(
      [this.program.programId.toBuffer()],
      new PublicKey('BPFLoaderUpgradeab1e11111111111111111111111')
    )[0];
  }

  async initialize() {
    await this.program.methods
      .initialize()
      .accounts({
        authority: this.keypair.publicKey,
        feePayer: this.keypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        secureWrapTokenProgram: this.program.programId,
        secureWrapTokenProgramData: this.secureWrapTokenProgramData,
      })
      .signers([this.keypair])
      .rpc();
  }

  setDefaultMint(mint: PublicKey) {
    this.mint = mint;
  }

  async createSecureWrappedToken(mint: PublicKey) {
    await this.program.methods
      .createSecureWrappedToken()
      .accounts({
        authority: this.keypair.publicKey,
        feePayer: this.keypair.publicKey,
        originalTokenMint: mint,
      })
      .signers([this.keypair])
      .rpc();
  }

  async getProgramSwtTokenBalance(mint?: PublicKey): Promise<number> {
    if (mint === undefined) {
      mint = this.mint;
    }
    const programWrappedTokenAccount = findProgramAddress(
      'program_wrapped_token_account',
      [mint]
    ).publicKey;
    return await getTokenBalance(programWrappedTokenAccount);
  }

  async getProgramOriginalTokenBalance(mint?: PublicKey): Promise<number> {
    if (mint === undefined) {
      mint = this.mint;
    }
    const programWrappedTokenAccount = findProgramAddress(
      'program_original_token_account',
      [mint]
    ).publicKey;
    return await getTokenBalance(programWrappedTokenAccount);
  }

  async setUnwrapDelay(unwrapDelaySeconds: number) {
    await this.program.methods
      .setUnwrapDelay(new anchor.BN(unwrapDelaySeconds))
      .accounts({
        authority: this.keypair.publicKey,
      })
      .signers([this.keypair])
      .rpc();
  }

  async freeze({accountToFreeze, freezePeriodSeconds}, mint?: PublicKey) {
    if (mint === undefined) {
      mint = this.mint;
    }
    await this.program.methods
      .freeze(new anchor.BN(freezePeriodSeconds))
      .accounts({
        authority: this.keypair.publicKey,
        feePayer: this.keypair.publicKey,
        originalTokenMint: mint,
        accountToFreeze: accountToFreeze,
      })
      .signers([this.keypair])
      .rpc();
  }

  async permanentFreeze(
    {accountToFreeze, originalTokenAccount},
    mint?: PublicKey
  ) {
    if (mint === undefined) {
      mint = this.mint;
    }
    await this.program.methods
      .permanentFreeze()
      .accounts({
        authority: this.keypair.publicKey,
        feePayer: this.keypair.publicKey,
        originalTokenMint: mint,
        accountToFreeze: accountToFreeze,
        originalTokenAccount: originalTokenAccount,
      })
      .signers([this.keypair])
      .rpc();
  }

  async distributeFrozenFunds(
    {permanentlyFrozenAccount, receivingAccount, amount},
    mint?: PublicKey
  ) {
    if (mint === undefined) {
      mint = this.mint;
    }
    await this.program.methods
      .distributeFrozenFunds(new anchor.BN(amount))
      .accounts({
        authority: this.keypair.publicKey,
        feePayer: this.keypair.publicKey,
        originalTokenMint: mint,
        permanentlyFrozenAccount: permanentlyFrozenAccount,
        receivingAccount: receivingAccount,
      })
      .signers([this.keypair])
      .rpc();
  }

  async haltOrders() {
    await this.program.methods
      .haltOrders()
      .accounts({
        authority: this.keypair.publicKey,
      })
      .signers([this.keypair])
      .rpc();
  }

  async resumeOrders() {
    await this.program.methods
      .resumeOrders()
      .accounts({
        authority: this.keypair.publicKey,
      })
      .signers([this.keypair])
      .rpc();
  }

  async haltWrapOrders() {
    await this.program.methods
      .haltWrapOrders()
      .accounts({
        authority: this.keypair.publicKey,
      })
      .signers([this.keypair])
      .rpc();
  }

  async resumeWrapOrders() {
    await this.program.methods
      .resumeWrapOrders()
      .accounts({
        authority: this.keypair.publicKey,
      })
      .signers([this.keypair])
      .rpc();
  }

  async haltUnwrap() {
    await this.program.methods
      .haltUnwrap()
      .accounts({
        authority: this.keypair.publicKey,
      })
      .signers([this.keypair])
      .rpc();
  }

  async resumeUnwrap() {
    await this.program.methods
      .resumeUnwrap()
      .accounts({
        authority: this.keypair.publicKey,
      })
      .signers([this.keypair])
      .rpc();
  }

  async immediateThaw(accountToThaw: PublicKey, mint?: PublicKey) {
    if (mint === undefined) {
      mint = this.mint;
    }
    await this.program.methods
      .immediateThaw()
      .accounts({
        authority: this.keypair.publicKey,
        feePayer: this.keypair.publicKey,
        originalTokenMint: mint,
        accountToThaw: accountToThaw,
      })
      .signers([this.keypair])
      .rpc();
  }

  async fillOrderProgram({orderOwner, tokenAccountToCredit}, mint?: PublicKey) {
    if (mint === undefined) {
      mint = this.mint;
    }
    const order = findUnwrapOrderAddress(mint, orderOwner.publicKey);
    await this.program.methods
      .fillOrderProgram({side: {unwrap: {}}})
      .accounts({
        authority: this.keypair.publicKey,
        originalTokenMint: mint,
        orderSecureWrapTokenAccount: orderOwner.swtAccount,
        orderTokenAccount: orderOwner.tokenAccount,
        tokenAccountToCredit: tokenAccountToCredit,
        orderOwner: orderOwner.publicKey,
        order: order,
      })
      .signers([this.keypair])
      .rpc();
  }

  public async cancelUnwrapProgram({user}, mint?: PublicKey): Promise<void> {
    if (mint === undefined) {
      mint = this.mint;
    }
    await this.program.methods
      .cancelUnwrapProgram()
      .accounts({
        authority: this.keypair.publicKey,
        originalTokenMint: mint,
        permanentlyFrozenUser: user.publicKey,
        secureWrapTokenAccount: user.swtAccount,
      })
      .signers([this.keypair])
      .rpc();
  }

  public async cancelOrderProgram(
    {side, user},
    mint?: PublicKey
  ): Promise<void> {
    if (mint === undefined) {
      mint = this.mint;
    }
    let sideParam, orderAddress;
    if (side === Side.Wrap) {
      sideParam = {wrap: {}};
      orderAddress = findWrapOrderAddress(mint, user.publicKey);
    } else {
      sideParam = {unwrap: {}};
      orderAddress = findUnwrapOrderAddress(mint, user.publicKey);
    }
    await this.program.methods
      .cancelOrderProgram({side: sideParam})
      .accounts({
        authority: this.keypair.publicKey,
        originalTokenMint: mint,
        permanentlyFrozenUser: user.publicKey,
        secureWrapTokenAccount: user.swtAccount,
        order: orderAddress,
      })
      .signers([this.keypair])
      .rpc();
  }
}
