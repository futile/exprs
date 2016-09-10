use ::{Node, Ref};

use std::ops::Add;

pub struct AddNode<LhsRes, RhsRes>
    where LhsRes: Add<RhsRes>
{
    lhs: Ref<Node<Output = LhsRes>>,
    rhs: Ref<Node<Output = RhsRes>>,
}

impl<LhsRes, RhsRes> Node for AddNode<LhsRes, RhsRes>
    where LhsRes: Add<RhsRes>
{
    type Output = <LhsRes as Add<RhsRes>>::Output;

    fn eval(&self) -> Self::Output {
        self.lhs.eval() + self.rhs.eval()
    }
}

impl<LhsRes, RhsRes> AddNode<LhsRes, RhsRes>
    where LhsRes: Add<RhsRes>
{
    pub fn new(lhs: Ref<Node<Output = LhsRes>>,
               rhs: Ref<Node<Output = RhsRes>>)
               -> Ref<AddNode<LhsRes, RhsRes>> {
        Ref::new(AddNode {
            lhs: lhs,
            rhs: rhs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::AddNode;
    use ::{InputNode, Node, Ref};

    #[test]
    fn add_uncached() {
        let input1 = InputNode::new(1.0f32);
        let add = AddNode::new(input1.clone(), Ref::new(2.0f32));

        assert_eq!(add.eval(), 3.0f32);
    }
}
