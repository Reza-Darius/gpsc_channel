use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    hash::Hash,
};

/// Container trait used by container for the gpsc channel
#[allow(clippy::len_without_is_empty)]
pub trait GpscContainer
where
    Self: Sized,
{
    type Message;

    /// if your collection isnt dynamically resizable, then the passed container to [take] on the receiver must match the cap
    fn new(cap: usize) -> Self;

    fn len(&self) -> usize;

    fn clear(&mut self);

    fn insert(&mut self, msg: Self::Message);
}

impl<T> GpscContainer for Vec<T> {
    type Message = T;

    fn new(cap: usize) -> Self {
        Self::with_capacity(cap)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn insert(&mut self, msg: Self::Message) {
        self.push(msg);
    }
}

impl<T> GpscContainer for VecDeque<T> {
    type Message = T;

    fn new(cap: usize) -> Self {
        Self::with_capacity(cap)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn insert(&mut self, msg: Self::Message) {
        self.push_back(msg);
    }
}

impl<K, V> GpscContainer for HashMap<K, V>
where
    K: Eq + Hash,
{
    type Message = (K, V);

    fn new(_: usize) -> Self {
        Self::new()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn insert(&mut self, msg: Self::Message) {
        HashMap::insert(self, msg.0, msg.1);
    }
}

impl<K, V> GpscContainer for BTreeMap<K, V>
where
    K: Eq + Hash + Ord,
{
    type Message = (K, V);

    fn new(_: usize) -> Self {
        Self::new()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn insert(&mut self, msg: Self::Message) {
        BTreeMap::insert(self, msg.0, msg.1);
    }
}
