use super::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct FakeTree {
    pub responsible: HashMap<i32, i32>,
    pub parent: HashMap<i32, i32>,
}

impl ProcessTree for FakeTree {
    fn responsible(&self, pid: i32) -> Option<i32> {
        self.responsible.get(&pid).copied()
    }

    fn parent(&self, pid: i32) -> Option<i32> {
        self.parent.get(&pid).copied()
    }
}
