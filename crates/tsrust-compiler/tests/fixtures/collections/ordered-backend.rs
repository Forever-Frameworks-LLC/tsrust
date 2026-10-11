fn main() {
    use std::cmp::Ordering;
    let numbers = [f64::NEG_INFINITY, -3.0, -0.0, 0.0, 2.0, f64::INFINITY, f64::NAN, f64::from_bits(0xfff8_0000_0000_0042)];
    for (a_index, &a) in numbers.iter().enumerate() {
        for (b_index, &b) in numbers.iter().enumerate() {
            let rank = |i| match i { 3 => 2, 7 => 6, _ => i };
            assert_eq!(TsrOrderedNumber::new(a).cmp(&TsrOrderedNumber::new(b)), rank(a_index).cmp(&rank(b_index)));
            for &c in &numbers {
                let (a,b,c) = (TsrOrderedNumber::new(a),TsrOrderedNumber::new(b),TsrOrderedNumber::new(c));
                if a <= b && b <= c { assert!(a <= c); }
            }
        }
    }
    assert_eq!(TsrOrderedNumber::new(-0.0).value().to_bits(), 0.0f64.to_bits());
    assert_eq!(TsrOrderedNumber::new(f64::NAN).cmp(&TsrOrderedNumber::new(f64::NAN)), Ordering::Equal);
    let mut map = TsrOrderedMap::new();
    let mut model = Vec::<(i64,Vec<i64>)>::new();
    for step in 0..2048_i64 {
        let key = (step * 37 + 13) % 79;
        match step % 5 {
            0..=2 => {
                let expected = model.iter_mut().find(|(k,_)| *k==key).map(|(_,v)| std::mem::replace(v,vec![step]));
                if expected.is_none() { model.push((key,vec![step])); }
                assert_eq!(map.insert(key,vec![step]),expected);
            }
            3 => {
                let expected=model.iter().position(|(k,_)| *k==key).map(|i|model.remove(i).1);
                assert_eq!(map.remove(&key),expected);
            }
            _ => assert_eq!(map.get(&key),model.iter().find(|(k,_)| *k==key).map(|(_,v)|v)),
        }
        model.sort_by_key(|(k,_)|*k);
        assert_eq!(map.len(),model.len());
        assert_eq!(map.iter().map(|(k,v)|(*k,v.clone())).collect::<Vec<_>>(),model);
        assert_eq!(map.contains_key(&key),model.iter().any(|(k,_)|*k==key));
    }
    let mut copy=map.clone();
    copy.insert(100,vec![42]);
    assert!(!map.contains_key(&100));
    if let Some((key,_))=model.first() {
        let mut moved=copy.remove(key).unwrap();moved.push(999);
        assert_eq!(map.get(key).unwrap().len(),1);
    }
    map.clear();assert_eq!(map.len(),0);
    let mut set=TsrOrderedSet::new();
    for word in ["🦀","é","a","a","界","\u{e000}",""] { set.insert(word.to_string()); }
    assert_eq!(set.iter().map(|v|v.as_str()).collect::<Vec<_>>(),["","a","é","界","\u{e000}","🦀"]);
    assert!(!set.insert("a".into()));assert!(set.contains(&"a".into()));
    assert!(set.remove(&"a".into()));assert!(!set.remove(&"a".into()));assert_eq!(set.len(),5);
    let saved=set.clone();set.clear();assert_eq!(set.len(),0);assert_eq!(saved.len(),5);
    let mut numeric=TsrOrderedSet::new();
    for n in numbers { numeric.insert(TsrOrderedNumber::new(n)); }
    assert_eq!(numeric.len(),6);
    assert!(numeric.iter().last().unwrap().value().is_nan());
    let mut numeric_map=TsrOrderedMap::new();
    assert_eq!(numeric_map.insert(TsrOrderedNumber::new(-0.0),1),None);
    assert_eq!(numeric_map.insert(TsrOrderedNumber::new(0.0),2),Some(1));
    assert_eq!(numeric_map.insert(TsrOrderedNumber::new(f64::NAN),3),None);
    assert_eq!(numeric_map.insert(TsrOrderedNumber::new(f64::from_bits(0xfff8_0000_0000_0042)),4),Some(3));
    assert_eq!(numeric_map.len(),2);
    assert_eq!(numeric_map.get(&TsrOrderedNumber::new(f64::NAN)),Some(&4));
    let mut numeric_heap=TsrHeap::new();
    for n in numbers { numeric_heap.push(TsrOrderedNumber::new(n)); }
    for expected in [f64::NAN,f64::NAN,f64::INFINITY,2.0,0.0,0.0,-3.0,f64::NEG_INFINITY] {
        let actual=numeric_heap.pop().unwrap().value();
        assert!(actual==expected || actual.is_nan() && expected.is_nan());
    }
    let mut heap=TsrHeap::new();let mut expected=Vec::new();
    for value in [5_i64,1,9,9,-3,0] {heap.push(value);expected.push(value);}
    expected.sort_by(|a,b|b.cmp(a));
    for value in expected {assert_eq!(heap.peek(),Some(&value));assert_eq!(heap.pop(),Some(value));}
    assert_eq!(heap.pop(),None);assert_eq!(heap.peek(),None);assert_eq!(heap.len(),0);
    heap.push(7);let copy=heap.clone();heap.clear();assert_eq!(heap.len(),0);assert_eq!(copy.peek(),Some(&7));
    struct Window<'a>(&'a [u8]);
    let bytes=[1,2,3];let mut borrowed=TsrOrderedMap::new();borrowed.insert(1,Window(&bytes[1..]));
    assert_eq!(borrowed.get(&1).unwrap().0.as_ptr(),bytes[1..].as_ptr());
    let moved=borrowed.remove(&1).unwrap();drop(borrowed);assert_eq!(moved.0,&[2,3]);
    struct Probe<'a>(&'a std::cell::Cell<usize>);
    impl Drop for Probe<'_>{fn drop(&mut self){self.0.set(self.0.get()+1);}}
    let drops=std::cell::Cell::new(0);
    {let mut values=TsrOrderedMap::new();values.insert(1,Probe(&drops));drop(values.insert(1,Probe(&drops)));assert_eq!(drops.get(),1);values.insert(2,Probe(&drops));let removed=values.remove(&2).unwrap();values.clear();assert_eq!(drops.get(),2);drop(removed);assert_eq!(drops.get(),3);values.insert(3,Probe(&drops));}
    assert_eq!(drops.get(),4);
}
