use std::cell::RefCell;

use core::*;

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

pub struct CachedNode<T: CachableNode> {
    inner_node: Ref<T>,
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
        *self.cached_value.borrow_mut() = self.inner_node.eval();

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
            inner_node: inner,
            cached_value: RefCell::new(value),
            revdeps: RefCell::new(RevdepVec::new()),
        });

        node.inner_node.forward_add_revdep(node.clone());

        node
    }
}

pub struct LazyCachedNode<T: CachableNode> {
    inner_node: Ref<T>,
    cached_value: RefCell<Option<T::Output>>,
    revdeps: RefCell<RevdepVec>,
}

impl<T: CachableNode> Node for LazyCachedNode<T>
    where T::Output: Clone
{
    type Output = T::Output;

    fn eval(&self) -> Self::Output {
        use std::ops::DerefMut;

        let mut cache_borrow = self.cached_value.borrow_mut();
        let cache = cache_borrow.deref_mut();

        if let &mut Some(ref cached) = cache {
            cached.clone()
        } else {
            let new = self.inner_node.eval();
            *cache = Some(new.clone());

            new
        }
    }
}

impl<T: CachableNode> UpdateableNode for LazyCachedNode<T> {
    fn update(&self) {
        *self.cached_value.borrow_mut() = None;

        self.revdeps.borrow_mut().update_all();
    }
}

impl<T: CachableNode> UpdatingNode for LazyCachedNode<T>
    where T::Output: Clone
{
    fn add_revdep(&self, revdep: Ref<UpdateableNode>) {
        self.revdeps.borrow_mut().add_revdep(revdep);
    }

    fn remove_revdep(&self, revdep: Ref<UpdateableNode>) {
        self.revdeps.borrow_mut().remove_revdep(revdep);
    }
}

impl<T: CachableNode + 'static> LazyCachedNode<T> {
    pub fn new(inner: Ref<T>) -> Ref<LazyCachedNode<T>> {
        let node = Ref::new(LazyCachedNode {
            inner_node: inner,
            cached_value: RefCell::new(None),
            revdeps: RefCell::new(RevdepVec::new()),
        });

        node.inner_node.forward_add_revdep(node.clone());

        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{Node, Ref, InputNode};

    #[test]
    fn basic_constant_cache() {
        let node = CachedNode::new(Ref::new(1.0f32));
        let cache_node = CachedNode::new(node.clone());

        assert_eq!(node.eval(), 1.0f32);
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

    #[test]
    fn lazy_cache() {
        let input = InputNode::new(1.0f32);
        let cache = LazyCachedNode::new(input.clone());

        assert_eq!(input.eval(), 1.0f32);
        assert_eq!(cache.eval(), 1.0f32);

        input.set(3.0f32);

        assert_eq!(input.eval(), 3.0f32);
        assert_eq!(cache.eval(), 3.0f32);
    }
}
