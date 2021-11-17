use super::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_testtool::ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_testtool::{ckb_error::assert_error_eq, ckb_script::ScriptError};
use ckb_testtool::ckb_error;

const MAX_CYCLES: u64 = 100_0000;


// errors
const ERROR_AMOUNT: i8 = 5;

fn build_test_context(
    inputs_token: Vec<u128>,
    outputs_token: Vec<u128>,
    is_owner_mode: bool,
) -> (Context, TransactionView) {
    // deploy my-sudt script
    let mut context = Context::default();
    let sudt_bin: Bytes = Loader::default().load_binary("my-sudt");
    // deploy_cell returns an OutPoint:
    // https://docs.rs/ckb-testtool/0.6.1/ckb_testtool/context/struct.Context.html#method.deploy_cell
    //
    // deploy sudt_bin script code in a livecell
    // grab the outpoint, see: https://docs.rs/ckb-types/0.100.0/ckb_types/packed/struct.OutPoint.html
    // it gives a reference to the cell by providing
    // tx_hash, index.
    let sudt_out_point = context.deploy_cell(sudt_bin);

    // deploy always_success script code in a livecell
    // grab the outpoint, see: https://docs.rs/ckb-types/0.100.0/ckb_types/packed/struct.OutPoint.html
    // it gives a reference to the cell by providing
    // tx_hash, index.
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // build lock script
    let lock_script = context
        .build_script(&always_success_out_point, // The outpoint here is used to grab code_hash of the lock script.
                      Default::default()) // default here indicates 0x
        .expect("script");

    // Builds a cell_dep for the lock script.
    // We need this before we can reference the lock script.
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    // build sudt script arguments
    // we just hash the always_success lock script.
    // and mark this as the required lock_script,
    // for this cell to be valid.
    let sudt_script_args: Bytes = if is_owner_mode {
        // use always_success script hash as owner's lock
        // we can always swap this in for a proper lock but... /shrug
        let lock_hash: [u8; 32] = lock_script.calc_script_hash().unpack();
        lock_hash.to_vec().into()
    } else {
        // use zero hash as owner's lock which implies we can never enter owner mode
        [0u8; 32].to_vec().into()
    };

    // build sudt script
    let sudt_script = context
        .build_script(&sudt_out_point, sudt_script_args)
        .expect("script");
    // build script dependencies again
    let sudt_script_dep = CellDep::new_builder().out_point(sudt_out_point).build();

    // prepare inputs
    // assign 1000 Bytes per input
    let input_ckb = Capacity::bytes(1000).unwrap().as_u64();
    let inputs = inputs_token.iter().map(|token| {
        let input_out_point = context.create_cell(
            // Magically initialize a cell with the proper lock scripts.
            // IRL we need to have an initial transaction,
            // which takes ckb from specified account,
            // and the following parts as output.
            CellOutput::new_builder()
                .capacity(input_ckb.pack()) // pass in the capacity declared above (1000)
                .lock(lock_script.clone())  // pass in the lock script declared above
                .type_(Some(sudt_script.clone()).pack()) // pass in the type script
                .build(),
            token.to_le_bytes().to_vec().into(),
        );
        let input = CellInput::new_builder()
            .previous_output(input_out_point)
            .build();
        input
    });

    // prepare outputs
    let output_ckb = input_ckb * inputs_token.len() as u64 / outputs_token.len() as u64;
    let outputs = outputs_token.iter().map(|_token| {
        CellOutput::new_builder()
            .capacity(output_ckb.pack()) // the token amount was specified above in the test_builder arguments.
            .lock(lock_script.clone()) // we should still preserve the original lock script
            .type_(Some(sudt_script.clone()).pack()) // and the type script as well.
            .build()
    });

    // We also need to construct the outputs...
    let outputs_data: Vec<_> = outputs_token
        .iter()
        .map(|token| Bytes::from(token.to_le_bytes().to_vec()))
        .collect();

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(sudt_script_dep)
        .build();

    // return the test context and transaction.
    (context, tx)
}

#[test]
fn test_basic() {
    let (mut context, tx) = build_test_context(vec![1000], vec![400, 600], false);
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("cycles: {}", cycles);
}

#[test]
fn test_destroy_udt() {
    let (mut context, tx) = build_test_context(vec![1000], vec![800, 100, 50], false);
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("cycles: {}", cycles);
}

#[test]
fn test_create_sudt_without_owner_mode() {
    let (mut context, tx) = build_test_context(vec![1000], vec![1200], false);
    let tx = context.complete_tx(tx);

    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    // assert_error_eq!(err, ScriptError::ValidationFailure(ERROR_AMOUNT));
    // TODO: Is there a way to cast error???
    // The above example is outdated.
    assert_error_eq!(err.kind(), ckb_error::ErrorKind::Script);
}

#[test]
fn test_create_sudt_with_owner_mode() {
    let (mut context, tx) = build_test_context(vec![1000], vec![1200], true);
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("cycles: {}", cycles);
}
