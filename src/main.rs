use error_chain::error_chain;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use bytes::BufMut;
use clap::Parser;
use prost::Message;

error_chain! {
    foreign_links {
        Utf8(std::str::Utf8Error);
        AddrParse(std::net::AddrParseError);
        Io(std::io::Error);
        SystemTimeError(std::time::SystemTimeError);
        Decode(prost::DecodeError);
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

fn main() -> Result<()> {
    // let args = Args::parse();

    let log_dir = "/Users/mattk/src/scratch/osmosis-hist/bank";

    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();
        let f = File::open(path)?;
        let mut gz = flate2::read::GzDecoder::new(f);

        let mut sz = [0u8; 4];
        gz.read(&mut sz[..])?;
        let sz = u32::from_le_bytes(sz);

        let mut proto = vec![0u8; sz as usize];
        gz.read(&mut proto[..])?;
        let mut bz = bytes::BytesMut::with_capacity(sz as usize);
        gz.read(&mut bz[..])?;
        bz.put(&proto[..]);
        let node = Node::decode(bz)?;
        println!("{:?}", node)
    }

    Ok(())
}