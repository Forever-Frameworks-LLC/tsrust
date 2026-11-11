struct Row{key:i64,label:std::string::String,values:Vec<f64>}
fn same(left:&Row,right:&Row)->bool{left.key==right.key&&left.label==right.label&&left.values.len()==right.values.len()&&left.values.iter().zip(&right.values).all(|(a,b)|a.to_bits()==b.to_bits()||(a.is_nan()&&b.is_nan()))}
fn function0(value0: i64, value1: i64) -> i64{let left=(0..8).map(|j|Row{key:j*j,label:"é".to_owned(),values:vec![f64::NAN,-0.0,j as f64]}).collect::<Vec<_>>();let right=(0..8).map(|j|Row{key:j*j,label:"é".to_owned(),values:vec![f64::NAN,-0.0,j as f64]}).collect::<Vec<_>>();let mut checksum=0;for i in 0..value0{let at=((i*13+value1)%8)as usize;assert!(same(&left[at],&right[at]));assert!(!same(&left[at],&right[(at+1)%8]));checksum=(checksum*131+left[at].key)%1000000007;}checksum}
fn main(){}
