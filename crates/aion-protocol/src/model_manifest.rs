//! Content-addressed model manifest, per docs/PROTOCOL.md. Weights are
//! never stored on-chain or in this struct directly -- only their content
//! identifiers (CIDs). CIDs remain plain strings on this struct (not a
//! typed `cid::Cid`) since the wire-format type only needs to carry an
//! identifier, not resolve one -- but as of milestone 30, those strings
//! can be REAL, IPFS-compatible CIDs: [`ModelManifest::with_computed_cids`]
//! computes them directly from each artifact's actual bytes via
//! `aion_storage::compute_cid` (crates/aion-storage, Phase 6), replacing
//! the historical pattern (still visible in this file's own tests, where
//! it's an intentional placeholder) of hand-typed strings like
//! `"cid-weights"` that don't refer to anything real.

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

    /// Builds a manifest whose CID fields are REAL, content-addressed
    /// identifiers computed directly from each artifact's actual bytes
    /// via `aion_storage::compute_cid` -- the same function
    /// `crates/aion-storage`'s `ContentStore` uses internally, so a CID
    /// produced here is exactly the identifier that store would compute
    /// (and require) for that same content. `benchmark_manifest` is
    /// `None` when a model has no associated network benchmark (an
    /// optional field on the wire format too, see `benchmark_manifest_cid`).
    pub fn with_computed_cids(
        metadata: ModelManifestMetadata,
        weights: &[u8],
        tokenizer: &[u8],
        config: &[u8],
        license: &[u8],
        benchmark_manifest: Option<&[u8]>,
    ) -> Self {
        Self {
            metadata,
            weights_cid: aion_storage::compute_cid(weights).to_string(),
            tokenizer_cid: aion_storage::compute_cid(tokenizer).to_string(),
            config_cid: aion_storage::compute_cid(config).to_string(),
            license_cid: aion_storage::compute_cid(license).to_string(),
            benchmark_manifest_cid: benchmark_manifest
                .map(|bytes| aion_storage::compute_cid(bytes).to_string()),
        }
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

    fn sample_metadata() -> ModelManifestMetadata {
        ModelManifestMetadata {
            id: "model-a".into(),
            creator: "creator-1".into(),
            architecture: "transformer".into(),
            version: "0.1.0".into(),
            parent_model: None,
            license: "apache-2.0".into(),
            size_bytes: 1_000_000,
            quantization: Some("q4_0".into()),
            runtime_requirements: vec!["ollama".into()],
        }
    }

    #[test]
    fn with_computed_cids_matches_aion_storages_own_compute_cid_directly() {
        // Cross-crate consistency: a CID this constructor produces must be
        // bit-identical to what aion_storage::compute_cid computes
        // directly on the same bytes -- not merely "some hash," but the
        // exact same real, content-addressed identifier.
        let weights = b"pretend model weights bytes";
        let manifest = ModelManifest::with_computed_cids(
            sample_metadata(),
            weights,
            b"pretend tokenizer bytes",
            b"pretend config bytes",
            b"pretend license bytes",
            None,
        );
        assert_eq!(
            manifest.weights_cid,
            aion_storage::compute_cid(weights).to_string()
        );
    }

    #[test]
    fn distinct_artifacts_get_distinct_cids() {
        let manifest = ModelManifest::with_computed_cids(
            sample_metadata(),
            b"weights bytes",
            b"tokenizer bytes",
            b"config bytes",
            b"license bytes",
            None,
        );
        let cids = [
            &manifest.weights_cid,
            &manifest.tokenizer_cid,
            &manifest.config_cid,
            &manifest.license_cid,
        ];
        for i in 0..cids.len() {
            for j in (i + 1)..cids.len() {
                assert_ne!(cids[i], cids[j], "CIDs at {i} and {j} unexpectedly matched");
            }
        }
    }

    #[test]
    fn benchmark_manifest_cid_is_none_without_benchmark_bytes_and_some_with_them() {
        let without =
            ModelManifest::with_computed_cids(sample_metadata(), b"w", b"t", b"c", b"l", None);
        assert_eq!(without.benchmark_manifest_cid, None);

        let with = ModelManifest::with_computed_cids(
            sample_metadata(),
            b"w",
            b"t",
            b"c",
            b"l",
            Some(b"benchmark manifest bytes"),
        );
        assert_eq!(
            with.benchmark_manifest_cid,
            Some(aion_storage::compute_cid(b"benchmark manifest bytes").to_string())
        );
    }

    #[test]
    fn a_manifests_weights_cid_actually_resolves_through_a_real_content_store() {
        // The end-to-end proof this wiring exists for: a ModelManifest's
        // CID field, once computed this way, is not just a string that
        // LOOKS like a CID -- it's the exact identifier a real
        // aion_storage::ContentStore uses to store and retrieve that same
        // content. Put the weights bytes into a real store, then confirm
        // the manifest's own weights_cid parses back into the same CID
        // the store handed back, and that CID actually retrieves the
        // original bytes.
        let weights = b"weights bytes actually stored in a real ContentStore";
        let dir = tempfile::tempdir().unwrap();
        let store = aion_storage::ContentStore::open(dir.path()).unwrap();
        let stored_cid = store.put(weights).unwrap();

        let manifest = ModelManifest::with_computed_cids(
            sample_metadata(),
            weights,
            b"tokenizer",
            b"config",
            b"license",
            None,
        );

        assert_eq!(manifest.weights_cid, stored_cid.to_string());
        let parsed_cid: cid::Cid = manifest.weights_cid.parse().unwrap();
        assert_eq!(store.get(&parsed_cid).unwrap(), weights);
    }
}
