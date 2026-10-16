#[test]
fn uuid_boundaries_preserve_all_bits_and_entropy_failures(){
 use tsrust_native_uuid::*;
 let text="67e55044-10b1-426f-9247-bb680e5fe0c8";
 let value=parse_uuid(text).unwrap();assert_eq!(format_uuid(&parse_uuid(&text.to_uppercase()).unwrap()),text);
 assert_eq!(format_uuid(&from_bytes(&to_bytes(&value)).unwrap()),text);assert_eq!(version(&value),4);
 for text in ["","67e5504410b1426f9247bb680e5fe0c8","{67e55044-10b1-426f-9247-bb680e5fe0c8}","67e55044-10b1-426f-9247-bb680e5fe0cg"]{assert!(parse_uuid(text).is_err())}
 for length in [0,15,17]{assert!(from_bytes(&vec![0;length]).is_err())}
 let nil=from_bytes(&[0;16]).unwrap();let max=from_bytes(&[255;16]).unwrap();assert_eq!(format_uuid(&nil),"00000000-0000-0000-0000-000000000000");assert_eq!(format_uuid(&max),"ffffffff-ffff-ffff-ffff-ffffffffffff");assert_eq!(compare(&nil,&max),-1);assert_eq!(compare(&value,&value),0);
 assert!(failed_entropy());let fixed=fixed_entropy();assert_eq!(format_uuid(&fixed),"00000000-0000-4000-8000-000000000000");
 let random=random_uuid().unwrap();assert_eq!(version(&random),4);assert_eq!(to_bytes(&random)[8]&0xc0,0x80);
}
