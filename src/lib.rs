use std::cell::RefCell;

mod ops;

pub use ops::AddNode;

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

pub type Ref<T> = std::rc::Rc<T>;
pub type WeakRef<T> = std::rc::Weak<T>;

pub trait Node {
    type Output;

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

impl<T> RevdepForwarder for T
    where T: UpdatingNode
{
    fn forward_add_revdep(&self, revdep: Ref<UpdateableNode>) {
        self.add_revdep(revdep);
    }

    fn forward_remove_revdep(&self, revdep: Ref<UpdateableNode>) {
        self.remove_revdep(revdep);
    }
}

pub trait CachableNode: Node + RevdepForwarder {}
impl<T: Node + RevdepForwarder> CachableNode for T where T::Output: Clone {}

struct RevdepVec(Vec<WeakRef<UpdateableNode>>);

impl RevdepVec {
    fn new() -> RevdepVec {
        RevdepVec(Vec::new())
    }

    fn add_revdep(&mut self, revdep: Ref<UpdateableNode>) {
        self.0.push(Ref::downgrade(&revdep));
    }

    fn remove_revdep(&mut self, revdep: Ref<UpdateableNode>) {
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

    fn update_all(&mut self) {
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

pub struct CachedNode<T: CachableNode> {
    inner_node: RefCell<Ref<T>>,
    cached_value: RefCell<T::Output>,
    revdeps: RefCell<RevdepVec>,
}

impl<T: CachableNode> Node for CachedNode<T>
    where T::Output: Clone
{
    type Output = T::Output;

    fn eval(&self) -> Self::Output {
        self.cached_value.borrow().clone()
    }
}

impl<T: CachableNode> UpdateableNode for CachedNode<T> {
    fn update(&self) {
        *self.cached_value.borrow_mut() = self.inner_node.borrow().eval();

        self.revdeps.borrow_mut().update_all();
    }
}

impl<T: CachableNode> UpdatingNode for CachedNode<T>
    where T::Output: Clone
{
    fn add_revdep(&self, revdep: Ref<UpdateableNode>) {
        self.revdeps.borrow_mut().add_revdep(revdep);
    }

    fn remove_revdep(&self, revdep: Ref<UpdateableNode>) {
        self.revdeps.borrow_mut().remove_revdep(revdep);
    }
}

impl<T: CachableNode + 'static> CachedNode<T> {
    pub fn new(inner: Ref<T>) -> Ref<CachedNode<T>> {
        let value = inner.eval();

        let node = Ref::new(CachedNode {
            inner_node: RefCell::new(inner),
            cached_value: RefCell::new(value),
            revdeps: RefCell::new(RevdepVec::new()),
        });

        node.inner_node.borrow().forward_add_revdep(node.clone());

        node
    }
}

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
    pub fn new(value: T) -> Ref<InputNode<T>> {
        Ref::new(InputNode {
            value: RefCell::new(value),
            revdeps: RefCell::new(RevdepVec::new()),
        })
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let node1 = CachedNode::new(Ref::new(1.0f32));
        let cache_node = CachedNode::new(node1.clone());

        assert_eq!(node1.eval(), 1.0f32);
        assert_eq!(cache_node.eval(), 1.0f32);
    }

    #[test]
    fn input_nodes() {
        let input = InputNode::new(1.0f32);
        let cache = CachedNode::new(input.clone());

        assert_eq!(input.eval(), 1.0f32);
        assert_eq!(cache.eval(), 1.0f32);

        input.set(3.0f32);

        assert_eq!(input.eval(), 3.0f32);
        assert_eq!(cache.eval(), 3.0f32);
    }
}
