//! Content-addressed model manifest, per docs/PROTOCOL.md. Weights are
//! never stored on-chain or in this struct directly -- only their content
//! identifiers (CIDs). CIDs are plain strings here rather than a typed CID
//! type because this crate does not yet depend on an IPFS/CID library
//! (that dependency belongs to crates/aion-storage, Phase 6); this struct
//! only needs to carry the identifiers, not resolve them.

#[derive(Debug, Clone, PartialEq)]
pub struct ModelManifestMetadata {
    pub id: String,
    pub creator: String,
    pub architecture: String,
    pub version: String,
    pub parent_model: Option<String>,
    pub license: String,
    pub size_bytes: u64,
    pub quantization: Option<String>,
    pub runtime_requirements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelManifest {
    pub metadata: ModelManifestMetadata,
    pub weights_cid: String,
    pub tokenizer_cid: String,
    pub config_cid: String,
    pub license_cid: String,
    pub benchmark_manifest_cid: Option<String>,
}

impl ModelManifest {
    /// True if this manifest's `parent_model` link would introduce a
    /// self-referential (single-node) cycle in the provenance DAG. Full
    /// multi-hop cycle detection requires walking the actual DAG (owned by
    /// a future model registry, not this struct) -- this is only the
    /// cheap, local check every manifest must pass regardless of registry
    /// state.
    pub fn has_self_referential_parent(&self) -> bool {
        self.metadata.parent_model.as_deref() == Some(self.metadata.id.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ModelManifest {
        ModelManifest {
            metadata: ModelManifestMetadata {
                id: "model-a".into(),
                creator: "creator-1".into(),
                architecture: "transformer".into(),
                version: "0.1.0".into(),
                parent_model: None,
                license: "apache-2.0".into(),
                size_bytes: 1_000_000,
                quantization: Some("q4_0".into()),
                runtime_requirements: vec!["ollama".into()],
            },
            weights_cid: "cid-weights".into(),
            tokenizer_cid: "cid-tokenizer".into(),
            config_cid: "cid-config".into(),
            license_cid: "cid-license".into(),
            benchmark_manifest_cid: None,
        }
    }

    #[test]
    fn normal_manifest_has_no_self_referential_parent() {
        assert!(!sample().has_self_referential_parent());
    }

    #[test]
    fn self_referential_parent_is_detected() {
        let mut m = sample();
        m.metadata.parent_model = Some("model-a".into()); // same as its own id
        assert!(m.has_self_referential_parent());
    }
}
