import {PublicKey} from '@solana/web3.js';
import {findProgramAddress} from './test_utils';

export enum Side {
  Wrap,
  Unwrap,
}

export function findWrapOrderAddress(mint: PublicKey, user: PublicKey) {
  return findProgramAddress('order', [mint, user, [0x01]]).publicKey;
}

export function findUnwrapOrderAddress(mint: PublicKey, user: PublicKey) {
  return findProgramAddress('order', [mint, user, [0x02]]).publicKey;
}
