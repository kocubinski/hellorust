mod test;

use error_chain::error_chain;
use std::fs;
use std::fs::{File};
use std::io::{Read};
use clap::Parser;
use prost::Message;

error_chain! {
    foreign_links {
        Utf8(std::str::Utf8Error);
        AddrParse(std::net::AddrParseError);
        Io(std::io::Error);
        SystemTimeError(std::time::SystemTimeError);
        Decode(prost::DecodeError);
        Sled(sled::Error);
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}


#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Node {
    #[prost(bytes, tag = "1")]
    pub key: Vec<u8>,

    #[prost(bytes, tag = "2")]
    pub value: Vec<u8>,

    #[prost(bool, tag = "3")]
    pub deleted: bool,

    #[prost(int64, tag = "4")]
    pub block: i64,

    #[prost(string, tag = "5")]
    pub store_key: String
}

fn sorted_files(dir: &str) -> Result<Vec<String>> {
    let mut files = fs::read_dir(dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>>>()?;
    files.sort();
    Ok(files)
}

fn main() -> Result<()> {
    // let args = Args::parse();

    let log_dir = "/Users/mattk/src/scratch/osmosis-hist/bank";
    let tree = sled::open("/Users/mattk/.costor/sled.db")?;

    let mut count = 0;
    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();
        println!("path: {:?}", path);
        let f = File::open(path)?;
        let mut gz = flate2::read::GzDecoder::new(f);

        loop {
            let mut sz = [0u8; 4];
            let mut n = gz.read(&mut sz[..])?;
            if n == 0 {
                println!("EOF");
                break;
            }
            if n < 4 {
                loop {
                    let m = gz.read(&mut sz[n..])?;
                    if n + m == 4 {
                        break;
                    }
                    n += m;
                }
            }
            let size = u32::from_le_bytes(sz) as usize;

            let mut proto = vec![0u8; size];
            n = gz.read(&mut proto[..])?;
            if n != size {
                loop {
                    let m = gz.read(&mut proto[n..])?;
                    if n + m == size {
                        break;
                    }
                    n += m;
                }
            }
            let node_res = Node::decode(&proto[..]);
            if node_res.is_err() {
                println!("node err: {:?}; len {:?}", node_res, proto.len());
                continue;
            }
            let node = node_res.unwrap();

            // key + block + deleted
            let key_len = proto.len() + 8 + 1;
            let mut key = vec![0u8; key_len];
            key[..proto.len()].copy_from_slice(&proto[..]);
            key[proto.len()..key_len-1].copy_from_slice(&node.block.to_be_bytes());
            key[key_len-1] = node.deleted as u8;

            tree.insert(key, node.value)?;

            if count % 100_000 == 0 {
                println!("count: {}", count);
                tree.flush()?;
            }
            count += 1;
        }
    }

    Ok(())
}