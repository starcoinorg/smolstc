use std::collections::HashSet;
pub struct Block;

pub trait DAG {
    /// Initializes the DAG.
    fn new() -> Self;

    /// Adds the given block to the DAG.
    fn add(&mut self, block: Block);

    /// Returns true if the block with the given global id is in the DAG.
    fn contains(&self, global_id: Block) -> bool;

    /// Returns the data of the block with the given global id if it exists in the DAG.
    fn get(&self, global_id: Block) -> Option<Block>;

    /// Returns an iterator on the DAG's blocks.
    fn iter(&self) -> std::collections::hash_set::Iter<Block>;

    /// Returns the number of blocks in the DAG.
    fn len(&self) -> usize;

    /// Returns a string representation of the DAG.
    fn to_string(&self) -> String;

    /// Returns a set containing the global ids of the parents of the virtual block.
    fn get_virtual_block_parents(&self) -> HashSet<Block>;

    /// Returns true if the block with global id a is before the block with global id b according to the DAG's ordering.
    /// Returns None if both blocks aren't in the DAG.
    fn is_a_before_b(&self, a: Block, b: Block) -> Option<bool>;

    /// Returns the depth in the "main" sub-DAG of the block with the given global id if it exists in the DAG.
    fn get_depth(&self, global_id: Block) -> Option<usize>;

}
