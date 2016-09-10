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

pub trait CachableNode: Node + RevdepForwarder {
}

impl<T: Node + RevdepForwarder> CachableNode for T
    where T::Output: Clone
{
}

pub struct CachedNode<T: CachableNode>
{
    inner_node: RefCell<Ref<T>>,
    cached_value: RefCell<T::Output>,
    revdeps: RefCell<Vec<WeakRef<UpdateableNode>>>,
}

impl<T: CachableNode> Node for CachedNode<T>
    where T::Output: Clone
{
    type Output = T::Output;

    fn eval(&self) -> Self::Output {
        self.cached_value.borrow().clone()
    }
}

impl<T: CachableNode> UpdateableNode for CachedNode<T>
{
    fn update(&self) {
        *self.cached_value.borrow_mut() = self.inner_node.borrow().eval();

        self.revdeps.borrow_mut().retain(|weak| {
            if let Some(revdep) = weak.upgrade() {
                revdep.update();
                true
            } else {
                false
            }
        });
    }
}

impl<T: CachableNode> UpdatingNode for CachedNode<T>
    where T::Output: Clone
{
    fn add_revdep(&self, revdep: Ref<UpdateableNode>) {
        self.revdeps.borrow_mut().push(Ref::downgrade(&revdep));
    }

    fn remove_revdep(&self, revdep: Ref<UpdateableNode>) {
        use std::ops::Deref;

        let needle = revdep.deref() as *const UpdateableNode;

        self.revdeps.borrow_mut().retain(|weak| {
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
}

impl<T: CachableNode> RevdepForwarder for CachedNode<T>
    where T::Output: Clone
{
    fn forward_add_revdep(&self, revdep: Ref<UpdateableNode>) {
        self.add_revdep(revdep);
    }

    fn forward_remove_revdep(&self, revdep: Ref<UpdateableNode>) {
        self.remove_revdep(revdep);
    }
}

impl<T: CachableNode + 'static> CachedNode<T>
{
    pub fn new(inner: Ref<T>) -> Ref<CachedNode<T>> {
        let value = inner.eval();

        let node = Ref::new(CachedNode {
            inner_node: RefCell::new(inner),
            cached_value: RefCell::new(value),
            revdeps: RefCell::new(Vec::new()),
        });

        node.inner_node.borrow().forward_add_revdep(node.clone());

        node
    }
}

pub trait RefCachedNodeExt<T: CachableNode + 'static> {
    fn set_inner(&self, new_inner: Ref<T>);
}

impl<T: CachableNode + 'static> RefCachedNodeExt<T> for Ref<CachedNode<T>> {
    fn set_inner(&self, new_inner: Ref<T>) {
        {
            let mut inner_node = self.inner_node.borrow_mut();
            inner_node.forward_remove_revdep(self.clone());
            *inner_node = new_inner;
            inner_node.forward_add_revdep(self.clone());
        }

        self.update();
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

        node1.set_inner(Ref::new(2.0f32));

        assert_eq!(node1.eval(), 2.0f32);
        assert_eq!(cache_node.eval(), 2.0f32);

        cache_node.set_inner(CachedNode::new(Ref::new(4.0f32)));

        node1.set_inner(Ref::new(3.0f32));

        assert_eq!(node1.eval(), 3.0f32);
        assert_eq!(cache_node.eval(), 4.0f32);

    }
}
