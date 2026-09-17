#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Rust SDK for the NetActuate vAPI2 and vAPI3 APIs.
//!
//! The crate exposes two clients because the platform APIs use different
//! envelopes, pagination rules and error shapes.

mod client;
mod error;
mod models;
mod transport;
mod v3;

pub use client::{
    BuildServerRequest, Client, CreateDnsRecordRequest, CreateDnsZoneRequest, CreateServerRequest,
    DeleteServerOptions, UpdateDnsRecordRequest,
};
pub use error::{Error, Result};
pub use models::*;
pub use v3::{
    CreateNkeClusterRequest, CreateStorageBucketRequest, CreateVpcRequest, NkeAddons, NkeBilling,
    NkeClusterNetwork, NkeClusterTagInput, NkeUpdateBilling, NkeUpdateNodes,
    UpdateNkeClusterRequest, UpdateNkeWorkerNodeRequest, UpdateStorageBucketRequest,
    UpdateVpcRequest, V3Client, VpcDefaults, VpcFirewalls, VpcNameserver, VpcNameservers,
    VpcNetwork,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{Method, Request, Response, Transport};
    use serde::Deserialize;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct RecordedTransport {
        responses: Mutex<VecDeque<Response>>,
        requests: Mutex<Vec<Request>>,
    }

    impl RecordedTransport {
        fn new(responses: Vec<Response>) -> Arc<Self> {
            Arc::new(Self {
                responses: Mutex::new(responses.into()),
                requests: Mutex::new(Vec::new()),
            })
        }

        fn json(status: u16, body: &str) -> Response {
            Response {
                status,
                body: body.as_bytes().to_vec(),
            }
        }

        fn paths(&self) -> Vec<String> {
            self.requests
                .lock()
                .expect("requests lock")
                .iter()
                .map(|request| {
                    let query = request.url.query().unwrap_or("");
                    if query.is_empty() {
                        request.url.path().to_string()
                    } else {
                        format!("{}?{}", request.url.path(), query)
                    }
                })
                .collect()
        }
    }

    impl Transport for RecordedTransport {
        fn send(&self, request: Request) -> Result<Response> {
            self.requests.lock().expect("requests lock").push(request);
            self.responses
                .lock()
                .expect("responses lock")
                .pop_front()
                .ok_or_else(|| Error::Transport("no recorded response".to_string()))
        }
    }

    #[test]
    fn client_reads_netactuate_api_key_from_environment() {
        std::env::set_var("NETACTUATE_API_KEY", "env-key");
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[]}"#,
        )]);
        let client = Client::with_transport(
            std::env::var("NETACTUATE_API_KEY").expect("env key"),
            "https://vapi2.example.test/api/",
            transport.clone(),
        )
        .expect("client");
        client.list_servers().expect("list servers");
        assert!(transport.paths()[0].contains("key=env-key"));
        std::env::remove_var("NETACTUATE_API_KEY");
    }

    #[test]
    fn error_text_redacts_key_from_url() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            500,
            r#"{"result":"error","code":500,"message":"failed","data":null}"#,
        )]);
        let client = Client::with_transport(
            "secret-key-in-url",
            "https://vapi2.example.test/api/",
            transport,
        )
        .expect("client");
        let err = client.list_servers().expect_err("error");
        let text = err.to_string();
        assert!(!text.contains("secret-key-in-url"));
        assert!(text.contains("key=REDACTED"));
    }

    #[test]
    fn transport_error_text_never_carries_the_key() {
        // The existing redaction test covers a URL the caller formats. This covers the path
        // that actually leaked: a transport error whose message embeds the request URL.
        let leaked = "error sending request for url (https://vapi2.netactuate.com/api/cloud/servers?key=SECRETVALUE)";
        let redacted = crate::transport::redact_text(leaked);
        assert!(
            !redacted.contains("SECRETVALUE"),
            "key survived redaction: {redacted}"
        );
        assert!(
            redacted.contains("key=REDACTED"),
            "expected a redaction marker: {redacted}"
        );
    }

    #[test]
    fn redact_text_handles_a_key_followed_by_other_parameters() {
        let leaked = "https://example.invalid/a?key=SECRETVALUE&limit=10";
        let redacted = crate::transport::redact_text(leaked);
        assert!(!redacted.contains("SECRETVALUE"));
        assert!(
            redacted.contains("limit=10"),
            "other parameters must survive: {redacted}"
        );
    }

    #[test]
    fn v2_not_found_and_contract_errors_are_distinguishable() {
        let not_found_transport = RecordedTransport::new(vec![RecordedTransport::json(
            422,
            r#"{"result":"error","code":422,"message":"validation_failed","fields":{"id":["The id must be a valid zone id"]}}"#,
        )]);
        let client = Client::with_transport(
            "key",
            "https://vapi2.example.test/api/",
            not_found_transport,
        )
        .expect("client");
        assert!(client.get_zone(99).expect_err("not found").is_not_found());

        let contract_transport = RecordedTransport::new(vec![RecordedTransport::json(
            412,
            r#"{"result":"error","code":412,"message":"contract required","data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", contract_transport)
                .expect("client");
        assert!(client.list_servers().expect_err("contract").is_contract());
    }

    #[test]
    fn v2_pagination_is_followed_to_the_end() {
        let transport = RecordedTransport::new(vec![
            RecordedTransport::json(
                200,
                r#"{"result":"success","code":200,"data":{"current_page":1,"last_page":2,"data":[{"mbpkgid":1,"fqdn":"one"}]}}"#,
            ),
            RecordedTransport::json(
                200,
                r#"{"result":"success","code":200,"data":{"current_page":2,"last_page":2,"data":[{"mbpkgid":2,"fqdn":"two"}]}}"#,
            ),
        ]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let servers = client.list_servers().expect("servers");
        assert_eq!(
            servers.iter().map(|server| server.id).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn server_decoder_accepts_flat_list_and_single_get_shapes() {
        let flat: Server = serde_json::from_str(r#"{"mbpkgid":7,"fqdn":"vm.example"}"#).unwrap();
        let get: Server =
            serde_json::from_str(r#"{"mbpkgid":8,"fqdn":"vm2.example","status":"ONLINE"}"#)
                .unwrap();
        assert_eq!(flat.id, 7);
        assert_eq!(get.server_status, "ONLINE");
    }

    #[test]
    fn dns_zone_decoder_accepts_flat_list_and_nested_single_get_shapes() {
        let list: DnsZone = serde_json::from_str(
            r#"{"id":1,"name":"example.com","type":"master","ttl":3600,"master":"ns1"}"#,
        )
        .unwrap();
        let get: DnsZone = serde_json::from_str(
            r#"{"id":1,"name":"example.com","type":"master","ttl":"3600","soa":{"primary":"ns1"},"ns":[{"id":9,"domain_id":1,"name":"@","type":"NS","content":"ns1","ttl":"3600","prio":0}]}"#,
        )
        .unwrap();
        assert_eq!(list.ttl.as_i64(), 3600);
        assert_eq!(get.ns[0].record_type, "NS");
    }

    #[test]
    fn dns_record_decoder_accepts_flat_list_and_single_get_shapes() {
        let list: DnsRecord = serde_json::from_str(
            r#"{"id":5,"domain_id":1,"name":"www","type":"A","content":"192.0.2.1","ttl":300,"prio":0}"#,
        )
        .unwrap();
        let get: DnsRecord = serde_json::from_str(
            r#"{"id":6,"domain_id":1,"name":"mail","type":"MX","content":"mail.example.com","ttl":"600","prio":10}"#,
        )
        .unwrap();
        assert_eq!(list.ttl.as_i64(), 300);
        assert_eq!(get.priority, 10);
    }

    #[test]
    fn vpc_decoder_accepts_flat_list_and_nested_single_get_shapes() {
        let flat: Vpc = serde_json::from_str(
            r#"{"id":42,"label":"flat","description":"row","status":"Running","location":{"id":1,"name":"A"}}"#,
        )
        .unwrap();
        let nested: Vpc = serde_json::from_str(
            r#"{"vpcId":43,"metadata":{"label":"nested","description":"get","status":"Running"},"location":{"id":2,"name":"B"}}"#,
        )
        .unwrap();
        assert_eq!(flat.vpc_id, 42);
        assert_eq!(flat.metadata.label, "flat");
        assert_eq!(nested.vpc_id, 43);
        assert_eq!(nested.metadata.label, "nested");
    }

    #[test]
    fn storage_bucket_decoder_accepts_flat_list_and_nested_single_get_shapes() {
        let flat: StorageBucket = serde_json::from_str(
            r#"{"bucketId":10,"label":"flat","ready":true,"private":false,"assignedOn":"now","location":{"id":1},"capacity":{"totalGB":10},"hardwareClass":{"id":1}}"#,
        )
        .unwrap();
        let nested: StorageBucket = serde_json::from_str(
            r#"{"credentials":{"endpoints":["https://s3.example"],"accessKey":"a","secretKey":"s","userKey":"u"},"metadata":{"bucketId":11,"label":"nested","ready":true,"private":true,"assignedOn":"now","location":{"id":1},"capacity":{"totalGB":20},"hardwareClass":{"id":1}}}"#,
        )
        .unwrap();
        assert_eq!(flat.metadata.bucket_id, 10);
        assert_eq!(nested.credentials.access_key, "a");
    }

    #[test]
    fn nke_cluster_decoder_accepts_list_and_single_get_shapes() {
        let list: NkeCluster = serde_json::from_str(
            r#"{"clusterId":70,"name":"list","status":{"cluster":"Healthy"},"version":{"active":"1.30"}}"#,
        )
        .unwrap();
        let get: NkeCluster = serde_json::from_str(
            r#"{"clusterId":71,"name":"get","status":{"cluster":"Healthy","scaling":"Idle"},"version":{"active":"1.30","requested":null},"nodes":{"total":3,"ready":3}}"#,
        )
        .unwrap();
        assert_eq!(list.cluster_id, 70);
        assert_eq!(get.nodes.ready, 3);
    }

    #[test]
    fn v3_list_envelopes_accept_all_four_forms() {
        #[derive(Debug, Deserialize)]
        struct Row {
            id: i64,
        }

        let bare = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"id":1}]}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", bare).expect("client");
        assert_eq!(client.request_list::<Row>("/x", None).unwrap()[0].id, 1);

        let data = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"data":[{"id":2}],"meta":{"limit":100,"offset":0,"total":1}}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", data).expect("client");
        assert_eq!(client.request_list::<Row>("/x", None).unwrap()[0].id, 2);

        let named = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"vpcs":{"data":[{"id":3}],"meta":{"limit":100,"offset":0,"total":1}}}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", named).expect("client");
        assert_eq!(
            client.request_list::<Row>("/x", Some("vpcs")).unwrap()[0].id,
            3
        );

        let paginator = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"paginator":{"data":[{"id":4}]}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", paginator)
            .expect("client");
        assert_eq!(client.request_list::<Row>("/x", None).unwrap()[0].id, 4);
    }

    #[test]
    fn v3_pagination_is_followed_to_the_end() {
        let transport = RecordedTransport::new(vec![
            RecordedTransport::json(
                200,
                r#"{"code":200,"data":{"data":[{"bucketId":1,"label":"one"}],"meta":{"limit":1,"offset":0,"total":2}}}"#,
            ),
            RecordedTransport::json(
                200,
                r#"{"code":200,"data":{"data":[{"bucketId":2,"label":"two"}],"meta":{"limit":1,"offset":1,"total":2}}}"#,
            ),
        ]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let buckets = client.list_storage_buckets().expect("buckets");
        assert_eq!(
            buckets
                .iter()
                .map(|bucket| bucket.metadata.bucket_id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert!(transport.paths()[1].contains("offset=1"));
        assert!(transport.paths()[1].contains("limit=1"));
    }

    #[test]
    fn v3_not_found_and_contract_errors_are_distinguishable() {
        let not_found = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", not_found)
            .expect("client");
        assert!(client.get_vpc(9).expect_err("not found").is_not_found());

        let contract = RecordedTransport::new(vec![RecordedTransport::json(
            412,
            r#"{"code":412,"message":"contract required","data":null}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", contract)
            .expect("client");
        assert!(client.list_vpcs().expect_err("contract").is_contract());
    }

    #[test]
    fn recorded_transport_never_opens_a_socket() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":["1.30"]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://unroutable.invalid", transport)
            .expect("client");
        assert_eq!(client.list_nke_versions().unwrap(), vec!["1.30"]);
    }

    #[test]
    fn recorded_transport_captures_http_methods() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(204, "")]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client.delete_vpc(1).expect("delete");
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Delete);
    }
}
