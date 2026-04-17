use std::collections::VecDeque;

pub trait BatchChanContainer
where
    Self: Sized,
{
    type Message;

    fn new(cap: usize) -> Self;

    fn len(&self) -> usize;

    fn clear(&mut self);

    fn insert(&mut self, msg: Self::Message);
}

impl<T> BatchChanContainer for Vec<T> {
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

impl<T> BatchChanContainer for VecDeque<T> {
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
