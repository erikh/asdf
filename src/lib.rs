pub type Port = u64;
pub type Position = u64;

pub type Network<const SIZE: usize> = [usize; SIZE];

pub struct Address<const SIZE: usize> {
    network: Network<SIZE>,
    position: Position,
    port: Port,
}

pub struct Node<const SIZE: usize> {
    address: Address<SIZE>,
    name: String,
}

pub type PropagateResult = anyhow::Result<(bool, bool)>;

pub impl Network {
    pub fn propagate() -> PropagateResult {
        Ok((true, true))
    }
}
