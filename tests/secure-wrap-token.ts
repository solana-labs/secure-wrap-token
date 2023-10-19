// To run this test suite, please use the included './test_script.sh'.
// Plain 'anchor test' does not work because it deploys the program without upgrade authority.
import * as anchor from '@coral-xyz/anchor';
import {expect} from 'chai';
import {getMintAndOwner, mintOwnerFreeze} from './mint_utils';
const chai = require('chai');
chai.use(require('chai-string'));

import * as utils from './test_utils';
import {AuthorityClient} from './authority_client';
import {UserClient} from './user_client';
import {Side} from './order_utils';

describe('secure-wrap-token', () => {
  const authorityClient = new AuthorityClient();

  before(async () => {
    // Fetch the default mint to use for testing, and initialize the SWT program.
    await authorityClient.initialize();
  });

  it('Create secure wrapped token', async () => {
    const [defaultMint] = await getMintAndOwner();
    await authorityClient.createSecureWrappedToken(defaultMint);
    authorityClient.setDefaultMint(defaultMint);
    // For the purposes of testing, set unwrap delay time to 0 seconds for immediate release.
    await authorityClient.setUnwrapDelay(0);
  });

  it('Expect duplicate create_secure_wrapped_token on the same mint to fail.', async () => {
    const [defaultMint] = await getMintAndOwner();
    await utils.assertRaisesError(async () => {
      await authorityClient.createSecureWrappedToken(defaultMint);
    }, 'Error processing Instruction 0: custom program error: 0x0');
  });

  it('basic token wrapping.', async () => {
    const programInitialTokenBalance =
      await authorityClient.getProgramOriginalTokenBalance();

    const user = await UserClient.createTestUser(1000);
    await user.wrap(100);

    // Check the user and program account balances after user wraps 100 tokens.
    expect(await authorityClient.getProgramOriginalTokenBalance()).to.equal(
      programInitialTokenBalance + 100
    );
    expect(await user.getOriginalTokenBalance()).to.equal(900);
    expect(await user.getSwtTokenBalance()).to.equal(100);

    // User sends 50 wrapped tokens to user2.
    const user2 = await UserClient.createTestUser(0);
    await user.sendSwtTokens({receivingAccount: user2.swtAccount, amount: 50});
    expect(await user.getSwtTokenBalance()).to.equal(50);
    expect(await user2.getSwtTokenBalance()).to.equal(50);

    // User2 now unwraps their SWT tokens.
    await user2.unwrap(50);
    // Before release, user2's SWT balance is lowered but original token balance is unchanged.
    expect(await user2.getSwtTokenBalance()).to.equal(0);
    expect(await user2.getOriginalTokenBalance()).to.equal(0);

    await user2.releaseUnwrap();

    expect(await user2.getSwtTokenBalance()).to.equal(0);
    expect(await user2.getOriginalTokenBalance()).to.equal(50);
  });

  it('Cancel unwrap', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(1000);

    await user.unwrap(1000);

    expect(await user.getOriginalTokenBalance()).to.equal(0);
    expect(await user.getSwtTokenBalance()).to.equal(0);

    await user.cancelUnwrap();

    expect(await user.getOriginalTokenBalance()).to.equal(0);
    expect(await user.getSwtTokenBalance()).to.equal(1000);
  });

  it('Unwrap fails when account has been frozen', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(1000);

    // Initiate unwrap and then authority freezes.
    await user.unwrap(1000);
    await authorityClient.freeze({
      accountToFreeze: user.swtAccount,
      freezePeriodSeconds: 1_000_000,
    });

    await utils.assertRaisesError(async () => {
      await user.releaseUnwrap();
    }, 'Funds cannot be unwrapped from a frozen account.');
  });

  it('Test unwrap delay set too long', async () => {
    await utils.assertRaisesErrorContains(async () => {
      await authorityClient.setUnwrapDelay(100_000);
    }, 'Maximum unwrap delay is 24 hours.');
    await utils.assertRaisesError(async () => {
      await authorityClient.setUnwrapDelay(
        new anchor.BN(2).pow(new anchor.BN(63))
      );
    }, 'Overflow in arithmetic operation.');
  });

  it('Unwrap and release error checks', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(1000);

    // set unwrap delay back to 1 day.
    await authorityClient.setUnwrapDelay(86400);

    await user.unwrap(1000);

    await utils.assertRaisesErrorContains(async () => {
      await user.releaseUnwrap(1000);
    }, 'Error Message: PendingUnwrap release requested too early.');
  });

  it('Unwrap halted', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(1000);

    const pendingUser = await UserClient.createTestUser(1000);
    await pendingUser.wrap(1000);
    await pendingUser.unwrap(1000);

    // After halting unwraps, it's impossible to: initiate an unwrap, and cancel/release an existing.
    await authorityClient.haltUnwrap();

    await utils.assertRaisesError(async () => {
      await user.unwrap(1000);
    }, 'Unwrap is currently halted.');

    await utils.assertRaisesError(async () => {
      await pendingUser.cancelUnwrap();
    }, 'Unwrap is currently halted.');

    await utils.assertRaisesError(async () => {
      await pendingUser.releaseUnwrap();
    }, 'Unwrap is currently halted.');

    // Resume unwrapping for other test cases.
    await authorityClient.resumeUnwrap();
  });

  it('Frozen account cannot transfer funds', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(1000);

    const user2 = await UserClient.createTestUser(1000);

    await authorityClient.freeze({
      accountToFreeze: user.swtAccount,
      freezePeriodSeconds: 2,
    });
    const freezeTimeout = new Promise(resolve => setTimeout(resolve, 2500));

    // Validate frozen user cannot send funds.
    await utils.assertRaisesError(async () => {
      await user.sendSwtTokens({
        receivingAccount: user2.swtAccount,
        amount: 500,
      });
    }, 'Error processing Instruction 0: custom program error: 0x11');

    // After waiting for freeze_period to pass, user thaws themselves and tries again.
    await freezeTimeout;
    await user.thaw();
    await user.sendSwtTokens({receivingAccount: user2.swtAccount, amount: 500});

    expect(await user2.getSwtTokenBalance()).to.equal(500);
  });

  it('Authority-only immediate thaw', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(1000);
    const user2 = await UserClient.createTestUser(1000);

    await authorityClient.freeze({
      accountToFreeze: user.swtAccount,
      freezePeriodSeconds: 1000,
    });
    await authorityClient.immediateThaw(user.swtAccount);

    // Verify after immediate thaw, the test user is able to transfer tokens.
    await user.sendSwtTokens({receivingAccount: user2.swtAccount, amount: 500});
    expect(await user2.getSwtTokenBalance()).to.equal(500);
  });

  it('Check Freeze/Thaw error conditions', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(500);

    // Test freeze period too long.
    await utils.assertRaisesErrorContains(async () => {
      await authorityClient.freeze({
        accountToFreeze: user.swtAccount,
        freezePeriodSeconds: 1_300_000,
      });
    }, 'Maximum freeze period is 14 days.');

    // Test thaw on an account that hasn't been frozen before.
    await utils.assertRaisesError(async () => {
      await user.thaw();
    }, 'The program expected this account to be already initialized.');

    // Test premature thaw.
    await authorityClient.freeze({
      accountToFreeze: user.swtAccount,
      freezePeriodSeconds: 2,
    });
    const freezeTimeout = new Promise(resolve => setTimeout(resolve, 2500));
    await utils.assertRaisesErrorContains(async () => {
      await user.thaw();
    }, 'Error Message: Thaw requested too early.');

    // Test account cannot be frozen again after freeze.
    await utils.assertRaisesErrorContains(async () => {
      await authorityClient.freeze({
        accountToFreeze: user.swtAccount,
        freezePeriodSeconds: 1000,
      });
    }, 'Account was recently frozen and is not yet eligible to freeze again');

    // Test account cannot be immediately frozen again after thaw.
    await freezeTimeout;
    await user.thaw();
    await utils.assertRaisesErrorContains(async () => {
      await authorityClient.freeze({
        accountToFreeze: user.swtAccount,
        freezePeriodSeconds: 1000,
      });
    }, 'Account was recently frozen and is not yet eligible to freeze again');
  });

  it('basic discount auction unwrap + wrap orders.', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(500);

    await user.placeOrder({side: Side.Unwrap, amountIn: 500, amountOut: 400});
    expect(await user.getSwtTokenBalance()).to.equal(0);

    const filler = await UserClient.createTestUser(1000);

    await filler.fillOrder({side: Side.Unwrap, orderOwner: user});

    // Check balances after filling the unwrap.
    expect(await user.getOriginalTokenBalance()).to.equal(900);
    expect(await user.getSwtTokenBalance()).to.equal(0);
    expect(await filler.getOriginalTokenBalance()).to.equal(600);
    expect(await filler.getSwtTokenBalance()).to.equal(500);

    await user.placeOrder({side: Side.Wrap, amountIn: 400, amountOut: 500});
    expect(await user.getOriginalTokenBalance()).to.equal(500);

    await filler.fillOrder({side: Side.Wrap, orderOwner: user});

    // Check balances after filling the wrap.
    expect(await user.getOriginalTokenBalance()).to.equal(500);
    expect(await user.getSwtTokenBalance()).to.equal(500);
    expect(await filler.getOriginalTokenBalance()).to.equal(1000);
    expect(await filler.getSwtTokenBalance()).to.equal(0);
  });

  it('discount auction cancel order.', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(500);
    const programInitialTokenBalance =
      await authorityClient.getProgramOriginalTokenBalance();
    const programInitialSwtBalance =
      await authorityClient.getProgramSwtTokenBalance();

    expect(await user.getOriginalTokenBalance()).to.equal(500);
    expect(await user.getSwtTokenBalance()).to.equal(500);

    await user.placeOrder({side: Side.Unwrap, amountIn: 500, amountOut: 100});

    expect(await user.getOriginalTokenBalance()).to.equal(500);
    expect(await user.getSwtTokenBalance()).to.equal(0);
    expect(await authorityClient.getProgramOriginalTokenBalance()).to.equal(
      programInitialTokenBalance
    );
    expect(await authorityClient.getProgramSwtTokenBalance()).to.equal(
      programInitialSwtBalance + 500
    );

    await user.placeOrder({side: Side.Wrap, amountIn: 500, amountOut: 1000});

    expect(await user.getOriginalTokenBalance()).to.equal(0);
    expect(await user.getSwtTokenBalance()).to.equal(0);
    expect(await authorityClient.getProgramOriginalTokenBalance()).to.equal(
      programInitialTokenBalance + 500
    );
    expect(await authorityClient.getProgramSwtTokenBalance()).to.equal(
      programInitialSwtBalance + 500
    );

    await user.cancelOrder({side: Side.Unwrap});
    await user.cancelOrder({side: Side.Wrap});

    expect(await user.getOriginalTokenBalance()).to.equal(500);
    expect(await user.getSwtTokenBalance()).to.equal(500);
    expect(await authorityClient.getProgramOriginalTokenBalance()).to.equal(
      programInitialTokenBalance
    );
    expect(await authorityClient.getProgramSwtTokenBalance()).to.equal(
      programInitialSwtBalance
    );
  });

  it('discount auction user can only have one outstanding order per side at any time.', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(500);

    await user.placeOrder({side: Side.Unwrap, amountIn: 10, amountOut: 5});
    // subsequent Unwrap order is expected to fail.
    await utils.assertRaisesError(async () => {
      await user.placeOrder({side: Side.Unwrap, amountIn: 10, amountOut: 5});
    }, 'custom program error: 0x0');

    // User can open another order on the wrap side.
    await user.placeOrder({side: Side.Wrap, amountIn: 5, amountOut: 10});
    // Similarly, subsequent Wrap order is expected to fail.
    await utils.assertRaisesError(async () => {
      await user.placeOrder({side: Side.Wrap, amountIn: 5, amountOut: 10});
    }, 'custom program error: 0x0');

    // Canceling the Unwrap order allows a new one to be placed.
    await user.cancelOrder({side: Side.Unwrap});
    await user.cancelOrder({side: Side.Wrap});
    await user.placeOrder({side: Side.Unwrap, amountIn: 20, amountOut: 10});
    await user.placeOrder({side: Side.Wrap, amountIn: 10, amountOut: 20});
    await user.cancelOrder({side: Side.Unwrap});
    await user.cancelOrder({side: Side.Wrap});
  });

  it('discount auction FillOrderProgram', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(1000);

    // Check the starting amount of SWT tokens escrowed by the program.
    const programInitialSwtBalance =
      await authorityClient.getProgramSwtTokenBalance();

    await user.placeOrder({side: Side.Unwrap, amountIn: 1000, amountOut: 990});

    expect(await authorityClient.getProgramSwtTokenBalance()).to.equal(
      programInitialSwtBalance + 1000
    );

    const creditor = await UserClient.createTestUser(0);

    // Authority fills the order
    await authorityClient.fillOrderProgram({
      orderOwner: user,
      tokenAccountToCredit: creditor.swtAccount,
    });
    // Check balances after filling order. Program escrow is back to original, user token balances change.
    expect(await authorityClient.getProgramSwtTokenBalance()).to.equal(
      programInitialSwtBalance
    );
    expect(await user.getOriginalTokenBalance()).to.equal(990);
    expect(await user.getSwtTokenBalance()).to.equal(0);
    expect(await creditor.getOriginalTokenBalance()).to.equal(0);
    expect(await creditor.getSwtTokenBalance()).to.equal(10);
  });

  it('discount auction error cases', async () => {
    const user = await UserClient.createTestUser(10000);
    await user.wrap(5000);

    await utils.assertRaisesErrorContains(async () => {
      await user.placeOrder({side: Side.Unwrap, amountIn: 500, amountOut: 501});
    }, 'Unwrap must have amount_in > amount_out. Wrap must have amount_out > amount_in.');

    await utils.assertRaisesErrorContains(async () => {
      await user.placeOrder({side: Side.Wrap, amountIn: 500, amountOut: 499});
    }, 'Unwrap must have amount_in > amount_out. Wrap must have amount_out > amount_in.');
  });

  it('permanent freeze and fund distribution', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(1000);

    const receiver = await UserClient.createTestUser(0);

    await authorityClient.freeze({
      accountToFreeze: user.swtAccount,
      freezePeriodSeconds: 2,
    });

    // Premature permanent freeze fails if original token authority has not frozen user's account yet.
    await utils.assertRaisesError(async () => {
      await authorityClient.permanentFreeze({
        accountToFreeze: user.swtAccount,
        originalTokenAccount: user.tokenAccount,
      });
    }, 'Permanent Freeze can only execute when the SWT *and* original token accounts are already frozen.');

    await mintOwnerFreeze(user.tokenAccount);

    // Now SWT permanent freeze works.
    await authorityClient.permanentFreeze({
      accountToFreeze: user.swtAccount,
      originalTokenAccount: user.tokenAccount,
    });

    await utils.assertRaisesError(async () => {
      await user.thaw();
    }, 'Permanently frozen account cannot be thawed.');

    // Distribute the test user's funds to user B.
    expect(await receiver.getSwtTokenBalance()).to.equal(0);
    await authorityClient.distributeFrozenFunds({
      permanentlyFrozenAccount: user.swtAccount,
      receivingAccount: receiver.swtAccount,
      amount: 500,
    });
    expect(await receiver.getSwtTokenBalance()).to.equal(500);

    await utils.assertRaisesErrorContains(async () => {
      await authorityClient.distributeFrozenFunds({
        permanentlyFrozenAccount: user.swtAccount,
        receivingAccount: receiver.swtAccount,
        amount: 501,
      });
    }, 'Error Message: Cannot distribute funds beyond what is in the permanently frozen account.');

    await authorityClient.distributeFrozenFunds({
      permanentlyFrozenAccount: user.swtAccount,
      receivingAccount: receiver.swtAccount,
      amount: 500,
    });
    expect(await receiver.getSwtTokenBalance()).to.equal(1000);
  });

  it('basic discount orders halted.', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(1000);

    await user.placeOrder({side: Side.Unwrap, amountIn: 1000, amountOut: 999});

    await authorityClient.haltOrders();

    const filler = await UserClient.createTestUser(1000);

    // Expect fill_order to fail due to halted orders.
    await utils.assertRaisesError(async () => {
      await filler.fillOrder({side: Side.Unwrap, orderOwner: user});
    }, 'Orders are currently halted.');

    await authorityClient.resumeOrders();

    // Now expect the order to be fillable.
    await filler.fillOrder({side: Side.Unwrap, orderOwner: user});
    expect(await filler.getOriginalTokenBalance()).to.equal(1);
    expect(await filler.getSwtTokenBalance()).to.equal(1000);
    expect(await user.getOriginalTokenBalance()).to.equal(999);
    expect(await user.getSwtTokenBalance()).to.equal(0);
  });

  it('Discount auction wrap order.', async () => {
    const filler = await UserClient.createTestUser(1_000_000);
    await filler.wrap(1_000_000);

    const user = await UserClient.createTestUser(1_000_000);

    await user.placeOrder({
      side: Side.Wrap,
      amountIn: 990_000,
      amountOut: 1_000_000,
    });
    // When user places order to wrap, they deposit the amountIn into escrow.
    expect(await user.getOriginalTokenBalance()).to.equal(10_000);
    expect(await user.getSwtTokenBalance()).to.equal(0);

    await filler.fillOrder({side: Side.Wrap, orderOwner: user});

    // After wrap order fills, the maker (user) gets the full amountOut while the taker gets the amountIn.
    expect(await filler.getOriginalTokenBalance()).to.equal(990_000);
    expect(await filler.getSwtTokenBalance()).to.equal(0);
    expect(await user.getOriginalTokenBalance()).to.equal(10_000);
    expect(await user.getSwtTokenBalance()).to.equal(1_000_000);
  });

  it('Wrap orders halted', async () => {
    const user = await UserClient.createTestUser(1000);
    expect(await user.getOriginalTokenBalance()).to.equal(1000);

    // Expect placing orders to fail while orders are halted.
    await authorityClient.haltWrapOrders();

    await utils.assertRaisesError(async () => {
      await user.placeOrder({
        side: Side.Wrap,
        amountIn: 1000,
        amountOut: 1001,
      });
    }, 'Wrap orders are currently halted.');

    await authorityClient.resumeWrapOrders();
    // After authority resumes, wrap orders are allowed.
    await user.placeOrder({
      side: Side.Wrap,
      amountIn: 1000,
      amountOut: 1001,
    });
    expect(await user.getOriginalTokenBalance()).to.equal(0);
  });

  it('program authority cancels permanently frozen user unwrap and order', async () => {
    const user = await UserClient.createTestUser(1000);
    await user.wrap(1000);

    const user2 = await UserClient.createTestUser(1000);
    await user2.wrap(1000);

    // User creates pending unwrap and order.
    await user.unwrap(500);
    await user.placeOrder({side: Side.Unwrap, amountIn: 500, amountOut: 400});

    // Permanently freeze the user
    await authorityClient.freeze({
      accountToFreeze: user.swtAccount,
      freezePeriodSeconds: 1000,
    });
    await mintOwnerFreeze(user.tokenAccount);
    await authorityClient.permanentFreeze({
      accountToFreeze: user.swtAccount,
      originalTokenAccount: user.tokenAccount,
    });

    expect(await user.getOriginalTokenBalance()).to.equal(0);
    expect(await user.getSwtTokenBalance()).to.equal(0);

    // Program authority cancels the user's existing unwrap and order.
    // After each, verify balance on the frozen user's account and that their account remains frozen.
    await authorityClient.cancelUnwrapProgram({user: user});
    expect(await user.getOriginalTokenBalance()).to.equal(0);
    expect(await user.getSwtTokenBalance()).to.equal(500);
    // Confirm user's SWT account remains frozen.
    utils.assertRaisesError(async () => {
      await user.sendSwtTokens({
        receivingAccount: user2.swtAccount,
        amount: 100,
      });
    }, 'custom program error: 0x11');

    await authorityClient.cancelOrderProgram({side: Side.Unwrap, user: user});
    expect(await user.getOriginalTokenBalance()).to.equal(0);
    expect(await user.getSwtTokenBalance()).to.equal(1000);
    // Confirm user's SWT account remains frozen.
    utils.assertRaisesError(async () => {
      await user.sendSwtTokens({
        receivingAccount: user2.swtAccount,
        amount: 100,
      });
    }, 'custom program error: 0x11');
  });

  it('program authority cancel_order_program, cancel_unwrap_program error cases', async () => {
    const user = await UserClient.createTestUser(10000);
    await user.wrap(1000);
    await user.unwrap(500);
    await user.placeOrder({side: Side.Unwrap, amountIn: 500, amountOut: 400});
    await user.placeOrder({side: Side.Wrap, amountIn: 400, amountOut: 500});

    // User has not been frozen by SWT program yet.
    await utils.assertRaisesError(async () => {
      await authorityClient.cancelUnwrapProgram({user: user});
    }, 'The program expected this account to be already initialized.');

    // User is frozen by SWT program, but not permanently frozen.
    await authorityClient.freeze({
      accountToFreeze: user.swtAccount,
      freezePeriodSeconds: 1000,
    });

    await utils.assertRaisesError(async () => {
      await authorityClient.cancelUnwrapProgram({user: user});
    }, 'Program can only cancel unwraps for permanently frozen users.');

    await utils.assertRaisesError(async () => {
      await authorityClient.cancelOrderProgram({side: Side.Unwrap, user: user});
    }, 'Program can only cancel UNWRAP orders for permanently frozen users.');

    // Now user is permanently frozen.
    await mintOwnerFreeze(user.tokenAccount);
    await authorityClient.permanentFreeze({
      accountToFreeze: user.swtAccount,
      originalTokenAccount: user.tokenAccount,
    });

    // Still, the program is not allowed to cancel Wrap orders.
    await utils.assertRaisesError(async () => {
      await authorityClient.cancelOrderProgram({side: Side.Wrap, user: user});
    }, 'Program can only cancel UNWRAP orders for permanently frozen users.');
  });
});
