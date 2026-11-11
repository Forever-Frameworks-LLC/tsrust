// Negative Rust lifetime witness: reader and writable view stay borrowed by the future.
include!(env!("TSRUST_IO_FUTURES_SOURCE"));mod owned_read;
fn main(){let mut reader=owned_read::OwnedRead::new(std::io::Cursor::new(vec![1]),8);let mut owner=[0u8;4];let destination=&mut owner[1..3];let pending=reader.read_into(destination);owner[1]=9;drop(pending);}
