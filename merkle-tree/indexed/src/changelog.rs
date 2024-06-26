use light_concurrent_merkle_tree::event::RawIndexedElement;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexedChangelogEntry<I, const NET_HEIGHT: usize>
where
    I: Clone,
{
    /// Element that was a subject to the change.
    pub element: RawIndexedElement<I>,
    /// Merkle proof of that operation.
    pub proof: [[u8; 32]; NET_HEIGHT], // TODO: add const generic HEIGHT - CANOPY_DEPTH
    /// Index of a changelog entry in `ConcurrentMerkleTree` corresponding to
    /// the same operation.
    pub changelog_index: usize,
}
