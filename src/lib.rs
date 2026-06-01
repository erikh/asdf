#![allow(dead_code)]
pub type Port = usize;
pub type Position = usize;

#[derive(Debug, Clone)]
pub struct Network<const SIZE: usize, const ADDR_SIZE: usize>([Node; SIZE]);

impl<const SIZE: usize, const ADDR_SIZE: usize> Default for Network<SIZE, ADDR_SIZE> {
    fn default() -> Self {
        let mut v = Vec::new();
        for i in 0..SIZE {
            v.push(Node::new_pos(i))
        }

        Self(v.as_array().unwrap().clone())
    }
}

pub struct NetworkIter<const SIZE: usize, const ADDR_SIZE: usize> {
    network: Network<SIZE, ADDR_SIZE>,
    idx: usize,
}

pub struct NetworkBroadcast<const SIZE: usize, const ADDR_SIZE: usize> {
    network: Network<SIZE, ADDR_SIZE>,
}

pub struct NetworkBroadcastIter<const SIZE: usize, const ADDR_SIZE: usize, const BATCH_SIZE: usize>
{
    network: Network<SIZE, ADDR_SIZE>,
    idx: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Address {
    position: Position,
    port: Port,
}

#[derive(Debug, Clone, Default)]
pub struct Node {
    address: Address,
    name: String,
    connected: bool,
}

impl Node {
    pub fn new_pos(pos: usize) -> Self {
        Self {
            address: Address {
                position: pos,
                port: 0,
            },
            name: "default".to_string(),
            connected: false,
        }
    }
}

impl<const ORIG: usize, const NEW: usize, const ADDR_SIZE: usize> From<[Node; ORIG]>
    for Network<NEW, ADDR_SIZE>
{
    fn from(value: [Node; ORIG]) -> Self {
        let mut this = Self::default();
        for (x, item) in value.iter().enumerate() {
            this.0[x] = item.clone()
        }

        this
    }
}

impl<const SIZE: usize, const ADDR_SIZE: usize> Network<SIZE, ADDR_SIZE> {
    pub fn new() -> Self {
        Self::default()
    }

    // returns a cursor
    pub fn slice_peers<const BATCH_SIZE: usize>(
        &self,
    ) -> anyhow::Result<(usize, Network<BATCH_SIZE, ADDR_SIZE>)> {
        Ok((BATCH_SIZE, self.0.clone().into()))
    }
}

impl<const SIZE: usize, const ADDR_SIZE: usize> Iterator for NetworkIter<SIZE, ADDR_SIZE> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > SIZE {
            None
        } else {
            Some(self.network.0[self.idx].clone())
        }
    }
}

impl<const SIZE: usize, const ADDR_SIZE: usize, const BATCH_SIZE: usize> Iterator
    for NetworkBroadcastIter<SIZE, ADDR_SIZE, BATCH_SIZE>
{
    type Item = Network<BATCH_SIZE, ADDR_SIZE>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > SIZE {
            None
        } else {
            let (idx, out) = self
                .network
                .slice_peers::<BATCH_SIZE>()
                .expect("invalid index");
            self.idx = idx;
            Some(out)
        }
    }
}
