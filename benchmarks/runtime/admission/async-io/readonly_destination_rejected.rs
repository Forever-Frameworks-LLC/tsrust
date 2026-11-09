include!(env!("TSRUST_IO_FUTURES_SOURCE"));mod owned_read;
fn main(){let mut reader=owned_read::OwnedRead::new(std::io::Cursor::new(vec![1]),8);let destination=[0u8;4];let pending=reader.read_into(&destination);drop(pending);}
