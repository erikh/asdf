type Port = u64;
type Position = u64;

type Network<const SIZE: usize> = [usize; SIZE];

struct Address<const SIZE: usize> {
    network: Network<SIZE>,
    position: Position,
    port: Port,
}

struct Node<const SIZE: usize> {
    address: Address<SIZE>,
    name: String,
}
