use crate::error::{Result, ScionicError};
use crate::types::{Dag, TransmissionPacket};
use std::fs;
use std::path::Path;

impl Dag {
    /// Serialize DAG to JSON
    pub fn to_json(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| ScionicError::Serialization(e.to_string()))
    }

    /// Serialize DAG to pretty JSON
    pub fn to_json_pretty(&self) -> Result<Vec<u8>> {
        serde_json::to_vec_pretty(self).map_err(|e| ScionicError::Serialization(e.to_string()))
    }

    /// Deserialize DAG from JSON
    pub fn from_json(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data).map_err(|e| ScionicError::Deserialization(e.to_string()))
    }

    /// Serialize DAG to CBOR
    pub fn to_cbor(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| ScionicError::Serialization(e.to_string()))
    }

    /// Deserialize DAG from CBOR
    pub fn from_cbor(data: &[u8]) -> Result<Self> {
        serde_cbor::from_slice(data).map_err(|e| ScionicError::Deserialization(e.to_string()))
    }

    /// Save DAG to file (CBOR format)
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let data = self.to_cbor()?;
        fs::write(path, data)?;
        Ok(())
    }

    /// Load DAG from file (CBOR format)
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let data = fs::read(path)?;
        Self::from_cbor(&data)
    }

    /// Get leaf sequence as transmission packets (for syncing)
    pub fn get_leaf_sequence(&self) -> Vec<TransmissionPacket> {
        let mut packets = Vec::new();

        for (hash, leaf) in &self.leaves {
            let parent_hash = self
                .find_parent_for_transmission(hash)
                .map(|p| p.hash.clone())
                .unwrap_or_default();

            let proofs = leaf.proofs.clone().unwrap_or_default();

            packets.push(TransmissionPacket {
                leaf: leaf.clone(),
                parent_hash,
                proofs,
            });
        }

        packets
    }

    fn find_parent_for_transmission(&self, child_hash: &str) -> Option<&crate::types::DagLeaf> {
        self.leaves
            .values()
            .find(|&leaf| leaf.has_link(child_hash))
            .map(|v| v as _)
    }

    /// Apply a transmission packet to this DAG
    pub fn apply_transmission_packet(&mut self, packet: TransmissionPacket) {
        self.leaves.insert(packet.leaf.hash.clone(), packet.leaf);
    }

    /// Verify and apply a transmission packet
    pub fn apply_and_verify_transmission_packet(
        &mut self,
        packet: TransmissionPacket,
    ) -> Result<()> {
        // Verify the leaf
        if packet.leaf.hash == self.root {
            packet.leaf.verify_root_leaf()?;
        } else {
            packet.leaf.verify_leaf()?;
        }

        // Apply the packet
        self.apply_transmission_packet(packet);

        Ok(())
    }
}

impl TransmissionPacket {
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| ScionicError::Serialization(e.to_string()))
    }

    /// Deserialize from JSON
    pub fn from_json(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data).map_err(|e| ScionicError::Deserialization(e.to_string()))
    }

    /// Serialize to CBOR
    pub fn to_cbor(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| ScionicError::Serialization(e.to_string()))
    }

    /// Deserialize from CBOR
    pub fn from_cbor(data: &[u8]) -> Result<Self> {
        serde_cbor::from_slice(data).map_err(|e| ScionicError::Deserialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::create_dag;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_json_serialization() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"Test data")?;

        let dag = create_dag(&file_path, false)?;

        // Serialize to JSON
        let json = dag.to_json()?;
        assert!(!json.is_empty());

        // Deserialize from JSON
        let dag2 = Dag::from_json(&json)?;
        assert_eq!(dag.root, dag2.root);
        assert_eq!(dag.leaves.len(), dag2.leaves.len());

        Ok(())
    }

    #[test]
    fn test_cbor_serialization() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"Test data")?;

        let dag = create_dag(&file_path, false)?;

        // Serialize to CBOR
        let cbor = dag.to_cbor()?;
        assert!(!cbor.is_empty());

        // Deserialize from CBOR
        let dag2 = Dag::from_cbor(&cbor)?;
        assert_eq!(dag.root, dag2.root);
        assert_eq!(dag.leaves.len(), dag2.leaves.len());

        Ok(())
    }

    #[test]
    fn test_file_save_load() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"Test data")?;

        let dag = create_dag(&file_path, false)?;

        // Save to file
        let dag_file = temp_dir.path().join("test.dag");
        dag.save_to_file(&dag_file)?;

        // Load from file
        let dag2 = Dag::load_from_file(&dag_file)?;
        assert_eq!(dag.root, dag2.root);
        assert_eq!(dag.leaves.len(), dag2.leaves.len());

        Ok(())
    }
}
