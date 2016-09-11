use super::{Node, Ref, RevdepForwarder, UpdateableNode};

use std::ops::*;

macro_rules! create_node_for_binary_op {
    ($name:ident, $ty:ident, $method:ident,) => (
        pub struct $name<LhsNode: Node, RhsNode: Node>
            where LhsNode::Output: $ty<RhsNode::Output>
        {
            lhs: Ref<LhsNode>,
            rhs: Ref<RhsNode>,
        }

        impl<LhsNode: Node, RhsNode: Node> Node for $name<LhsNode, RhsNode>
            where LhsNode::Output: $ty<RhsNode::Output>
        {
            type Output = <LhsNode::Output as $ty<RhsNode::Output>>::Output;

            fn eval(&self) -> Self::Output {
                self.lhs.eval().$method(self.rhs.eval())
            }
        }

        impl<LhsNode: Node, RhsNode: Node> $name<LhsNode, RhsNode>
            where LhsNode::Output: $ty<RhsNode::Output>
        {
            pub fn new(lhs: Ref<LhsNode>, rhs: Ref<RhsNode>) -> Ref<$name<LhsNode, RhsNode>> {
                Ref::new($name {
                    lhs: lhs,
                    rhs: rhs,
                })
            }
        }

        impl<LhsNode: Node, RhsNode: Node> RevdepForwarder for $name<LhsNode, RhsNode>
            where LhsNode::Output: $ty<RhsNode::Output>,
                  LhsNode: RevdepForwarder,
                  RhsNode: RevdepForwarder
        {
            fn forward_add_revdep(&self, revdep: Ref<UpdateableNode>) {
                self.lhs.forward_add_revdep(revdep.clone());
                self.rhs.forward_add_revdep(revdep);
            }

            fn forward_remove_revdep(&self, revdep: Ref<UpdateableNode>) {
                self.lhs.forward_remove_revdep(revdep.clone());
                self.rhs.forward_remove_revdep(revdep);
            }
        }
    );

    ($name:ident, $ty:ident, $method:ident, $($name2:ident, $ty2:ident, $method2:ident,)*) => (
        create_node_for_binary_op!($name, $ty, $method,);
        create_node_for_binary_op!($($name2, $ty2, $method2,)*);
    );
}

create_node_for_binary_op!(
    AddNode, Add, add,
    SubNode, Sub, sub,
    MulNode, Mul, mul,
    DivNode, Div, div,
    RemNode, Rem, rem,
    ShlNode, Shl, shl,
    ShrNode, Shr, shr,
    BitAndNode, BitAnd, bitand,
    BitOrNode, BitOr, bitor,
    BitXorNode, BitXor, bitxor,
);

#[cfg(test)]
mod tests {
    use super::*;
    use ::{InputNode, Node, Ref, CachedNode};

    #[test]
    fn add_uncached() {
        let input = InputNode::new(1.0f32);
        let add = AddNode::new(input.clone(), Ref::new(2.0f32));

        assert_eq!(add.eval(), 3.0f32);

        input.set(2.0f32);

        assert_eq!(add.eval(), 4.0f32);
    }

    #[test]
    fn add_cached() {
        let input = InputNode::new(1.0f32);
        let add = CachedNode::new(AddNode::new(input.clone(), Ref::new(2.0f32)));

        assert_eq!(add.eval(), 3.0f32);

        input.set(2.0f32);

        assert_eq!(add.eval(), 4.0f32);
    }

    #[test]
    fn sub_cached() {
        let input = InputNode::new(1.0f32);
        let add = CachedNode::new(SubNode::new(input.clone(), Ref::new(2.0f32)));

        assert_eq!(add.eval(), -1.0f32);

        input.set(2.0f32);

        assert_eq!(add.eval(), 0.0f32);
    }

    #[test]
    fn shl_cached() {
        let input = InputNode::new(1u8);
        let add = CachedNode::new(ShlNode::new(input.clone(), Ref::new(1u8)));

        assert_eq!(add.eval(), 2u8);

        input.set(2u8);

        assert_eq!(add.eval(), 4u8);
    }
}
