include!(env!("TSRUST_IO_FUTURES_SOURCE"));mod owned_read;
fn main(){let mut reader=owned_read::OwnedRead::new(std::io::Cursor::new(vec![1]),8);let mut first=[0u8;4];let mut second=[0u8;4];let pending=reader.read_into(&mut first);let other=reader.read_into(&mut second);drop(pending);drop(other);}
