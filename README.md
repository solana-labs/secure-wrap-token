# Secure Wrap Token

## This code is not audited. Use at your own risk.

## Introduction

Secure Wrap Token (SWT) is an open-source reference implementation of a custodial token wrap program.

It's an additive program that a DeFi protocol team can independently deploy on-chain to enhance the security over their user assets.

Users wrap their SPL-Tokens to receive (mint) wrapped tokens 1:1. The SWT program retains full custody of the original tokens.
Users can always unwrap (burn) wrapped tokens 1:1 for original tokens. This unwrapping has a time delay of e.g. 24 hours.

## Motivation

Numerous DeFi protocols have had bugs, allowing attackers to drain the protocols of their custodied user assets.
These attacks are possible because the trade execution layer and the custody layer have been tightly coupled.

With Secure Wrap Token, custody is decoupled into a separate program.

Upon an exploit of a DeFi protocol using SWT, the attacker would still obtain illicit wrapped tokens.
But the attacker faces a time delay to unwrap their stolen tokens. During this time, the DeFi protocol team and relevant authorities can intervene to prevent permanent loss of user assets.

### Actors

There are two actors: Program Authority, User.

User capabilities:

- Wrap any `SPL-Token` with the program, receiving wrapped tokens in exchange. Wrapped tokens are still `SPL-Token`. Original tokens are custodied by the program. Wrapped tokens are custodied by the user and can be transfered at will.
- Unwrap any amount at any time.  Unwraps are "released" after a time delay.
- Participate in Wrap/Unwrap market to facilitate other users who want to unwrap immediately. Supports two-sided order making and taking. Orders will always be at a discount or premium to directly wrapping/unwrapping 1:1. **Importantly, the marketplace is peer-to-peer. Participants are liable for clawback if they accept stolen tokens.**

Program Authority capabilities:

- Freeze any User's wrapped token account for a period of time, maximum up to 14 days.
  The subsequent thaw is permissionless and self-executable by the user.
  Once frozen, an account cannot be frozen again until 3 days after its thaw. The authority cannot freeze a frozen account to indefinitely extend its freeze period.
- Permanently freeze and redistribute funds from an account as necessary.  Described in next section.
- Temporarily halt all token unwraps and orders.  Usually done during incidents to prevent permanent loss of funds.

At a high level, the User trusts their assets to the Program Authority in exchange for remediation when things go wrong.

### Permanent Freeze & Clawback Mechanism

The Program Authority retains the right to clawback the funds from any SWT account.

This clawback is only executable after the SWT account has been "permanently frozen" by the authority.

To permanently freeze an SWT account, the program requires that the user's original token account (same `owner` as the SWT account) has also been frozen by the original token authority -- most likely in compliance to legal court orders.

Once permanently frozen, an SWT account can never be thawed.
The SWT protocol allows the authority to mint new wrapped tokens exactly up to the balance of the permanently frozen account. The authority distributes these newly minted tokens to whichever addresses it wishes -- presumably to return funds to victims from an attack.

## Quick start

### Setup Environment

1. Clone the repository from <https://github.com/solana-labs/secure-wrap-token.git>.
2. Install the latest Solana tools from <https://docs.solana.com/cli/install-solana-cli-tools>. If you already have Solana tools, run `solana-install update` to get the latest compatible version.
3. Install the latest Rust stable from <https://rustup.rs/>. If you already have Rust, run `rustup update` to get the latest version.
4. Install the latest Anchor framework from <https://www.anchor-lang.com/docs/installation>. If you already have Anchor, run `avm update` to get the latest version.
Rustfmt is used to format the code. It requires `nightly` features to be activated:
5. Install `nightly` rust toolchain. <https://rust-lang.github.io/rustup/installation/index.html#installing-nightly>

#### [Optional] Vscode setup

1. Install `rust-analyzer` extension
2. If formatting doesn't work, make sure that `rust-analyzer.rustfmt.extraArgs` is set to `+nightly`

### Build

First, generate a new key for the program address with `solana-keygen new -o <PROG_ID_JSON>`. Then replace the existing program ID with the newly generated address in `Anchor.toml` and `programs/secure-wrap-token/src/lib.rs`.
Also, ensure the path to your wallet in `Anchor.toml` is correct. Alternatively, when running Anchor deploy or test commands, you can specify your wallet with `--provider.wallet` argument. The wallet's pubkey will be set as an upgrade authority upon initial deployment of the program. It is strongly recommended to make upgrade authority a multisig when deploying to the mainnet.
To build the program run `anchor build` command from the `secure-wrap-token/` directory:

```sh
anchor build
```

### Test

Integration tests (Typescript) can be run in the `secure-wrap-token/` directory using the provided test script command:

```sh
npm install
./test_script.sh
```

By default, integration tests are executed on a local validator, so it won't cost you any SOL.

### Deploy

To deploy the program to the devnet and upload the IDL use the following commands:

```sh
anchor deploy --provider.cluster devnet --program-keypair <PROG_ID_JSON>
anchor idl init --provider.cluster devnet --filepath ./target/idl/secure_wrap_token.json <PROGRAM ID>
```

### Initialize

TODO: Add instructions and npx utility for managing SWT deployment.

## Support

If you are experiencing technical difficulties while working with the Secure Wrap Token codebase, open an issue on [Github](https://github.com/solana-labs/secure-wrap-token/issues). For more general questions about programming on Solana blockchain use [StackExchange](https://solana.stackexchange.com).
If you find a bug in the code, you can raise an issue on [Github](https://github.com/solana-labs/secure-wrap-token/issues). But if this is a security issue, please don't disclose it on Github or in public channels. Send information to <defi@solana.com> instead.

## Contributing

Contributions are very welcome. Please refer to the [Contributing](https://github.com/solana-labs/solana/blob/master/CONTRIBUTING.md) guidelines for more information.

## License

Solana Secure Wrap Token codebase is released under [Apache License 2.0](LICENSE).

## Disclaimer

By accessing or using Solana Secure Wrap Token or any of its components, you accept and agree with the [Disclaimer](DISCLAIMER.md).
