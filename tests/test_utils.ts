import * as anchor from '@coral-xyz/anchor';
import {LAMPORTS_PER_SOL, PublicKey} from '@solana/web3.js';
import {Program} from '@coral-xyz/anchor';
import {SecureWrapToken} from '../target/types/secure_wrap_token';

import {expect} from 'chai';
const chai = require('chai');
chai.use(require('chai-string'));

const provider = anchor.AnchorProvider.env();
const connection = provider.connection;
const program = anchor.workspace.SecureWrapToken as Program<SecureWrapToken>;

export async function assertRaisesError(
  testSnippet: () => Promise<void>,
  endsWith: string
): Promise<void> {
  try {
    await testSnippet();
    throw new Error('error expected');
  } catch (error) {
    expect(error.message).to.endWith(endsWith);
  }
}

export async function assertRaisesErrorContains(
  testSnippet: () => Promise<void>,
  contains: string
): Promise<void> {
  try {
    await testSnippet();
    throw new Error('error expected');
  } catch (error) {
    expect(error.message).to.containIgnoreSpaces(contains);
  }
}

export async function requestSolAirdrop(addresses: PublicKey[]) {
  await Promise.all(
    addresses.map(address => {
      return connection.requestAirdrop(address, 1 * LAMPORTS_PER_SOL);
    })
  );
}

export function findProgramAddress(label: string, extraSeeds?: []) {
  const seeds = [Buffer.from(anchor.utils.bytes.utf8.encode(label))];
  if (extraSeeds) {
    for (const extraSeed of extraSeeds) {
      if (typeof extraSeed === 'string') {
        seeds.push(Buffer.from(anchor.utils.bytes.utf8.encode(extraSeed)));
      } else if (Array.isArray(extraSeed)) {
        seeds.push(Buffer.from(extraSeed));
      } else {
        seeds.push(extraSeed.toBuffer());
      }
    }
  }
  const res = PublicKey.findProgramAddressSync(seeds, program.programId);
  return {publicKey: res[0], bump: res[1]};
}
