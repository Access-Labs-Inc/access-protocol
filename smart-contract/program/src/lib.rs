#![warn(missing_docs)]
/*!
Access protocol

## Overview

## Central state

The [`CentralState`][`state::CentralState`] contains the information about:

- ACCESS token mint
- ACCESS token inflation

The inflation schedule can be modified by the `authority` key contained in the [`CentralState`][`state::CentralState`] by using the [`change_inflation`][`fn@instruction::change_inflation`] instruction.

The [`CentralState`][`state::CentralState`] is the mint authority of the ACCESS token.

## Stake pool

[`Stake pools`][`state::StakePool`] are created by content publishers. In order to get access to the publisher's content users need to stake ACCESS tokens in the [`StakePool`][`state::StakePool`] of the publisher.

A [`Stake pools`][`state::StakePool`] is made of a header ([`StakePoolHeader`][`state::StakePoolHeader`]) and circular buffer that contains the pool balances multiplied by the current inflation at each crank time.

The circular buffer is updated using a permissionless [`crank`][`fn@instruction::crank`].


## Stake accounts

[`Stake accounts`][`state::StakeAccount`] are used to deposit funds in a stake pool. Stake accounts allow users to access the content of the publisher and earn yield in ACCESS tokens at the same time.

## Bonds

[`Bonds`][`state::Bonds`] represent locked ACCESS tokens sold by the ACCESS DAO. The lifecycle of a bond is as follow:

- [`create_bond`][`fn@instruction::create_bond`]: This instruction creates an inactive bond. The bond account contains the information about the price of the bond, the buyer, the unlock schedule and the sellers.
- [`sign_bond`][`fn@instruction::sign_bond`]: This instruction allows DAO members to approve the sell.
- [`claim_bond`][`fn@instruction::claim_bond`]: Once the bond has been signed by enough DAO members, the buyer can claim the bond.

Bond tokens can be staked like regular ACCESS tokens.

*/

use solana_program::declare_id;
#[doc(hidden)]
pub mod entrypoint;
#[doc(hidden)]
pub mod error;
/// Program instructions and their CPI-compatible bindings
pub mod instruction;
/// Describes the different data structres that the program uses to encode state
pub mod state;

#[doc(hidden)]
pub(crate) mod processor;
pub(crate) mod utils;

#[allow(missing_docs)]
pub mod cpi;

declare_id!("FuGuhWkaMCWfk2sg3VsxTL39zurSNQgtCV5zjFcVaTio");
