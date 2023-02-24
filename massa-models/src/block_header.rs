use crate::block_id::BlockId;
use crate::config::THREAD_COUNT;
use crate::endorsement::{
    Endorsement, EndorsementDeserializerLW, EndorsementId, EndorsementSerializer,
    EndorsementSerializerLW, SecureShareEndorsement,
};
use crate::secure_share::{
    SecureShare, SecureShareContent, SecureShareDeserializer, SecureShareSerializer,
};
use crate::slot::{Slot, SlotDeserializer, SlotSerializer};
use massa_hash::{Hash, HashDeserializer};
use massa_serialization::{
    Deserializer, SerializeError, Serializer, U32VarIntDeserializer, U32VarIntSerializer,
};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::error::{context, ContextError, ParseError};
use nom::multi::{count, length_count};
use nom::sequence::{preceded, tuple};
use nom::{IResult, Parser};
use serde::{Deserialize, Serialize};
use std::collections::Bound::{Excluded, Included};
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;

const NB_BLOCKS_CONSIDERED: usize = 5;
const THRESHOLD: usize = 2;

/// Struct used to keep track of announced versions in previous blocks
/// Contains additional utilities
pub struct VersioningMiddleware {
    latest_announcements: VecDeque<u32>,
    counts: HashMap<u32, usize>,
    current_active_version: u32,
    current_supported_versions: Vec<u32>,
    current_announced_version: u32,
}

impl Default for VersioningMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl VersioningMiddleware {
    /// Creates a new empty versioning middleware.
    pub fn new() -> Self {
        VersioningMiddleware {
            latest_announcements: VecDeque::with_capacity(NB_BLOCKS_CONSIDERED + 1),
            counts: HashMap::new(),
            current_active_version: 0,
            current_supported_versions: Vec::new(),
            current_announced_version: 0,
        }
    }

    /// Add the announced version of a new block
    /// If needed, remove the info related to a ancient block
    pub fn new_block(&mut self, version: u32) {
        self.latest_announcements.push_back(version);

        *self.counts.entry(version).or_default() += 1;

        // If the queue is too large, remove the most ancient block and its associated count
        if self.latest_announcements.len() > NB_BLOCKS_CONSIDERED {
            let prev_version = self.latest_announcements.pop_front();
            if let Some(prev_version) = prev_version {
                *self.counts.entry(prev_version).or_insert(1) -= 1;
            }
        }
    }

    /// Get version that was announced the most in the previous blocks,
    /// as well as the count
    pub fn most_announced_version(&mut self) -> (u32, usize) {
        match self.counts.iter().max_by_key(|(_, &v)| v) {
            Some((max_count_version, count)) => (*max_count_version, *count),
            None => (0, 0),
        }
    }

    /// Checks whether a given version should become active
    pub fn is_count_above_threshold(&mut self, version: u32) -> bool {
        match self.counts.get(&version) {
            Some(count) => *count >= THRESHOLD,
            None => false,
        }
    }

    /// Get the current active version
    pub fn get_current_active_version(&self) -> u32 {
        self.current_active_version
    }

    /// Set the current active version
    pub fn set_current_active_version(&mut self, version: u32) {
        self.current_active_version = version;
    }

    /// Get the current supported versions
    pub fn get_current_supported_versions(&self) -> Vec<u32> {
        self.current_supported_versions.clone()
    }

    /// Add a new version to the currently supported versions list
    pub fn add_supported_version(&mut self, version: u32) {
        if !self.current_supported_versions.contains(&version) {
            self.current_supported_versions.push(version)
        }
    }

    /// Remove a version to the currently supported versions list
    pub fn remove_supported_version(&mut self, version: u32) {
        self.current_supported_versions.retain(|&x| x != version)
    }

    /// Get the current version to announce
    pub fn get_current_announced_version(&self) -> u32 {
        self.current_announced_version
    }

    /// Set the current version to announce
    pub fn set_current_announced_version(&mut self, version: u32) {
        self.current_announced_version = version;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_versioning_middleware() {
        let mut vm = VersioningMiddleware::new();

        vm.new_block(1);
        println!("{:?}", vm.most_announced_version());
        vm.new_block(1);
        println!("{:?}", vm.most_announced_version());
        vm.new_block(2);
        println!("{:?}", vm.most_announced_version());
        vm.new_block(2);
        println!("{:?}", vm.most_announced_version());
        vm.new_block(2);
        println!("{:?}", vm.most_announced_version());
        vm.new_block(2);
        println!("{:?}", vm.most_announced_version());
        vm.new_block(2);
        println!("{:?}", vm.most_announced_version());
        vm.new_block(2);
        println!("{:?}", vm.most_announced_version());
        vm.new_block(2);
        println!("{:?}", vm.most_announced_version());
    }
}

/// block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// slot
    pub slot: Slot,
    /// announced version for BIP-34 impl
    pub announced_version: u32,
    /// parents
    pub parents: Vec<BlockId>,
    /// all operations hash
    pub operation_merkle_root: Hash,
    /// endorsements
    pub endorsements: Vec<SecureShareEndorsement>,
}

// TODO: gh-issue #3398
#[cfg(any(test, feature = "testing"))]
impl BlockHeader {
    fn assert_invariants(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.slot.period == 0 {
            if !self.parents.is_empty() {
                return Err("Invariant broken: genesis block with parent(s)".into());
            }
            if !self.endorsements.is_empty() {
                return Err("Invariant broken: genesis block with endorsement(s)".into());
            }
        } else {
            if self.parents.len() != crate::config::THREAD_COUNT as usize {
                return Err(
                    "Invariant broken: non-genesis block with incorrect number of parents".into(),
                );
            }
            if self.endorsements.len() > crate::config::ENDORSEMENT_COUNT as usize {
                return Err("Invariant broken: endorsement count too high".into());
            }

            let parent_id = self.parents[self.slot.thread as usize];
            for endo in self.endorsements.iter() {
                if endo.content.endorsed_block != parent_id {
                    return Err("Invariant broken: endorsement doesn't match parent".into());
                }
            }
        }

        // assert that the endorsement indexes are all unique...
        let mut set = HashSet::new();
        for endo in self.endorsements.iter() {
            // ...and check signatures + invariants while at it
            endo.check_invariants()?;

            if !set.insert(endo.content.index) {
                return Err("Endorsement duplicate index found".into());
            }
        }
        Ok(())
    }
}

/// BlockHeader wrapped up alongside verification data
pub type SecuredHeader = SecureShare<BlockHeader, BlockId>;

impl SecuredHeader {
    /// gets the header fitness
    pub fn get_fitness(&self) -> u64 {
        (self.content.endorsements.len() as u64) + 1
    }
    // TODO: gh-issue #3398
    #[allow(dead_code)]
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn assert_invariants(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.content.assert_invariants()?;
        self.verify_signature()
            .map_err(|er| format!("{}", er).into())
    }
}

impl SecureShareContent for BlockHeader {}

/// Serializer for `BlockHeader`
pub struct BlockHeaderSerializer {
    slot_serializer: SlotSerializer,
    endorsement_serializer: SecureShareSerializer,
    endorsement_content_serializer: EndorsementSerializerLW,
    u32_serializer: U32VarIntSerializer,
}

impl BlockHeaderSerializer {
    /// Creates a new `BlockHeaderSerializer`
    pub fn new() -> Self {
        Self {
            slot_serializer: SlotSerializer::new(),
            endorsement_serializer: SecureShareSerializer::new(),
            u32_serializer: U32VarIntSerializer::new(),
            endorsement_content_serializer: EndorsementSerializerLW::new(),
        }
    }
}

impl Default for BlockHeaderSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializer<BlockHeader> for BlockHeaderSerializer {
    /// ## Example:
    /// ```rust
    /// use massa_models::{block_id::BlockId, block_header::BlockHeader, block_header::BlockHeaderSerializer};
    /// use massa_models::endorsement::{Endorsement, EndorsementSerializer};
    /// use massa_models::secure_share::SecureShareContent;
    /// use massa_models::{config::THREAD_COUNT, slot::Slot};
    /// use massa_hash::Hash;
    /// use massa_signature::KeyPair;
    /// use massa_serialization::Serializer;
    ///
    /// let keypair = KeyPair::generate();
    /// let parents = (0..THREAD_COUNT)
    ///   .map(|i| BlockId(Hash::compute_from(&[i])))
    ///   .collect();
    /// let header = BlockHeader {
    ///   slot: Slot::new(1, 1),
    ///   parents,
    ///   operation_merkle_root: Hash::compute_from("mno".as_bytes()),
    ///   endorsements: vec![
    ///     Endorsement::new_verifiable(
    ///        Endorsement {
    ///          slot: Slot::new(1, 1),
    ///          index: 1,
    ///          endorsed_block: BlockId(Hash::compute_from("blk1".as_bytes())),
    ///        },
    ///     EndorsementSerializer::new(),
    ///     &keypair,
    ///     )
    ///     .unwrap(),
    ///     Endorsement::new_verifiable(
    ///       Endorsement {
    ///         slot: Slot::new(4, 0),
    ///         index: 3,
    ///         endorsed_block: BlockId(Hash::compute_from("blk2".as_bytes())),
    ///       },
    ///     EndorsementSerializer::new(),
    ///     &keypair,
    ///     )
    ///     .unwrap(),
    ///    ],
    /// };
    /// let mut buffer = vec![];
    /// BlockHeaderSerializer::new().serialize(&header, &mut buffer).unwrap();
    /// ```
    fn serialize(&self, value: &BlockHeader, buffer: &mut Vec<u8>) -> Result<(), SerializeError> {
        self.slot_serializer.serialize(&value.slot, buffer)?;

        self.u32_serializer
            .serialize(&value.announced_version, buffer)?;

        // parents (note: there should be none if slot period=0)
        if value.parents.is_empty() {
            buffer.push(0);
        } else {
            buffer.push(1);
        }
        for parent_h in value.parents.iter() {
            buffer.extend(parent_h.0.to_bytes());
        }

        // operations merkle root
        buffer.extend(value.operation_merkle_root.to_bytes());

        self.u32_serializer.serialize(
            &value.endorsements.len().try_into().map_err(|err| {
                SerializeError::GeneralError(format!("too many endorsements: {}", err))
            })?,
            buffer,
        )?;
        for endorsement in value.endorsements.iter() {
            self.endorsement_serializer.serialize_with(
                &self.endorsement_content_serializer,
                endorsement,
                buffer,
            )?;
        }
        Ok(())
    }
}

/// Deserializer for `BlockHeader`
pub struct BlockHeaderDeserializer {
    slot_deserializer: SlotDeserializer,
    endorsement_serializer: EndorsementSerializer,
    announced_version_deserializer: U32VarIntDeserializer,
    length_endorsements_deserializer: U32VarIntDeserializer,
    hash_deserializer: HashDeserializer,
    thread_count: u8,
    endorsement_count: u32,
}

impl BlockHeaderDeserializer {
    /// Creates a new `BlockHeaderDeserializerLW`
    pub const fn new(thread_count: u8, endorsement_count: u32) -> Self {
        Self {
            slot_deserializer: SlotDeserializer::new(
                (Included(0), Included(u64::MAX)),
                (Included(0), Excluded(thread_count)),
            ),
            endorsement_serializer: EndorsementSerializer::new(),
            length_endorsements_deserializer: U32VarIntDeserializer::new(
                Included(0),
                Included(endorsement_count),
            ),
            announced_version_deserializer: U32VarIntDeserializer::new(
                Included(0),
                Included(u32::MAX),
            ),
            hash_deserializer: HashDeserializer::new(),
            thread_count,
            endorsement_count,
        }
    }
}

impl Deserializer<BlockHeader> for BlockHeaderDeserializer {
    /// ## Example:
    /// ```rust
    /// use massa_models::block_header::{BlockHeader, BlockHeaderDeserializer, BlockHeaderSerializer};
    /// use massa_models::block_id::{BlockId};
    /// use massa_models::{config::THREAD_COUNT, slot::Slot, secure_share::SecureShareContent};
    /// use massa_models::endorsement::{Endorsement, EndorsementSerializer};
    /// use massa_hash::Hash;
    /// use massa_signature::KeyPair;
    /// use massa_serialization::{Serializer, Deserializer, DeserializeError};
    ///
    /// let keypair = KeyPair::generate();
    /// let parents: Vec<BlockId> = (0..THREAD_COUNT)
    ///   .map(|i| BlockId(Hash::compute_from(&[i])))
    ///   .collect();
    /// let header = BlockHeader {
    ///   slot: Slot::new(1, 1),
    ///   parents: parents.clone(),
    ///   operation_merkle_root: Hash::compute_from("mno".as_bytes()),
    ///   endorsements: vec![
    ///     Endorsement::new_verifiable(
    ///        Endorsement {
    ///          slot: Slot::new(1, 1),
    ///          index: 0,
    ///          endorsed_block: parents[1].clone(),
    ///        },
    ///     EndorsementSerializer::new(),
    ///     &keypair,
    ///     )
    ///     .unwrap(),
    ///     Endorsement::new_verifiable(
    ///       Endorsement {
    ///         slot: Slot::new(1, 1),
    ///         index: 1,
    ///         endorsed_block: parents[1].clone(),
    ///       },
    ///     EndorsementSerializer::new(),
    ///     &keypair,
    ///     )
    ///     .unwrap(),
    ///    ],
    /// };
    /// let mut buffer = vec![];
    /// BlockHeaderSerializer::new().serialize(&header, &mut buffer).unwrap();
    /// let (rest, deserialized_header) = BlockHeaderDeserializer::new(32, 9).deserialize::<DeserializeError>(&buffer).unwrap();
    /// assert_eq!(rest.len(), 0);
    /// let mut buffer2 = Vec::new();
    /// BlockHeaderSerializer::new().serialize(&deserialized_header, &mut buffer2).unwrap();
    /// assert_eq!(buffer, buffer2);
    /// ```
    fn deserialize<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        &self,
        buffer: &'a [u8],
    ) -> IResult<&'a [u8], BlockHeader, E> {
        let (rest, (slot, announced_version, parents, operation_merkle_root)): (
            &[u8],
            (Slot, u32, Vec<BlockId>, Hash),
        ) = context("Failed BlockHeader deserialization", |input| {
            let (rest, (slot, announced_version, parents)) = tuple((
                context("Failed slot deserialization", |input| {
                    self.slot_deserializer.deserialize(input)
                }),
                context("Failed announced_version deserialization", |input| {
                    self.announced_version_deserializer.deserialize(input)
                }),
                context(
                    "Failed parents deserialization",
                    alt((
                        preceded(tag(&[0]), |input| Ok((input, Vec::new()))),
                        preceded(
                            tag(&[1]),
                            count(
                                context("Failed block_id deserialization", |input| {
                                    self.hash_deserializer
                                        .deserialize(input)
                                        .map(|(rest, hash)| (rest, BlockId(hash)))
                                }),
                                self.thread_count as usize,
                            ),
                        ),
                    )),
                ),
            ))
            .parse(input)?;

            // validate the parent/slot invariats before moving on to other fields
            if slot.period == 0 && !parents.is_empty() {
                return Err(nom::Err::Failure(ContextError::add_context(
                    rest,
                    "Genesis block cannot contain parents",
                    ParseError::from_error_kind(rest, nom::error::ErrorKind::Fail),
                )));
            } else if slot.period != 0 && parents.len() != THREAD_COUNT as usize {
                return Err(nom::Err::Failure(ContextError::add_context(
                    rest,
                    const_format::formatcp!("Non-genesis block must have {} parents", THREAD_COUNT),
                    ParseError::from_error_kind(rest, nom::error::ErrorKind::Fail),
                )));
            }

            let (rest, merkle) = context("Failed operation_merkle_root", |input| {
                self.hash_deserializer.deserialize(input)
            })
            .parse(rest)?;
            Ok((rest, (slot, announced_version, parents, merkle)))
        })
        .parse(buffer)?;

        if parents.is_empty() {
            let res = BlockHeader {
                slot,
                announced_version,
                parents,
                operation_merkle_root,
                endorsements: Vec::new(),
            };

            // TODO: gh-issue #3398
            #[cfg(any(test, feature = "testing"))]
            res.assert_invariants().unwrap();

            return Ok((
                &rest[1..], // Because there is 0 endorsements, we have a remaining 0 in rest and we don't need it
                res,
            ));
        }
        // Now deser the endorsements (which were light-weight serialized)
        let endorsement_deserializer =
            SecureShareDeserializer::new(EndorsementDeserializerLW::new(
                self.endorsement_count,
                slot,
                parents[slot.thread as usize],
            ));

        let parent_id = parents[slot.thread as usize];
        let (rest, endorsements): (&[u8], Vec<SecureShare<Endorsement, EndorsementId>>) = context(
            "Failed endorsements deserialization",
            length_count::<&[u8], SecureShare<Endorsement, EndorsementId>, u32, E, _, _>(
                context("Failed length deserialization", |input| {
                    self.length_endorsements_deserializer.deserialize(input)
                }),
                context("Failed endorsement deserialization", |input| {
                    let (rest, endo) = endorsement_deserializer
                        .deserialize_with(&self.endorsement_serializer, input)?;

                    if endo.content.endorsed_block != parent_id {
                        return Err(nom::Err::Failure(ContextError::add_context(
                            rest,
                            "Endorsement does not match block parents",
                            ParseError::from_error_kind(rest, nom::error::ErrorKind::Fail),
                        )));
                    }

                    Ok((rest, endo))
                }),
            ),
        )
        .parse(rest)?;

        let mut set = HashSet::new();

        for end in endorsements.iter() {
            if !set.insert(end.content.index) {
                return Err(nom::Err::Failure(ContextError::add_context(
                    rest,
                    "Duplicate endorsement index found",
                    ParseError::from_error_kind(rest, nom::error::ErrorKind::Fail),
                )));
            }
        }

        let header = BlockHeader {
            slot,
            announced_version,
            parents,
            operation_merkle_root,
            endorsements,
        };

        // TODO: gh-issue #3398
        #[cfg(any(test, feature = "testing"))]
        header.assert_invariants().unwrap();

        Ok((rest, header))
    }
}

impl std::fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\t(period: {}, thread: {})",
            self.slot.period, self.slot.thread,
        )?;
        writeln!(f, "\tMerkle root: {}", self.operation_merkle_root,)?;
        writeln!(f, "\tParents: ")?;
        for id in self.parents.iter() {
            let str_id = id.to_string();
            writeln!(f, "\t\t{}", str_id)?;
        }
        if self.parents.is_empty() {
            writeln!(f, "No parents found: This is a genesis header")?;
        }
        writeln!(f, "\tEndorsements:")?;
        for ed in self.endorsements.iter() {
            writeln!(f, "\t\t-----")?;
            writeln!(f, "\t\tId: {}", ed.id)?;
            writeln!(f, "\t\tIndex: {}", ed.content.index)?;
            writeln!(f, "\t\tEndorsed slot: {}", ed.content.slot)?;
            writeln!(
                f,
                "\t\tEndorser's public key: {}",
                ed.content_creator_pub_key
            )?;
            writeln!(f, "\t\tEndorsed block: {}", ed.content.endorsed_block)?;
            writeln!(f, "\t\tSignature: {}", ed.signature)?;
        }
        if self.endorsements.is_empty() {
            writeln!(f, "\tNo endorsements found")?;
        }
        Ok(())
    }
}
