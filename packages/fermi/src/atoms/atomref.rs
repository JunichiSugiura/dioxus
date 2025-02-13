use crate::{AtomId, AtomRoot, Readable};
use std::cell::RefCell;

pub struct AtomRefBuilder;
pub type AtomRef<T> = fn(AtomRefBuilder) -> T;

impl<V> Readable<RefCell<V>> for AtomRef<V> {
    fn read(&self, _root: AtomRoot) -> Option<RefCell<V>> {
        todo!()
    }

    fn init(&self) -> RefCell<V> {
        RefCell::new((*self)(AtomRefBuilder))
    }

    fn unique_id(&self) -> AtomId {
        *self as *const ()
    }
}

#[test]
fn atom_compiles() {
    static TEST_ATOM: AtomRef<Vec<String>> = |_| vec![];
    dbg!(TEST_ATOM.init());
}
