use std::cell::RefCell;

macro_rules! impl_node_for {
    ($ty:ty) => (
        impl Node for $ty {
            type Output = $ty;

            fn eval(&self) -> Self::Output {
                *self
            }
        }

        impl RevdepForwarder for $ty {
            fn forward_add_revdep(&self, _revdep: Ref<UpdateableNode>) {
            }
            fn forward_remove_revdep(&self, _revdep: Ref<UpdateableNode>) {
            }
        }
    );

    ($ty:ty, $($rest:ty),*) => (
        impl_node_for!($ty);
        impl_node_for!($($rest),*);
    );
}

/// The type of reference used for `Node`s.
pub type Ref<T> = ::std::rc::Rc<T>;
pub type WeakRef<T> = ::std::rc::Weak<T>;

/// The `Node` trait forms the basis for all expressions in this crate.
///
/// # Examples
///
/// ```
/// use exprs::{Ref, Node};
/// 
/// let node: Ref<Node<Output=f32>> = Ref::new(1.0f32);
/// let node2 = Ref::new(1.0f32);
///
/// assert_eq!(node.eval(), node2.eval());
/// ```
pub trait Node {
    /// The type of values produced by this `Node`.
    type Output;

    /// Evaluates the `Node`, resulting in a value of type `Output`.
    fn eval(&self) -> Self::Output;
}

impl_node_for!(bool,
               u8,
               u16,
               u32,
               u64,
               usize,
               i8,
               i16,
               i32,
               i64,
               isize,
               f32,
               f64);

pub trait UpdateableNode {
    fn update(&self);
}

pub trait UpdatingNode: Node {
    fn add_revdep(&self, revdep: Ref<UpdateableNode>);
    fn remove_revdep(&self, revdep: Ref<UpdateableNode>);
}

pub trait RevdepForwarder {
    fn forward_add_revdep(&self, revdep: Ref<UpdateableNode>);
    fn forward_remove_revdep(&self, revdep: Ref<UpdateableNode>);
}

pub struct RevdepVec(Vec<WeakRef<UpdateableNode>>);

impl RevdepVec {
    pub fn new() -> RevdepVec {
        RevdepVec(Vec::new())
    }

    pub fn add_revdep(&mut self, revdep: Ref<UpdateableNode>) {
        self.0.push(Ref::downgrade(&revdep));
    }

    pub fn remove_revdep(&mut self, revdep: Ref<UpdateableNode>) {
        use std::ops::Deref;

        let needle = revdep.deref() as *const UpdateableNode;

        self.0.retain(|weak| {
            let strong = match weak.upgrade() {
                None => return false,
                Some(r) => r,
            };

            if strong.deref() as *const UpdateableNode == needle {
                false
            } else {
                true
            }
        });
    }

    pub fn update_all(&mut self) {
        self.0.retain(|weak| {
            if let Some(revdep) = weak.upgrade() {
                revdep.update();
                true
            } else {
                false
            }
        });
    }
}

/// An `InputNode`. Contains a value that can be changed.
///
/// # Examples
///
/// ```
/// use exprs::{Ref, Node, InputNode};
/// 
/// let input = InputNode::new(100u8);
/// assert_eq!(input.eval(), 100u8);
///
/// input.set(101u8);
/// assert_eq!(input.eval(), 101u8);
/// ```
pub struct InputNode<T: Clone> {
    value: RefCell<T>,
    revdeps: RefCell<RevdepVec>,
}

impl<T: Clone> Node for InputNode<T> {
    type Output = T;

    fn eval(&self) -> Self::Output {
        self.value.borrow().clone()
    }
}

impl<T: Clone> InputNode<T> {
    /// Create a new `InputNode` given an initial input value.
    pub fn new(value: T) -> Ref<InputNode<T>> {
        Ref::new(InputNode {
            value: RefCell::new(value),
            revdeps: RefCell::new(RevdepVec::new()),
        })
    }

    /// Change the value contained in this `InputNode`.
    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;

        self.revdeps.borrow_mut().update_all();
    }
}

impl<T: Clone> UpdatingNode for InputNode<T> {
    fn add_revdep(&self, revdep: Ref<UpdateableNode>) {
        self.revdeps.borrow_mut().add_revdep(revdep);
    }
    fn remove_revdep(&self, revdep: Ref<UpdateableNode>) {
        self.revdeps.borrow_mut().remove_revdep(revdep);
    }
}
