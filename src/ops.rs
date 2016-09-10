use ::{Node, Ref, RevdepForwarder, UpdateableNode};

use std::ops::Add;

pub struct AddNode<LhsNode: Node, RhsNode: Node>
    where LhsNode::Output: Add<RhsNode::Output>
{
    lhs: Ref<LhsNode>,
    rhs: Ref<RhsNode>,
}

impl<LhsNode: Node, RhsNode: Node> Node for AddNode<LhsNode, RhsNode>
    where LhsNode::Output: Add<RhsNode::Output>
{
    type Output = <LhsNode::Output as Add<RhsNode::Output>>::Output;

    fn eval(&self) -> Self::Output {
        self.lhs.eval() + self.rhs.eval()
    }
}

impl<LhsNode: Node, RhsNode: Node> AddNode<LhsNode, RhsNode>
    where LhsNode::Output: Add<RhsNode::Output>
{
    pub fn new(lhs: Ref<LhsNode>, rhs: Ref<RhsNode>) -> Ref<AddNode<LhsNode, RhsNode>> {
        Ref::new(AddNode {
            lhs: lhs,
            rhs: rhs,
        })
    }
}

impl<LhsNode: Node, RhsNode: Node> RevdepForwarder for AddNode<LhsNode, RhsNode>
    where LhsNode::Output: Add<RhsNode::Output>,
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

#[cfg(test)]
mod tests {
    use super::AddNode;
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
}
