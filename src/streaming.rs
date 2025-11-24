use crate::error::{Result, ScionicError};
use crate::types::{Dag, DagLeaf, DagLeafBuilder, LeafType, DEFAULT_CHUNK_SIZE};
use std::collections::HashMap;
use std::io::Read;

/// Streaming DAG builder for large files
pub struct StreamingDagBuilder {
    file_name: String,
    chunk_size: usize,
    chunks: Vec<DagLeaf>,
    chunk_count: usize,
}

impl StreamingDagBuilder {
    pub fn new(file_name: impl Into<String>) -> Self {
        Self {
            file_name: file_name.into(),
            chunk_size: DEFAULT_CHUNK_SIZE,
            chunks: Vec::new(),
            chunk_count: 0,
        }
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Process a chunk of data and return the current root CID
    pub fn add_chunk(&mut self, data: Vec<u8>) -> Result<String> {
        if data.is_empty() {
            return Err(ScionicError::InvalidLeaf("Empty chunk".to_string()));
        }

        // Create chunk leaf
        let chunk_name = format!("{}/{}", self.file_name, self.chunk_count);
        let chunk_leaf = DagLeafBuilder::new(chunk_name)
            .set_type(LeafType::Chunk)
            .set_data(data)
            .build_leaf(None)?;

        self.chunks.push(chunk_leaf);
        self.chunk_count += 1;

        // Build current root
        self.build_current_root()
    }

    /// Build the current root CID with all chunks so far
    fn build_current_root(&self) -> Result<String> {
        if self.chunks.is_empty() {
            return Err(ScionicError::InvalidDag("No chunks yet".to_string()));
        }

        // Build parent file leaf
        let mut leaf_builder = DagLeafBuilder::new(self.file_name.clone())
            .set_type(LeafType::File);

        for chunk in &self.chunks {
            leaf_builder = leaf_builder.add_link(chunk.hash.clone());
        }

        let parent = leaf_builder.build_leaf(None)?;
        Ok(parent.hash)
    }

    /// Finalize and return the complete DAG
    pub fn finalize(self) -> Result<Dag> {
        if self.chunks.is_empty() {
            return Err(ScionicError::InvalidDag("No chunks to finalize".to_string()));
        }

        let mut leaves = HashMap::new();

        // Add all chunk leaves
        for chunk in &self.chunks {
            leaves.insert(chunk.hash.clone(), chunk.clone());
        }

        // Build root file leaf
        let mut root_builder = DagLeafBuilder::new(self.file_name.clone())
            .set_type(LeafType::File);

        for chunk in &self.chunks {
            root_builder = root_builder.add_link(chunk.hash.clone());
        }

        let root = root_builder.build_root_leaf(&leaves, None)?;
        let root_hash = root.hash.clone();

        leaves.insert(root_hash.clone(), root);

        Ok(Dag {
            root: root_hash,
            leaves,
            labels: None,
        })
    }

    /// Stream from a reader, calling callback with CID after each chunk
    pub fn stream_from_reader<R: Read, F>(
        mut self,
        mut reader: R,
        mut callback: F,
    ) -> Result<Dag>
    where
        F: FnMut(&str),
    {
        let mut buffer = vec![0u8; self.chunk_size];
        let mut chunk_data = Vec::new();

        loop {
            match reader.read(&mut buffer)? {
                0 => break, // EOF
                n => {
                    chunk_data.extend_from_slice(&buffer[..n]);

                    // If we've accumulated a full chunk, process it
                    if chunk_data.len() >= self.chunk_size {
                        let cid = self.add_chunk(chunk_data.clone())?;
                        callback(&cid);
                        chunk_data.clear();
                    }
                }
            }
        }

        // Process remaining data as final chunk
        if !chunk_data.is_empty() {
            let cid = self.add_chunk(chunk_data)?;
            callback(&cid);
        }

        self.finalize()
    }
}

/// Create a streaming DAG from a reader
pub fn create_dag_from_stream<R: Read, F>(
    reader: R,
    file_name: impl Into<String>,
    callback: F,
) -> Result<Dag>
where
    F: FnMut(&str),
{
    let builder = StreamingDagBuilder::new(file_name);
    builder.stream_from_reader(reader, callback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_streaming_small_file() -> Result<()> {
        let data = b"Hello, streaming world!";
        let reader = Cursor::new(data);

        let mut cids = Vec::new();
        let dag = create_dag_from_stream(reader, "test.txt", |cid| {
            cids.push(cid.to_string());
        })?;

        assert!(!dag.root.is_empty());
        assert_eq!(cids.len(), 1); // Small file = 1 chunk = 1 callback

        Ok(())
    }

    #[test]
    fn test_streaming_large_file() -> Result<()> {
        // Create 5MB of data (2.5 chunks at 2MB each)
        let size = 5 * 1024 * 1024;
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let reader = Cursor::new(data);

        let mut cids = Vec::new();
        let dag = create_dag_from_stream(reader, "large.bin", |cid| {
            cids.push(cid.to_string());
        })?;

        assert!(!dag.root.is_empty());
        assert_eq!(cids.len(), 3); // 5MB = 3 chunks (2MB + 2MB + 1MB)

        // Verify all CIDs changed as chunks were added
        assert_ne!(cids[0], cids[1]);
        assert_ne!(cids[1], cids[2]);

        Ok(())
    }

    #[test]
    fn test_streaming_incremental_cid() -> Result<()> {
        let mut builder = StreamingDagBuilder::new("test.txt");

        let cid1 = builder.add_chunk(b"chunk1".to_vec())?;
        let cid2 = builder.add_chunk(b"chunk2".to_vec())?;
        let cid3 = builder.add_chunk(b"chunk3".to_vec())?;

        // Each addition should produce different root CID
        assert_ne!(cid1, cid2);
        assert_ne!(cid2, cid3);

        let dag = builder.finalize()?;
        assert_eq!(dag.leaves.len(), 4); // 3 chunks + 1 parent

        Ok(())
    }
}
