#![allow(dead_code)]
pub type Port = usize;
pub type Position = usize;

#[derive(Debug, Clone)]
pub struct Network<const SIZE: usize>([usize; SIZE]);

impl<const SIZE: usize> Default for Network<SIZE> {
    fn default() -> Self {
        Network([0_usize; SIZE])
    }
}

pub struct Address<const SIZE: usize> {
    network: Network<SIZE>,
    position: Position,
    port: Port,
}

pub struct Node<const SIZE: usize> {
    address: Address<SIZE>,
    name: String,
    connected: bool,
}

pub type PropagateResult = anyhow::Result<(bool, bool)>;

impl<const SIZE: usize> Network<SIZE> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn propagate() -> PropagateResult {
        Ok((true, true))
    }
}
