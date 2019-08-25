use crate::geo::{Bbox, Vec2};

const LEAF_SIZE: usize = 128;
const MIN_BBOX_AREA: f64 = 1e0;

#[derive(Debug)]
pub struct Bvh<Elem> {
    root: BvhNode<Elem>,
}

#[derive(Debug)]
pub enum BvhNode<Elem> {
    Leaf {
        elems: Vec<(Elem, Bbox)>,
        bbox: Bbox,
    },
    Branch {
        bbox: Bbox,
        children: Box<[BvhNode<Elem>; 4]>,
    },
}

impl<Elem: Copy> Bvh<Elem> {
    pub fn new(bbox: Bbox) -> Self {
        Bvh {
            root: BvhNode::Leaf {
                elems: Vec::with_capacity(LEAF_SIZE),
                bbox,
            },
        }
    }

    /// Depth of the Bvh. _Not_ O(1).
    pub fn depth(&self) -> usize {
        self.root.depth()
    }

    /// Length of the Bvh (potentially greater than the number of distinct elements). _Not_ O(1).
    pub fn len(&self) -> usize {
        self.root.len()
    }

    pub fn insert(&mut self, e: Elem, bbox: Bbox) {
        self.root.insert(e, bbox);
    }

    pub fn remove(&mut self, e: &Elem, bbox: Bbox)
    where
        Elem: Eq,
    {
        self.root.remove(e, bbox)
    }

    /// Return all the elements that contain the given refpoint. Might return the same elemnt
    /// multiple times.
    pub fn enclosing(
        &self,
        refpoint: Vec2,
        contains: impl Fn(&Elem, Vec2) -> bool,
    ) -> impl Iterator<Item = &Elem> {
        self.root.enclosing(refpoint, contains)
    }
}

impl<Elem: Copy> BvhNode<Elem> {
    fn split(elems: &mut Vec<(Elem, Bbox)>, bbox: &mut Bbox) -> Self {
        let pivot = bbox.center();
        let quads = bbox.split(pivot);

        let mut children = Box::new([
            BvhNode::Leaf {
                elems: Vec::with_capacity(LEAF_SIZE),
                bbox: quads[0],
            },
            BvhNode::Leaf {
                elems: Vec::with_capacity(LEAF_SIZE),
                bbox: quads[1],
            },
            BvhNode::Leaf {
                elems: Vec::with_capacity(LEAF_SIZE),
                bbox: quads[2],
            },
            BvhNode::Leaf {
                elems: Vec::with_capacity(LEAF_SIZE),
                bbox: quads[3],
            },
        ]);

        for (e, e_bbox) in elems.drain(0..) {
            for child in children.iter_mut() {
                if child.intersects(e_bbox) {
                    child.insert(e, e_bbox);
                }
            }
        }

        BvhNode::Branch {
            children,
            bbox: *bbox,
        }
    }

    pub fn insert(&mut self, e: Elem, e_bbox: Bbox) {
        match self {
            BvhNode::Leaf { elems, bbox } => {
                elems.push((e, e_bbox));

                if elems.len() > LEAF_SIZE && bbox.area() > MIN_BBOX_AREA {
                    *self = BvhNode::split(elems, bbox);
                }
            }
            BvhNode::Branch { children, .. } => {
                for child in children.iter_mut() {
                    if child.intersects(e_bbox) {
                        child.insert(e, e_bbox);
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, e: &Elem, bbox: Bbox)
    where
        Elem: Eq,
    {
        match self {
            BvhNode::Leaf { elems, .. } => {
                elems.retain(|(ee, _)| ee != e);
            }
            BvhNode::Branch { children, .. } => {
                for child in children.iter_mut() {
                    if child.intersects(bbox) {
                        child.remove(e, bbox);
                    }
                }
            }
        }
    }

    pub fn enclosing(
        &self,
        query_point: Vec2,
        contains: impl Fn(&Elem, Vec2) -> bool,
    ) -> impl Iterator<Item = &Elem> {
        let mut nodes = vec![self];
        let mut cur_elems = [].iter();

        std::iter::from_fn(move || loop {
            for (e, _) in cur_elems.by_ref() {
                if contains(e, query_point) {
                    return Some(e);
                }
            }

            match nodes.pop()? {
                BvhNode::Leaf { elems, .. } => cur_elems = elems.iter(),
                BvhNode::Branch { children, bbox } => {
                    if bbox.contains(query_point) {
                        nodes.extend(children.iter());
                    }
                }
            }
        })
    }

    fn intersects(&self, e_bbox: Bbox) -> bool {
        let bbox = match self {
            BvhNode::Branch { bbox, .. } | BvhNode::Leaf { bbox, .. } => bbox,
        };

        bbox.intersection(e_bbox).is_some()
    }

    pub fn depth(&self) -> usize {
        match self {
            BvhNode::Leaf { .. } => 1,
            BvhNode::Branch { children, .. } => {
                children.iter().map(BvhNode::depth).max().unwrap() + 1
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            BvhNode::Leaf { elems, .. } => elems.len(),
            BvhNode::Branch { children, .. } => children.iter().map(BvhNode::len).sum(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    #[test]
    fn test_basic() {
        let mut bbox = Bbox::new(Vec2::zero());
        bbox.enlarge(LEAF_SIZE as f64 * 20.0);

        let mut bvh = Bvh::new(bbox);

        for y in 0..LEAF_SIZE {
            for x in 0..LEAF_SIZE {
                let ip = (x, y);

                let x = x as f64 * 20.0;
                let y = y as f64 * 20.0;

                let mut b = Bbox::new(Vec2::new(x, y));
                b.enlarge(10.0);

                bvh.insert(ip, b);

                assert!(bvh
                    .enclosing(b.center(), |(x, y), p| {
                        let mut b = Bbox::new(Vec2::new(*x as f64, *y as f64));
                        b.enlarge(10.0);
                        b.contains(p)
                    })
                    .all(|ipp| *ipp == ip));
            }
        }

        let mut b = Bbox::new(Vec2::zero());
        b.enlarge(10.0);
        bvh.remove(&(0, 0), b);
        assert!(!bvh
            .enclosing(b.center(), |(x, y), p| {
                let mut b = Bbox::new(Vec2::new(*x as f64, *y as f64));
                b.enlarge(10.0);
                b.contains(p)
            })
            .any(|ipp| *ipp == (0, 0)));
    }

    proptest! {
        #[test]
        fn prop_enclosing_gives_same_result_as_bruteforce(
            pts in prop::collection::vec((0_u32..30_000, 0_u32..30_000), 1..200),
            to_search in prop::collection::vec((0_u32..30_000, 0_u32..30_000), 1..200),
        ) {
            use std::collections::HashSet;

            let mut bbox = Bbox::new(Vec2::new(f64::from(pts[0].0), f64::from(pts[0].1)));
            for &(x, y) in &pts[1..] {
                bbox.expand(Vec2::new(x.into(), y.into()));
            }

            let mut bvh = Bvh::new(bbox);
            for &(x, y) in &pts {
                let mut bbox = Bbox::new(Vec2::new(x.into(), y.into()));
                bbox.enlarge(5.0);

                bvh.insert((x, y), bbox);
            }

            let to_search = pts.iter().chain(to_search.iter());
            for &(x, y) in to_search {
                let pf = Vec2::new(x.into(), y.into());

                let mut b = Bbox::new(pf);
                b.enlarge(5.0);

                let enclosing = bvh
                    .enclosing(Vec2::new(x.into(), y.into()), |&(x, y), p| {
                        let mut b = Bbox::new(Vec2::new(x.into(), y.into()));
                        b.enlarge(5.0);
                        b.contains(p)
                    })
                    .collect::<HashSet<_>>();

                let brute_force_enclosing = pts
                    .iter()
                    .filter(|&&(x, y)| {
                        let mut b = Bbox::new(Vec2::new(x.into(), y.into()));
                        b.enlarge(5.0);
                        b.contains(pf)
                    })
                    .collect::<HashSet<_>>();

                prop_assert_eq!(enclosing, brute_force_enclosing);
            }
        }

    }
}
