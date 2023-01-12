use massa_hash::Hash;
use massa_models::{
    block::{Block, BlockSerializer, SecureShareBlock},
    block_header::{BlockHeader, BlockHeaderSerializer},
    secure_share::SecureShareContent,
    slot::Slot,
};
use massa_models::block_v0::BlockV0;
use massa_signature::KeyPair;

/// Create an empty block for testing. Can be used to generate genesis blocks.
pub fn create_empty_block(keypair: &KeyPair, slot: &Slot) -> SecureShareBlock {
    let header = BlockHeader::new_verifiable(
        BlockHeader {
            block_version_current: 0,
            block_version_next: 0,
            slot: *slot,
            parents: Vec::new(),
            operation_merkle_root: Hash::compute_from(&Vec::new()),
            endorsements: Vec::new(),
        },
        BlockHeaderSerializer::new(),
        keypair,
    )
    .unwrap();

    Block::new_verifiable(
        Block::V0(BlockV0{
            header,
            operations: Default::default(),
        }),
        BlockSerializer::new(),
        keypair,
    )
    .unwrap()
}
