mod crank;
mod error;
mod pools;
mod settings;
mod utils;
mod distribute_fees;

use solana_client::rpc_client::RpcClient;

use {
    solana_program::pubkey::Pubkey,
    std::{thread::sleep, time::Duration},
    tokio::{runtime::Runtime, task},
};
use access_protocol::state::{CentralStateV2};
use crate::settings::PROGRAM_ID;
use borsh::de::BorshDeserialize;

fn process() -> Result<(), error::ProgramError> {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    println!("PROGRAM: {}", *PROGRAM_ID);

    let (central_key, _) =
        Pubkey::find_program_address(&[&PROGRAM_ID.to_bytes()], &PROGRAM_ID);

    let connection = RpcClient::new(crate::settings::RPC_URL.as_str());
    let acc = connection
        .get_account(&central_key)
        .unwrap();
    let central_state = CentralStateV2::deserialize(&mut &acc.data[..]).unwrap();

    let all_pools = pools::get_all_pools(connection, central_state.creation_time as u64)?;
    let mut join_handles = Vec::with_capacity(all_pools.len() + 1);

    for pool in all_pools {
        let handle = task::spawn(async move { crank::crank_pool(pool, central_key).await });
        join_handles.push(handle)
    }

    join_handles.push(task::spawn(async move {  if let Err(e) = distribute_fees::distribute_fees(central_key).await {
        println!("Error distributing fees: {}", e);
    }}));

    for t in join_handles {
        rt.block_on(t).unwrap();
    }

    Ok(())
}

fn main() {
    loop {
        let result = process();
        match result {
            Ok(()) => println!("Finished cranking cycle"),
            Err(e) => println!("Error: {}", e),
        }
        sleep(Duration::from_secs(settings::CYCLE_INTERVAL))
    }
}
