use crate::geo::{Bbox, Vec2};

const LEAF_SIZE: usize = 128;
const MIN_BBOX_AREA: f64 = 1e-3;

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

    pub fn depth(&self) -> usize {
        self.root.depth()
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

    pub fn intersects(&self, e_bbox: Bbox) -> bool {
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
}
