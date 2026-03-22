
use std::sync::{Arc, RwLock};

pub struct Sender<T> {
    inner: Arc<RwLock<Option<T>>>,
}

pub struct Receiver<T> {
    inner: Arc<RwLock<Option<T>>>,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner_for_sender = Arc::new(RwLock::new(Option::None));
    let inner_for_receiver = Arc::clone(&inner_for_sender);
    (
        Sender{inner: inner_for_sender},
        Receiver{inner: inner_for_receiver},
    )
}

impl<T> Sender<T> {
    pub fn send(&self, t: T) {
        *self.inner.write().unwrap() = Option::Some(t);
    }
}

impl<T: Clone> Receiver<T> {
    pub fn recv(&self) -> Option<T> {

        // read value
        let maybe_t = self.inner.read().expect("Failed to aquire read guard!").clone();

        // replace value
        *self.inner.write().expect("Failed to acquire write guard!") = Option::None;

        return maybe_t;
    }
}