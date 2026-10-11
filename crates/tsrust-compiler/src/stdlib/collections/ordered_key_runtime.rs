trait TsrOrderedKey {
    fn tsr_compare(&self, other: &Self) -> std::cmp::Ordering;
    fn tsr_normalize(self) -> Self where Self: Sized { self }
}
impl TsrOrderedKey for i64 { fn tsr_compare(&self, other: &Self) -> std::cmp::Ordering { self.cmp(other) } }
impl TsrOrderedKey for bool { fn tsr_compare(&self, other: &Self) -> std::cmp::Ordering { self.cmp(other) } }
impl TsrOrderedKey for String { fn tsr_compare(&self, other: &Self) -> std::cmp::Ordering { { let left: &str = self.as_ref(); let right: &str = other.as_ref(); left.cmp(right) } } }
impl TsrOrderedKey for f64 {
    fn tsr_compare(&self, other: &Self) -> std::cmp::Ordering { TsrOrderedNumber::new(*self).cmp(&TsrOrderedNumber::new(*other)) }
    fn tsr_normalize(self) -> Self { TsrOrderedNumber::new(self).value() }
}
#[derive(Clone)]
struct TsrOrdered<T>(T);
impl<T:TsrOrderedKey> TsrOrdered<T> { fn new(value:T)->Self{Self(value.tsr_normalize())} }
impl<T:TsrOrderedKey> PartialEq for TsrOrdered<T>{fn eq(&self,other:&Self)->bool{self.cmp(other)==std::cmp::Ordering::Equal}}
impl<T:TsrOrderedKey> Eq for TsrOrdered<T>{}
impl<T:TsrOrderedKey> PartialOrd for TsrOrdered<T>{fn partial_cmp(&self,other:&Self)->Option<std::cmp::Ordering>{Some(self.cmp(other))}}
impl<T:TsrOrderedKey> Ord for TsrOrdered<T>{fn cmp(&self,other:&Self)->std::cmp::Ordering{self.0.tsr_compare(&other.0)}}

impl<T:TsrOrderedKey> TsrHeap<TsrOrdered<T>> {
    fn key_iter(&self) -> impl Iterator<Item=&T> { self.values.iter().map(|value| &value.0) }
}
