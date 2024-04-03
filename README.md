# Hatom Liquid Staking Smart Contract

![main](https://github.com/HatomProtocol/hatom-liquid-staking/actions/workflows/actions.yml/badge.svg)

Hatom Liquid Staking protocol is a decentralized liquid staking protocol that makes EGLD staking as secure and
decentralized as possible. The protocol allows users to stake their EGLD and receive its liquid representation, sEGLD,
in exchange. sEGLD can be used in DeFi applications (such as the Hatom Lending Protocol), traded on exchanges, or simple
peer-to-peer transfers. sEGLD accrues staking rewards at every epoch change, and users can redeem their sEGLD for EGLD
at any time but with an unbonding period given by the underlying MultiversX staking solution.

## :tada: Getting Started

This repository contains the source code for the Hatom Liquid Staking Protocol. The protocol is built on top of the
MultiversX blockchain and consists on only one smart contract that implements all the logic.

To compile the project, make sure you have installed [Rust](https://www.rust-lang.org/tools/install). After downloading
the repository and checking out the intended branch, you can build the project by simply running:

```bash
$ cargo build
```

## :sparkles: Protocol Overview

The Hatom Liquid Staking Protocol whitelists many Delegation Smart Contracts or Staking Providers which will receive
either delegations or undelegations from users. The protocol is responsible for managing the delegation algorithm, which
is composed of the following steps:

1. Each Staking Provider has a delegation score given by its total value locked (TVL) and annual percentage rate (APR).
   Lower TVLs and higher APRs will result in higher scores.
2. Staking Providers with the higher scores (for delegations) or lower scores (for undelegations) are selected.
3. Finally, the protocol runs a weighted sample to select the final Staking Provider that will receive the delegation or
   undelegation. This random selection is weighted by the service fee of the Staking Provider.

## :busts_in_silhouette: Users

The main interactions that users can perform with the protocol are:

- `delegate`: Stake EGLD and receive sEGLD in exchange.
- `unDelegate`: Redeem sEGLD for an undelegate NFT that can be redeemed for EGLD after the unbonding period.
- `withdraw`: Redeem the undelegate NFT for EGLD after the unbonding period has elapsed.

Notice that delegations, undelegations and withdrawals do not perform the actual operations at the underlying Delegation
Smart Contracts. Instead, they only mint and burn sEGLD and/or NFTs, run the delegation algorithm and update storage
variables. This way, these endpoints do not perform any async operation and allow for easier integrations with the
protocol.

The actual async operations are performed by the following public endpoints:

- `delegatePendingAmount`: Delegate the pending amount of EGLD to the underlying Staking Provider.
- `unDelegatePendingAmount`: Undelegate the pending amount of EGLD from the underlying Staking Provider.
- `withdrawFrom`: Withdraw EGLD from the underlying Staking Provider.
- `claimRewards`: Claim rewards from the underlying Staking Provider.
- `delegateRewards`: Delegate rewards to a Staking Provider.

All these actions are currently performed by Hatom's bots, running with meaningful frequencies.

## :monocle_face: Audits

The Hatom Liquid Staking Protocol has been extensively audited before its launch. Multiple firms and parties have looked
and analyzed the code, including:

- Various Hatom Core Developers
- [MultiversX](https://multiversx.com/)
- [Runtime Verification](https://runtimeverification.com/)
- [Certik](https://www.certik.com/)
- [Arda](https://arda.run/)

## :handshake: Connect with the community

- [Web](https://hatom.com/)
- [App](https://app.hatom.com/liquid)
- [Discord](https://discord.com/invite/WekwfUDXGp)
- [Blog](https://mirror.xyz/0xDac8B6141d28C46765255607f3572c73A064383f)
- [Twitter](https://twitter.com/HatomProtocol)
- [Telegram](https://t.me/+tfGNdvZpgcoxNDM0)
