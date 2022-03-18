use {
    access_protocol::error::AccessError,
    solana_client::client_error::ClientError,
    solana_program::instruction::InstructionError,
    solana_sdk::signature::Signature,
    std::fmt::Debug,
    std::time::{SystemTime, UNIX_EPOCH},
    tokio::task,
};

pub async fn retry<F, T, K, E, R>(arg: T, f: F, e: R) -> K
where
    F: Fn(&T) -> Result<K, E>,
    E: Debug,
    R: Fn(Result<K, E>) -> Result<K, E>,
{
    loop {
        let res = e(f(&arg));
        if res.is_ok() {
            return res.unwrap();
        }

        let error = res.err().unwrap();

        println!("Failed task with {:#?}, retrying", error);
        task::yield_now().await;
    }
}

pub fn no_op_filter(r: Result<Signature, ClientError>) -> Result<Signature, ClientError> {
    if let Err(e) = &r {
        match &e.kind {
            solana_client::client_error::ClientErrorKind::RpcError(
                solana_client::rpc_request::RpcError::RpcResponseError {
                    code: _,
                    message: _,
                    data,
                },
            ) => {
                if let solana_client::rpc_request::RpcResponseErrorData::SendTransactionPreflightFailure(f) = data {
                    match f.err {
                        Some(solana_sdk::transaction::TransactionError::InstructionError(_, InstructionError::Custom(c))) if c == AccessError::NoOp as u32=> {
                            println!("Operation was a no-op");
                            Ok(Signature::new(&[0;64]))
                        }
                        _ => r
                    }
                } else {
                    r
                }
            }
            _ => r,
        }
    } else {
        r
    }
}

pub fn current_time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    let now = since_the_epoch.as_secs();
    now
}
