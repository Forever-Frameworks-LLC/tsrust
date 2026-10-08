//! Ordered prefixes and repeated tails bound array provenance at loop joins.
const MAX_PREFIX: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Sequence<T> {
    pub(super) prefix: Vec<T>,
    pub(super) tail: Option<T>,
}

#[derive(Clone, Debug)]
pub(super) struct Removal<T> { pub(super) present:Option<T>, pub(super) may_be_empty:bool }

impl<T: Clone> Sequence<T> {
    pub(super) fn exact(values: Vec<T>) -> Self { Self { prefix: values, tail: None } }
    pub(super) fn exact_values(&self)->Option<&[T]> {self.tail.is_none().then_some(self.prefix.as_slice())}
    pub(super) fn iter(&self)->impl Iterator<Item=&T> {self.prefix.iter().chain(self.tail.iter())}
    pub(super) fn get(&self,index:usize)->Option<&T> {self.prefix.get(index).or(self.tail.as_ref())}
    pub(super) fn map<U>(&self,convert:impl Fn(&T)->U)->Sequence<U> {Sequence {prefix:self.prefix.iter().map(&convert).collect(),tail:self.tail.as_ref().map(convert)}}
    pub(super) fn join(&self, incoming:&Self, merge:impl Fn(&T,&T)->T)->Self {
        if self.tail.is_none() && incoming.tail.is_none() && self.prefix.len()==incoming.prefix.len() {
            return Self::exact(self.prefix.iter().zip(&incoming.prefix).map(|(a,b)|merge(a,b)).collect());
        }
        let count=self.prefix.len().min(incoming.prefix.len()).min(MAX_PREFIX);
        let prefix=self.prefix[..count].iter().zip(&incoming.prefix[..count]).map(|(a,b)|merge(a,b)).collect();
        let mut tail=None;
        for value in self.prefix[count..].iter().chain(incoming.prefix[count..].iter()).chain(self.tail.iter()).chain(incoming.tail.iter()) {
            tail=Some(tail.map_or_else(||value.clone(),|previous|merge(&previous,value)));
        }
        Self {prefix,tail}
    }
    pub(super) fn push(&mut self, value:T, merge:impl Fn(&T,&T)->T) {
        if let Some(tail)=&mut self.tail {*tail=merge(tail,&value);} else {self.prefix.push(value);}
    }
    pub(super) fn unshift(&mut self, value:T, merge:impl Fn(&T,&T)->T) {
        self.prefix.insert(0,value);
        if self.tail.is_some() && self.prefix.len()>MAX_PREFIX {
            let displaced=self.prefix.split_off(MAX_PREFIX);
            for value in displaced {let tail=self.tail.as_mut().unwrap();*tail=merge(tail,&value);}
        }
    }
    pub(super) fn shift(&mut self)->Removal<T> {
        if !self.prefix.is_empty() {return Removal {present:Some(self.prefix.remove(0)),may_be_empty:false};}
        Removal {present:self.tail.clone(),may_be_empty:true}
    }
    pub(super) fn pop(&mut self, merge:impl Fn(&T,&T)->T)->Removal<T> {
        let may_be_empty=self.prefix.is_empty();
        let Some(tail)=self.tail.clone() else {return Removal {present:self.prefix.pop(),may_be_empty};};
        let result=if let Some(last)=self.prefix.pop() {
            let combined=merge(&last,&tail);self.tail=Some(combined.clone());combined
        } else {tail};
        Removal {present:Some(result),may_be_empty}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn merge(a:&u8,b:&u8)->u8 {*a|*b}
    fn admits(shape:&Sequence<u8>,values:&[u8])->bool {
        values.len()>=shape.prefix.len()
            && shape.prefix.iter().zip(values).all(|(allowed,value)|allowed&value==*value)
            && if let Some(tail)=shape.tail {values[shape.prefix.len()..].iter().all(|value|tail&value==*value)} else {values.len()==shape.prefix.len()}
    }
    #[test]
    fn different_lengths_keep_ordered_prefix_and_all_tail_values() {
        let short=Sequence::exact(vec![1]);let long=Sequence::exact(vec![1,2,4]);
        let joined=short.join(&long,merge);
        assert!(admits(&joined,&[1]));assert!(admits(&joined,&[1,2,4]));
        assert_eq!(joined.prefix,vec![1]);assert_eq!(joined.tail,Some(6));
        assert_eq!(joined,long.join(&short,merge));
    }
    fn words()->Vec<Vec<u8>> {
        let mut result=vec![vec![]];
        for length in 1..=4 {for bits in 0..(1<<length) {result.push((0..length).map(|index|if bits&(1<<index)==0 {1}else{2}).collect());}}
        result
    }
    #[test]
    fn joined_transitions_cover_every_concrete_result_and_empty_case() {
        let words=words();
        for a in &words {for b in &words {
            let shape=Sequence::exact(a.clone()).join(&Sequence::exact(b.clone()),merge);
            for concrete in [a,b] {
                assert!(admits(&shape,concrete));
                for front in [true,false] {
                    let mut observed=concrete.clone();let actual=if front {if observed.is_empty(){None}else{Some(observed.remove(0))}} else {observed.pop()};
                    let mut abstracted=shape.clone();let result=if front {abstracted.shift()}else{abstracted.pop(merge)};
                    assert!(admits(&abstracted,&observed),"{shape:?} {concrete:?}");
                    if let Some(actual)=actual {assert_eq!(result.present.unwrap()&actual,actual);}else{assert!(result.may_be_empty);}
                }
                for front in [true,false] {
                    let mut observed=concrete.clone();let mut abstracted=shape.clone();
                    if front {observed.insert(0,4);abstracted.unshift(4,merge);}else{observed.push(4);abstracted.push(4,merge);}
                    assert!(admits(&abstracted,&observed));
                }
            }
        }}
    }
    #[test]
    fn repeated_loop_growth_converges_and_keeps_prefix_order() {
        let mut state=Sequence::exact(vec![1]);
        for _ in 0..1024 {
            let mut next=state.clone();next.push(2,merge);state=state.join(&next,merge);
            assert!(state.prefix.len()<=MAX_PREFIX);
        }
        let before=state.clone();let mut next=state.clone();next.push(2,merge);
        assert_eq!(state.join(&next,merge),before);
        assert_eq!(state.prefix,vec![1]);assert_eq!(state.tail,Some(2));
        assert_eq!(state.shift().present,Some(1));
        assert_eq!(state.shift().present,Some(2));
        assert!(state.shift().may_be_empty);
    }
    #[test]
    fn joins_commute_and_do_not_forget_an_admitted_sequence() {
        let words=words();
        for a in &words {for b in &words {
            let left=Sequence::exact(a.clone());let right=Sequence::exact(b.clone());
            let joined=left.join(&right,merge);
            assert_eq!(joined,right.join(&left,merge));assert_eq!(joined,joined.join(&joined,merge));
            for c in &words {
                if admits(&joined,c) {assert!(admits(&joined.join(&Sequence::exact(vec![4]),merge),c));}
            }
        }}
    }
}
