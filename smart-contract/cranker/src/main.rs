mod crank;
mod error;
mod pools;
mod settings;
mod utils;

use {
    solana_program::pubkey::Pubkey,
    std::{thread::sleep, time::Duration},
    tokio::{runtime::Runtime, task},
};

fn process() -> Result<(), error::ProgramError> {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let (central_key, _) =
        Pubkey::find_program_address(&[&access_protocol::ID.to_bytes()], &access_protocol::ID);
    let all_pools = pools::get_all_pools()?;

    let mut join_handles = Vec::with_capacity(all_pools.len());

    for pool in all_pools {
        let handle = task::spawn(async move { crank::crank_pool(pool, central_key).await });
        join_handles.push(handle)
    }

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
