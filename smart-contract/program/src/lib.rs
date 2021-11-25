use solana_program::declare_id;

pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod state;

pub(crate) mod processor;
pub(crate) mod utils;

pub mod cpi;

declare_id!("FuGuhWkaMCWfk2sg3VsxTL39zurSNQgtCV5zjFcVaTio");
