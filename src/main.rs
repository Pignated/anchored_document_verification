use std::fmt;

use alloy::{
    primitives::address,
    providers::{Provider, ProviderBuilder},
    sol,
};

// sol!(
//     #![sol(all_derives)]
//     #[sol(rpc)]
//     interface IFileReg {
//         enum HashType {RawFileHash, MerkleRoot}
//         event AnchorRecord(address indexed user, bytes32 indexed hashValue, HashType recordType, uint256 timestamp);
//         event PermissionChanged(address indexed user, bool allowed);
//
//     }
// );
sol!(
    #[sol(rpc)]
    "foundry/src/FileRegistry.sol"
);

impl fmt::Debug for FileRegister::HashType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileRegister::HashType::MerkleRoot => write!(f, "Merkle Root"),
            FileRegister::HashType::RawFileHash => write!(f, "File Hash"),
            FileRegister::HashType::__Invalid => write!(f, "Invalid"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection_string = "http://127.0.0.1:8545";
    let provider = match ProviderBuilder::new().connect(connection_string).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Couldn't connect to {connection_string}");
            return Err(e.into());
        }
    };
    let contract_addr = address!("0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6");
    let contract = FileRegister::new(contract_addr, provider.clone());
    let filter = contract.AnchorRecord_filter().from_block(0);
    let logs = contract.provider().get_logs(&filter.filter).await?;
    let mut first = true;
    for log in logs {
        if let Ok(decoded) = log.log_decode::<FileRegister::AnchorRecord>() {
            if first {
                first = false
            } else {
                println!("--------------------------------------------");
            }
            let anchor = decoded.inner.data;
            println!("User: {:?}", anchor.user);
            println!("Hash Value: {:?}", anchor.hashValue);
            println!("Record Type: {:?}", anchor.recordType);
            let unix_seconds = anchor.timestamp.to::<u64>() as i64;
            if let Ok(datetime) = time::OffsetDateTime::from_unix_timestamp(unix_seconds) {
                println!("Timestamp: {}", datetime);
            } else {
                println!("Timestamp: {:?}", anchor.timestamp);
            }
        }
    }
    Ok(())
}
