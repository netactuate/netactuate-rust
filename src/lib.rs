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

/// Builds a manual `Debug` impl for a struct that has one or more credential fields, showing
/// those fields as the literal string "REDACTED" while every other field prints normally.
///
/// Usage: `redacted_debug!(StructName; normal_field_a, normal_field_b; secret_field_a, secret_field_b);`
#[macro_export]
macro_rules! redacted_debug {
    ($t:ident; $($f:ident),* $(,)? ; $($s:ident),+ $(,)?) => {
        impl ::std::fmt::Debug for $t {
            fn fmt(&self, out: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                out.debug_struct(stringify!($t))
                    $(.field(stringify!($f), &self.$f))*
                    $(.field(stringify!($s), &"REDACTED"))*
                    .finish()
            }
        }
    };
}

pub use client::{
    AttemptSshRequest, BgpDashboardOptions, BindBgpGroupFirewallSetRequest, BindFirewallSetRequest,
    BuildServerRequest, BuyBgpPrefixesRequest, BuyBuildDedicatedServerRequest, Client,
    CreateBgpGroupRequest, CreateDnsRecordRequest, CreateDnsZoneRequest, CreateFirewallRuleRequest,
    CreateImageRequest, CreateImageResponse, CreateServerRequest, CreateTagRequest,
    CreateTicketRequest, CreateUsageContractRequest, DedicatedDeviceFilterOptions,
    DedicatedIpv4ReverseRequest, DedicatedServerActionRequest, DedicatedServerBuildRequest,
    DeleteImageResponse, DeleteServerOptions, DeleteServerRequest, DeploySizesRequest,
    FirewallAvailableVmOptions, FirewallRelatedSetOptions, PlatformLookingGlassExecuteOptions,
    RebuildDedicatedServerRequest, ReorderFirewallRulesRequest, ReplaceImageRequest,
    ReplyTicketRequest, RescueStartRequest, ResetRootPasswordRequest, ScaleServerRequest,
    ScalingOptionsRequest, ServerActionRequest, ServerNicAttachRequest, ServerNicUpdateRequest,
    TicketListOptions, UpdateDnsRecordRequest, UpdateServerOptionsRequest, UpdateTagRequest,
};
pub use error::{Error, Result};
pub use models::*;
pub use v3::{
    AddMeshRouterRequest, CloudFloatingIpv4VmRef, CreateCloudFloatingIpv4Request,
    CreateHttpLbGroupRequest, CreateMagicMeshRequest, CreateNkeAddonRequest,
    CreateNkeClusterRequest, CreateNlbGroupRequest, CreateOidcClientKeyRequest,
    CreateOidcClientRequest, CreateRouterPrefixListRequest, CreateRouterRequest,
    CreateRouterStaticRouteRequest, CreateRouterVrfBgpNeighborRequest,
    CreateRouterVrfDnatRuleRequest, CreateRouterVrfInterfaceRequest,
    CreateRouterVrfInterfaceWireguardPeerRequest, CreateRouterVrfIpSecPeerRequest,
    CreateRouterVrfRequest, CreateRouterVrfSnatRuleRequest, CreateRouterVrfTunnelRequest,
    CreateSslCertificateRequest, CreateStorageBlockNamespaceRequest,
    CreateStorageBlockVolumeRequest, CreateStorageBucketRequest, CreateStorageObjectStoreRequest,
    CreateVpcBackendRequest, CreateVpcBackendTemplateRequest, CreateVpcDnatRuleRequest,
    CreateVpcFirewallRuleRequest, CreateVpcFloatingIpRequest, CreateVpcRequest,
    CreateVpcSnatRuleRequest, EnableVpcSshKeyRequest, GrantCloudFloatingIpv4VmsRequest,
    MeshRouterEntry, NkeAddons, NkeBilling, NkeClusterNetwork, NkeClusterTagInput,
    NkeDnsAddonWriteConfig, NkeStorageAddonWriteConfig, NkeUpdateBilling, NkeUpdateNodes,
    ReplaceHttpLbGroupRequest, ReplaceNlbGroupRequest, ReplaceVpcBackendTemplateRequest,
    ReplaceVpcBackendsRequest, ReplaceVpcNameserversRequest, ReplaceVpcNameserversResponse,
    RevokeCloudFloatingIpv4VmsRequest, RouterRoutingViewRequest, RouterRoutingViewSelector,
    RouterVrfBgpAsn, RouterVrfTunnelRemoteEndpoint, UpdateMagicMeshRequest, UpdateNkeAddonRequest,
    UpdateNkeClusterRequest, UpdateNkeWorkerNodeRequest, UpdateOidcClientKeyRequest,
    UpdateOidcClientRequest, UpdateRouterIpSecConfigRequest, UpdateRouterNtpConfigRequest,
    UpdateRouterPrefixListRequest, UpdateRouterRequest, UpdateRouterStaticRouteRequest,
    UpdateRouterVrfBgpNeighborRequest, UpdateRouterVrfBgpRequest, UpdateRouterVrfDhcpRequest,
    UpdateRouterVrfDnatRuleRequest, UpdateRouterVrfInterfaceRequest,
    UpdateRouterVrfIpSecPeerRequest, UpdateRouterVrfRequest, UpdateRouterVrfSnatRuleRequest,
    UpdateRouterVrfTunnelRequest, UpdateSslCertificateRequest, UpdateStorageBlockNamespaceRequest,
    UpdateStorageBlockVolumeRequest, UpdateStorageBucketRequest, UpdateStorageObjectStoreRequest,
    UpdateVpcBackendRequest, UpdateVpcBackendTemplateRequest, UpdateVpcDnatRuleRequest,
    UpdateVpcFirewallRuleRequest, UpdateVpcFloatingIpRequest, UpdateVpcRequest,
    UpdateVpcSnatRuleRequest, UpdateVpcSshSettingsRequest, V3Client, VpcDefaults, VpcFirewalls,
    VpcNameserver, VpcNameservers, VpcNetwork,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{Method, Request, Response, Transport};
    use serde::Deserialize;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

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
    fn get_ssh_key_treats_a_null_body_as_not_found() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let err = client.get_ssh_key(9).expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn get_ssh_key_returns_the_key_when_present() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":9,"name":"laptop","ssh_key":"ssh-ed25519 AAAA","fingerprint":"aa:bb"}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let key = client.get_ssh_key(9).expect("ssh key");
        assert_eq!(key.id, 9);
        assert_eq!(key.name, "laptop");
        assert_eq!(key.key, "ssh-ed25519 AAAA");
    }

    #[test]
    fn create_ssh_key_sends_a_form_encoded_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"name":"laptop","ssh_key":"ssh-ed25519 AAAA","fingerprint":"aa:bb"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let key = client
            .create_ssh_key("laptop", "ssh-ed25519 AAAA")
            .expect("create");
        assert_eq!(key.fingerprint, "aa:bb");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("name=laptop"));
        assert!(body.contains("ssh_key="));
    }

    #[test]
    fn get_tag_finds_the_matching_tag_by_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"name":"prod"},{"id":2,"name":"staging"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let tag = client.get_tag(2).expect("tag");
        assert_eq!(tag.name, "staging");
    }

    #[test]
    fn get_tag_returns_not_found_when_no_tag_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"name":"prod"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let err = client.get_tag(99).expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn assign_tag_resource_sends_the_identifier_as_a_string() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .assign_tag_resource(5, "server", 4242)
            .expect("assign");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"identifier\":\"4242\""));
        assert!(requests[0].url.path().ends_with("/tags/5/assign-resource"));
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
    fn firewall_set_decodes_enabled_and_is_draft_from_mixed_types() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[
                {"id":1,"name":"one","enabled":true,"is_draft":"0"},
                {"id":2,"name":"two","enabled":"1","is_draft":1}
            ]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let sets = client.get_firewall_sets().expect("sets");
        assert!(sets[0].enabled);
        assert!(!sets[0].is_draft);
        assert!(sets[1].enabled);
        assert!(sets[1].is_draft);
    }

    #[test]
    fn get_firewall_set_reads_the_single_set_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":5,"name":"prod"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let set = client.get_firewall_set(5).expect("set");
        assert_eq!(set.name, "prod");
        assert!(transport.requests.lock().unwrap()[0]
            .url
            .path()
            .ends_with("/firewall/sets/5"));
    }

    #[test]
    fn create_firewall_set_sends_name_description_and_enabled_even_when_empty() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"name":"","enabled":true}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .create_firewall_set("", "", true)
            .expect("create set");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("name="));
        assert!(body.contains("description="));
        assert!(body.contains("enabled=1"));
        assert!(requests[0].url.path().ends_with("/firewall/sets"));
    }

    #[test]
    fn delete_firewall_set_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        client.delete_firewall_set(5).expect("idempotent delete");
    }

    #[test]
    fn enable_firewall_set_puts_an_empty_form_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.enable_firewall_set(5).expect("enable");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Put);
        assert!(requests[0].url.path().ends_with("/firewall/sets/5/enable"));
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/x-www-form-urlencoded")
        );
    }

    #[test]
    fn create_draft_firewall_set_posts_to_the_create_draft_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":9,"is_draft":true}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let draft = client.create_draft_firewall_set(5).expect("draft");
        assert!(draft.is_draft);
        assert!(transport.requests.lock().unwrap()[0]
            .url
            .path()
            .ends_with("/firewall/sets/5/create-draft"));
    }

    #[test]
    fn delete_draft_firewall_set_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        client
            .delete_draft_firewall_set(9)
            .expect("idempotent delete");
    }

    #[test]
    fn get_firewall_rules_coerces_the_firewall_set_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"firewall_set_id":"5","enabled":"1"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let rules = client.get_firewall_rules(5).expect("rules");
        assert_eq!(rules[0].firewall_set_id, 5);
        assert!(rules[0].enabled);
    }

    #[test]
    fn reorder_firewall_rules_posts_json_to_the_re_order_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = ReorderFirewallRulesRequest {
            move_id: 1,
            after_id: Some(2),
            before_id: None,
        };
        client.reorder_firewall_rules(5, &request).expect("reorder");
        let requests = transport.requests.lock().unwrap();
        assert!(requests[0]
            .url
            .path()
            .ends_with("/firewall/sets/5/rules/re-order"));
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"after_id\":2"));
        assert!(!body.contains("before_id"));
    }

    #[test]
    fn create_firewall_rule_always_sends_the_match_criteria_field() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":11,"ip_version":"ipv4"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = CreateFirewallRuleRequest {
            ip_version: "ipv4".to_string(),
            action: "accept".to_string(),
            enabled: true,
            ..Default::default()
        };
        let rule = client
            .create_firewall_rule(5, &request)
            .expect("create rule");
        assert_eq!(rule.id, 11);
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"match_criteria\":null"));
        assert!(requests[0].url.path().ends_with("/firewall/sets/5/rules"));
    }

    #[test]
    fn update_firewall_rule_uses_the_gona_endpoint_quirk() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":11,"action":"drop"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = CreateFirewallRuleRequest {
            action: "drop".to_string(),
            ..Default::default()
        };
        let rule = client
            .update_firewall_rule(5, 11, &request)
            .expect("update rule");
        assert_eq!(rule.action, "drop");
        assert!(transport.requests.lock().unwrap()[0]
            .url
            .path()
            .ends_with("/firewall/5/11"));
    }

    #[test]
    fn delete_firewall_rule_uses_the_gona_endpoint_quirk_and_is_idempotent() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .delete_firewall_rule(5, 11)
            .expect("idempotent delete");
        assert!(transport.requests.lock().unwrap()[0]
            .url
            .path()
            .ends_with("/firewall/5/rules/11"));
    }

    #[test]
    fn get_firewall_set_vms_coerces_ids_sent_as_strings() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":"1","mbpkgid":"42","interface_id":"3","firewall_set_id":"5","set_priority":"0"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let vms = client.get_firewall_set_vms(5).expect("vms");
        assert_eq!(vms[0].mbpkgid, 42);
        assert_eq!(vms[0].interface_id, 3);
    }

    #[test]
    fn get_firewall_set_available_vms_builds_the_query_from_set_options() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let options = FirewallAvailableVmOptions {
            vpc_id: Some(9),
            include_bandwidth: Some(true),
            disable_interface_id_filter: Some(false),
            ..Default::default()
        };
        client
            .get_firewall_set_available_vms(5, &options)
            .expect("available vms");
        let query = transport.paths()[0].clone();
        assert!(query.contains("vpc_id=9"));
        assert!(query.contains("bw=1"));
        assert!(query.contains("disable_interface_id_filter=0"));
        assert!(!query.contains("extref_acct_id"));
    }

    #[test]
    fn attach_firewall_set_vm_sends_a_single_entry_vm_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"mbpkgid":42}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let vms = client.attach_firewall_set_vm(5, 42, 3, 1).expect("attach");
        assert_eq!(vms[0].mbpkgid, 42);
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(
            body.contains("\"vm_list\":[{\"mbpkgid\":42,\"interface_id\":3,\"set_priority\":1}]")
        );
        assert!(requests[0]
            .url
            .path()
            .ends_with("/firewall/sets/5/vm/attach"));
    }

    #[test]
    fn detach_firewall_set_vm_posts_to_the_detach_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.detach_firewall_set_vm(5, 42).expect("detach");
        assert!(transport.requests.lock().unwrap()[0]
            .url
            .path()
            .ends_with("/firewall/sets/5/vm/detach/42"));
    }

    #[test]
    fn firewall_external_ip_set_keeps_the_raw_response() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"name":"office","extra_field":"kept"}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let set = client.get_firewall_external_ip_set(1).expect("ip set");
        assert_eq!(set.name, "office");
        assert_eq!(set.raw["extra_field"], "kept");
    }

    #[test]
    fn firewall_manage_enabled_decodes_the_flag() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"enabled":true}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let enabled = client.get_firewall_manage_enabled().expect("enabled");
        assert!(enabled.enabled);
    }

    #[test]
    fn filter_dedicated_devices_decodes_a_bare_array() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"device_id":1,"cpu_type":"epyc"}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let options = DedicatedDeviceFilterOptions {
            per_page: Some(50),
            cpu_type: Some("epyc".to_string()),
            ..Default::default()
        };
        let devices = client.filter_dedicated_devices(&options).expect("devices");
        assert_eq!(devices[0]["device_id"], 1);
        let query = transport.paths()[0].clone();
        assert!(query.contains("per_page=50"));
        assert!(query.contains("cpu_type=epyc"));
    }

    #[test]
    fn filter_dedicated_devices_decodes_the_nested_paginator_envelope() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"devices":{"paginator":{"data":[{"device_id":2}]}},"columns":["device_id"]}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let devices = client
            .filter_dedicated_devices(&DedicatedDeviceFilterOptions::default())
            .expect("devices");
        assert_eq!(devices[0]["device_id"], 2);
    }

    #[test]
    fn list_dedicated_locations_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"short_name":"DAL","pub_description":"Dallas, TX","location_id":9}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let locations = client.list_dedicated_locations().expect("locations");
        assert_eq!(locations[0].short_name, "DAL");
        assert_eq!(locations[0].location_id, 9);
    }

    #[test]
    fn list_dedicated_device_os_profiles_sends_is_buyable_and_decodes_pascal_case_fields() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"OSID":5,"Name":"Ubuntu 24.04","DiskLayouts":[{"ID":1,"Name":"default"}]}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let profiles = client
            .list_dedicated_device_os_profiles(42, Some(true))
            .expect("profiles");
        assert_eq!(profiles[0].os_id, 5);
        assert_eq!(profiles[0].name, "Ubuntu 24.04");
        assert_eq!(profiles[0].disk_layouts[0].name, "default");
        assert!(transport.paths()[0].contains("is_buyable=1"));
        assert!(transport.paths()[0].contains("/dedicated/os/device/42"));
    }

    #[test]
    fn list_dedicated_plans_decodes_untyped_rows() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"plan":"metal-1"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let plans = client.list_dedicated_plans(3).expect("plans");
        assert_eq!(plans[0]["plan"], "metal-1");
    }

    #[test]
    fn deploy_dedicated_server_sends_a_json_body_with_the_disklayout_field_name() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"mbpkgid":1,"status":"queued","build":9}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = DedicatedServerBuildRequest {
            fqdn: "host.example.com".to_string(),
            profile: 12,
            disk_layout: Some(2),
            ..Default::default()
        };
        let build = client.deploy_dedicated_server(1, &request).expect("deploy");
        assert_eq!(build.build, 9);
        let requests = transport.requests.lock().unwrap();
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/json")
        );
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"disklayout\":2"));
        assert!(!body.contains("root_password"));
        assert!(requests[0]
            .url
            .path()
            .ends_with("/dedicated/server/build/1"));
    }

    #[test]
    fn buy_dedicated_server_sends_an_empty_form_encoded_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"mbpkgid":1,"status":"queued","build":9}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.buy_dedicated_server(77).expect("buy");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].body.as_deref(), Some(&[][..]));
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/x-www-form-urlencoded")
        );
        assert!(requests[0].url.path().ends_with("/dedicated/server/buy/77"));
    }

    #[test]
    fn update_dedicated_server_ipv4_reverse_puts_a_json_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = DedicatedIpv4ReverseRequest {
            mbpkgid: 1,
            id: 2,
            reverse: "host.example.com".to_string(),
        };
        client
            .update_dedicated_server_ipv4_reverse(&request)
            .expect("update reverse");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Put);
        assert!(requests[0]
            .url
            .path()
            .ends_with("/dedicated/server/ipv4_reverse"));
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"mbpkgid":1,"id":2,"reverse":"host.example.com"}"#);
    }

    #[test]
    fn soft_reset_dedicated_server_uses_the_soft_reset_path_quirk() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.soft_reset_dedicated_server(5).expect("soft reset");
        let requests = transport.requests.lock().unwrap();
        assert!(requests[0]
            .url
            .path()
            .ends_with("/dedicated/server/soft-reset/5"));
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/json")
        );
        assert_eq!(requests[0].body.as_deref(), Some(&[][..]));
    }

    #[test]
    fn delete_dedicated_server_sends_the_force_flag_when_given() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = DedicatedServerActionRequest {
            force: Some(true),
            password: None,
        };
        client
            .delete_dedicated_server(5, Some(&request))
            .expect("delete");
        let requests = transport.requests.lock().unwrap();
        assert!(requests[0]
            .url
            .path()
            .ends_with("/dedicated/server/5/delete"));
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"force":true}"#);
    }

    #[test]
    fn reboot_dedicated_server_sends_an_empty_json_body_when_no_request_is_given() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.reboot_dedicated_server(5, None).expect("reboot");
        let requests = transport.requests.lock().unwrap();
        assert!(requests[0]
            .url
            .path()
            .ends_with("/dedicated/server/5/reboot"));
        assert_eq!(requests[0].body.as_deref(), Some(&[][..]));
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/json")
        );
    }

    #[test]
    fn get_dedicated_server_power_status_decodes_untyped_fields() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"power_status":"running"}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let status = client
            .get_dedicated_server_power_status(5, None)
            .expect("status");
        assert_eq!(status["power_status"], "running");
    }

    #[test]
    fn list_dedicated_servers_coerces_ob_id_and_keeps_the_raw_building_value() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[
                {"id":1,"mbpkgid":100,"ob_id":42,"building":null},
                {"id":2,"mbpkgid":101,"ob_id":"43","building":{"status":"running"}}
            ]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let servers = client.list_dedicated_servers().expect("servers");
        assert_eq!(servers[0].ob_id, "42");
        assert!(servers[0].building.is_null());
        assert_eq!(servers[1].ob_id, "43");
        assert_eq!(servers[1].building["status"], "running");
    }

    #[test]
    fn get_my_images_and_get_image_decode_the_os_enabled_alias() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"os":"Ubuntu 24.04","type":"linux","os_enabled":1}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let images = client.get_my_images().expect("images");
        assert_eq!(images[0].name, "Ubuntu 24.04");
        assert_eq!(images[0].image_type, "linux");
        assert_eq!(images[0].enabled, Some(1));
    }

    #[test]
    fn create_image_form_encodes_only_the_fields_that_are_set() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"queue_id":7}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = CreateImageRequest {
            mbpkgid: 9,
            image_name: "snapshot".to_string(),
            image_description: String::new(),
            keep_ssh_userdirs: true,
        };
        let response = client.create_image(&request).expect("create image");
        assert_eq!(response.queue_id, 7);
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("mbpkgid=9"));
        assert!(body.contains("image_name=snapshot"));
        assert!(!body.contains("image_description"));
        assert!(body.contains("keep_ssh_userdirs=1"));
        assert!(requests[0].url.path().ends_with("/cloud/images/create"));
    }

    #[test]
    fn edit_image_patches_a_json_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .edit_image(3, "renamed", "new description")
            .expect("edit");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Patch);
        assert!(requests[0].url.path().ends_with("/cloud/images/3/edit"));
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"os":"renamed","description":"new description"}"#);
    }

    #[test]
    fn delete_image_decodes_the_queued_job_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"queue_id":11}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let response = client.delete_image(3).expect("delete");
        assert_eq!(response.queue_id, 11);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Delete);
    }

    #[test]
    fn get_image_queue_status_decodes_the_status() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"status":"Running","percent":40}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let status = client.get_image_queue_status(11).expect("status");
        assert_eq!(status.status, "Running");
        assert_eq!(status.percent, 40);
    }

    #[test]
    fn wait_for_image_queue_returns_immediately_when_already_complete() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"status":"Complete","percent":100}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let status = client.wait_for_image_queue(11).expect("complete");
        assert_eq!(status.status, "Complete");
    }

    #[test]
    fn wait_for_image_queue_fails_fast_on_a_failed_job() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"status":"Failed","response":"disk full"}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let err = client.wait_for_image_queue(11).expect_err("failed job");
        assert!(matches!(err, Error::Api { .. }));
        assert!(err.to_string().contains("disk full"));
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

    #[test]
    fn delete_vpc_standby_gateway_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_vpc_standby_gateway(1)
            .expect("idempotent delete");
    }

    #[test]
    fn add_vpc_standby_gateway_posts_to_the_gateway_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client.add_vpc_standby_gateway(7).expect("add gateway");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Post);
        assert!(requests[0].url.path().ends_with("/vpcs/7/gateway/standby"));
    }

    #[test]
    fn get_vpc_ip_reservations_decodes_the_response() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"gateways":[1,2],"interfaces":[],"vms":{"count":3}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let reservations = client.get_vpc_ip_reservations(9).expect("reservations");
        assert_eq!(reservations.gateways, serde_json::json!([1, 2]));
        assert_eq!(reservations.vms, serde_json::json!({"count": 3}));
    }

    #[test]
    fn vpc_ssh_settings_accepts_an_object_port() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"enabled":true,"port":{"port":2222},"bastion":{"ipv4":"192.0.2.10","ipv6":"2001:db8::10"}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let settings = client.get_vpc_ssh_settings(55).expect("settings");
        assert_eq!(settings.port, Some(2222));
        assert!(settings.enabled);
        assert_eq!(settings.bastion.unwrap().ipv4, "192.0.2.10");
    }

    #[test]
    fn vpc_ssh_settings_accepts_a_key_map() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"enabled":true,"port":2222,"keys":{"123":{"id":123,"sshKeyId":123,"name":"ops","dates":{"created":"2026-01-01","enabled":"2026-01-01"}}},"bastion":{"ipv4":"192.0.2.10","ipv6":"2001:db8::10"}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let settings = client.get_vpc_ssh_settings(55).expect("settings");
        assert_eq!(settings.keys.len(), 1);
        assert_eq!(settings.keys["123"].ssh_key_id, 123);
    }

    #[test]
    fn vpc_ssh_settings_accepts_keys_wrapped_in_a_list_envelope() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"enabled":false,"port":null,"keys":{"data":[{"id":9,"sshKeyId":9,"name":"laptop"}]}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let settings = client.get_vpc_ssh_settings(55).expect("settings");
        assert!(settings.port.is_none());
        assert_eq!(settings.keys["9"].name, "laptop");
    }

    #[test]
    fn list_vpc_backend_templates_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"backendTemplateId":1,"name":"web","backendHosts":[{"backendHostId":11,"address":"10.0.0.1"}]}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let templates = client
            .list_vpc_backend_templates(3)
            .expect("backend templates");
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].backend_hosts[0].address, "10.0.0.1");
    }

    #[test]
    fn list_vpc_backends_decodes_the_backend_hosts_wrapper_not_a_bare_array() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"backendHosts":[{"backendHostId":1,"address":"10.0.0.1"},{"backendHostId":2,"address":"10.0.0.2"}]}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let backends = client.list_vpc_backends(3, 11).expect("backends");
        assert_eq!(backends.len(), 2);
        assert_eq!(backends[1].backend_host_id, 2);
    }

    #[test]
    fn replace_vpc_backends_decodes_the_backend_hosts_wrapper_not_a_bare_array() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"backendHosts":[{"backendHostId":5,"address":"10.0.0.5"}]}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = ReplaceVpcBackendsRequest {
            backend_hosts: vec![],
        };
        let backends = client
            .replace_vpc_backends(3, 11, &request)
            .expect("replace backends");
        assert_eq!(backends.len(), 1);
        assert_eq!(backends[0].backend_host_id, 5);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn get_vpc_backend_finds_the_matching_backend_by_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"backendHosts":[{"backendHostId":1,"name":"a"},{"backendHostId":2,"name":"b"}]}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let backend = client.get_vpc_backend(3, 11, 2).expect("backend");
        assert_eq!(backend.name, "b");
    }

    #[test]
    fn get_vpc_backend_returns_not_found_when_no_backend_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"backendHosts":[{"backendHostId":1,"name":"a"}]}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client.get_vpc_backend(3, 11, 99).expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn delete_vpc_backend_template_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_vpc_backend_template(3, 11)
            .expect("idempotent delete");
    }

    #[test]
    fn delete_vpc_backend_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_vpc_backend(3, 11, 1)
            .expect("idempotent delete");
    }

    #[test]
    fn create_vpc_backend_template_sends_the_backend_hosts_field() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"backendTemplateId":1,"name":"web"}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateVpcBackendTemplateRequest {
            name: "web".to_string(),
            description: String::new(),
            backend_hosts: vec![VpcBackend {
                address: "10.0.0.1".to_string(),
                ..Default::default()
            }],
        };
        let template = client
            .create_vpc_backend_template(3, &request)
            .expect("create template");
        assert_eq!(template.backend_template_id, 1);
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"backendHosts\""));
        assert!(body.contains("10.0.0.1"));
    }

    #[test]
    fn update_vpc_ssh_settings_sends_the_port_field_even_when_none() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"enabled":true,"port":2222}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let settings = client
            .update_vpc_ssh_settings(55, &UpdateVpcSshSettingsRequest { port: None })
            .expect("update settings");
        assert_eq!(settings.port, Some(2222));
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Patch);
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"port":null}"#);
    }

    #[test]
    fn get_vpc_ssh_key_matches_by_the_underlying_account_key_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"id":0,"sshKeyId":9,"name":"laptop"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let key = client.get_vpc_ssh_key(55, 9).expect("ssh key");
        assert_eq!(key.name, "laptop");
    }

    #[test]
    fn get_vpc_ssh_key_returns_not_found_when_no_key_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"id":1,"sshKeyId":1,"name":"laptop"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client.get_vpc_ssh_key(55, 99).expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn enable_vpc_ssh_key_patches_the_enabled_flag() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client.enable_vpc_ssh_key(55, 9, true).expect("enable");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Patch);
        assert!(requests[0].url.path().ends_with("/vpcs/55/ssh/keys/9"));
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"enabled":true}"#);
    }

    #[test]
    fn delete_vpc_ssh_key_disables_it() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client.delete_vpc_ssh_key(55, 9).expect("delete");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"enabled":false}"#);
    }

    #[test]
    fn create_vpc_floating_ip_decodes_the_new_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"floatingIpId":42}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateVpcFloatingIpRequest {
            ip_version: 4,
            ptr: String::new(),
        };
        let id = client
            .create_vpc_floating_ip(7, &request)
            .expect("create floating ip");
        assert_eq!(id, 42);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Post);
    }

    #[test]
    fn get_vpc_floating_ip_finds_the_matching_floating_ip_by_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"floatingIpId":1,"address":"203.0.113.1"},{"floatingIpId":2,"address":"203.0.113.2"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let floating_ip = client.get_vpc_floating_ip(7, 2).expect("floating ip");
        assert_eq!(floating_ip.address, "203.0.113.2");
    }

    #[test]
    fn get_vpc_floating_ip_returns_not_found_when_no_floating_ip_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"floatingIpId":1,"address":"203.0.113.1"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client.get_vpc_floating_ip(7, 99).expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn update_vpc_floating_ip_sends_the_ptr_field() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateVpcFloatingIpRequest {
            ptr: "host.example.com".to_string(),
        };
        client
            .update_vpc_floating_ip(7, 2, &request)
            .expect("update floating ip");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Patch);
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"ptr":"host.example.com"}"#);
    }

    #[test]
    fn delete_vpc_floating_ip_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_vpc_floating_ip(7, 2)
            .expect("idempotent delete");
    }

    #[test]
    fn create_vpc_firewall_rule_decodes_the_new_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"firewallRuleId":11}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateVpcFirewallRuleRequest {
            ip_version: 4,
            direction: "inbound".to_string(),
            ..Default::default()
        };
        let id = client
            .create_vpc_firewall_rule(7, &request)
            .expect("create firewall rule");
        assert_eq!(id, 11);
        assert!(transport.requests.lock().unwrap()[0]
            .url
            .path()
            .ends_with("/vpcs/7/gateway/rules/firewall"));
    }

    #[test]
    fn list_vpc_firewall_rules_uses_the_ip_version_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"firewallRuleId":1,"direction":"inbound"}]}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let rules = client.list_vpc_firewall_rules(7, 6).expect("rules");
        assert_eq!(rules[0].firewall_rule_id, 1);
        assert!(transport.paths()[0].contains("/gateway/rules/firewall/ipv6"));
    }

    #[test]
    fn get_vpc_firewall_rule_returns_not_found_when_no_rule_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"firewallRuleId":1}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client
            .get_vpc_firewall_rule(7, 99, 4)
            .expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn update_vpc_firewall_rule_decodes_the_updated_rule() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"firewallRuleId":11,"direction":"outbound"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let request = UpdateVpcFirewallRuleRequest {
            direction: "outbound".to_string(),
            ..Default::default()
        };
        let rule = client
            .update_vpc_firewall_rule(7, 11, &request)
            .expect("update firewall rule");
        assert_eq!(rule.direction, "outbound");
    }

    #[test]
    fn delete_vpc_firewall_rule_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_vpc_firewall_rule(7, 11)
            .expect("idempotent delete");
    }

    #[test]
    fn apply_vpc_firewall_changes_posts_to_the_apply_changes_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client.apply_vpc_firewall_changes(7).expect("apply changes");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Post);
        assert!(requests[0]
            .url
            .path()
            .ends_with("/gateway/rules/firewall/apply-changes"));
    }

    #[test]
    fn create_vpc_snat_rule_decodes_the_nested_match_and_translation() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"snatRuleId":5,"ipVersion":4,"match":{"internalCidr":"10.0.0.0/24"},"translation":{"address":{"start":"203.0.113.1","end":"203.0.113.1"}}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let request = CreateVpcSnatRuleRequest {
            ip_version: 4,
            match_criteria: Some(VpcSnatMatch {
                internal_cidr: "10.0.0.0/24".to_string(),
            }),
            ..Default::default()
        };
        let rule = client
            .create_vpc_snat_rule(7, &request)
            .expect("create snat rule");
        assert_eq!(rule.snat_rule_id, 5);
        assert_eq!(
            rule.match_criteria.expect("match").internal_cidr,
            "10.0.0.0/24"
        );
    }

    #[test]
    fn get_vpc_snat_rule_returns_not_found_when_no_rule_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"snatRuleId":1}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client.get_vpc_snat_rule(7, 99, 4).expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn delete_vpc_snat_rule_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_vpc_snat_rule(7, 5)
            .expect("idempotent delete");
    }

    #[test]
    fn apply_vpc_snat_changes_posts_to_the_apply_changes_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client.apply_vpc_snat_changes(7).expect("apply changes");
        assert!(transport.requests.lock().unwrap()[0]
            .url
            .path()
            .ends_with("/gateway/rules/snat/apply-changes"));
    }

    #[test]
    fn create_vpc_dnat_rule_always_sends_the_translation_field() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"dnatRuleId":9,"ipVersion":4}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateVpcDnatRuleRequest {
            ip_version: 4,
            ..Default::default()
        };
        let rule = client
            .create_vpc_dnat_rule(7, &request)
            .expect("create dnat rule");
        assert_eq!(rule.dnat_rule_id, 9);
        let body =
            String::from_utf8(transport.requests.lock().unwrap()[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"translation\":null"));
    }

    #[test]
    fn get_vpc_dnat_rule_returns_not_found_when_no_rule_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"dnatRuleId":1}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client.get_vpc_dnat_rule(7, 99, 4).expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn delete_vpc_dnat_rule_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_vpc_dnat_rule(7, 9)
            .expect("idempotent delete");
    }

    #[test]
    fn storage_object_store_decoder_accepts_flat_list_and_nested_single_get_shapes() {
        let flat: StorageObjectStore = serde_json::from_str(
            r#"{"objectStoreId":20,"label":"flat","ready":true,"assignedOn":"now","location":{"id":1},"capacity":{"totalGB":10},"hardwareClass":{"id":1}}"#,
        )
        .unwrap();
        let nested: StorageObjectStore = serde_json::from_str(
            r#"{"credentials":{"endpoints":["https://s3.example"],"accessKey":"a","secretKey":"s","userKey":"u"},"metadata":{"objectStoreId":21,"label":"nested","ready":true,"assignedOn":"now","location":{"id":1},"capacity":{"totalGB":20},"hardwareClass":{"id":1}}}"#,
        )
        .unwrap();
        assert_eq!(flat.metadata.object_store_id, 20);
        assert_eq!(nested.credentials.access_key, "a");
        assert_eq!(nested.metadata.object_store_id, 21);
    }

    #[test]
    fn storage_block_namespace_decoder_accepts_flat_list_and_nested_single_get_shapes() {
        let flat: StorageBlockNamespace = serde_json::from_str(
            r#"{"blockNamespaceId":30,"label":"flat","ready":true,"assignedOn":"now","location":{"id":1},"capacity":{"totalGB":10},"hardwareClass":{"id":1}}"#,
        )
        .unwrap();
        let nested: StorageBlockNamespace = serde_json::from_str(
            r#"{"credentials":{"endpoints":["https://block.example"],"userKey":"u","secretKey":"s","pool":"p","namespace":"n","clusterId":"c"},"metadata":{"blockNamespaceId":31,"label":"nested","ready":true,"assignedOn":"now","location":{"id":1},"capacity":{"totalGB":20},"hardwareClass":{"id":1}}}"#,
        )
        .unwrap();
        assert_eq!(flat.metadata.block_namespace_id, 30);
        assert_eq!(nested.credentials.pool, "p");
        assert_eq!(nested.metadata.block_namespace_id, 31);
    }

    #[test]
    fn storage_block_volume_decoder_accepts_flat_list_and_nested_single_get_shapes() {
        let flat: StorageBlockVolume = serde_json::from_str(
            r#"{"blockVolumeId":40,"label":"flat","ready":true,"assignedOn":"now","location":{"id":1},"capacity":{"totalGB":10},"hardwareClass":{"id":1}}"#,
        )
        .unwrap();
        let nested: StorageBlockVolume = serde_json::from_str(
            r#"{"credentials":{"endpoints":["https://block.example"],"userKey":"u","secretKey":"s","pool":"p","namespace":"n","clusterId":"c","imageName":"img"},"metadata":{"blockVolumeId":41,"label":"nested","ready":true,"assignedOn":"now","location":{"id":1},"capacity":{"totalGB":20},"hardwareClass":{"id":1}}}"#,
        )
        .unwrap();
        assert_eq!(flat.metadata.block_volume_id, 40);
        assert_eq!(nested.credentials.image_name, "img");
        assert_eq!(nested.metadata.block_volume_id, 41);
    }

    #[test]
    fn get_storage_block_volume_recovers_the_id_from_the_platform_defect() {
        // The single-volume GET can answer with object-store shaped metadata that carries no
        // blockVolumeId at all, even though the id resolved correctly. The client must fill
        // in the id it was asked for rather than surface a zero id.
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"objectStoreId":5,"label":"vol","ready":true,"private":false,"assignedOn":"now","location":{"id":1},"capacity":{"totalGB":10},"hardwareClass":{"id":1},"versioning":false}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let volume = client.get_storage_block_volume(42).expect("volume");
        assert_eq!(volume.metadata.block_volume_id, 42);
        assert!(volume.metadata.ready);
    }

    #[test]
    fn list_storage_types_decodes_a_plain_array() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"type":"bucket","name":"Bucket"},{"type":"block","name":"Block"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let types = client.list_storage_types().expect("storage types");
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].type_code, "bucket");
    }

    #[test]
    fn list_storage_types_decodes_a_map_keyed_by_type_code() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"block":{"name":"Block"},"bucket":{"name":"Bucket"}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let types = client.list_storage_types().expect("storage types");
        let codes: Vec<&str> = types.iter().map(|t| t.type_code.as_str()).collect();
        assert_eq!(codes, vec!["block", "bucket"]);
    }

    #[test]
    fn create_storage_object_store_decodes_the_new_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"objectStoreId":9}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let request = CreateStorageObjectStoreRequest {
            location_id: 1,
            label: "store".to_string(),
            ..Default::default()
        };
        let id = client
            .create_storage_object_store(&request)
            .expect("create object store");
        assert_eq!(id, 9);
    }

    #[test]
    fn create_storage_block_namespace_decodes_the_new_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"blockNamespaceId":12}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let request = CreateStorageBlockNamespaceRequest {
            location_id: 1,
            label: "namespace".to_string(),
            ..Default::default()
        };
        let id = client
            .create_storage_block_namespace(&request)
            .expect("create block namespace");
        assert_eq!(id, 12);
    }

    #[test]
    fn create_storage_block_volume_decodes_the_new_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"blockVolumeId":13}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let request = CreateStorageBlockVolumeRequest {
            location_id: 1,
            label: "volume".to_string(),
            ..Default::default()
        };
        let id = client
            .create_storage_block_volume(&request)
            .expect("create block volume");
        assert_eq!(id, 13);
    }

    #[test]
    fn delete_storage_object_store_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_storage_object_store(9)
            .expect("idempotent delete");
    }

    #[test]
    fn delete_storage_block_namespace_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_storage_block_namespace(12)
            .expect("idempotent delete");
    }

    #[test]
    fn delete_storage_block_volume_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_storage_block_volume(13)
            .expect("idempotent delete");
    }

    #[test]
    fn list_storage_locations_decodes_the_location_and_hardware_class() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"location":{"id":1,"name":"DAL"},"hardware":{"id":2,"name":"nvme"}}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let locations = client.list_storage_locations().expect("locations");
        assert_eq!(locations[0].location.name, "DAL");
        assert_eq!(locations[0].hardware.name, "nvme");
    }

    #[test]
    fn wait_for_ready_returns_immediately_when_the_first_check_is_ready() {
        use std::time::Duration;
        let mut calls = 0;
        crate::v3::wait_for_ready(Duration::from_millis(1), Duration::from_millis(1), || {
            calls += 1;
            Ok(true)
        })
        .expect("ready");
        assert_eq!(calls, 1);
    }

    #[test]
    fn wait_for_ready_polls_until_the_condition_is_met() {
        use std::time::Duration;
        let mut calls = 0;
        crate::v3::wait_for_ready(Duration::from_millis(1), Duration::from_millis(200), || {
            calls += 1;
            Ok(calls >= 3)
        })
        .expect("ready");
        assert_eq!(calls, 3);
    }

    #[test]
    fn wait_for_ready_times_out_when_never_ready() {
        use std::time::Duration;
        let err =
            crate::v3::wait_for_ready(Duration::from_millis(1), Duration::from_millis(5), || {
                Ok(false)
            })
            .expect_err("timeout");
        assert!(err.is_timeout());
    }

    #[test]
    fn wait_for_ready_propagates_a_check_error() {
        use std::time::Duration;
        let err =
            crate::v3::wait_for_ready(Duration::from_millis(1), Duration::from_millis(5), || {
                Err(Error::Decode("boom".to_string()))
            })
            .expect_err("propagated error");
        assert!(matches!(err, Error::Decode(message) if message == "boom"));
    }

    #[test]
    fn list_nke_addon_catalog_decodes_entries() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"addonId":1,"addonType":"netactuate-dns","isDefault":true,"requiresVpc":true}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let catalog = client.list_nke_addon_catalog().expect("catalog");
        assert_eq!(catalog[0].addon_type, "netactuate-dns");
        assert!(catalog[0].is_default);
        assert!(catalog[0].requires_vpc);
    }

    #[test]
    fn list_nke_cluster_addons_decodes_nested_health_and_config() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"id":1,"addonType":"storage","state":"Installed","health":{"summary":"ok"},"config":{"integrations":[{"storageIntegrationId":9,"isDefaultClass":true}]}}]}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let addons = client.list_nke_cluster_addons(7).expect("addons");
        assert_eq!(addons[0].addon_type, "storage");
        assert_eq!(addons[0].health.summary, "ok");
        assert!(addons[0].config.integrations[0].is_default_class);
        assert!(transport.requests.lock().unwrap()[0]
            .url
            .path()
            .ends_with("/nke/clusters/7/addons"));
    }

    #[test]
    fn get_nke_cluster_addon_reads_the_addon_type_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"id":1,"addonType":"netactuate-dns","state":"Installed"}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let addon = client
            .get_nke_cluster_addon(7, "netactuate-dns")
            .expect("addon");
        assert_eq!(addon.state, "Installed");
        assert!(transport.requests.lock().unwrap()[0]
            .url
            .path()
            .ends_with("/nke/clusters/7/addons/netactuate-dns"));
    }

    #[test]
    fn create_nke_cluster_addon_sends_the_addon_type_and_config() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"id":1,"addonType":"netactuate-dns"}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateNkeAddonRequest {
            addon_type: "netactuate-dns".to_string(),
            config: Some(
                serde_json::to_value(NkeDnsAddonWriteConfig {
                    zone: "example.com".to_string(),
                    mode: String::new(),
                })
                .unwrap(),
            ),
            ..Default::default()
        };
        let addon = client
            .create_nke_cluster_addon(7, &request)
            .expect("create addon");
        assert_eq!(addon.addon_type, "netactuate-dns");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Post);
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"addonType\":\"netactuate-dns\""));
        assert!(body.contains("\"zone\":\"example.com\""));
        assert!(!body.contains("\"version\""));
    }

    #[test]
    fn update_nke_cluster_addon_patches_the_addon_type_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"id":1,"addonType":"storage","version":"2.0"}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateNkeAddonRequest {
            version: "2.0".to_string(),
            ..Default::default()
        };
        let addon = client
            .update_nke_cluster_addon(7, "storage", &request)
            .expect("update addon");
        assert_eq!(addon.version, "2.0");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Patch);
        assert!(requests[0]
            .url
            .path()
            .ends_with("/nke/clusters/7/addons/storage"));
    }

    #[test]
    fn delete_nke_cluster_addon_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_nke_cluster_addon(7, "storage")
            .expect("idempotent delete");
    }

    #[test]
    fn list_nke_cluster_dns_zones_decodes_the_zone_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"dnsZoneId":1,"clusterId":7,"zone":"example.com","state":"Active"}]}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let zones = client.list_nke_cluster_dns_zones(7).expect("dns zones");
        assert_eq!(zones[0].zone, "example.com");
        assert_eq!(zones[0].state, "Active");
        assert!(transport.requests.lock().unwrap()[0]
            .url
            .path()
            .ends_with("/nke/clusters/7/dns-zones"));
    }

    #[test]
    fn wait_for_nke_cluster_healthy_returns_immediately_when_already_healthy() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"clusterId":7,"status":{"cluster":"Healthy"}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .wait_for_nke_cluster_healthy(7)
            .expect("already healthy");
    }

    #[test]
    fn wait_for_nke_cluster_healthy_fails_fast_on_a_failed_cluster() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"clusterId":7,"status":{"cluster":"Failed"}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client
            .wait_for_nke_cluster_healthy(7)
            .expect_err("failed cluster");
        assert!(matches!(err, Error::Api { .. }));
        assert!(err.to_string().contains("entered failed state"));
    }

    #[test]
    fn wait_for_nke_worker_nodes_skips_the_request_when_minimum_is_not_positive() {
        let transport = RecordedTransport::new(vec![]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .wait_for_nke_worker_nodes(7, 0)
            .expect("no minimum to wait for");
        assert!(transport.requests.lock().unwrap().is_empty());
    }

    #[test]
    fn wait_for_nke_worker_nodes_returns_immediately_once_the_minimum_is_met() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"workerNodeId":1},{"workerNodeId":2}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .wait_for_nke_worker_nodes(7, 2)
            .expect("minimum already met");
    }

    #[test]
    fn delete_oidc_client_key_treats_a_revoked_key_as_idempotent_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            400,
            r#"{"code":400,"message":"The key is revoked","data":null}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_oidc_client_key(5, 9)
            .expect("idempotent delete");
    }

    #[test]
    fn delete_oidc_client_key_treats_not_found_as_idempotent_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_oidc_client_key(5, 9)
            .expect("idempotent delete");
    }

    #[test]
    fn delete_oidc_client_key_propagates_an_unrelated_400() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            400,
            r#"{"code":400,"message":"malformed request","data":null}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client.delete_oidc_client_key(5, 9).expect_err("propagated");
        assert!(!err.is_not_found());
    }

    #[test]
    fn get_oidc_client_returns_not_found_when_missing_from_the_client_list() {
        let transport = RecordedTransport::new(vec![
            RecordedTransport::json(
                200,
                r#"{"code":200,"data":{"metadata":{"createdOn":"t","label":"l","description":"d"},"keys":{"data":[],"meta":{}},"logs":{"changes":{"data":[],"meta":{}},"auth":{"data":[],"meta":{}}}}}"#,
            ),
            RecordedTransport::json(
                200,
                r#"{"code":200,"data":{"tenant":1,"clients":{"data":[],"meta":{"limit":1000,"offset":0,"total":0}}}}"#,
            ),
        ]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client.get_oidc_client(42).expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn list_oidc_clients_decodes_the_tenant_and_the_jwks_https_url_fallback() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"tenant":288,"clients":{"data":[{"clientId":7,"createdOn":"2024-01-01","label":"prod","description":"","jwksHttpsUrl":"https://example.test/jwks","accountDefault":true,"defaultAudience":"aud","ttl":3600,"enforceAllowList":1}],"meta":{"limit":1000,"offset":0,"total":1}}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let clients = client.list_oidc_clients().expect("clients");
        assert_eq!(clients.len(), 1);
        let oidc_client = &clients[0];
        assert_eq!(oidc_client.client_id, 7);
        assert_eq!(oidc_client.tenant, "288");
        assert_eq!(
            oidc_client.jwks_uri.as_deref(),
            Some("https://example.test/jwks")
        );
        assert!(oidc_client.enforce_allow_list);
    }

    #[test]
    fn create_oidc_client_decodes_the_new_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"clientId":42}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateOidcClientRequest {
            label: "svc".to_string(),
            enforce_allow_list: true,
            ..Default::default()
        };
        let id = client.create_oidc_client(&request).expect("create");
        assert_eq!(id, 42);
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains(r#""enforceAllowList":true"#));
    }

    #[test]
    fn add_oidc_client_vms_sends_the_expected_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client.add_oidc_client_vms(5, &[10, 20]).expect("add vms");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"vms":[{"mbpkgid":10},{"mbpkgid":20}]}"#);
    }

    #[test]
    fn list_oidc_client_keys_decodes_rows() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"keys":{"data":[{"keyId":1,"label":"k1","providedOn":"t","revokedOn":"","type":"rsa","publicKey":"pk"}],"meta":{"limit":1000,"offset":0,"total":1}}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let keys = client.list_oidc_client_keys(5).expect("keys");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].key_id, 1);
        assert_eq!(keys[0].public_key, "pk");
    }

    #[test]
    fn get_vlans_decodes_provisioned_locations() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"mbid":2,"private":1,"allow_sriov":0,"display_name":"vlan-1","description":"","last_updated":"","created":"","provisioned_locations":[{"provisioned":true,"name":"DAL","location_id":9,"flag":"us","iata_code":"DAL"}]}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let vlans = client.get_vlans().expect("vlans");
        assert_eq!(vlans.len(), 1);
        assert_eq!(vlans[0].provisioned_locations[0].iata_code, "DAL");
    }

    #[test]
    fn attach_server_nic_decodes_a_response_wrapped_in_a_one_element_array() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"nic_id":11,"mbpkgid":99,"customer_vlan_id":4,"attach_order":2}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let request = ServerNicAttachRequest {
            customer_vlan_id: 4,
        };
        let nic = client.attach_server_nic(99, &request).expect("attach nic");
        assert_eq!(nic.nic_id, 11);
        assert_eq!(nic.customer_vlan_id, 4);
    }

    #[test]
    fn detach_server_nic_treats_not_found_as_idempotent_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        client.detach_server_nic(99, 11).expect("idempotent detach");
    }

    #[test]
    fn list_cloud_floating_ipv4_decodes_rows() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"data":[{"floatingIpv4Id":3,"address":"203.0.113.9","vlanId":4,"ptrDomain":null,"location":{"id":1,"name":"DAL","flag":"us","latitude":"0","longitude":"0"}}],"meta":{"limit":1000,"offset":0,"total":1}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let ips = client.list_cloud_floating_ipv4().expect("floating ips");
        assert_eq!(ips.len(), 1);
        assert_eq!(ips[0].address, "203.0.113.9");
        assert_eq!(ips[0].location.as_ref().unwrap().name, "DAL");
    }

    #[test]
    fn grant_cloud_floating_ipv4_vms_sends_the_expected_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = GrantCloudFloatingIpv4VmsRequest {
            revoke_existing: Some(true),
            vms: vec![CloudFloatingIpv4VmRef { mbpkgid: 55 }],
        };
        client
            .grant_cloud_floating_ipv4_vms(3, &request)
            .expect("grant");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"revokeExisting":true,"vms":[{"mbpkgid":55}]}"#);
    }

    #[test]
    fn delete_cloud_floating_ipv4_treats_not_found_as_idempotent_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_cloud_floating_ipv4(3)
            .expect("idempotent delete");
    }

    #[test]
    fn list_cloud_networking_locations_decodes_rows() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"locationId":1,"datacenterId":9}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let locations = client.list_cloud_networking_locations().expect("locations");
        assert_eq!(
            locations,
            vec![CloudNetworkingLocation {
                location_id: 1,
                datacenter_id: 9,
            }]
        );
    }

    #[test]
    fn unlink_server_posts_an_empty_form_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.unlink_server(7).expect("unlink");
        let requests = transport.requests.lock().unwrap();
        assert!(requests[0].url.path().ends_with("/cloud/server/7/unlink"));
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/x-www-form-urlencoded")
        );
    }

    #[test]
    fn start_and_stop_server_post_a_json_null_body() {
        let transport = RecordedTransport::new(vec![
            RecordedTransport::json(200, r#"{"result":"success","code":200,"data":null}"#),
            RecordedTransport::json(200, r#"{"result":"success","code":200,"data":null}"#),
        ]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.start_server(7).expect("start");
        client.stop_server(7).expect("stop");
        let requests = transport.requests.lock().unwrap();
        assert!(requests[0].url.path().ends_with("/cloud/server/7/start"));
        assert!(requests[1].url.path().ends_with("/cloud/server/7/shutdown"));
        for request in requests.iter() {
            assert_eq!(request.content_type.as_deref(), Some("application/json"));
            assert_eq!(request.body.as_deref(), Some(b"null".as_slice()));
        }
    }

    #[test]
    fn scale_server_sends_the_scale_request_and_decodes_the_job_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":42}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = ScaleServerRequest {
            pkg_name: "bigger".to_string(),
            pkg_id: 0,
            allow_reboot: true,
        };
        let job_id = client.scale_server(7, &request).expect("scale");
        assert_eq!(job_id, 42);
        let requests = transport.requests.lock().unwrap();
        assert!(requests[0].url.path().ends_with("/cloud/scale/7"));
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"pkg_name\":\"bigger\""));
        assert!(!body.contains("pkg_id"));
        assert!(body.contains("\"allow_reboot\":true"));
    }

    #[test]
    fn get_job_status_decodes_the_job() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":42,"ts_insert":"2026-01-01","command":"scale_vm","status":1}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let job = client.get_job_status("scale_vm", 42).expect("job status");
        assert_eq!(job.id, 42);
        assert_eq!(job.command, "scale_vm");
        assert_eq!(job.status, 1);
    }

    #[test]
    fn get_services_keeps_the_raw_response_alongside_known_fields() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"description":"colo","extra":"kept"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let services = client.get_services().expect("services");
        assert_eq!(services[0].id, 1);
        assert_eq!(services[0].raw["extra"], "kept");
    }

    #[test]
    fn get_colocation_services_omits_the_filter_when_none() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.get_colocation_services(None).expect("services");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/api/services/colocation"
        );
    }

    #[test]
    fn get_ip_transit_services_applies_the_service_id_filter() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"service_id":9,"datacenter_id":2,"bgp_group_id":3}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let services = client
            .get_ip_transit_services(Some(9))
            .expect("ip transit services");
        assert!(transport.paths()[0].contains("service_id=9"));
        assert_eq!(services.len(), 1);
    }

    #[test]
    fn get_transport_service_decodes_a_single_record() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":5,"service_id":9,"datacenter_id":2,"description":"backbone"}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let service = client.get_transport_service(5).expect("transport service");
        assert_eq!(service.description, "backbone");
    }

    #[test]
    fn get_platform_status_sorts_services_and_locations() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{
                "web":{"component_id":"c2","locations":{"nyc":{"status":"up","container_id":"n1","last_updated":"t1"}}},
                "api":{"component_id":"c1","locations":{"lax":{"status":"up","container_id":"l1","last_updated":"t2"},"ams":{"status":"down","container_id":"a1","last_updated":"t3"}}}
            }}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let status = client.get_platform_status().expect("platform status");
        assert_eq!(status.len(), 2);
        assert_eq!(status[0].service, "api");
        assert_eq!(status[0].component_id, "c1");
        assert_eq!(status[0].locations[0].location, "ams");
        assert_eq!(status[0].locations[1].location, "lax");
        assert_eq!(status[1].service, "web");
    }

    #[test]
    fn get_platform_change_log_entry_tolerates_a_numeric_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":42,"title":"Upgrade","short_description":"short","status":"published"}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let entry = client
            .get_platform_change_log_entry(42)
            .expect("change log entry");
        assert_eq!(entry.change_log_id, "42");
        assert_eq!(entry.title, "Upgrade");
        assert_eq!(entry.raw["status"], "published");
    }

    #[test]
    fn get_platform_datacenters_escapes_the_location_segment() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"name":"Building A","iata":"nyc"}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let datacenters = client
            .get_platform_datacenters("new york")
            .expect("datacenters");
        assert_eq!(datacenters[0].iata, "nyc");
        assert!(transport.paths()[0].contains("/platform/datacenters/new+york"));
    }

    #[test]
    fn execute_platform_looking_glass_builds_the_query_from_options() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"output":"ok"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let options = PlatformLookingGlassExecuteOptions {
            action: Some("ping".to_string()),
            target: Some("203.0.113.1".to_string()),
            location: None,
            full: Some(1),
        };
        let result = client
            .execute_platform_looking_glass(&options)
            .expect("looking glass");
        assert_eq!(result.raw["output"], "ok");
        let query = transport.paths()[0].clone();
        assert!(query.contains("action=ping"));
        assert!(query.contains("target=203.0.113.1"));
        assert!(query.contains("full=1"));
        assert!(!query.contains("location"));
    }

    #[test]
    fn get_platform_incidents_decodes_active_upcoming_and_historic_events() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{
                "active":[{"event_id":"1","type":"outage","name":"Link down","status":"active","start_time":"t1","end_time":"","components":["router"],"containers":["c1"]}],
                "upcoming":[],
                "historic":[]
            }}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let events = client
            .get_platform_incidents("nyc")
            .expect("platform incidents");
        assert_eq!(events.active.len(), 1);
        assert_eq!(events.active[0].event_type, "outage");
        assert_eq!(events.active[0].components, vec!["router".to_string()]);
    }

    #[test]
    fn get_tickets_builds_the_query_from_list_options_and_keeps_the_raw_payload() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":"42","subject":"help","status":"open"}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let options = TicketListOptions {
            open: Some("1".to_string()),
            include_stats: Some("1".to_string()),
        };
        let tickets = client.get_tickets(&options).expect("tickets");
        assert_eq!(tickets[0].id, "42");
        assert_eq!(tickets[0].raw["subject"], "help");
        let query = transport.paths()[0].clone();
        assert!(query.contains("open=1"));
        assert!(query.contains("include_stats=1"));
    }

    #[test]
    fn create_ticket_sends_a_json_body_and_omits_empty_optional_fields() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":"7","subject":"help"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = CreateTicketRequest {
            subject: "help".to_string(),
            message: "please help".to_string(),
            department: 3,
            ..Default::default()
        };
        let ticket = client.create_ticket(&request).expect("create ticket");
        assert_eq!(ticket.id, "7");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"department\":3"));
        assert!(!body.contains("urgency"));
        assert!(!body.contains("files"));
    }

    #[test]
    fn get_ticket_attachment_appends_without_data_to_the_query() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"name":"log.txt","content_type":"text/plain","size":10}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let attachment = client
            .get_ticket_attachment("42", "ticket", "42", 0, Some(1))
            .expect("attachment");
        assert_eq!(attachment.name, "log.txt");
        assert_eq!(attachment.size, 10);
        let query = transport.paths()[0].clone();
        assert!(query.contains("/support/tickets/42/attachment/ticket/42/0"));
        assert!(query.contains("without_data=1"));
    }

    #[test]
    fn download_ticket_attachment_returns_raw_bytes_without_envelope_decoding() {
        let transport =
            RecordedTransport::new(vec![RecordedTransport::json(200, "raw-file-bytes")]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let bytes = client
            .download_ticket_attachment("42", "ticket", "42", 0)
            .expect("download");
        assert_eq!(bytes, b"raw-file-bytes");
    }

    #[test]
    fn download_ticket_attachment_reports_a_failure_status_without_treating_it_as_not_found() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(404, "gone")]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let err = client
            .download_ticket_attachment("42", "ticket", "42", 0)
            .expect_err("download failure");
        assert!(!err.is_not_found());
        assert!(err.to_string().contains("404"));
    }

    #[test]
    fn close_ticket_posts_an_empty_json_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.close_ticket("42").expect("close");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Post);
        assert_eq!(requests[0].body.as_ref().map(Vec::len), Some(0));
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/json")
        );
        assert!(requests[0]
            .url
            .path()
            .ends_with("/support/tickets/42/close"));
    }

    #[test]
    fn reply_to_ticket_alias_posts_to_the_alternate_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":"9","message":"thanks"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = ReplyTicketRequest {
            message: "thanks".to_string(),
            files: Vec::new(),
        };
        let reply = client.reply_to_ticket_alias("42", &request).expect("reply");
        assert_eq!(reply.message, "thanks");
        assert!(transport.requests.lock().unwrap()[0]
            .url
            .path()
            .ends_with("/support/tickets/reply/42"));
    }

    #[test]
    fn get_secret_list_values_coerces_a_string_secret_list_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"secret_list_id":"9","secret_key":"k","secret_value":"v"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let values = client.get_secret_list_values(9).expect("values");
        assert_eq!(values[0].secret_list_id, 9);
        assert_eq!(values[0].secret_key, "k");
    }

    #[test]
    fn create_secret_list_sends_a_form_encoded_name() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"name":"prod"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let list = client.create_secret_list("prod").expect("create list");
        assert_eq!(list.name, "prod");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, "name=prod");
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/x-www-form-urlencoded")
        );
    }

    #[test]
    fn update_secret_list_posts_rather_than_puts() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"name":"renamed"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.update_secret_list(1, "renamed").expect("update");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Post);
        assert!(requests[0].url.path().ends_with("/secrets/lists/1"));
    }

    #[test]
    fn create_secret_list_value_sends_the_key_and_value_fields() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"secret_list_id":9,"secret_key":"k","secret_value":"v"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let value = client
            .create_secret_list_value(9, "k", "v")
            .expect("create value");
        assert_eq!(value.secret_key, "k");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("secret_key=k"));
        assert!(body.contains("secret_value=v"));
    }

    #[test]
    fn delete_secret_list_value_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        client
            .delete_secret_list_value(1, 2)
            .expect("idempotent delete");
    }

    #[test]
    fn create_bgp_group_sends_a_json_body_and_omits_an_empty_group_type() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"name":"g","description":"d","group_type":""}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = CreateBgpGroupRequest {
            name: "g".to_string(),
            description: "d".to_string(),
            group_type: String::new(),
        };
        let group = client.create_bgp_group(&request).expect("create group");
        assert_eq!(group.id, 1);
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Post);
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/json")
        );
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(!body.contains("group_type"));
    }

    #[test]
    fn get_bgp_group_decodes_the_record() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":7,"name":"g","description":"d","group_type":"bgp"}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let group = client.get_bgp_group(7).expect("get group");
        assert_eq!(group.group_type, "bgp");
    }

    #[test]
    fn list_bgp_groups_applies_the_group_type_filter() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"name":"g","description":"","group_type":"anycast"}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let groups = client.list_bgp_groups("anycast").expect("list groups");
        assert_eq!(groups.len(), 1);
        assert!(transport.paths()[0].contains("group_type=anycast"));
    }

    #[test]
    fn list_bgp_groups_omits_the_filter_when_empty() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.list_bgp_groups("").expect("list groups");
        assert!(!transport.paths()[0].contains("group_type"));
        assert!(transport.paths()[0].starts_with("/api/bgp/bgpgroups"));
    }

    #[test]
    fn list_bgp_groups_treats_a_filter_match_of_nothing_as_an_empty_list() {
        // A filtered list that matches nothing answers 404 rather than 200 with an empty
        // array. The fix under test is that this reads back as zero groups, not an error.
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let groups = client
            .list_bgp_groups("anycast")
            .expect("empty list, not an error");
        assert!(groups.is_empty());
    }

    #[test]
    fn list_bgp_prefixes_treats_a_filter_match_of_nothing_as_an_empty_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let prefixes = client
            .list_bgp_prefixes("bgp")
            .expect("empty list, not an error");
        assert!(prefixes.is_empty());
    }

    #[test]
    fn list_bgp_asns_treats_a_filter_match_of_nothing_as_an_empty_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let asns = client
            .list_bgp_asns("bgp")
            .expect("empty list, not an error");
        assert!(asns.is_empty());
    }

    #[test]
    fn buy_bgp_prefixes_sends_a_json_body_and_omits_zero_fields() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"name":"p","prefix":"192.0.2.0/24","group_id":0,"asn_id":0,"anycast_profile":0,"agreement_id":9}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = BuyBgpPrefixesRequest {
            name: "p".to_string(),
            group_id: 0,
            asn_id: 0,
            anycast_profile: 0,
            agreement_id: 9,
        };
        let prefix = client.buy_bgp_prefixes(&request).expect("buy prefixes");
        assert_eq!(prefix.agreement_id, 9);
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].url.path(), "/api/bgp/bgpbuyprefixes");
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(!body.contains("group_id"));
        assert!(body.contains("agreement_id"));
    }

    #[test]
    fn get_bgp_asn_uses_a_query_parameter_rather_than_a_path_segment() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"asn":65000,"name":"a","group_type":"bgp"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let asn = client.get_bgp_asn(1).expect("get asn");
        assert_eq!(asn.asn, 65000);
        assert!(transport.paths()[0].starts_with("/api/bgp/bgpasn?id=1"));
    }

    #[test]
    fn list_account_agreements_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"name":"tos","title":"Terms","description":"d","version":"1"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let agreements = client.list_account_agreements().expect("list agreements");
        assert_eq!(agreements[0].title, "Terms");
    }

    #[test]
    fn bind_bgp_group_firewall_set_posts_a_json_body_to_the_dashed_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"bgp2_group_id":5,"firewall_set_id":9,"interface_number":0,"set_priority":1}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = BindBgpGroupFirewallSetRequest {
            id: 0,
            firewall_set_id: 9,
            interface_number: 0,
            set_priority: 1,
        };
        let binding = client
            .bind_bgp_group_firewall_set(5, &request)
            .expect("bind firewall set");
        assert_eq!(binding.bgp_group_id, 5);
        let requests = transport.requests.lock().unwrap();
        assert_eq!(
            requests[0].url.path(),
            "/api/bgp/bgp-groups/5/firewall-sets"
        );
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/json")
        );
    }

    #[test]
    fn unbind_bgp_group_firewall_set_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        client
            .unbind_bgp_group_firewall_set(5, 9)
            .expect("idempotent unbind");
    }

    #[test]
    fn bgp_group_session_actions_send_an_empty_form_encoded_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.start_bgp_group_sessions(3).expect("start group");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].url.path(), "/api/bgp/bgpgroup/3/start");
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/x-www-form-urlencoded")
        );
        assert_eq!(requests[0].body.as_deref(), Some(&[][..]));
    }

    #[test]
    fn bgp_session_actions_use_the_bgpsession_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.stop_bgp_session(11).expect("stop session");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/api/bgp/bgpsession/11/stop"
        );
    }

    #[test]
    fn get_bgp_summary_decodes_the_raw_map() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"sessions_up":3,"sessions_down":1}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let summary = client.get_bgp_summary().expect("summary");
        assert_eq!(summary["sessions_up"], 3);
    }

    #[test]
    fn get_bgp_dashboard_applies_the_group_type_and_flap_window_filters() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let options = BgpDashboardOptions {
            group_type: "anycast".to_string(),
            flap_window: Some(300),
        };
        client.get_bgp_dashboard(&options).expect("dashboard");
        let path = &transport.paths()[0];
        assert!(path.contains("group_type=anycast"));
        assert!(path.contains("flap_window=300"));
    }

    #[test]
    fn get_bandwidth_stats_applies_the_date_filter_when_given() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"in":1}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .get_bandwidth_stats(7, Some("2026-09-01"))
            .expect("bandwidth stats");
        let path = &transport.paths()[0];
        assert!(path.starts_with("/api/cloud/bw_stats/7"));
        assert!(path.contains("date=2026-09-01"));
    }

    #[test]
    fn get_bandwidth_stats_omits_the_date_filter_when_absent() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .get_bandwidth_stats(7, None)
            .expect("bandwidth stats");
        assert!(!transport.paths()[0].contains("date="));
    }

    #[test]
    fn get_base_images_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"os":"Ubuntu 24.04","type":"linux"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let images = client.get_base_images().expect("base images");
        assert_eq!(images[0].name, "Ubuntu 24.04");
    }

    #[test]
    fn get_private_images_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":2,"os":"my-snapshot","type":"custom"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let images = client.get_private_images().expect("private images");
        assert_eq!(images[0].id, 2);
    }

    #[test]
    fn replace_image_sends_a_json_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"queue_id":5}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .replace_image(9, &ReplaceImageRequest { replace_id: 3 })
            .expect("replace image");
        let requests = transport.requests.lock().unwrap();
        assert!(requests[0]
            .url
            .path()
            .ends_with("/cloud/images/9/replace_image"));
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"replace_id":3}"#);
    }

    #[test]
    fn update_cloud_ipv4_reverse_dns_puts_a_json_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .update_cloud_ipv4_reverse_dns(11, "host.example.test")
            .expect("update reverse dns");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Put);
        assert!(requests[0].url.path().ends_with("/cloud/ipv4/11"));
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"reverse":"host.example.test"}"#);
    }

    #[test]
    fn update_cloud_ipv6_reverse_dns_uses_the_ipv6_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .update_cloud_ipv6_reverse_dns(11, "host6.example.test")
            .expect("update reverse dns");
        assert!(transport.paths()[0].contains("/cloud/ipv6/11"));
    }

    #[test]
    fn get_kernels_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"name":"stock-4"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let kernels = client.get_kernels().expect("kernels");
        assert_eq!(kernels[0].name, "stock-4");
    }

    #[test]
    fn get_cloud_location_decodes_the_row() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":5,"name":"NYC","location":"New York","city":"New York","country":"US","iata_code":"JFK","flag":"us.png","latitude":null,"longitude":null}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let location = client.get_cloud_location(5).expect("cloud location");
        assert_eq!(location.iata_code, "JFK");
    }

    #[test]
    fn get_cloud_pool_decodes_the_row() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":2,"name":"pool-a","description":"","required_vcpu":"EPYC-Milan","hard_capabilities":[],"soft_capabilities":[],"private":0,"backup_cloud_pool_id":null,"default_ram_price":"0","default_cpu_price":"0","default_disk_price":"0","last_updated":"2026-01-01"}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let pool = client.get_cloud_pool(2).expect("cloud pool");
        assert_eq!(pool.required_vcpu.as_deref(), Some("EPYC-Milan"));
    }

    #[test]
    fn get_scaling_options_encodes_booleans_as_true_false_not_one_zero() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = ScalingOptionsRequest {
            include_current_plan: Some(true),
            min_ram: Some(1024),
            max_ram: None,
            min_cpus: None,
            max_cpus: Some(8),
        };
        client
            .get_scaling_options(4, &request)
            .expect("scaling options");
        let path = &transport.paths()[0];
        assert!(path.contains("include_current_plan=true"));
        assert!(path.contains("min_ram=1024"));
        assert!(path.contains("max_cpus=8"));
        assert!(!path.contains("max_ram="));
    }

    #[test]
    fn get_server_build_status_decodes_the_row() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"status":"Complete","percent":100,"response":"done"}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let status = client.get_server_build_status(1).expect("build status");
        assert_eq!(status.status, "Complete");
        assert_eq!(status.percent, 100);
    }

    #[test]
    fn get_server_deployment_info_applies_the_contract_type_filter() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .get_server_deployment_info(Some("dedicated"))
            .expect("deployment info");
        assert!(transport.paths()[0].contains("contract_type=dedicated"));
    }

    #[test]
    fn update_server_options_omits_unset_fields() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = UpdateServerOptionsRequest {
            fqdn: Some("host.example.test".to_string()),
            ..Default::default()
        };
        client
            .update_server_options(6, &request)
            .expect("update options");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Put);
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"fqdn":"host.example.test"}"#);
    }

    #[test]
    fn delete_server_with_options_posts_a_json_body_and_returns_the_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":42}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = DeleteServerRequest {
            cancel_billing: Some(true),
            ..Default::default()
        };
        let response = client
            .delete_server_with_options(42, &request)
            .expect("delete with options");
        assert_eq!(response.id, 42);
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Post);
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/json")
        );
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"cancel_billing":true}"#);
    }

    #[test]
    fn run_server_fsck_posts_with_no_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.run_server_fsck(6).expect("fsck");
        let requests = transport.requests.lock().unwrap();
        assert!(requests[0].body.is_none());
        assert!(requests[0].url.path().ends_with("/cloud/server/6/fsck"));
    }

    #[test]
    fn get_server_ipv4_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"ip":"203.0.113.5","type":"public"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let addresses = client.get_server_ipv4(6).expect("ipv4 addresses");
        assert_eq!(addresses[0].ip, "203.0.113.5");
        assert_eq!(addresses[0].address_type.as_deref(), Some("public"));
    }

    #[test]
    fn get_server_ipv6_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":2,"ip":"2001:db8::1"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let addresses = client.get_server_ipv6(6).expect("ipv6 addresses");
        assert_eq!(addresses[0].ip, "2001:db8::1");
    }

    #[test]
    fn list_server_jobs_and_get_server_job_decode() {
        let transport = RecordedTransport::new(vec![
            RecordedTransport::json(
                200,
                r#"{"result":"success","code":200,"data":[{"id":1,"ts_insert":"2026-01-01","command":"scale_vm","status":1}]}"#,
            ),
            RecordedTransport::json(
                200,
                r#"{"result":"success","code":200,"data":{"id":1,"ts_insert":"2026-01-01","command":"scale_vm","status":1}}"#,
            ),
        ]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let jobs = client.list_server_jobs(6).expect("list jobs");
        assert_eq!(jobs[0].command, "scale_vm");
        let job = client.get_server_job(6, 1).expect("get job");
        assert_eq!(job.id, 1);
    }

    #[test]
    fn reconfigure_server_network_posts_with_no_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.reconfigure_server_network(6).expect("netconfig");
        assert!(transport.paths()[0].contains("/cloud/server/6/netconfig"));
    }

    #[test]
    fn reset_server_root_password_renames_the_field_to_rootpass() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = ResetRootPasswordRequest {
            root_pass: "s3cret".to_string(),
            password: None,
        };
        client
            .reset_server_root_password(6, &request)
            .expect("reset password");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"rootpass":"s3cret"}"#);
    }

    #[test]
    fn reboot_server_sends_a_literal_null_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.reboot_server(6).expect("reboot");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].body.as_deref(), Some(b"null".as_slice()));
    }

    #[test]
    fn reboot_server_with_options_sends_the_force_flag() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":null}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .reboot_server_with_options(6, Some(&ServerActionRequest { force: Some(true) }))
            .expect("reboot with options");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"force":true}"#);
    }

    #[test]
    fn shutdown_and_start_server_with_options_use_the_expected_paths() {
        let transport = RecordedTransport::new(vec![
            RecordedTransport::json(200, r#"{"result":"success","code":200,"data":null}"#),
            RecordedTransport::json(200, r#"{"result":"success","code":200,"data":null}"#),
        ]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .shutdown_server_with_options(7, None)
            .expect("shutdown");
        client.start_server_with_options(7, None).expect("start");
        let paths = transport.paths();
        assert!(paths[0].contains("/cloud/server/7/shutdown"));
        assert!(paths[1].contains("/cloud/server/7/start"));
    }

    #[test]
    fn start_server_rescue_and_stop_server_rescue() {
        let transport = RecordedTransport::new(vec![
            RecordedTransport::json(200, r#"{"result":"success","code":200,"data":{}}"#),
            RecordedTransport::json(200, r#"{"result":"success","code":200,"data":null}"#),
        ]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .start_server_rescue(
                6,
                &RescueStartRequest {
                    rescue_pass: "pw".to_string(),
                    password: None,
                },
            )
            .expect("start rescue");
        client.stop_server_rescue(6).expect("stop rescue");
        let paths = transport.paths();
        assert!(paths[0].contains("/cloud/server/6/rescue_start"));
        assert!(paths[1].contains("/cloud/server/6/rescue_stop"));
    }

    #[test]
    fn get_server_bgp_sessions_applies_the_group_type_filter() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .get_server_bgp_sessions(6, Some("anycast"))
            .expect("bgp sessions");
        assert!(transport.paths()[0].contains("group_type=anycast"));
    }

    #[test]
    fn get_server_status_decodes_the_row() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"status":"active","state":"running"}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let status = client.get_server_status(6).expect("server status");
        assert_eq!(status.state, "running");
    }

    #[test]
    fn start_server_vnc_sends_a_literal_null_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.start_server_vnc(6).expect("start vnc");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].body.as_deref(), Some(b"null".as_slice()));
    }

    #[test]
    fn attempt_ssh_connection_sends_a_json_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .attempt_ssh_connection(&AttemptSshRequest {
                mbpkgid: 6,
                username: "root".to_string(),
                password: "pw".to_string(),
            })
            .expect("attempt ssh");
        assert!(transport.paths()[0].contains("/cloud/servers/attempt-ssh"));
    }

    #[test]
    fn get_current_server_decodes_the_row() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"fqdn":"current.example.test","mbpkgid":9}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let server = client.get_current_server().expect("current server");
        assert_eq!(server.id, 9);
    }

    #[test]
    fn get_virtual_server_contract_decodes_nullable_fields() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"contract_mbpkgid":6,"parent_contract_id":null,"brand":"na","mb_id":1,"contract_type":"metered","is_free":0,"include_bandwidth":1,"customer_po":null,"customer_description":null,"po_monthly_limit":null,"monthly_discount":null,"hourly_discount":null,"max_cpus":null,"max_ram":null,"max_disk":null,"allow_overage":null}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let contract = client.get_virtual_server_contract(6).expect("contract");
        assert_eq!(contract.contract_mbpkgid, Some(6));
        assert_eq!(contract.max_cpus, None);
    }

    #[test]
    fn get_plan_id_encodes_the_plan_name_in_the_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":123}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client.get_plan_id("USA CALIFORNIA 1").expect("plan id");
        assert!(transport.paths()[0].contains("USA+CALIFORNIA+1"));
    }

    #[test]
    fn get_deploy_sizes_applies_filters_and_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"plan_id":1,"plan":"1c1g","ram":"1024","disk":"25","transfer":"1000","price":"5.00","cpu":1,"port":"1000","available":10.0}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let request = DeploySizesRequest {
            min_cpu: Some(1),
            min_ram: None,
        };
        let sizes = client
            .get_deploy_sizes("NYC", &request)
            .expect("deploy sizes");
        assert_eq!(sizes[0].plan, "1c1g");
        assert!(transport.paths()[0].contains("min_cpu=1"));
    }

    #[test]
    fn get_deploy_sizes_treats_a_filter_match_of_nothing_as_an_empty_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let sizes = client
            .get_deploy_sizes("NYC", &DeploySizesRequest::default())
            .expect("empty list, not an error");
        assert!(sizes.is_empty());
    }

    #[test]
    fn get_storage_locations_applies_the_cloud_pool_filter() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .get_storage_locations(Some(3))
            .expect("storage locations");
        assert!(transport.paths()[0].contains("cloud_pool_id=3"));
    }

    #[test]
    fn bind_and_unbind_cloud_firewall_set() {
        let transport = RecordedTransport::new(vec![
            RecordedTransport::json(200, r#"{"result":"success","code":200,"data":{}}"#),
            RecordedTransport::json(200, r#"{"result":"success","code":200,"data":null}"#),
        ]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .bind_cloud_firewall_set(
                6,
                &BindFirewallSetRequest {
                    firewall_set_id: 1,
                    interface_id: 2,
                    set_priority: 0,
                },
            )
            .expect("bind firewall set");
        client
            .unbind_cloud_firewall_set(6, "1")
            .expect("unbind firewall set");
        let paths = transport.paths();
        assert!(paths[0].contains("/cloud/6/firewall-sets"));
        assert!(paths[1].contains("/cloud/6/firewall-sets/1"));
    }

    #[test]
    fn create_usage_contract_sends_the_mb_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"mb_id":9}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let contract = client
            .create_usage_contract(&CreateUsageContractRequest { mb_id: 9 })
            .expect("create usage contract");
        assert_eq!(contract.mb_id, Some(9));
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert_eq!(body, r#"{"mb_id":9}"#);
    }

    #[test]
    fn parse_cloud_init_uploads_a_multipart_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .parse_cloud_init("init.yaml", b"#cloud-config\nruncmd: []\n")
            .expect("parse cloud init");
        let requests = transport.requests.lock().unwrap();
        assert!(requests[0]
            .content_type
            .as_deref()
            .unwrap()
            .starts_with("multipart/form-data; boundary="));
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("filename=\"init.yaml\""));
        assert!(body.contains("#cloud-config"));
    }

    #[test]
    fn list_magic_meshes_requests_the_high_limit_page() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"data":[{"meshId":1,"name":"m","description":null}],"meta":{"limit":1000,"offset":0,"total":1}}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let meshes = client.list_magic_meshes().expect("list meshes");
        assert_eq!(meshes[0].mesh_id, 1);
        assert!(transport.paths()[0].contains("limit=1000"));
    }

    #[test]
    fn create_magic_mesh_returns_the_new_mesh_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"meshId":42}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateMagicMeshRequest {
            name: "m".to_string(),
            description: Some("d".to_string()),
            routers: vec![MeshRouterEntry { router_id: 7 }],
        };
        let mesh_id = client.create_magic_mesh(&request).expect("create mesh");
        assert_eq!(mesh_id, 42);
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"routerId\":7"));
    }

    #[test]
    fn get_magic_mesh_decodes_the_record() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"meshId":1,"name":"m","description":"d"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let mesh = client.get_magic_mesh(1).expect("get mesh");
        assert_eq!(mesh.description.as_deref(), Some("d"));
    }

    #[test]
    fn update_magic_mesh_patches_a_json_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateMagicMeshRequest {
            name: Some("renamed".to_string()),
            description: None,
        };
        client.update_magic_mesh(1, &request).expect("update mesh");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Patch);
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("renamed"));
        assert!(!body.contains("description"));
    }

    #[test]
    fn delete_magic_mesh_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client.delete_magic_mesh(1).expect("idempotent delete");
    }

    #[test]
    fn list_mesh_routers_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"data":[{"routerId":9,"name":"r","description":"d","ipv4Address":"10.0.0.1"}],"meta":{"limit":100,"offset":0,"total":1}}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let routers = client.list_mesh_routers(1).expect("list routers");
        assert_eq!(routers[0].ipv4_address, "10.0.0.1");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/meshes/1/routers"
        );
    }

    #[test]
    fn add_router_to_mesh_posts_the_router_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .add_router_to_mesh(1, &AddMeshRouterRequest { router_id: 9 })
            .expect("add router");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Post);
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"routerId\":9"));
    }

    #[test]
    fn remove_router_from_mesh_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .remove_router_from_mesh(1, 9)
            .expect("idempotent remove");
    }

    #[test]
    fn list_routers_requests_the_high_limit_page() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"data":[{"name":"r1","hasDefaultVrf":true,"canJoinMagicMesh":true}],"meta":{"limit":1000,"offset":0,"total":1}}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let routers = client.list_routers().expect("list routers");
        assert_eq!(routers[0].name, "r1");
        assert!(transport.paths()[0].contains("limit=1000"));
    }

    #[test]
    fn get_router_decodes_build_progress() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"name":"r1","readyOn":null,"hasDefaultVrf":false,"canJoinMagicMesh":false,"meshId":null,"build":[{"text":"Cloud Router configured","date":"2026-01-01T00:00:00Z"},{"text":"BGP configured","date":null}]}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let router = client.get_router(1).expect("get router");
        assert!(router.ready_on.is_none());
        assert_eq!(router.build.len(), 2);
        assert!(router.build[0].date.is_some());
        assert!(router.build[1].date.is_none());
    }

    #[test]
    fn create_router_returns_the_new_router_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"routerId":55}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateRouterRequest {
            package_id: 10,
            location_id: 20,
            name: Some("r1".to_string()),
            description: None,
        };
        let router_id = client.create_router(&request).expect("create router");
        assert_eq!(router_id, 55);
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"packageId\":10"));
        assert!(!body.contains("description"));
    }

    #[test]
    fn update_router_decodes_the_updated_router() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"name":"renamed","hasDefaultVrf":true,"canJoinMagicMesh":true}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterRequest {
            name: Some("renamed".to_string()),
            description: None,
        };
        let router = client.update_router(1, &request).expect("update router");
        assert_eq!(router.name, "renamed");
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Patch);
    }

    #[test]
    fn delete_router_propagates_not_found_rather_than_swallowing_it() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client.delete_router(1).expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn wait_for_router_ready_timeout_returns_immediately_when_already_ready() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"name":"r1","readyOn":"2026-01-01T00:00:00Z"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .wait_for_router_ready_timeout(1, Duration::from_secs(60))
            .expect("already ready");
    }

    #[test]
    fn wait_for_router_ready_timeout_names_the_pending_step_when_it_gives_up() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"name":"r1","readyOn":null,"build":[{"text":"BGP configured","date":null}]}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client
            .wait_for_router_ready_timeout(1, Duration::from_nanos(1))
            .expect_err("times out immediately");
        assert!(err.is_timeout());
    }

    #[test]
    fn list_router_vrfs_decodes_the_map() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"1":{"vrfId":1,"name":"default"}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let vrfs = client.list_router_vrfs(1).expect("list vrfs");
        assert_eq!(vrfs.get("1").expect("vrf 1").vrf_id, 1);
    }

    #[test]
    fn create_router_vrf_returns_the_new_vrf_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"vrfId":3}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateRouterVrfRequest {
            name: Some("v1".to_string()),
            description: None,
        };
        let vrf_id = client.create_router_vrf(1, &request).expect("create vrf");
        assert_eq!(vrf_id, 3);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Post);
    }

    #[test]
    fn get_router_vrf_decodes_the_record() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"vrfId":3,"name":"v1"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let vrf = client.get_router_vrf(1, 3).expect("get vrf");
        assert_eq!(vrf.name, "v1");
    }

    #[test]
    fn update_router_vrf_puts_and_returns_the_vrf_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"vrfId":3}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterVrfRequest {
            name: Some("renamed".to_string()),
            description: None,
        };
        let vrf_id = client
            .update_router_vrf(1, 3, &request)
            .expect("update vrf");
        assert_eq!(vrf_id, 3);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn delete_router_vrf_sends_the_delete_request() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client.delete_router_vrf(1, 3).expect("delete vrf");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/routers/1/config/vrfs/3"
        );
    }

    #[test]
    fn get_router_vrf_bgp_decodes_the_record() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"localAsn":"65001","routerId":"10.0.0.1","networks":[{"subnet":"10.0.0.0/24"}],"neighbors":[]}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let bgp = client.get_router_vrf_bgp(1, 3).expect("get bgp");
        assert_eq!(bgp.router_id, "10.0.0.1");
        assert_eq!(bgp.networks[0].subnet, "10.0.0.0/24");
    }

    #[test]
    fn update_router_vrf_bgp_decodes_the_numeric_router_id_quirk() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"localAsn":"65001","routerId":1,"networks":[],"neighbors":[]}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterVrfBgpRequest {
            networks: vec![RouterVrfBgpNetwork {
                subnet: "10.0.0.0/24".to_string(),
            }],
            asn: Some(RouterVrfBgpAsn {
                local: Some("65001".to_string()),
            }),
        };
        let result = client
            .update_router_vrf_bgp(1, 3, &request)
            .expect("update bgp");
        assert_eq!(result.router_id, 1);
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Put);
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("10.0.0.0/24"));
    }

    #[test]
    fn list_router_vrf_bgp_neighbors_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"neighborId":1,"address":"10.0.0.2","asn":{"remote":65002},"enabledIpVersion":{"ipv4":true,"ipv6":false}}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let neighbors = client
            .list_router_vrf_bgp_neighbors(1, 3)
            .expect("list neighbors");
        assert_eq!(neighbors[0].address, "10.0.0.2");
        assert_eq!(neighbors[0].asn.remote, 65002);
    }

    #[test]
    fn create_router_vrf_bgp_neighbor_returns_the_new_neighbor_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"neighborId":9}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateRouterVrfBgpNeighborRequest {
            address: "10.0.0.2".to_string(),
            is_shutdown: false,
            do_as_override: false,
            do_next_help_self: false,
            source: None,
            enabled_ip_version: BgpNeighborEnabledIpVersion {
                ipv4: true,
                ipv6: false,
            },
            ebgp_multihop: None,
            asn: BgpNeighborAsn { remote: 65002 },
            md5_secret: String::new(),
            import: None,
            export: None,
            name: String::new(),
            description: String::new(),
        };
        let neighbor_id = client
            .create_router_vrf_bgp_neighbor(1, 3, &request)
            .expect("create neighbor");
        assert_eq!(neighbor_id, 9);
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(!body.contains("md5Secret"));
    }

    #[test]
    fn get_router_vrf_bgp_neighbor_decodes_the_record() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"neighborId":9,"address":"10.0.0.2"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let neighbor = client
            .get_router_vrf_bgp_neighbor(1, 3, 9)
            .expect("get neighbor");
        assert_eq!(neighbor.neighbor_id, 9);
    }

    #[test]
    fn update_router_vrf_bgp_neighbor_returns_the_neighbor_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"neighborId":9}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterVrfBgpNeighborRequest {
            address: "10.0.0.3".to_string(),
            is_shutdown: true,
            do_as_override: false,
            do_next_help_self: false,
            source: None,
            enabled_ip_version: BgpNeighborEnabledIpVersion {
                ipv4: true,
                ipv6: false,
            },
            ebgp_multihop: None,
            asn: BgpNeighborAsn { remote: 65002 },
            md5_secret: String::new(),
            import: None,
            export: None,
            name: String::new(),
            description: String::new(),
        };
        let neighbor_id = client
            .update_router_vrf_bgp_neighbor(1, 3, 9, &request)
            .expect("update neighbor");
        assert_eq!(neighbor_id, 9);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn delete_router_vrf_bgp_neighbor_sends_the_delete_request() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .delete_router_vrf_bgp_neighbor(1, 3, 9)
            .expect("delete neighbor");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/routers/1/config/vrfs/3/bgp/neighbors/9"
        );
    }

    #[test]
    fn list_router_static_routes_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"staticRouteId":1,"network":"0.0.0.0/0","via":{"nextHop":"10.0.0.1"}}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let routes = client.list_router_static_routes(1, 3).expect("list routes");
        assert_eq!(routes[0].network, "0.0.0.0/0");
    }

    #[test]
    fn get_router_static_route_returns_not_found_when_no_route_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"staticRouteId":1,"network":"0.0.0.0/0"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client
            .get_router_static_route(1, 3, 99)
            .expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn get_router_static_route_finds_the_matching_route() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"staticRouteId":1,"network":"0.0.0.0/0"},{"staticRouteId":2,"network":"10.0.0.0/8"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let route = client.get_router_static_route(1, 3, 2).expect("get route");
        assert_eq!(route.network, "10.0.0.0/8");
    }

    #[test]
    fn create_router_static_route_returns_the_new_route_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"staticRouteId":4}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateRouterStaticRouteRequest {
            network: "10.0.0.0/8".to_string(),
            via: StaticRouteVia {
                next_hop: "10.0.0.1".to_string(),
                ..Default::default()
            },
            description: String::new(),
            distance: None,
        };
        let route_id = client
            .create_router_static_route(1, 3, &request)
            .expect("create route");
        assert_eq!(route_id, 4);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Post);
    }

    #[test]
    fn update_router_static_route_returns_the_route_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"staticRouteId":4}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterStaticRouteRequest {
            network: "10.0.0.0/8".to_string(),
            via: StaticRouteVia {
                next_hop: "10.0.0.2".to_string(),
                ..Default::default()
            },
            description: String::new(),
            distance: None,
        };
        let route_id = client
            .update_router_static_route(1, 3, 4, &request)
            .expect("update route");
        assert_eq!(route_id, 4);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn delete_router_static_route_sends_the_delete_request() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .delete_router_static_route(1, 3, 4)
            .expect("delete route");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/routers/1/config/vrfs/3/static-routes/4"
        );
    }

    #[test]
    fn create_router_prefix_list_returns_the_new_list_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"prefixListId":6}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateRouterPrefixListRequest {
            name: "p1".to_string(),
            ip_version: 4,
            description: String::new(),
            rules: vec![PrefixListRule {
                action: "permit".to_string(),
                prefix: "10.0.0.0/8".to_string(),
            }],
        };
        let list_id = client
            .create_router_prefix_list(1, &request)
            .expect("create prefix list");
        assert_eq!(list_id, 6);
    }

    #[test]
    fn list_router_prefix_lists_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"prefixListId":6,"name":"p1","ipVersion":4}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let lists = client.list_router_prefix_lists(1).expect("list lists");
        assert_eq!(lists[0].name, "p1");
    }

    #[test]
    fn get_router_prefix_list_returns_not_found_when_no_list_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"prefixListId":6}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client.get_router_prefix_list(1, 99).expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn update_router_prefix_list_returns_the_list_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"prefixListId":6}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterPrefixListRequest {
            name: "p1-renamed".to_string(),
            ip_version: 4,
            description: String::new(),
            rules: vec![],
        };
        let list_id = client
            .update_router_prefix_list(1, 6, &request)
            .expect("update prefix list");
        assert_eq!(list_id, 6);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn delete_router_prefix_list_sends_the_delete_request() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .delete_router_prefix_list(1, 6)
            .expect("delete prefix list");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/routers/1/config/prefix-lists/6"
        );
    }

    #[test]
    fn get_router_ntp_config_decodes_the_record() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"enabled":true,"interfaceId":null,"upstreams":[{"domain":"pool.ntp.org"}]}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let ntp = client.get_router_ntp_config(1).expect("get ntp");
        assert!(ntp.enabled);
        assert_eq!(ntp.upstreams[0].domain, "pool.ntp.org");
    }

    #[test]
    fn update_router_ntp_config_always_sends_every_field() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"routerId":1}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterNtpConfigRequest {
            enabled: Some(true),
            interface_id: None,
            upstreams: vec![],
        };
        let router_id = client
            .update_router_ntp_config(1, &request)
            .expect("update ntp");
        assert_eq!(router_id, 1);
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("\"interfaceId\":null"));
        assert!(body.contains("\"upstreams\":[]"));
    }

    #[test]
    fn get_router_routing_views_posts_the_selector_and_returns_raw_data() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"routes":[]}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = RouterRoutingViewRequest {
            views: vec![RouterRoutingViewSelector {
                id: String::new(),
                ip_version: 4,
                name: "bgp".to_string(),
                filter: String::new(),
            }],
        };
        let data = client
            .get_router_routing_views(1, 3, &request)
            .expect("get routing views");
        assert!(data.get("routes").is_some());
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Post);
        assert_eq!(
            requests[0].url.path(),
            "/cloud-routing/routers/1/view/routing/3"
        );
    }

    #[test]
    fn get_router_routing_overview_returns_raw_data() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"summary":"ok"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let data = client
            .get_router_routing_overview(1, 3)
            .expect("get overview");
        assert_eq!(data["summary"], "ok");
    }

    #[test]
    fn get_router_ipsec_config_decodes_the_record() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"ikeGroup":{"encryption":"aes256","hash":"sha256"},"espGroup":{"encryption":"aes256","hash":"sha256"}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let config = client.get_router_ipsec_config(1).expect("get ipsec config");
        assert_eq!(config.ike_group.encryption, "aes256");
        assert_eq!(config.esp_group.hash, "sha256");
    }

    #[test]
    fn update_router_ipsec_config_sends_a_put_to_the_config_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterIpSecConfigRequest {
            ike_group: RouterIpSecIkeGroup {
                encryption: "aes256".to_string(),
                ..Default::default()
            },
            esp_group: RouterIpSecEspGroup::default(),
        };
        client
            .update_router_ipsec_config(1, &request)
            .expect("update ipsec config");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].method, Method::Put);
        assert_eq!(
            requests[0].url.path(),
            "/cloud-routing/routers/1/config/ipSec"
        );
    }

    #[test]
    fn list_router_vrf_ipsec_peers_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"ipSecPeerId":1,"name":"peer-a"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let peers = client
            .list_router_vrf_ipsec_peers(1, 3)
            .expect("list peers");
        assert_eq!(peers[0].name, "peer-a");
    }

    #[test]
    fn get_router_vrf_ipsec_peer_returns_not_found_when_no_peer_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"ipSecPeerId":1}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client
            .get_router_vrf_ipsec_peer(1, 3, 99)
            .expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn get_router_vrf_ipsec_peer_finds_the_matching_peer() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"ipSecPeerId":1,"name":"a"},{"ipSecPeerId":2,"name":"b"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let peer = client.get_router_vrf_ipsec_peer(1, 3, 2).expect("get peer");
        assert_eq!(peer.name, "b");
    }

    #[test]
    fn create_router_vrf_ipsec_peer_returns_the_new_peer_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"ipSecPeerId":7}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateRouterVrfIpSecPeerRequest {
            name: "peer-a".to_string(),
            remote_id: "remote@example.test".to_string(),
            psk_secret: "secret".to_string(),
            do_initiate_connection: true,
            peer_address: "203.0.113.1".to_string(),
            overlay_network: RouterVrfIpSecOverlayNetwork {
                ipv4: Some("169.254.0.1/30".to_string()),
                ipv6: None,
            },
            ..Default::default()
        };
        let peer_id = client
            .create_router_vrf_ipsec_peer(1, 3, &request)
            .expect("create peer");
        assert_eq!(peer_id, 7);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Post);
    }

    #[test]
    fn update_router_vrf_ipsec_peer_returns_the_peer_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"ipSecPeerId":7}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterVrfIpSecPeerRequest {
            name: "peer-a-renamed".to_string(),
            remote_id: "remote@example.test".to_string(),
            psk_secret: "secret".to_string(),
            do_initiate_connection: true,
            overlay_network: RouterVrfIpSecOverlayNetwork::default(),
            ..Default::default()
        };
        let peer_id = client
            .update_router_vrf_ipsec_peer(1, 3, 7, &request)
            .expect("update peer");
        assert_eq!(peer_id, 7);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn delete_router_vrf_ipsec_peer_sends_the_delete_request() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .delete_router_vrf_ipsec_peer(1, 3, 7)
            .expect("delete peer");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/routers/1/config/vrfs/3/ipSec/peers/7"
        );
    }

    #[test]
    fn list_router_vrf_interfaces_decodes_the_map() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"9":{"interfaceId":9,"vrfId":3,"type":"wireguard","name":"wg0"}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let interfaces = client
            .list_router_vrf_interfaces(1, 3)
            .expect("list interfaces");
        assert_eq!(interfaces["9"].interface_type, "wireguard");
    }

    #[test]
    fn create_router_vrf_interface_returns_the_new_interface_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"interfaceId":9}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateRouterVrfInterfaceRequest {
            interface_type: "wireguard".to_string(),
            name: "wg0".to_string(),
            wireguard_port: Some(51820),
            ..Default::default()
        };
        let interface_id = client
            .create_router_vrf_interface(1, 3, &request)
            .expect("create interface");
        assert_eq!(interface_id, 9);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Post);
    }

    #[test]
    fn get_router_vrf_interface_decodes_wireguard_peers() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"interfaceId":9,"vrfId":3,"type":"wireguard","name":"wg0","peers":[{"wireguardPeerId":1,"publicKey":"pub","privateKey":"priv"}]}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let interface = client
            .get_router_vrf_interface(1, 3, 9)
            .expect("get interface");
        assert_eq!(interface.peers[0].public_key, "pub");
    }

    #[test]
    fn update_router_vrf_interface_returns_the_interface_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"interfaceId":9}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterVrfInterfaceRequest {
            interface_type: "wireguard".to_string(),
            name: "wg0-renamed".to_string(),
            ..Default::default()
        };
        let interface_id = client
            .update_router_vrf_interface(1, 3, 9, &request)
            .expect("update interface");
        assert_eq!(interface_id, 9);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn delete_router_vrf_interface_sends_the_delete_request() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .delete_router_vrf_interface(1, 3, 9)
            .expect("delete interface");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/routers/1/config/vrfs/3/interfaces/9"
        );
    }

    #[test]
    fn create_router_vrf_interface_wireguard_peer_returns_the_new_peer_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"wireguardPeerId":4}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateRouterVrfInterfaceWireguardPeerRequest {
            allowed_ips: vec![WireguardPeerAllowedIp {
                network: "10.10.0.0/24".to_string(),
            }],
            ..Default::default()
        };
        let peer_id = client
            .create_router_vrf_interface_wireguard_peer(1, 3, 9, &request)
            .expect("create wireguard peer");
        assert_eq!(peer_id, 4);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Post);
    }

    #[test]
    fn get_router_vrf_interface_wireguard_peer_decodes_the_record() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"wireguardPeerId":4,"publicKey":"pub","privateKey":"priv","allowedIps":[{"network":"10.10.0.0/24"}]}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let peer = client
            .get_router_vrf_interface_wireguard_peer(1, 3, 9, 4)
            .expect("get wireguard peer");
        assert_eq!(peer.allowed_ips[0].network, "10.10.0.0/24");
    }

    #[test]
    fn delete_router_vrf_interface_wireguard_peer_sends_the_delete_request() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .delete_router_vrf_interface_wireguard_peer(1, 3, 9, 4)
            .expect("delete wireguard peer");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/routers/1/config/vrfs/3/interfaces/9/wireguard-peers/4"
        );
    }

    #[test]
    fn create_router_vrf_snat_rule_decodes_the_nested_match_and_translation() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"snatRuleId":5,"ipVersion":4,"match":{"interfaceId":9,"network":"10.0.0.0/24"},"translation":{"network":"203.0.113.1/32"}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let request = CreateRouterVrfSnatRuleRequest {
            ip_version: 4,
            protocol: "any".to_string(),
            match_criteria: Some(RouterVrfSnatMatch {
                interface_id: 9,
                network: "10.0.0.0/24".to_string(),
                port: None,
            }),
            translation: Some(RouterVrfSnatTranslation {
                network: "203.0.113.1/32".to_string(),
                port: None,
            }),
            ..Default::default()
        };
        let rule_id = client
            .create_router_vrf_snat_rule(1, 3, &request)
            .expect("create snat rule");
        assert_eq!(rule_id, 5);
    }

    #[test]
    fn list_router_vrf_snat_rules_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"snatRuleId":1,"name":"rule-a"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let rules = client.list_router_vrf_snat_rules(1, 3).expect("list rules");
        assert_eq!(rules[0].name, "rule-a");
    }

    #[test]
    fn get_router_vrf_snat_rule_returns_not_found_when_no_rule_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"snatRuleId":1}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client
            .get_router_vrf_snat_rule(1, 3, 99)
            .expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn update_router_vrf_snat_rule_returns_the_rule_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"snatRuleId":5}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterVrfSnatRuleRequest {
            ip_version: 4,
            protocol: "any".to_string(),
            ..Default::default()
        };
        let rule_id = client
            .update_router_vrf_snat_rule(1, 3, 5, &request)
            .expect("update snat rule");
        assert_eq!(rule_id, 5);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn delete_router_vrf_snat_rule_sends_the_delete_request() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .delete_router_vrf_snat_rule(1, 3, 5)
            .expect("delete snat rule");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/routers/1/config/vrfs/3/snat-rules/5"
        );
    }

    #[test]
    fn create_router_vrf_dnat_rule_decodes_the_nested_match_and_translation() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"dnatRuleId":6,"ipVersion":4,"match":{"interfaceId":9,"network":"203.0.113.1/32"},"translation":{"network":"10.0.0.5/32"}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let request = CreateRouterVrfDnatRuleRequest {
            ip_version: 4,
            protocol: "tcp".to_string(),
            match_criteria: Some(RouterVrfDnatMatch {
                interface_id: 9,
                network: "203.0.113.1/32".to_string(),
                port: None,
            }),
            translation: Some(RouterVrfDnatTranslation {
                network: "10.0.0.5/32".to_string(),
                port: None,
            }),
            ..Default::default()
        };
        let rule_id = client
            .create_router_vrf_dnat_rule(1, 3, &request)
            .expect("create dnat rule");
        assert_eq!(rule_id, 6);
    }

    #[test]
    fn list_router_vrf_dnat_rules_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"dnatRuleId":1,"name":"rule-a"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let rules = client.list_router_vrf_dnat_rules(1, 3).expect("list rules");
        assert_eq!(rules[0].name, "rule-a");
    }

    #[test]
    fn get_router_vrf_dnat_rule_returns_not_found_when_no_rule_matches() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"dnatRuleId":1}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client
            .get_router_vrf_dnat_rule(1, 3, 99)
            .expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn update_router_vrf_dnat_rule_returns_the_rule_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"dnatRuleId":6}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterVrfDnatRuleRequest {
            ip_version: 4,
            protocol: "tcp".to_string(),
            ..Default::default()
        };
        let rule_id = client
            .update_router_vrf_dnat_rule(1, 3, 6, &request)
            .expect("update dnat rule");
        assert_eq!(rule_id, 6);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn delete_router_vrf_dnat_rule_sends_the_delete_request() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .delete_router_vrf_dnat_rule(1, 3, 6)
            .expect("delete dnat rule");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/routers/1/config/vrfs/3/dnat-rules/6"
        );
    }

    #[test]
    fn list_router_vrf_tunnels_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"tunnelId":1,"name":"tun0","mtu":"1476"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let tunnels = client.list_router_vrf_tunnels(1, 3).expect("list tunnels");
        assert_eq!(tunnels[0].mtu, "1476");
    }

    #[test]
    fn get_router_vrf_tunnel_uses_the_direct_endpoint() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"tunnelId":1,"name":"tun0","endpointAddress":{"source":"192.0.2.1","remote":"198.51.100.1"}}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let tunnel = client.get_router_vrf_tunnel(1, 3, 1).expect("get tunnel");
        assert_eq!(tunnel.endpoint_address.remote, "198.51.100.1");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/routers/1/config/vrfs/3/tunnels/1"
        );
    }

    #[test]
    fn create_router_vrf_tunnel_returns_the_new_tunnel_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"tunnelId":2}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateRouterVrfTunnelRequest {
            ip_key: 42,
            name: "tun0".to_string(),
            mtu: 1476,
            endpoint_address: RouterVrfTunnelRemoteEndpoint {
                remote: "198.51.100.1".to_string(),
            },
            ..Default::default()
        };
        let tunnel_id = client
            .create_router_vrf_tunnel(1, 3, &request)
            .expect("create tunnel");
        assert_eq!(tunnel_id, 2);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Post);
    }

    #[test]
    fn update_router_vrf_tunnel_returns_the_tunnel_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"tunnelId":2}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterVrfTunnelRequest {
            ip_key: 42,
            name: "tun0-renamed".to_string(),
            mtu: 1476,
            endpoint_address: RouterVrfTunnelRemoteEndpoint {
                remote: "198.51.100.2".to_string(),
            },
            ..Default::default()
        };
        let tunnel_id = client
            .update_router_vrf_tunnel(1, 3, 2, &request)
            .expect("update tunnel");
        assert_eq!(tunnel_id, 2);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn delete_router_vrf_tunnel_sends_the_delete_request() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .delete_router_vrf_tunnel(1, 3, 2)
            .expect("delete tunnel");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud-routing/routers/1/config/vrfs/3/tunnels/2"
        );
    }

    #[test]
    fn get_router_vrf_dhcp_decodes_the_record() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"enabled":true,"interfaceId":9,"subnet":"10.0.0.0/24","range":{"firstAddress":"10.0.0.10","lastAddress":"10.0.0.200"},"domainNameServers":[{"address":"1.1.1.1"}]}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let dhcp = client.get_router_vrf_dhcp(1, 3).expect("get dhcp config");
        assert!(dhcp.enabled);
        assert_eq!(dhcp.domain_name_servers[0].address, "1.1.1.1");
        assert_eq!(dhcp.range.expect("range").first_address, "10.0.0.10");
    }

    #[test]
    fn update_router_vrf_dhcp_returns_the_router_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"routerId":1}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateRouterVrfDhcpRequest {
            enabled: true,
            interface_id: 9,
            subnet: "10.0.0.0/24".to_string(),
            ..Default::default()
        };
        let router_id = client
            .update_router_vrf_dhcp(1, 3, &request)
            .expect("update dhcp config");
        assert_eq!(router_id, 1);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn get_datacenter_by_iata_returns_the_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":42,"name":"New York","iata":"nyc"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let id = client.get_datacenter_by_iata("nyc").expect("datacenter id");
        assert_eq!(id, 42);
        assert!(transport.paths()[0].contains("platform/datacenters-by-iata/nyc"));
    }

    #[test]
    fn get_sizes_hits_the_location_free_endpoint() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"plan_id":1,"plan":"1c1g","ram":"1024","disk":"25","transfer":"1000","price":"5.00","cpu":1,"port":"1000","available":10.0}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let sizes = client.get_sizes().expect("sizes");
        assert_eq!(sizes[0].cpu, 1);
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/api/cloud/sizes"
        );
    }

    #[test]
    fn get_plans_calls_the_same_endpoint_as_get_sizes() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"plan_id":2,"plan":"2c2g","ram":"2048","disk":"50","transfer":"2000","price":"10.00","cpu":2,"port":"1000","available":5.0}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let plans = client.get_plans().expect("plans");
        assert_eq!(plans[0].plan, "2c2g");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/api/cloud/sizes"
        );
    }

    #[test]
    fn get_contract_usage_decodes_account_totals() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":7,"contract_mbpkgid":100,"max_cpus":16}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let usage = client.get_contract_usage().expect("contract usage");
        assert_eq!(usage.id, Some(7));
        assert_eq!(usage.max_cpus, Some(16));
    }

    #[test]
    fn get_boot_profiles_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":3,"name":"Ubuntu","type":"linux","description":"","builder":"qemu","kernel":"","boot":"hd","serial":"","disk_represent":"virtio","image_template":0,"last_updated":"2024-01-01","pae":1,"acpi":1,"apic":1,"xlocaltime":0,"sdl":0,"vnc":1,"vncconsole":0,"vncunused":0,"hide":0,"kvm":1}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let profiles = client.get_boot_profiles().expect("boot profiles");
        assert_eq!(profiles[0].boot_profile_id, 3);
        assert_eq!(profiles[0].profile_type, "linux");
        assert_eq!(profiles[0].kvm, 1);
    }

    #[test]
    fn get_server_disks_decodes_each_element() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"size":25},{"size":50}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let disks = client.get_server_disks(555).expect("server disks");
        assert_eq!(disks.len(), 2);
        assert!(transport.paths()[0].contains("cloud/disks/555"));
    }

    #[test]
    fn get_dedicated_os_profiles_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"OSID":5,"Name":"Ubuntu 24.04","GroupName":"linux"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let profiles = client
            .get_dedicated_os_profiles()
            .expect("dedicated os profiles");
        assert_eq!(profiles[0].os_id, 5);
        assert_eq!(profiles[0].group_name, "linux");
    }

    #[test]
    fn get_dedicated_rescue_os_profiles_hits_the_rescue_endpoint() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"OSID":9,"Name":"Rescue Linux"}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let profiles = client
            .get_dedicated_rescue_os_profiles()
            .expect("dedicated rescue os profiles");
        assert_eq!(profiles[0].os_id, 9);
        assert!(transport.paths()[0].contains("dedicated/os/rescue-system-list"));
    }

    #[test]
    fn get_dedicated_disk_layouts_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"name":"default","profile":"linux","min_disks":1}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let layouts = client.get_dedicated_disk_layouts(5).expect("disk layouts");
        assert_eq!(layouts[0].layout_id, 1);
        assert_eq!(layouts[0].min_disks, 1);
        assert!(transport.paths()[0].contains("dedicated/disklayouts/5"));
    }

    #[test]
    fn create_ssl_certificate_returns_the_new_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"sslCertificateId":9}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateSslCertificateRequest {
            name: "example".to_string(),
            certificate: "test-certificate".to_string(),
            private_key: "test-private-key".to_string(),
            ..Default::default()
        };
        let id = client
            .create_ssl_certificate(&request)
            .expect("create certificate");
        assert_eq!(id, 9);
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Post);
    }

    #[test]
    fn list_ssl_certificates_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"sslCertificateId":1,"name":"example","domains":["example.test"],"isActive":true,"status":"active"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let certs = client.list_ssl_certificates().expect("list certificates");
        assert_eq!(certs[0].ssl_certificate_id, 1);
        assert!(certs[0].is_active);
        assert_eq!(certs[0].domains, vec!["example.test".to_string()]);
    }

    #[test]
    fn list_ssl_certificates_propagates_not_found_rather_than_an_empty_list() {
        // gona's getList does not swallow a semantic not-found into an empty list; this
        // matches that encoding exactly rather than the v2 list_or_empty convention.
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client.list_ssl_certificates().expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn get_ssl_certificate_decodes_dates() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"sslCertificateId":1,"name":"example","dates":{"created":"2024-01-01","notBefore":"2024-01-01","expiration":"2025-01-01"}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let cert = client.get_ssl_certificate(1).expect("get certificate");
        let dates = cert.dates.expect("dates");
        assert_eq!(dates.not_before, "2024-01-01");
        assert_eq!(dates.expiration, "2025-01-01");
    }

    #[test]
    fn update_ssl_certificate_sends_a_patch() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":null}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = UpdateSslCertificateRequest {
            name: "renamed".to_string(),
            ..Default::default()
        };
        client
            .update_ssl_certificate(1, &request)
            .expect("update certificate");
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Patch);
    }

    #[test]
    fn delete_ssl_certificate_treats_already_gone_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client
            .delete_ssl_certificate(1)
            .expect("delete already gone");
    }

    #[test]
    fn create_nlb_group_decodes_the_group() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"networkGroupId":10,"name":"web","ipVersion":4,"algorithm":"round_robin","match":{"address":"0.0.0.0/0"},"healthCheck":{"enabled":true,"method":"tcp","interval":10,"retries":3,"delay":5,"timeout":5},"rules":[{"protocol":"tcp","ports":{"match":80,"internal":8080}}],"backends":[{"name":"b1","internalAddress":"10.0.0.1"}]}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = CreateNlbGroupRequest {
            name: "web".to_string(),
            ip_version: 4,
            algorithm: "round_robin".to_string(),
            matcher: NlbGroupMatch {
                address: "0.0.0.0/0".to_string(),
            },
            health_check: NlbGroupHealthCheck {
                enabled: true,
                method: "tcp".to_string(),
                interval: 10,
                retries: 3,
                delay: 5,
                timeout: 5,
            },
            rules: vec![NlbGroupRule {
                protocol: "tcp".to_string(),
                network_rule_id: 0,
                ports: NlbGroupRulePorts {
                    match_port: 80,
                    internal: 8080,
                },
            }],
            backends: vec![NlbGroupBackend {
                name: "b1".to_string(),
                internal_address: "10.0.0.1".to_string(),
                is_online: false,
                network_backend_id: 0,
            }],
            ..Default::default()
        };
        let group = client.create_nlb_group(3, &request).expect("create group");
        assert_eq!(group.network_group_id, 10);
        assert_eq!(group.rules[0].ports.match_port, 80);
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/network-loadbalancers/3/groups"
        );
    }

    #[test]
    fn list_nlb_groups_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"networkGroupId":10,"name":"web"}]}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let groups = client.list_nlb_groups(3).expect("list groups");
        assert_eq!(groups[0].network_group_id, 10);
    }

    #[test]
    fn list_nlb_groups_propagates_not_found_rather_than_an_empty_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let err = client.list_nlb_groups(3).expect_err("not found");
        assert!(err.is_not_found());
    }

    #[test]
    fn get_nlb_group_decodes_backends_and_rules() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"networkGroupId":10,"name":"web","rules":[{"protocol":"tcp","networkRuleId":4,"ports":{"match":80,"internal":8080}}],"backends":[{"name":"b1","internalAddress":"10.0.0.1","isOnline":true,"networkBackendId":7}]}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let group = client.get_nlb_group(3, 10).expect("get group");
        assert_eq!(group.rules[0].network_rule_id, 4);
        assert!(group.backends[0].is_online);
        assert_eq!(group.backends[0].network_backend_id, 7);
    }

    #[test]
    fn replace_nlb_group_sends_a_put() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"networkGroupId":10,"name":"web2"}}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let request = ReplaceNlbGroupRequest {
            name: "web2".to_string(),
            ip_version: 4,
            algorithm: "round_robin".to_string(),
            ..Default::default()
        };
        let group = client
            .replace_nlb_group(3, 10, &request)
            .expect("replace group");
        assert_eq!(group.name, "web2");
        assert_eq!(transport.requests.lock().unwrap()[0].method, Method::Put);
    }

    #[test]
    fn delete_nlb_group_treats_already_gone_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"code":404,"data":{"message":"not found"}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        client.delete_nlb_group(3, 10).expect("delete already gone");
    }

    #[test]
    fn query_statistics_sends_the_metric_selector_body_and_decodes_the_result() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[{"metric":{"cpu.usage":{}},"service":"compute","data":[{"count":1.0,"resources":2.0,"avg":3.0,"sum":4.0}]}]}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let results = client
            .query_statistics(&["cpu.usage".to_string()])
            .expect("query statistics");
        assert_eq!(results[0].metric, "cpu.usage");
        assert_eq!(results[0].service, "compute");
        assert_eq!(results[0].data[0].sum, 4.0);
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].url.path(), "/cloud/statistics");
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("cpu.usage"));
    }

    #[test]
    fn query_networking_statistics_hits_the_networking_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[]}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .query_networking_statistics(&["bandwidth.rx".to_string()])
            .expect("query networking statistics");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud/networking/statistics"
        );
    }

    #[test]
    fn query_anycast_statistics_hits_the_anycast_path() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":[]}"#,
        )]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        client
            .query_anycast_statistics(&["announcements".to_string()])
            .expect("query anycast statistics");
        assert_eq!(
            transport.requests.lock().unwrap()[0].url.path(),
            "/cloud/networking/anycast/statistics"
        );
    }

    #[test]
    fn get_metric_names_sorts_keys_and_extracts_the_time_window() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"__timeWindow":{"start":"2024-01-01","end":"2024-01-02","seconds":86400},"cpu.usage":{"service":"compute","resources":2,"avg":{"sum":1.0,"avg":1.0,"min":1.0,"max":1.0},"last":{},"sum":{}},"bandwidth.rx":{"service":"network","resources":1,"avg":{},"last":{},"sum":{}}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let names = client.get_metric_names().expect("get metric names");
        assert_eq!(names.time_window.seconds, 86400);
        assert_eq!(names.metrics.len(), 2);
        assert_eq!(names.metrics[0].metric, "bandwidth.rx");
        assert_eq!(names.metrics[1].metric, "cpu.usage");
        assert_eq!(names.metrics[1].resources, 2);
    }

    // --- DDoS ---

    #[test]
    fn get_ddos_attacks_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"ip":"203.0.113.5","pps":900000}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let attacks = client.get_ddos_attacks().expect("attacks");
        assert_eq!(attacks[0].attack_id, 1);
        assert_eq!(attacks[0].ip, "203.0.113.5");
        assert_eq!(attacks[0].pps, 900000);
    }

    #[test]
    fn get_ddos_dashboard_sends_every_query_param() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"total_attacks":3,"active_rules":1,"longest_attack_seconds":120,"top_attacks":[],"period":7}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let dashboard = client
            .get_ddos_dashboard(&DdosDashboardOptions {
                period: Some(7),
                include_ended: Some(true),
                limit: Some(5),
            })
            .expect("dashboard");
        assert_eq!(dashboard.total_attacks, 3);
        assert_eq!(dashboard.period, 7);
        let path = transport.paths()[0].clone();
        assert!(path.contains("period=7"));
        assert!(path.contains("include_ended=1"));
        assert!(path.contains("limit=5"));
    }

    // --- Access control subnets ---

    #[test]
    fn access_control_subnet_create_and_delete_round_trip() {
        let transport = RecordedTransport::new(vec![
            RecordedTransport::json(
                200,
                r#"{"result":"success","code":200,"data":{"id":9,"label":"office","subnet":"203.0.113.0/24"}}"#,
            ),
            RecordedTransport::json(
                404,
                r#"{"result":"error","code":404,"message":"not found","data":null}"#,
            ),
        ]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let subnet = client
            .create_access_control_subnet(&CreateAccessControlSubnetRequest {
                label: "office".to_string(),
                subnet: "203.0.113.0/24".to_string(),
            })
            .expect("create");
        assert_eq!(subnet.id, 9);
        assert_eq!(subnet.subnet, "203.0.113.0/24");

        client
            .delete_access_control_subnet(9)
            .expect("delete already-gone subnet is success");
    }

    // --- HTTP load balancer groups ---

    #[test]
    fn http_lb_group_create_list_and_delete() {
        let transport = RecordedTransport::new(vec![
            RecordedTransport::json(
                200,
                r#"{"code":200,"data":{"httpGroupId":5,"name":"web","algorithm":"round-robin","match":{"address":"0.0.0.0","ports":"80"},"healthCheck":{"active":{"enabled":true,"timeout":null},"passive":{"enabled":false}},"rules":[],"backends":[]}}"#,
            ),
            RecordedTransport::json(
                200,
                r#"{"code":200,"data":[{"httpGroupId":5,"name":"web"}]}"#,
            ),
            RecordedTransport::json(404, r#"{"code":404,"message":"not found","data":null}"#),
        ]);
        let client =
            V3Client::with_transport("key", "https://vapi3.example.test", transport.clone())
                .expect("client");
        let group = client
            .create_http_lb_group(
                3,
                &CreateHttpLbGroupRequest {
                    name: "web".to_string(),
                    algorithm: "round-robin".to_string(),
                    matcher: HttpLbGroupMatch {
                        address: "0.0.0.0".to_string(),
                        ports: "80".to_string(),
                    },
                    ..Default::default()
                },
            )
            .expect("create group");
        assert_eq!(group.http_group_id, 5);
        assert!(transport.paths()[0].starts_with("/http-loadbalancers/3/groups"));

        let groups = client.list_http_lb_groups(3).expect("list groups");
        assert_eq!(groups.len(), 1);

        client
            .delete_http_lb_group(3, 5)
            .expect("delete already-gone group is success");
    }

    // --- Dedicated server builds (metal) ---

    #[test]
    fn buy_build_dedicated_server_sends_a_form_encoded_body() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"mbpkgid":42,"status":"queued","build":7}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let build = client
            .buy_build_dedicated_server(&BuyBuildDedicatedServerRequest {
                location: 1,
                device_id: 2,
                hostname: "host.example.test".to_string(),
                profile: 3,
                ..Default::default()
            })
            .expect("buy build");
        assert_eq!(build.server_id, 42);
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("location=1"));
        assert!(body.contains("device_id=2"));
        assert!(body.contains("fqdn=host.example.test"));
        assert!(body.contains("profile=3"));
    }

    #[test]
    fn rebuild_dedicated_server_always_sends_mbpkgid() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"mbpkgid":42,"status":"queued","build":8}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .rebuild_dedicated_server(
                42,
                &RebuildDedicatedServerRequest {
                    mbpkgid: 42,
                    ..Default::default()
                },
            )
            .expect("rebuild");
        assert!(transport.paths()[0].starts_with("/api/dedicated/server/re_build/42"));
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("mbpkgid=42"));
    }

    // --- BGP sessions ---

    #[test]
    fn list_bgp_sessions_keeps_only_sessions_matching_the_packages_ips() {
        let transport = RecordedTransport::new(vec![
            RecordedTransport::json(
                200,
                r#"{"result":"success","code":200,"data":[{"id":1,"customer_peer_ip":"203.0.113.5"},{"id":2,"customer_peer_ip":"203.0.113.9"}]}"#,
            ),
            RecordedTransport::json(
                200,
                r#"{"result":"success","code":200,"data":{"IPv4":[{"id":1,"ip":"203.0.113.5"}],"IPv6":[]}}"#,
            ),
            RecordedTransport::json(
                200,
                r#"{"result":"success","code":200,"data":{"id":1,"customer_peer_ip":"203.0.113.5"}}"#,
            ),
        ]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let sessions = client.list_bgp_sessions(99).expect("sessions");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, 1);
        assert!(transport.paths()[1].starts_with("/api/cloud/networkips/99"));
    }

    #[test]
    fn create_bgp_sessions_sets_ipv6_and_redundant_flags_only_when_true() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"id":1,"customer_peer_ip":"203.0.113.5"}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        client
            .create_bgp_sessions(99, 4, true, false)
            .expect("create sessions");
        let requests = transport.requests.lock().unwrap();
        let body = String::from_utf8(requests[0].body.clone().unwrap()).unwrap();
        assert!(body.contains("mbpkgid=99"));
        assert!(body.contains("group_id=4"));
        assert!(body.contains("ipv6=1"));
        assert!(!body.contains("redundant"));
    }

    #[test]
    fn delete_bgp_session_treats_not_found_as_success() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        client
            .delete_bgp_session(1)
            .expect("delete already-gone session is success");
    }

    // --- Capacity ---

    #[test]
    fn get_cloud_capacity_treats_not_found_as_empty() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            404,
            r#"{"result":"error","code":404,"message":"not found","data":null}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let capacity = client.get_cloud_capacity(1, 2).expect("capacity");
        assert!(capacity.is_empty());
    }

    #[test]
    fn get_billing_packages_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"name":"vps-1","packageid":10,"billingcycle":"Monthly"}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let packages = client.get_billing_packages().expect("packages");
        assert_eq!(packages[0].package_id, 10);
        assert_eq!(packages[0].billing_cycle, "Monthly");
    }

    // --- Non-cloud packages ---

    #[test]
    fn get_colocation_packages_sorts_the_map_by_billing_package_id() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"2":{"mbpkgid":2},"1":{"mbpkgid":1}}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let packages = client.get_colocation_packages().expect("packages");
        assert_eq!(
            packages.iter().map(|pkg| pkg.mbpkgid).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    // --- Cloud packages ---

    #[test]
    fn get_packages_tolerates_quoted_and_unquoted_numbers() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"mbpkgid":"1","locked":0,"name":"a","installed":"1"},{"mbpkgid":2,"locked":"1","name":"b","installed":0}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let packages = client.get_packages().expect("packages");
        assert_eq!(packages[0].id, 1);
        assert_eq!(packages[0].installed, 1);
        assert_eq!(packages[1].id, 2);
        assert_eq!(packages[1].locked, 1);
    }

    // --- Longtail ---

    #[test]
    fn get_location_by_current_ip_keeps_the_full_response() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"ip":"203.0.113.5","location":"chi"}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let location = client
            .get_location_by_current_ip()
            .expect("location by current ip");
        assert_eq!(location.ip, "203.0.113.5");
        assert_eq!(location.location, "chi");
    }

    #[test]
    fn get_graph_sends_port_and_time_query_params() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"points":[1,2,3]}}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let graph = client.get_graph(7, "daily").expect("graph");
        assert_eq!(graph.raw["points"][0], 1);
        assert!(transport.paths()[0].contains("port=7"));
        assert!(transport.paths()[0].contains("time=daily"));
    }

    // --- Locations, OS catalog and network IPs ---

    #[test]
    fn get_locations_decodes_the_list() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"name":"NYC1","iata_code":"JFK","continent":"NA","disabled":0}]}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let locations = client.get_locations().expect("locations");
        assert_eq!(locations[0].iata_code, "JFK");
        assert_eq!(locations[0].continent, "NA");
    }

    #[test]
    fn get_oss_hits_the_bare_images_endpoint() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":[{"id":1,"os":"Ubuntu 24.04","type":"linux"}]}"#,
        )]);
        let client =
            Client::with_transport("key", "https://vapi2.example.test/api/", transport.clone())
                .expect("client");
        let images = client.get_oss().expect("os catalog");
        assert_eq!(images[0].name, "Ubuntu 24.04");
        assert!(transport.paths()[0].starts_with("/api/cloud/images"));
    }

    #[test]
    fn get_ips_decodes_the_capitalized_ipv4_and_ipv6_keys() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"result":"success","code":200,"data":{"IPv4":[{"id":1,"ip":"203.0.113.5"}],"IPv6":[{"id":2,"ip":"2001:db8::1"}]}}"#,
        )]);
        let client = Client::with_transport("key", "https://vapi2.example.test/api/", transport)
            .expect("client");
        let ips = client.get_ips(1).expect("ips");
        assert_eq!(ips.ipv4[0].ip, "203.0.113.5");
        assert_eq!(ips.ipv6[0].ip, "2001:db8::1");
    }

    // --- VPC nameservers and account limits (vAPI3) ---

    #[test]
    fn get_vpc_nameservers_reads_the_dhcp_block_from_the_vpc() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"vpcId":1,"metadata":{},"dhcp":{"nameservers":{"ipv4":["198.51.100.1"],"ipv6":[]}}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let nameservers = client.get_vpc_nameservers(1).expect("nameservers");
        assert_eq!(nameservers.ipv4[0].server, "198.51.100.1");
        assert!(nameservers.ipv6.is_empty());
    }

    #[test]
    fn get_vpc_nameservers_returns_empty_when_the_vpc_has_no_dhcp_block() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"vpcId":1,"metadata":{}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let nameservers = client.get_vpc_nameservers(1).expect("nameservers");
        assert!(nameservers.ipv4.is_empty());
        assert!(nameservers.ipv6.is_empty());
    }

    #[test]
    fn replace_vpc_nameservers_tolerates_an_empty_response_body() {
        let transport = RecordedTransport::new(vec![Response {
            status: 204,
            body: Vec::new(),
        }]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let response = client
            .replace_vpc_nameservers(
                1,
                &ReplaceVpcNameserversRequest {
                    nameservers: vec![VpcNameserver {
                        server: "198.51.100.1".to_string(),
                    }],
                },
            )
            .expect("replace nameservers");
        assert!(response.nameservers.is_empty());
    }

    #[test]
    fn get_account_limits_decodes_a_map_of_limits() {
        let transport = RecordedTransport::new(vec![RecordedTransport::json(
            200,
            r#"{"code":200,"data":{"vms":{"used":3,"max":10,"allowedPlans":["standard"]}}}"#,
        )]);
        let client = V3Client::with_transport("key", "https://vapi3.example.test", transport)
            .expect("client");
        let limits = client.get_account_limits().expect("account limits");
        let vms = limits.get("vms").expect("vms limit");
        assert_eq!(vms.used, 3);
        assert_eq!(vms.max, 10);
        assert_eq!(vms.allowed_plans, vec!["standard".to_string()]);
    }
}
