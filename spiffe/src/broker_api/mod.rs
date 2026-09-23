//! Experimental bindings for the [SPIFFE Broker API].
//!
//! A broker, such as a node proxy, uses the Broker API to get SVIDs and bundles for the
//! workloads it serves. Every request names one workload with a [`pb::WorkloadReference`].
//! Build one from a [`pb::WorkloadPidReference`] or a [`pb::KubernetesObjectReference`]
//! with [`From`], which also sets the type URL that providers match.
//!
//! The [SPIFFE Broker Endpoint] requires mutual TLS with X509-SVIDs, so the caller builds
//! the [`tonic::transport::Channel`]. Every request must also carry the security header.
//! Use [`add_security_header`] as the interceptor of the generated client:
//!
//! ```no_run
//! # fn example(channel: tonic::transport::Channel) {
//! use spiffe::broker_api::{add_security_header, pb};
//!
//! let _client = pb::api_client::ApiClient::with_interceptor(channel, add_security_header);
//! let _pod = pb::WorkloadReference::from(pb::KubernetesObjectReference {
//!     r#type: Some(pb::KubernetesObjectType {
//!         plural: "pods".into(),
//!         group: "core".into(),
//!     }),
//!     key: Some(pb::KubernetesObjectKey {
//!         namespace: "default".into(),
//!         name: "web-0".into(),
//!     }),
//!     uid: "a1b2c3d4-0000-0000-0000-000000000000".into(),
//! });
//! # }
//! ```
//!
//! This module is experimental. The Broker API is new, and SPIRE ships it under its
//! `experimental` configuration, so breaking changes can follow the specification.
//!
//! [SPIFFE Broker API]: https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE_Broker_API.md
//! [SPIFFE Broker Endpoint]: https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE_Broker_Endpoint.md

use prost::Name;

/// Generated protobuf bindings for the `spiffe.broker` package.
///
/// Copied from `standards/brokerapi.proto` at spiffe/spiffe commit `69464c3`.
/// Regenerate with `cargo run -p xtask -- gen spiffe` from the repo root.
#[expect(
    clippy::allow_attributes_without_reason,
    clippy::derive_partial_eq_without_eq,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::too_long_first_doc_paragraph,
    unused_qualifications,
    unused_results
)]
pub mod pb {
    include!("pb/broker.rs");
}

/// The gRPC metadata key that every Broker Endpoint request must carry.
pub const SECURITY_HEADER_KEY: &str = "broker.spiffe.io";

/// The value of [`SECURITY_HEADER_KEY`]. It is case sensitive.
pub const SECURITY_HEADER_VALUE: &str = "true";

/// Adds the Broker Endpoint security header to a request.
///
/// A provider rejects a request without this header with `InvalidArgument`.
///
/// # Errors
///
/// Never. The signature matches [`tonic::service::Interceptor`].
pub fn add_security_header(
    mut request: tonic::Request<()>,
) -> Result<tonic::Request<()>, tonic::Status> {
    let _previous = request.metadata_mut().insert(
        SECURITY_HEADER_KEY,
        tonic::metadata::MetadataValue::from_static(SECURITY_HEADER_VALUE),
    );
    Ok(request)
}

impl From<pb::WorkloadPidReference> for pb::WorkloadReference {
    fn from(reference: pb::WorkloadPidReference) -> Self {
        pack(&reference)
    }
}

impl From<pb::KubernetesObjectReference> for pb::WorkloadReference {
    fn from(reference: pb::KubernetesObjectReference) -> Self {
        pack(&reference)
    }
}

fn pack<M: Name>(reference: &M) -> pb::WorkloadReference {
    pb::WorkloadReference {
        reference: Some(prost_types::Any {
            type_url: M::type_url(),
            value: reference.encode_to_vec(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pod() -> pb::KubernetesObjectReference {
        pb::KubernetesObjectReference {
            r#type: Some(pb::KubernetesObjectType {
                plural: "pods".into(),
                group: "core".into(),
            }),
            key: Some(pb::KubernetesObjectKey {
                namespace: "default".into(),
                name: "web-0".into(),
            }),
            uid: "a1b2c3d4".into(),
        }
    }

    // SPIRE matches these exact strings; a missing domain is rejected as unsupported.
    #[test]
    fn references_carry_full_type_urls() {
        let pid = pb::WorkloadReference::from(pb::WorkloadPidReference { pid: 42 });
        assert_eq!(
            pid.reference.unwrap().type_url,
            "type.googleapis.com/spiffe.broker.WorkloadPIDReference"
        );
        let object = pb::WorkloadReference::from(pod());
        assert_eq!(
            object.reference.unwrap().type_url,
            "type.googleapis.com/spiffe.broker.KubernetesObjectReference"
        );
    }

    #[test]
    fn kubernetes_reference_round_trips() {
        let any = pb::WorkloadReference::from(pod()).reference.unwrap();
        assert_eq!(
            any.to_msg::<pb::KubernetesObjectReference>().unwrap(),
            pod()
        );
    }

    #[test]
    fn interceptor_adds_security_header() {
        let request = add_security_header(tonic::Request::new(())).unwrap();
        assert_eq!(
            request.metadata().get(SECURITY_HEADER_KEY).unwrap(),
            SECURITY_HEADER_VALUE
        );
    }
}
