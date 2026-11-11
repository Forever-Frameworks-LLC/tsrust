// Owning standard iterators over existing array storage, without payload clones.
enum TsrIteratorCursor<T>{
    Heap(std::vec::IntoIter<T>),
    Inline(std::iter::Flatten<std::array::IntoIter<Option<T>,4>>),
    Numeric(std::iter::Take<std::array::IntoIter<T,4>>),
}
impl<T> Iterator for TsrIteratorCursor<T>{type Item=T;fn next(&mut self)->Option<T>{match self{Self::Heap(values)=>values.next(),Self::Inline(values)=>values.next(),Self::Numeric(values)=>values.next()}}}
trait TsrIntoIteratorCursor<T>{fn tsr_into_iterator_cursor(self)->TsrIteratorCursor<T>;}
impl<T> TsrIntoIteratorCursor<T> for Vec<T>{fn tsr_into_iterator_cursor(self)->TsrIteratorCursor<T>{TsrIteratorCursor::Heap(self.into_iter())}}
