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
        // Copy at most NEW items so a shrink (ORIG > NEW) never indexes past
        // the destination array.
        for (x, item) in value.iter().take(NEW).enumerate() {
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
        if self.idx >= SIZE {
            None
        } else {
            let node = self.network.0[self.idx].clone();
            self.idx += 1;
            Some(node)
        }
    }
}

impl<const SIZE: usize, const ADDR_SIZE: usize, const BATCH_SIZE: usize> Iterator
    for NetworkBroadcastIter<SIZE, ADDR_SIZE, BATCH_SIZE>
{
    type Item = Network<BATCH_SIZE, ADDR_SIZE>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= SIZE {
            return None;
        }

        // Window BATCH_SIZE peers starting at the current cursor, then
        // advance the cursor by a full batch so each call emits the next
        // consecutive chunk and the iterator terminates.
        let mut batch = Network::<BATCH_SIZE, ADDR_SIZE>::default();
        for i in 0..BATCH_SIZE {
            let src = self.idx + i;
            if src >= SIZE {
                break;
            }
            batch.0[i] = self.network.0[src].clone();
        }
        self.idx += BATCH_SIZE;
        Some(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pull out the `position` of every node in a network, in order.
    /// Lets the tests assert on which peers landed where without poking at
    /// every private field inline.
    fn positions<const SIZE: usize, const ADDR_SIZE: usize>(
        net: &Network<SIZE, ADDR_SIZE>,
    ) -> Vec<Position> {
        net.0.iter().map(|n| n.address.position).collect()
    }

    // ---- const generic SIZE wiring -------------------------------------

    /// `default()` must produce exactly SIZE nodes, addressed 0..SIZE.
    /// Guards the `for i in 0..SIZE` / `as_array` plumbing against an
    /// off-by-one in the fill loop.
    #[test]
    fn default_fills_exactly_size_nodes() {
        let net = Network::<5, 8>::default();
        assert_eq!(net.0.len(), 5, "array length must equal the SIZE generic");
        assert_eq!(
            positions(&net),
            vec![0, 1, 2, 3, 4],
            "every slot 0..SIZE must be initialized once, in order"
        );
    }

    /// A degenerate empty network must be constructible and contain nothing.
    #[test]
    fn default_handles_zero_size() {
        let net = Network::<0, 8>::default();
        assert_eq!(net.0.len(), 0);
        assert!(positions(&net).is_empty());
    }

    // ---- From<[Node; ORIG]> resizing -----------------------------------

    /// Converting a *larger* array into a *smaller* network must copy only
    /// the destination's NEW slots and never index past them.
    /// Currently the loop walks all ORIG items and writes `this.0[x]`,
    /// so this panics with an out-of-bounds when ORIG > NEW.
    #[test]
    fn from_shrinks_without_overrunning_destination() {
        let src: [Node; 5] = std::array::from_fn(Node::new_pos);
        let net: Network<3, 8> = src.into();
        assert_eq!(net.0.len(), 3);
        assert_eq!(
            positions(&net),
            vec![0, 1, 2],
            "shrinking must keep the first NEW peers, not overrun the array"
        );
    }

    /// Converting a *smaller* array into a *larger* network must copy the
    /// ORIG items and leave the remaining slots at their defaults.
    #[test]
    fn from_grows_and_leaves_tail_defaulted() {
        let src: [Node; 2] = std::array::from_fn(Node::new_pos);
        let net: Network<4, 8> = src.into();
        assert_eq!(net.0.len(), 4);
        assert_eq!(positions(&net)[..2], [0, 1]);
    }

    // ---- slice_peers / BATCH_SIZE generic ------------------------------

    /// `slice_peers::<B>` must hand back a network whose array length is the
    /// BATCH_SIZE generic, holding the first B peers — not the whole SIZE
    /// array crammed into a B-slot network.
    #[test]
    fn slice_peers_yields_a_batch_sized_network() {
        let net = Network::<6, 8>::default();
        let (cursor, batch) = net
            .slice_peers::<2>()
            .expect("slicing a 2-peer batch out of 6 must succeed");
        assert_eq!(batch.0.len(), 2, "result length must equal BATCH_SIZE");
        assert_eq!(positions(&batch), vec![0, 1]);
        assert!(
            cursor <= 6,
            "returned cursor must stay within the source network, got {cursor}"
        );
    }

    // ---- NetworkIter: array bounds + advancement -----------------------

    /// The boundary off-by-one: once the cursor reaches SIZE there is no
    /// valid index left, so `next()` must return None. `idx > SIZE` instead
    /// of `idx >= SIZE` lets `idx == SIZE` through and indexes out of bounds.
    #[test]
    fn iter_stops_when_cursor_reaches_size() {
        let mut iter = NetworkIter {
            network: Network::<3, 8>::new(),
            idx: 3, // == SIZE: one past the last valid index (0..=2)
        };
        assert!(
            iter.next().is_none(),
            "index == SIZE is out of bounds and must yield None"
        );
    }

    /// Walking the iterator must visit each node exactly once, in order,
    /// then terminate. Guards against the missing `idx` increment (which
    /// would otherwise replay node 0 forever).
    #[test]
    fn iter_visits_each_node_once_then_ends() {
        let mut iter = NetworkIter {
            network: Network::<4, 8>::new(),
            idx: 0,
        };

        let mut seen = Vec::new();
        // Bounded so a non-advancing iterator fails the assert instead of
        // hanging the test run.
        for _ in 0..4 {
            match iter.next() {
                Some(node) => seen.push(node.address.position),
                None => break,
            }
        }
        assert_eq!(seen, vec![0, 1, 2, 3], "each node visited once, in order");
        assert!(iter.next().is_none(), "iterator must be exhausted after SIZE items");
    }

    // ---- NetworkBroadcastIter: batching + termination ------------------

    /// Broadcasting in batches must chunk the SIZE nodes into BATCH_SIZE
    /// groups, advance the cursor each step, and terminate. The cursor is
    /// currently pinned to BATCH_SIZE every call, so it never makes progress.
    #[test]
    fn broadcast_chunks_network_and_terminates() {
        let mut iter = NetworkBroadcastIter::<6, 8, 2> {
            network: Network::<6, 8>::new(),
            idx: 0,
        };

        let mut batches = Vec::new();
        // Hard cap well above the expected 3 batches: a non-terminating
        // iterator fails this assert rather than spinning forever.
        for _ in 0..16 {
            match iter.next() {
                Some(batch) => batches.push(positions(&batch)),
                None => break,
            }
        }

        assert_eq!(
            batches,
            vec![vec![0, 1], vec![2, 3], vec![4, 5]],
            "6 peers in batches of 2 must produce three consecutive chunks"
        );
        assert!(
            iter.next().is_none(),
            "broadcast must terminate once every peer has been emitted"
        );
    }
}
