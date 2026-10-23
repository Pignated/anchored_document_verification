use std::{fmt, str::FromStr};

use alloy::{
    hex::FromHexError,
    primitives::{Address, address},
    providers::{Provider, ProviderBuilder},
    sol,
};
use clap::{
    Arg, ArgAction, Command, arg,
    builder::{RangedU64ValueParser, ValueParser},
    value_parser,
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

fn parse_user_address(input: &str) -> Result<Address, FromHexError> {
    let clean_input = input.trim();
    Address::from_str(clean_input)
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let connection_string = "http://127.0.0.1:8545";
    let provider = match ProviderBuilder::new().connect(connection_string).await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Couldn't connect to {connection_string}");
            return Err(());
        }
    };
    let contract_addr: Address = address!("0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6");
    let contract = FileRegister::new(contract_addr, provider.clone());
    let cmd = Command::new("filever")
        .bin_name("filever")
        .subcommand(
            Command::new("search")
                .about("Search for specific transactions")
                .arg(
                    arg!(-a --address <ADDRESS> "user address")
                        .required(false)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("from-block")
                        .long("from-block")
                        .help("The block to start the query at")
                        .required(false)
                        .value_parser(RangedU64ValueParser::<u64>::new()),
                )
                .arg(
                    Arg::new("to-block")
                        .long("to-block")
                        .help("The block to end the query at")
                        .required(false)
                        .value_parser(RangedU64ValueParser::<u64>::new()),
                )
                .arg(
                    Arg::new("file")
                        .short('f')
                        .long("file")
                        .help("Show only events referring to file hashes")
                        .required(false)
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("merkle")
                        .short('m')
                        .long("merkle")
                        .help("Show only events referring to merkle roots")
                        .required(false)
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("upload")
                .subcommand(
                    Command::new("file").about("upload a file hash").arg(
                        Arg::new("filepath")
                            .value_parser(ValueParser::path_buf())
                            .help("Path to file"),
                    ),
                )
                .subcommand(
                    Command::new("merkle").about("upload a merkle root").arg(
                        Arg::new("filepath")
                            .value_parser(ValueParser::path_buf())
                            .help("Path to file"),
                    ),
                )
                .arg(
                    Arg::new("upload_addr")
                        .short('a')
                        .long("addr")
                        .alias("address")
                        .value_parser(ValueParser::string())
                        .help("Uploader's address")
                        .required(false),
                ),
        );
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("search", search_matches)) => {
            let mut filter = contract.AnchorRecord_filter();
            if let Some(address) = search_matches.get_one::<String>("address") {
                match parse_user_address(address) {
                    Ok(addr) => {
                        filter = filter.topic1(addr);
                    }
                    Err(e) => {
                        match e {
                            FromHexError::InvalidHexCharacter { c, index } => {
                                eprintln!(
                                    "Invalid hex character in address: index: {index}, char: {c}"
                                )
                            }
                            FromHexError::InvalidStringLength | FromHexError::OddLength => {
                                eprintln!("Must be 32 bytes worth of hex")
                            }
                        };
                        return Err(());
                    }
                }
            }
            if let Some(from) = search_matches.get_one::<u64>("from-block") {
                filter = filter.from_block(*from);
            } else {
                filter = filter.from_block(0);
            }
            if let Some(to) = search_matches.get_one::<u64>("to-block") {
                filter = filter.to_block(*to);
            }
            let file_flag = search_matches.get_flag("file");
            let merkle_flag = search_matches.get_flag("merkle");
            let logs = match contract.provider().get_logs(&filter.filter).await {
                Ok(logs) => logs,
                Err(_) => {
                    println!("Failed sending request to provider");
                    return Err(());
                }
            };

            let mut first = true;
            for log in logs {
                if let Ok(decoded) = log.log_decode::<FileRegister::AnchorRecord>() {
                    let anchor = decoded.inner.data;
                    match anchor.recordType {
                        FileRegister::HashType::MerkleRoot => {
                            if file_flag && !merkle_flag {
                                continue;
                            }
                        }
                        FileRegister::HashType::RawFileHash => {
                            if merkle_flag && !file_flag {
                                continue;
                            }
                        }
                        FileRegister::HashType::__Invalid => {
                            continue; //Should never happen
                        }
                    }
                    if first {
                        first = false
                    } else {
                        println!("--------------------------------------------");
                    }
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
        }
        Some((_, _)) => {
            println!("rip");
        }
        None => todo!(),
    }
    Ok(())
}
