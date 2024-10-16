use super::NodeWeightDiscriminants;
use crate::workspace_snapshot::{
    content_address::ContentAddress,
    graph::LineageId,
    node_weight::traits::{CorrectExclusiveOutgoingEdge, CorrectTransforms, SiNodeWeight},
    NodeId,
};
use std::collections::HashSet;

use crate::workspace_snapshot::graph::detect_updates::Update;
use crate::workspace_snapshot::node_weight::traits::CorrectTransformsResult;
use crate::{EdgeWeightKindDiscriminants, Timestamp, WorkspaceSnapshotGraphVCurrent};
use dal_macros::SiNodeWeight;
use jwt_simple::prelude::{Deserialize, Serialize};
use petgraph::prelude::EdgeRef;
use petgraph::Incoming;
use si_events::merkle_tree_hash::MerkleTreeHash;
use si_events::ulid::Ulid;
use si_events::ContentHash;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SiNodeWeight)]
#[si_node_weight(discriminant = NodeWeightDiscriminants::Geometry)]
pub struct GeometryNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    #[si_node_weight(node_hash = "self.content_address.content_hash().as_bytes()")]
    content_address: ContentAddress,
    timestamp: Timestamp,
}

impl GeometryNodeWeightV1 {
    pub fn new(id: Ulid, lineage_id: Ulid, content_hash: ContentHash) -> Self {
        Self {
            id,
            lineage_id,
            content_address: ContentAddress::Geometry(content_hash),
            merkle_tree_hash: MerkleTreeHash::default(),
            timestamp: Timestamp::now(),
        }
    }

    pub fn new_content_hash(&mut self, new_content_hash: ContentHash) {
        self.content_address = ContentAddress::Geometry(new_content_hash);
    }
}

impl CorrectTransforms for GeometryNodeWeightV1 {}

impl CorrectExclusiveOutgoingEdge for GeometryNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[EdgeWeightKindDiscriminants::Represents]
    }
}
