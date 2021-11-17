// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec, vec::Vec};

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    debug,
    high_level::{QueryIter, load_script, load_tx_hash, load_cell_lock_hash, load_cell_data},
    ckb_types::{bytes::Bytes, prelude::*},
    ckb_constants::Source,
    error::SysError,
};

// use crate::error::Error;

/// Error
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding, // number 4 -- ValidationFailure(4)
    Amount
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}


pub fn main() -> Result<(), Error> {
    // load current script
    // check verification branch is owner mode or normal mode
    let script = load_script()?; // Loads the script data structure
    let args: Bytes = script.args().unpack(); // Grabs the arguments from there

    // return success if owner mode is true
    if check_owner_mode(&args)? { // If arguments indicate that the cell is owner
                                  // we are in owner mode...
                                  // we also know the owner lock script will get run
                                  // so as long as we used a proper lock script,
                                  // this is secure.
        return Ok(()); // Let the user do whatever, because they're the owner
                       // This allows the owner to mint tokens.
    }

    // unpack the Script#args field
    // let args: Vec<u8> = script.args().unpack();

    // Otherwise this cell is just a normal cell
    // we should only permit
    let inputs_amount = collect_inputs_amount()?;
    let outputs_amount = collect_outputs_amount()?;

    if inputs_amount < outputs_amount {
        return Err(Error::Amount);
    }

    return Ok(());
}


fn check_owner_mode(args: &Bytes) -> Result<bool, Error> {
    // With owner lock script extracted, we will look through each input in the
    // current transaction to see if any unlocked cell uses owner lock.
    let is_owner_mode = QueryIter::new(load_cell_lock_hash, Source::Input)
        .find(|lock_hash| args[..] == lock_hash[..]).is_some();
    Ok(is_owner_mode)
}

const UDT_LEN: usize = 16;

fn collect_inputs_amount() -> Result<u128, Error> {
    // let's loop through all input cells containing current UDTs,
    // and gather the sum of all input tokens.
    let mut buf = [0u8; UDT_LEN];

    let udt_list = QueryIter::new(load_cell_data, Source::GroupInput)
        .map(|data|{
            if data.len() == UDT_LEN {
                buf.copy_from_slice(&data);
                // u128 is 16 bytes
                Ok(u128::from_le_bytes(buf))
            } else {
                Err(Error::Encoding)
            }
        }).collect::<Result<Vec<_>, Error>>()?;
    Ok(udt_list.into_iter().sum::<u128>())
}

fn collect_outputs_amount() -> Result<u128, Error> {
    // With the sum of all input UDT tokens gathered, let's now iterate through
    // output cells to grab the sum of all output UDT tokens.
    let mut buf = [0u8; UDT_LEN];

    let udt_list = QueryIter::new(load_cell_data, Source::GroupOutput)
        .map(|data|{
            if data.len() == UDT_LEN {
                buf.copy_from_slice(&data);
                // u128 is 16 bytes
                Ok(u128::from_le_bytes(buf))
            } else {
                Err(Error::Encoding)
            }
        }).collect::<Result<Vec<_>, Error>>()?;
    Ok(udt_list.into_iter().sum::<u128>())
}
