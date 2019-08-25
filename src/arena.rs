use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Arena<T> {
    data: Vec<Node<T>>,
    first_free: Option<usize>,
}

#[derive(Debug)]
enum Node<T> {
    Free { next_free: Option<usize> },
    Occupied(T),
}

#[derive(Debug)]
pub struct ArenaId<Tag> {
    ix: usize,
    tag: std::marker::PhantomData<Tag>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena {
            data: vec![],
            first_free: None,
        }
    }

    pub fn push(&mut self, v: T) -> ArenaId<T> {
        match self.first_free {
            None => {
                self.data.push(Node::Occupied(v));
                ArenaId::new(self.data.len() - 1)
            }
            Some(f) => {
                match self.data[f] {
                    Node::Free { next_free } => {
                        self.first_free = next_free;
                    }
                    Node::Occupied(_) => panic!("bug"),
                };

                self.data[f] = Node::Occupied(v);
                ArenaId::new(f)
            }
        }
    }

    pub fn remove(&mut self, id: ArenaId<T>) -> Option<T> {
        match self.data.get_mut(id.ix)? {
            cell @ Node::Occupied(_) => {
                let mut out = Node::Free {
                    next_free: self.first_free,
                };

                std::mem::swap(&mut out, cell);
                self.first_free = Some(id.ix);

                match out {
                    Node::Occupied(t) => Some(t),
                    Node::Free { .. } => unreachable!(),
                }
            }
            Node::Free { .. } => None,
        }
    }

    pub fn get(&self, id: ArenaId<T>) -> Option<&T> {
        match self.data.get(id.ix)? {
            Node::Occupied(t) => Some(t),
            Node::Free { .. } => None,
        }
    }

    pub fn get_mut(&mut self, id: ArenaId<T>) -> Option<&mut T> {
        match self.data.get_mut(id.ix)? {
            Node::Occupied(t) => Some(t),
            Node::Free { .. } => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.enumerate().map(|(_, n)| n)
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (ArenaId<T>, &T)> {
        self.data.iter().enumerate().filter_map(|(i, n)| match n {
            Node::Occupied(t) => Some((ArenaId::new(i), t)),
            Node::Free { .. } => None,
        })
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena::new()
    }
}

impl<T> Index<ArenaId<T>> for Arena<T> {
    type Output = T;

    fn index(&self, ix: ArenaId<T>) -> &Self::Output {
        self.get(ix).unwrap()
    }
}

impl<T> IndexMut<ArenaId<T>> for Arena<T> {
    fn index_mut(&mut self, ix: ArenaId<T>) -> &mut T {
        self.get_mut(ix).unwrap()
    }
}

impl<Tag> ArenaId<Tag> {
    fn new(ix: usize) -> Self {
        ArenaId {
            ix,
            tag: PhantomData,
        }
    }
}

impl<T> Copy for ArenaId<T> {}
impl<T> Clone for ArenaId<T> {
    fn clone(&self) -> Self {
        ArenaId {
            ix: self.ix,
            tag: self.tag,
        }
    }
}

impl<T> Eq for ArenaId<T> {}
impl<T> PartialEq for ArenaId<T> {
    fn eq(&self, rhs: &ArenaId<T>) -> bool {
        self.ix == rhs.ix
    }
}

impl<T> Hash for ArenaId<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.ix.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut arena: Arena<u32> = Arena::new();
        assert!(arena.get(ArenaId::new(0)).is_none());

        let id42 = arena.push(42);
        assert_eq!(arena[id42], 42);

        let id73 = arena.push(73);
        arena.remove(id42);
        assert!(arena.get(id42).is_none());
        assert_eq!(arena[id73], 73);

        let id0 = arena.push(0);
        assert_eq!(arena[id73], 73);
        assert_eq!(arena[id0], 0);

        arena[id0] = 69;
        assert_eq!(arena[id73], 73);
        assert_eq!(arena[id0], 69);
    }

    #[test]
    fn test_iterators() {
        let mut arena = Arena::new();
        arena.push(42);
        arena.push(73);
        arena.push(0);

        assert_eq!(arena.iter().collect::<Vec<_>>(), vec![&42, &73, &0]);

        for (i, e) in arena.enumerate() {
            assert_eq!(&arena[i], e);
        }
    }

}
