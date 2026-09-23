//! Experimental bindings for the [SPIFFE Broker API].
//!
//! A broker, such as a node proxy, uses the Broker API to get SVIDs and bundles for the
//! workloads it serves. Every request names one workload with a [`WorkloadReference`].
//! Build one from a [`WorkloadPidReference`] or a [`KubernetesObjectReference`] with
//! [`From`], which also sets the type URL that providers match.
//!
//! The [SPIFFE Broker Endpoint] requires mutual TLS with X509-SVIDs, so the caller builds
//! the [`tonic::transport::Channel`]. The channel must also check that the provider presents
//! its expected SPIFFE ID. Every request must also carry the security header.
//! Use [`add_security_header`] as the interceptor of the generated client:
//!
//! ```no_run
//! # fn example(channel: tonic::transport::Channel) {
//! use spiffe::broker_api::add_security_header;
//! use spiffe::broker_api::pb::spiffe::broker::{
//!     api_client::ApiClient, KubernetesObjectKey, KubernetesObjectReference,
//!     KubernetesObjectType, WorkloadReference,
//! };
//!
//! let _client = ApiClient::with_interceptor(channel, add_security_header);
//! let _pod = WorkloadReference::from(KubernetesObjectReference {
//!     r#type: Some(KubernetesObjectType {
//!         plural: "pods".into(),
//!         group: "core".into(),
//!     }),
//!     key: Some(KubernetesObjectKey {
//!         namespace: "default".into(),
//!         name: "web-0".into(),
//!     }),
//!     uid: "a1b2c3d4-0000-0000-0000-000000000000".into(),
//! });
//! # }
//! ```
//!
//! This module is experimental. The Broker API is new, and SPIRE ships it under its
//! `experimental` configuration, so breaking changes can follow the specification. The
//! generated types also expose `prost` and `tonic`, so a major upgrade of either can break
//! users of this module.
//!
//! [`WorkloadReference`]: pb::spiffe::broker::WorkloadReference
//! [`WorkloadPidReference`]: pb::spiffe::broker::WorkloadPidReference
//! [`KubernetesObjectReference`]: pb::spiffe::broker::KubernetesObjectReference
//! [SPIFFE Broker API]: https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE_Broker_API.md
//! [SPIFFE Broker Endpoint]: https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE_Broker_Endpoint.md

use crate::broker_api::pb::spiffe::broker::{
    Jwtsvid, KubernetesObjectReference, WorkloadPidReference, WorkloadReference, X509svid,
};
use prost::Name;
use tonic::metadata::{Ascii, MetadataKey, MetadataValue};

/// Generated protobuf bindings for the SPIFFE Broker API.
///
/// **This module contains generated code. Do not edit these files manually.**
///
/// Generated from `standards/brokerapi.proto` at spiffe/spiffe commit `69464c3`.
/// Regenerate with: `cargo run -p xtask -- gen spiffe` from the repo root.
#[expect(
    clippy::allow_attributes_without_reason,
    clippy::derive_partial_eq_without_eq,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::too_long_first_doc_paragraph,
    missing_docs,
    unused_qualifications,
    unused_results
)]
pub mod pb {
    pub mod spiffe {
        pub mod broker {
            include!("pb/broker.rs");
        }
    }
}

const BROKER_HEADER_KEY: &str = "broker.spiffe.io";
const BROKER_HEADER_VALUE: &str = "true";

// These are fixed ASCII string literals, so parsing always succeeds.
static PARSED_HEADER_KEY: std::sync::LazyLock<MetadataKey<Ascii>> =
    std::sync::LazyLock::new(|| MetadataKey::from_static(BROKER_HEADER_KEY));

static PARSED_HEADER_VALUE: std::sync::LazyLock<MetadataValue<Ascii>> =
    std::sync::LazyLock::new(|| MetadataValue::from_static(BROKER_HEADER_VALUE));

/// Adds the Broker Endpoint security header, `broker.spiffe.io: true`, to a request.
///
/// A provider rejects a request without this header with `InvalidArgument`.
///
/// # Errors
///
/// Never. The signature matches [`tonic::service::Interceptor`].
pub fn add_security_header(
    mut request: tonic::Request<()>,
) -> Result<tonic::Request<()>, tonic::Status> {
    request
        .metadata_mut()
        .insert(PARSED_HEADER_KEY.clone(), PARSED_HEADER_VALUE.clone());
    Ok(request)
}

impl From<WorkloadPidReference> for WorkloadReference {
    fn from(reference: WorkloadPidReference) -> Self {
        pack(&reference)
    }
}

impl From<KubernetesObjectReference> for WorkloadReference {
    fn from(reference: KubernetesObjectReference) -> Self {
        pack(&reference)
    }
}

// The SVID messages skip the generated `Debug`, so their private key or token never
// reaches a log.
impl std::fmt::Debug for X509svid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("X509svid")
            .field("spiffe_id", &self.spiffe_id)
            .field("hint", &self.hint)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for Jwtsvid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Jwtsvid")
            .field("spiffe_id", &self.spiffe_id)
            .field("hint", &self.hint)
            .finish_non_exhaustive()
    }
}

fn pack<M: Name>(reference: &M) -> WorkloadReference {
    WorkloadReference {
        reference: Some(prost_types::Any {
            type_url: M::type_url(),
            value: reference.encode_to_vec(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker_api::pb::spiffe::broker::{KubernetesObjectKey, KubernetesObjectType};

    fn pod() -> KubernetesObjectReference {
        KubernetesObjectReference {
            r#type: Some(KubernetesObjectType {
                plural: "pods".into(),
                group: "core".into(),
            }),
            key: Some(KubernetesObjectKey {
                namespace: "default".into(),
                name: "web-0".into(),
            }),
            uid: "a1b2c3d4".into(),
        }
    }

    // SPIRE matches these exact strings; a missing domain is rejected as unsupported.
    #[test]
    fn references_carry_full_type_urls() {
        let pid = WorkloadReference::from(WorkloadPidReference { pid: 42 });
        assert_eq!(
            pid.reference.unwrap().type_url,
            "type.googleapis.com/spiffe.broker.WorkloadPIDReference"
        );
        let object = WorkloadReference::from(pod());
        assert_eq!(
            object.reference.unwrap().type_url,
            "type.googleapis.com/spiffe.broker.KubernetesObjectReference"
        );
    }

    #[test]
    fn kubernetes_reference_round_trips() {
        let any = WorkloadReference::from(pod()).reference.unwrap();
        assert_eq!(any.to_msg::<KubernetesObjectReference>().unwrap(), pod());
    }

    #[test]
    fn debug_hides_svid_secrets() {
        let x509 = X509svid {
            spiffe_id: "spiffe://td/w".into(),
            x509_svid_key: "SECRET-KEY".into(),
            ..Default::default()
        };
        let jwt = Jwtsvid {
            spiffe_id: "spiffe://td/w".into(),
            svid: "SECRET-TOKEN".into(),
            ..Default::default()
        };
        let printed = format!("{x509:?} {jwt:?}");
        assert!(printed.contains("spiffe://td/w"));
        assert!(!printed.contains("SECRET"));
    }

    #[test]
    fn interceptor_adds_security_header() {
        let request = add_security_header(tonic::Request::new(())).unwrap();
        assert_eq!(
            request.metadata().get(BROKER_HEADER_KEY).unwrap(),
            BROKER_HEADER_VALUE
        );
    }
}
