use crate::{
    entry::{Entry, EntryMut},
    EytzingerTree, Node, NodeMut,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum WalkAction {
    Stop,
    Parent,
    Child(usize),
}

pub trait Walkable {
    type Item;

    fn walk<H>(&self, handler: &mut H)
    where
        H: WalkHandler<Item = Self::Item>;
}
pub trait WalkHandler {
    type Item;

    fn on_node(&mut self, entry: &Entry<Self::Item>) -> WalkAction;
}

impl<'a, N> Walkable for Entry<'a, N> {
    type Item = N;

    fn walk<H>(&self, handler: &mut H)
    where
        H: WalkHandler<Item = Self::Item>,
    {
        use self::WalkAction::*;
        use crate::entry::Entry::*;

        let mut current = *self;
        loop {
            match handler.on_node(&current) {
                Stop => break,
                Parent => {
                    if let Some(parent) = self.parent() {
                        current = Occupied(parent);
                    } else {
                        break;
                    }
                }
                Child(index) => match current {
                    Occupied(node) => current = node.child_entry(index),
                    Vacant(_) => break,
                },
            }
        }
    }
}

impl<'a, N> Walkable for Node<'a, N> {
    type Item = N;

    fn walk<H>(&self, handler: &mut H)
    where
        H: WalkHandler<Item = Self::Item>,
    {
        Entry::Occupied(*self).walk(handler)
    }
}

impl<'a, N> Walkable for NodeMut<'a, N> {
    type Item = N;

    fn walk<H>(&self, handler: &mut H)
    where
        H: WalkHandler<Item = Self::Item>,
    {
        self.as_node().walk(handler)
    }
}

impl<'a, N> Walkable for EntryMut<'a, N> {
    type Item = N;

    fn walk<H>(&self, handler: &mut H)
    where
        H: WalkHandler<Item = Self::Item>,
    {
        Entry::from(self).walk(handler)
    }
}

impl<N> Walkable for EytzingerTree<N> {
    type Item = N;

    fn walk<H>(&self, handler: &mut H)
    where
        H: WalkHandler<Item = Self::Item>,
    {
        self.root_entry().walk(handler)
    }
}

pub trait WalkableMut {
    type Item;

    fn walk_mut<H>(self, handler: &mut H) -> Self
    where
        H: WalkMutHandler<Item = Self::Item>;
}

pub trait WalkMutHandler {
    type Item;

    fn on_node(&mut self, entry: &mut EntryMut<Self::Item>) -> WalkAction;
}

impl<'a, N> WalkableMut for EntryMut<'a, N> {
    type Item = N;

    fn walk_mut<H>(self, handler: &mut H) -> Self
    where
        H: WalkMutHandler<Item = Self::Item>,
    {
        use self::WalkAction::*;
        use crate::entry::EntryMut::*;

        let mut levels = 0usize;
        let mut current = self;
        loop {
            match handler.on_node(&mut current) {
                Parent => {
                    if levels == 0 {
                        break;
                    }

                    match current.to_parent() {
                        Ok(parent) => {
                            current = parent.into();
                            levels -= 1;
                        }
                        Err(x) => {
                            current = x;
                            break;
                        }
                    }
                }
                Child(index) => match current {
                    Occupied(node) => {
                        current = node.to_child_entry(index);
                        levels += 1;
                    }
                    Vacant(_) => break,
                },
                Stop => break,
            }
        }

        for _ in 0..levels {
            current = current
                .to_parent()
                .ok()
                .expect("should have a parent")
                .into()
        }
        current
    }
}

impl<'a, N> WalkableMut for &'a mut EytzingerTree<N> {
    type Item = N;

    fn walk_mut<H>(self, handler: &mut H) -> Self
    where
        H: WalkMutHandler<Item = Self::Item>,
    {
        self.root_entry_mut().walk_mut(handler);
        self
    }
}
