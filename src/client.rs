use crate::error::{Error, Result};
use crate::models::{
    decode_platform_status, decode_required, AccessControlSubnet, AccountAgreement, BgpAsn,
    BgpDashboard, BgpGroup, BgpGroupFirewallSetBinding, BgpPrefix, BgpSession, BgpSummary,
    BillingPackage, BootProfile, CancelPackageRequest, CloudCapacity, CloudLocation, CloudPool,
    ColocationPackage, ColocationService, ContractUsage, CreateAccessControlSubnetRequest,
    Datacenter, DdosAttack, DdosDashboard, DdosDashboardOptions, DdosRule, DedicatedCapacity,
    DedicatedDevice, DedicatedDiskLayout, DedicatedLocation, DedicatedOsProfile, DedicatedPlan,
    DedicatedPowerStatus, DedicatedRescueOs, DedicatedServer, DedicatedServerBuild,
    DedicatedServerBuildStatus, DeleteServerResponse, DnsRecord, DnsZone, FirewallExternalIpSet,
    FirewallManageEnabled, FirewallMatchCriteria, FirewallRule, FirewallSet, FirewallSetVm, Graph,
    Image, ImageQueueStatus, IpTransitIpAddress, IpTransitPort, IpTransitService, JobStatus,
    Kernel, Location, LocationByCurrentIp, NetworkIps, Package, PlatformChangeLogEntry,
    PlatformEvents, PlatformLookingGlassInit, PlatformLookingGlassResult, PlatformMaintenanceInfo,
    PlatformStatusService, SecretList, SecretListValue, Server, ServerBuild, ServerBuildStatus,
    ServerDisk, ServerIpAddress, ServerNic, ServerStatus, Service, Size, SshKey, Tag, TagLog,
    TagResource, Ticket, TicketAttachment, TicketDepartment, TicketReply, TransitPackage,
    TransportPort, TransportService, UpdateAccessControlSubnetRequest, Vlan,
};
#[cfg(feature = "blocking")]
use crate::transport::ReqwestTransport;
use crate::transport::{api_key_from_env, build_url, redact_url, DynTransport, Method, Request};
use crate::v3::wait_for_ready;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

const BASE_ENDPOINT: &str = "https://vapi2.netactuate.com/api/";

const IMAGE_QUEUE_POLL_INTERVAL: Duration = Duration::from_secs(3);
const IMAGE_QUEUE_TIMEOUT: Duration = Duration::from_secs(1800);

/// vAPI2 client.
#[derive(Clone)]
pub struct Client {
    base_url: Url,
    api_key: String,
    transport: DynTransport,
}

impl Client {
    /// Creates a production vAPI2 client with an explicit API key.
    #[cfg(feature = "blocking")]
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_base_url(api_key, "")
    }

    /// Creates a production vAPI2 client from `NETACTUATE_API_KEY`.
    #[cfg(feature = "blocking")]
    pub fn from_env() -> Result<Self> {
        Self::new(api_key_from_env()?)
    }

    /// Creates a vAPI2 client with an explicit base URL. An empty base URL selects production.
    #[cfg(feature = "blocking")]
    pub fn with_base_url(api_key: impl Into<String>, base_url: &str) -> Result<Self> {
        let base = if base_url.is_empty() {
            BASE_ENDPOINT
        } else {
            base_url
        };
        Self::with_transport(api_key, base, Arc::new(ReqwestTransport::new()))
    }

    /// Creates a vAPI2 client with a caller-supplied transport.
    pub fn with_transport(
        api_key: impl Into<String>,
        base_url: &str,
        transport: DynTransport,
    ) -> Result<Self> {
        let base = if base_url.is_empty() {
            BASE_ENDPOINT
        } else {
            base_url
        };
        Ok(Self {
            base_url: Url::parse(base)?,
            api_key: api_key.into(),
            transport,
        })
    }

    /// Returns all cloud servers visible to the account.
    pub fn list_servers(&self) -> Result<Vec<Server>> {
        self.request_json(Method::Get, "cloud/servers", None, None)
    }

    /// Returns one cloud server by package id.
    pub fn get_server(&self, id: i64) -> Result<Server> {
        self.request_json(Method::Get, &format!("cloud/server/{id}"), None, None)
    }

    /// Buys and builds a new cloud server.
    pub fn create_server(&self, request: &CreateServerRequest) -> Result<ServerBuild> {
        let body = request.to_form();
        self.request_json(
            Method::Post,
            "cloud/server/buy_build",
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Rebuilds an existing cloud server.
    pub fn build_server(&self, id: i64, request: &BuildServerRequest) -> Result<ServerBuild> {
        let body = request.to_form();
        self.request_json(
            Method::Post,
            &format!("cloud/server/build/{id}"),
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Deletes a cloud server.
    pub fn delete_server(&self, id: i64, options: DeleteServerOptions) -> Result<()> {
        let body = options.to_form();
        self.request_empty(
            Method::Post,
            &format!("cloud/server/{id}/delete"),
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Unlinks a server's billing package from its location, freeing the location for reuse.
    pub fn unlink_server(&self, id: i64) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("cloud/server/{id}/unlink"),
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Boots a server.
    pub fn start_server(&self, id: i64) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("cloud/server/{id}/start"),
            Some(b"null".to_vec()),
            Some("application/json"),
        )
    }

    /// Shuts down a server.
    pub fn stop_server(&self, id: i64) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("cloud/server/{id}/shutdown"),
            Some(b"null".to_vec()),
            Some("application/json"),
        )
    }

    /// Scales a server to a different package, returning the id of the queued job. Poll it
    /// with [`Client::get_job_status`] using the command `scale_vm`.
    pub fn scale_server(&self, id: i64, request: &ScaleServerRequest) -> Result<i64> {
        self.request_json(
            Method::Post,
            &format!("cloud/scale/{id}"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Gets the status of a queued job by its command name and job id.
    pub fn get_job_status(&self, command: &str, job_id: i64) -> Result<JobStatus> {
        let path = format!("cloud/jobs/{}/{job_id}", encode(command));
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists DNS zones of a type such as `master`.
    pub fn list_zones(&self, zone_type: &str) -> Result<Vec<DnsZone>> {
        let path = format!("dns/zones?type={}", encode(zone_type));
        self.request_json(Method::Get, &path, None, None)
    }

    /// Gets a DNS zone by id.
    pub fn get_zone(&self, id: i64) -> Result<DnsZone> {
        self.request_json(Method::Get, &format!("dns/zone/{id}"), None, None)
    }

    /// Creates a DNS zone.
    pub fn create_zone(&self, request: &CreateDnsZoneRequest) -> Result<DnsZone> {
        let body = request.to_form();
        self.request_json(
            Method::Post,
            "dns/zone",
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Deletes a DNS zone. A zone that is already gone is treated as success.
    pub fn delete_zone(&self, id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("dns/zone/{id}"), None, None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists DNS records for a zone.
    pub fn list_records(&self, zone_id: i64) -> Result<Vec<DnsRecord>> {
        self.request_json(Method::Get, &format!("dns/records/{zone_id}"), None, None)
    }

    /// Gets a DNS record by id.
    pub fn get_record(&self, id: i64) -> Result<DnsRecord> {
        self.request_json(Method::Get, &format!("dns/record/{id}"), None, None)
    }

    /// Creates a DNS record.
    pub fn create_record(&self, request: &CreateDnsRecordRequest) -> Result<DnsRecord> {
        let body = request.to_form();
        self.request_json(
            Method::Post,
            "dns/record",
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Updates a DNS record.
    pub fn update_record(&self, request: &UpdateDnsRecordRequest) -> Result<DnsRecord> {
        let body = request.to_form();
        self.request_json(
            Method::Put,
            "dns/record",
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Deletes a DNS record. A record that is already gone is treated as success.
    pub fn delete_record(&self, id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("dns/record/{id}"), None, None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists the SSH keys installed on the account.
    pub fn get_ssh_keys(&self) -> Result<Vec<SshKey>> {
        self.request_json(Method::Get, "account/ssh_keys", None, None)
    }

    /// Gets one SSH key by id.
    ///
    /// A key that no longer exists does not fail the call: the API answers with a
    /// 200 and a null body, which decodes to the zero value. A real key always has
    /// a non-zero id, so a zero id after a successful call means the key is gone.
    pub fn get_ssh_key(&self, id: i64) -> Result<SshKey> {
        let path = format!("account/ssh_key/{id}");
        let key: SshKey = self.request_json(Method::Get, &path, None, None)?;
        if key.id == 0 {
            return Err(Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 200,
                api_code: 200,
                message: format!("ssh key {id} returned a null body, so it does not exist"),
            });
        }
        Ok(key)
    }

    /// Creates an SSH key from a name and its public key content.
    pub fn create_ssh_key(&self, name: &str, key: &str) -> Result<SshKey> {
        let mut pairs = Vec::new();
        push_str(&mut pairs, "ssh_key", key);
        push_str(&mut pairs, "name", name);
        let body = form_encode(pairs);
        self.request_json(
            Method::Post,
            "account/ssh_key",
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Updates an SSH key's name and content.
    pub fn update_ssh_key(&self, id: i64, name: &str, key: &str) -> Result<SshKey> {
        let body = serde_json::to_vec(&SshKeyUpdate {
            name: name.to_string(),
            ssh_key: key.to_string(),
        })?;
        self.request_json(
            Method::Patch,
            &format!("account/ssh_key/{id}"),
            Some(body),
            Some("application/json"),
        )
    }

    /// Deletes an SSH key. A key that is already gone is treated as success.
    pub fn delete_ssh_key(&self, id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("account/ssh_key/{id}"), None, None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists every tag on the account, each with its assigned resources embedded.
    pub fn get_tags(&self) -> Result<Vec<Tag>> {
        self.request_json(Method::Get, "tags", None, None)
    }

    /// Gets one tag by id.
    ///
    /// There is no single-tag endpoint, so this filters the full tag list, the same
    /// source the portal uses. Returns [`Error::NotFound`] when no tag has this id.
    pub fn get_tag(&self, id: i64) -> Result<Tag> {
        let path = format!("tags/{id}");
        self.get_tags()?
            .into_iter()
            .find(|tag| tag.id == id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!("tag {id} not found"),
            })
    }

    /// Creates a tag.
    pub fn create_tag(&self, request: &CreateTagRequest) -> Result<Tag> {
        self.request_json(
            Method::Post,
            "tags",
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Updates a tag's name, description, icon, color and dashboard/lock flags.
    pub fn update_tag(&self, id: i64, request: &UpdateTagRequest) -> Result<Tag> {
        self.request_json(
            Method::Put,
            &format!("tags/{id}"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Deletes a tag. The API refuses to delete a locked or still-assigned tag with
    /// a contract error. A tag that is already gone is treated as success.
    pub fn delete_tag(&self, id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("tags/{id}"), None, None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Attaches a tag to a resource. Repeating an existing assignment is a no-op.
    pub fn assign_tag_resource(
        &self,
        tag_id: i64,
        resource_name: &str,
        identifier: i64,
    ) -> Result<()> {
        let body = serde_json::to_vec(&TagResourceRef {
            resource_name: resource_name.to_string(),
            identifier: identifier.to_string(),
        })?;
        self.request_empty(
            Method::Post,
            &format!("tags/{tag_id}/assign-resource"),
            Some(body),
            Some("application/json"),
        )
    }

    /// Detaches a tag from a resource. This removes only the assignment; the tag
    /// itself is never deleted.
    pub fn remove_tag_resource(
        &self,
        tag_id: i64,
        resource_name: &str,
        identifier: i64,
    ) -> Result<()> {
        let body = serde_json::to_vec(&TagResourceRef {
            resource_name: resource_name.to_string(),
            identifier: identifier.to_string(),
        })?;
        self.request_empty(
            Method::Post,
            &format!("tags/{tag_id}/remove-resource"),
            Some(body),
            Some("application/json"),
        )
    }

    /// Lists the tags currently assigned to a resource.
    pub fn get_resource_tags(&self, resource_name: &str, resource_id: i64) -> Result<Vec<Tag>> {
        let path = format!("tags/resource/{}/id/{resource_id}", encode(resource_name));
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists the resources currently assigned to a tag.
    pub fn get_tag_resources(&self, tag_id: i64) -> Result<Vec<TagResource>> {
        self.request_json(Method::Get, &format!("tags/{tag_id}/resources"), None, None)
    }

    /// Lists the log entries recorded for a tag.
    pub fn get_tag_logs(&self, tag_id: i64) -> Result<Vec<TagLog>> {
        self.request_json(Method::Get, &format!("tags/{tag_id}/logs"), None, None)
    }

    /// Lists the external IP sets available to reference from firewall rules.
    pub fn get_firewall_external_ip_sets(&self) -> Result<Vec<FirewallExternalIpSet>> {
        self.request_json(Method::Get, "firewall/external-ipsets", None, None)
    }

    /// Gets one external IP set by id.
    pub fn get_firewall_external_ip_set(&self, id: i64) -> Result<FirewallExternalIpSet> {
        self.request_json(
            Method::Get,
            &format!("firewall/external-ipsets/{id}"),
            None,
            None,
        )
    }

    /// Reports whether cloud firewall management is available to the account.
    pub fn get_firewall_manage_enabled(&self) -> Result<FirewallManageEnabled> {
        self.request_json(Method::Get, "firewall/manage/enabled", None, None)
    }

    /// Lists every firewall set on the account.
    pub fn get_firewall_sets(&self) -> Result<Vec<FirewallSet>> {
        self.request_json(Method::Get, "firewall/sets", None, None)
    }

    /// Gets one firewall set by id.
    pub fn get_firewall_set(&self, id: i64) -> Result<FirewallSet> {
        self.request_json(Method::Get, &format!("firewall/sets/{id}"), None, None)
    }

    /// Creates a firewall set.
    pub fn create_firewall_set(
        &self,
        name: &str,
        description: &str,
        enabled: bool,
    ) -> Result<FirewallSet> {
        let body = firewall_set_form(name, description, enabled);
        self.request_json(
            Method::Post,
            "firewall/sets",
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Updates a firewall set's name, description and enabled flag.
    pub fn update_firewall_set(
        &self,
        id: i64,
        name: &str,
        description: &str,
        enabled: bool,
    ) -> Result<FirewallSet> {
        let body = firewall_set_form(name, description, enabled);
        self.request_json(
            Method::Put,
            &format!("firewall/sets/{id}"),
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Deletes a firewall set. A set that is already gone is treated as success.
    pub fn delete_firewall_set(&self, id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("firewall/sets/{id}"), None, None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Enables a firewall set.
    pub fn enable_firewall_set(&self, id: i64) -> Result<()> {
        self.request_empty(
            Method::Put,
            &format!("firewall/sets/{id}/enable"),
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Disables a firewall set.
    pub fn disable_firewall_set(&self, id: i64) -> Result<()> {
        self.request_empty(
            Method::Put,
            &format!("firewall/sets/{id}/disable"),
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Creates an editable draft copy of a published firewall set.
    pub fn create_draft_firewall_set(&self, id: i64) -> Result<FirewallSet> {
        self.request_json(
            Method::Post,
            &format!("firewall/sets/{id}/create-draft"),
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Publishes a draft firewall set, replacing the set it was drafted from.
    pub fn publish_draft_firewall_set(&self, draft_id: i64) -> Result<FirewallSet> {
        self.request_json(
            Method::Post,
            &format!("firewall/sets/publish-draft/{draft_id}"),
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Deletes a draft firewall set without publishing it. A draft that is already gone is
    /// treated as success.
    pub fn delete_draft_firewall_set(&self, draft_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("firewall/sets/delete-draft/{draft_id}"),
            None,
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Synchronizes a firewall set's rules to every VM it is attached to.
    pub fn sync_firewall_set_rules(&self, set_id: i64) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("firewall/sets/{set_id}/vm/sync-all"),
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Lists the rules belonging to a firewall set.
    pub fn get_firewall_rules(&self, set_id: i64) -> Result<Vec<FirewallRule>> {
        self.request_json(
            Method::Get,
            &format!("firewall/sets/{set_id}/rules"),
            None,
            None,
        )
    }

    /// Moves a rule within a draft firewall set, placing it after or before another rule.
    pub fn reorder_firewall_rules(
        &self,
        set_id: i64,
        request: &ReorderFirewallRulesRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("firewall/sets/{set_id}/rules/re-order"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Gets one rule from a firewall set by id.
    pub fn get_firewall_rule(&self, set_id: i64, rule_id: i64) -> Result<FirewallRule> {
        self.request_json(
            Method::Get,
            &format!("firewall/sets/{set_id}/rules/{rule_id}"),
            None,
            None,
        )
    }

    /// Adds a rule to a firewall set.
    pub fn create_firewall_rule(
        &self,
        set_id: i64,
        request: &CreateFirewallRuleRequest,
    ) -> Result<FirewallRule> {
        self.request_json(
            Method::Post,
            &format!("firewall/sets/{set_id}/rules"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Updates a rule on a firewall set.
    ///
    /// The endpoint path mirrors gona exactly (`firewall/{set_id}/{rule_id}`, without the
    /// `sets`/`rules` segments every other rule endpoint uses); this is a platform quirk, not
    /// a transcription error.
    pub fn update_firewall_rule(
        &self,
        set_id: i64,
        rule_id: i64,
        request: &CreateFirewallRuleRequest,
    ) -> Result<FirewallRule> {
        self.request_json(
            Method::Put,
            &format!("firewall/{set_id}/{rule_id}"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Deletes a rule from a firewall set. A rule that is already gone is treated as success.
    ///
    /// The endpoint path mirrors gona exactly (`firewall/{set_id}/rules/{rule_id}`, without the
    /// `sets` segment every other rule endpoint uses); this is a platform quirk, not a
    /// transcription error.
    pub fn delete_firewall_rule(&self, set_id: i64, rule_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("firewall/{set_id}/rules/{rule_id}"),
            None,
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists the VMs attached to a firewall set.
    pub fn get_firewall_set_vms(&self, set_id: i64) -> Result<Vec<FirewallSetVm>> {
        self.request_json(
            Method::Get,
            &format!("firewall/sets/{set_id}/vm-list"),
            None,
            None,
        )
    }

    /// Lists the VMs that can be attached to a firewall set.
    pub fn get_firewall_set_available_vms(
        &self,
        set_id: i64,
        options: &FirewallAvailableVmOptions,
    ) -> Result<Vec<FirewallSetVm>> {
        let mut pairs = Vec::new();
        push_query_opt_i64(&mut pairs, "extref_acct_id", options.extref_account_id);
        push_query_opt_i64(&mut pairs, "vpc_id", options.vpc_id);
        push_query_opt_bool(&mut pairs, "bw", options.include_bandwidth);
        push_query_opt_bool(&mut pairs, "ul", options.include_ul);
        push_query_opt_bool(&mut pairs, "check_vpc", options.check_vpc);
        push_query_opt_bool(
            &mut pairs,
            "disable_interface_id_filter",
            options.disable_interface_id_filter,
        );

        let path = query_path(&format!("firewall/sets/{set_id}/available-vm-list"), pairs);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists the firewall set attachments for a VM, identified by its package id.
    pub fn get_firewall_set_related_vms(
        &self,
        mbpkgid: i64,
        options: &FirewallRelatedSetOptions,
    ) -> Result<Vec<FirewallSetVm>> {
        let mut pairs = Vec::new();
        push_query_opt_bool(
            &mut pairs,
            "disable_interface_id_filter",
            options.disable_interface_id_filter,
        );

        let path = query_path(&format!("firewall/sets/vm/{mbpkgid}/related"), pairs);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Attaches a VM's network interface to a firewall set.
    pub fn attach_firewall_set_vm(
        &self,
        set_id: i64,
        mbpkgid: i64,
        interface_id: i64,
        set_priority: i64,
    ) -> Result<Vec<FirewallSetVm>> {
        let body = serde_json::to_vec(&AttachFirewallSetVmRequest {
            vm_list: vec![AttachFirewallSetVmEntry {
                mbpkgid,
                interface_id,
                set_priority,
            }],
        })?;
        self.request_json(
            Method::Post,
            &format!("firewall/sets/{set_id}/vm/attach"),
            Some(body),
            Some("application/json"),
        )
    }

    /// Detaches a VM from a firewall set.
    pub fn detach_firewall_set_vm(&self, set_id: i64, mbpkgid: i64) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("firewall/sets/{set_id}/vm/detach/{mbpkgid}"),
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Detaches a VM from a firewall set by attachment relation id.
    pub fn detach_firewall_set_vm_relation(&self, relation_id: i64) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("firewall/sets/vm/detach/{relation_id}"),
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Detaches every VM from a firewall set.
    pub fn detach_all_firewall_set_vms(&self, set_id: i64) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("firewall/sets/{set_id}/vm/detach-all"),
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Lists every VLAN provisioned on the account.
    pub fn get_vlans(&self) -> Result<Vec<Vlan>> {
        self.request_json(Method::Get, "cloud/networking/vlans", None, None)
    }

    /// Gets one customer VLAN by id.
    pub fn get_customer_vlan(&self, customer_vlan_id: i64) -> Result<Vlan> {
        self.request_json(
            Method::Get,
            &format!("cloud/networking/vlans/{customer_vlan_id}"),
            None,
            None,
        )
    }

    /// Lists the customer VLANs available at a location.
    pub fn list_customer_vlans_at_location(&self, location_id: i64) -> Result<Vec<Vlan>> {
        self.request_json(
            Method::Get,
            &format!("cloud/networking/locations/{location_id}/vlans"),
            None,
            None,
        )
    }

    /// Lists the network interfaces attached to a server.
    pub fn get_server_nics(&self, mbpkg_id: i64) -> Result<Vec<ServerNic>> {
        self.request_json(
            Method::Get,
            &format!("cloud/networking/nics/{mbpkg_id}"),
            None,
            None,
        )
    }

    /// Attaches a new network interface to a server on the given customer VLAN.
    pub fn attach_server_nic(
        &self,
        mbpkg_id: i64,
        request: &ServerNicAttachRequest,
    ) -> Result<ServerNic> {
        let payload = self.request_payload(
            Method::Post,
            &format!("cloud/networking/nics/{mbpkg_id}"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )?;
        server_nic_from_payload(&payload)
    }

    /// Updates a network interface's VLAN and attachment order.
    pub fn update_server_nic(
        &self,
        nic_id: i64,
        request: &ServerNicUpdateRequest,
    ) -> Result<ServerNic> {
        let payload = self.request_payload(
            Method::Put,
            &format!("cloud/networking/nics/{nic_id}"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )?;
        server_nic_from_payload(&payload)
    }

    /// Detaches a network interface from a server. An interface that is already detached is
    /// treated as success.
    pub fn detach_server_nic(&self, mbpkg_id: i64, nic_id: i64) -> Result<()> {
        let path = format!("cloud/networking/nics/{nic_id}?mbpkgid={mbpkg_id}");
        match self.request_empty(Method::Delete, &path, None, None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists every service on the account.
    pub fn get_services(&self) -> Result<Vec<Service>> {
        self.request_json(Method::Get, "services", None, None)
    }

    /// Lists colocation services, optionally filtered to one owning service id.
    pub fn get_colocation_services(
        &self,
        service_id: Option<i64>,
    ) -> Result<Vec<ColocationService>> {
        let path = service_list_path("services/colocation", "service_id", service_id);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Gets one colocation service by id.
    pub fn get_colocation_service(&self, id: i64) -> Result<ColocationService> {
        self.request_json(
            Method::Get,
            &format!("services/colocation/{id}"),
            None,
            None,
        )
    }

    /// Lists IP transit services, optionally filtered to one owning service id.
    pub fn get_ip_transit_services(
        &self,
        service_id: Option<i64>,
    ) -> Result<Vec<IpTransitService>> {
        let path = service_list_path("services/iptransit", "service_id", service_id);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Gets one IP transit service by id.
    pub fn get_ip_transit_service(&self, id: i64) -> Result<IpTransitService> {
        self.request_json(Method::Get, &format!("services/iptransit/{id}"), None, None)
    }

    /// Lists IP addresses, optionally filtered to one owning IP transit service id.
    pub fn get_ip_transit_ip_addresses(
        &self,
        service_iptransit_id: Option<i64>,
    ) -> Result<Vec<IpTransitIpAddress>> {
        let path = service_list_path(
            "services/iptransit/ips",
            "service_iptransit_id",
            service_iptransit_id,
        );
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists ports, optionally filtered to one owning IP transit service id.
    pub fn get_ip_transit_ports(
        &self,
        service_iptransit_id: Option<i64>,
    ) -> Result<Vec<IpTransitPort>> {
        let path = service_list_path(
            "services/iptransit/ports",
            "service_iptransit_id",
            service_iptransit_id,
        );
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists transport services, optionally filtered to one owning service id.
    pub fn get_transport_services(&self, service_id: Option<i64>) -> Result<Vec<TransportService>> {
        let path = service_list_path("services/transport", "service_id", service_id);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Gets one transport service by id.
    pub fn get_transport_service(&self, id: i64) -> Result<TransportService> {
        self.request_json(Method::Get, &format!("services/transport/{id}"), None, None)
    }

    /// Lists ports, optionally filtered to one owning transport service id.
    pub fn get_transport_ports(
        &self,
        service_transport_id: Option<i64>,
    ) -> Result<Vec<TransportPort>> {
        let path = service_list_path(
            "services/transport/ports",
            "service_transport_id",
            service_transport_id,
        );
        self.request_json(Method::Get, &path, None, None)
    }

    /// Creates an account BGP group.
    pub fn create_bgp_group(&self, request: &CreateBgpGroupRequest) -> Result<BgpGroup> {
        self.request_json(
            Method::Post,
            "bgp/bgpgroup",
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Gets an account BGP group by id.
    pub fn get_bgp_group(&self, id: i64) -> Result<BgpGroup> {
        self.request_json(Method::Get, &format!("bgp/bgpgroup/{id}"), None, None)
    }

    /// Lists account BGP groups, optionally filtered to one group type. An empty `group_type`
    /// returns every group.
    ///
    /// A filter that matches nothing answers with a 404 rather than an empty list; that is
    /// treated as zero groups rather than an error.
    pub fn list_bgp_groups(&self, group_type: &str) -> Result<Vec<BgpGroup>> {
        let path = bgp_group_type_path("bgp/bgpgroups", group_type);
        self.list_or_empty(&path)
    }

    /// Purchases anycast BGP prefixes for the account.
    pub fn buy_bgp_prefixes(&self, request: &BuyBgpPrefixesRequest) -> Result<BgpPrefix> {
        self.request_json(
            Method::Post,
            "bgp/bgpbuyprefixes",
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Gets an account BGP prefix by id.
    pub fn get_bgp_prefix(&self, id: i64) -> Result<BgpPrefix> {
        self.request_json(Method::Get, &format!("bgp/bgpprefix/{id}"), None, None)
    }

    /// Lists account BGP prefixes, optionally filtered to one group type. An empty `group_type`
    /// returns every prefix.
    ///
    /// A filter that matches nothing answers with a 404 rather than an empty list; that is
    /// treated as zero prefixes rather than an error.
    pub fn list_bgp_prefixes(&self, group_type: &str) -> Result<Vec<BgpPrefix>> {
        let path = bgp_group_type_path("bgp/bgpprefixes", group_type);
        self.list_or_empty(&path)
    }

    /// Lists account ASNs, optionally filtered to one group type. An empty `group_type` returns
    /// every ASN.
    ///
    /// A filter that matches nothing answers with a 404 rather than an empty list; that is
    /// treated as zero ASNs rather than an error.
    pub fn list_bgp_asns(&self, group_type: &str) -> Result<Vec<BgpAsn>> {
        let path = bgp_group_type_path("bgp/bgpasns", group_type);
        self.list_or_empty(&path)
    }

    /// Gets an account ASN by id.
    ///
    /// The platform answers this lookup as a query parameter rather than a path segment.
    pub fn get_bgp_asn(&self, id: i64) -> Result<BgpAsn> {
        self.request_json(Method::Get, &format!("bgp/bgpasn?id={id}"), None, None)
    }

    /// Lists the legal agreements available to the account.
    pub fn list_account_agreements(&self) -> Result<Vec<AccountAgreement>> {
        self.request_json(Method::Get, "account/agreements", None, None)
    }

    /// Binds a firewall set to an interface of a BGP group.
    pub fn bind_bgp_group_firewall_set(
        &self,
        group_id: i64,
        request: &BindBgpGroupFirewallSetRequest,
    ) -> Result<BgpGroupFirewallSetBinding> {
        let path = format!("bgp/bgp-groups/{group_id}/firewall-sets");
        self.request_json(
            Method::Post,
            &path,
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Removes a firewall set binding from a BGP group. A binding that is already gone is
    /// treated as success.
    pub fn unbind_bgp_group_firewall_set(&self, group_id: i64, firewall_set_id: i64) -> Result<()> {
        let path = format!("bgp/bgp-groups/{group_id}/firewall-sets/{firewall_set_id}");
        match self.request_empty(Method::Delete, &path, None, None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Asks the platform to refresh every session in a BGP group.
    pub fn refresh_bgp_group_sessions(&self, group_id: i64) -> Result<()> {
        self.post_bgp_group_action(group_id, "refresh")
    }

    /// Asks the platform to start every session in a BGP group.
    pub fn start_bgp_group_sessions(&self, group_id: i64) -> Result<()> {
        self.post_bgp_group_action(group_id, "start")
    }

    /// Asks the platform to stop every session in a BGP group.
    pub fn stop_bgp_group_sessions(&self, group_id: i64) -> Result<()> {
        self.post_bgp_group_action(group_id, "stop")
    }

    /// Asks the platform to refresh a BGP session.
    pub fn refresh_bgp_session(&self, session_id: i64) -> Result<()> {
        self.post_bgp_session_action(session_id, "refresh")
    }

    /// Asks the platform to start a BGP session.
    pub fn start_bgp_session(&self, session_id: i64) -> Result<()> {
        self.post_bgp_session_action(session_id, "start")
    }

    /// Asks the platform to stop a BGP session.
    pub fn stop_bgp_session(&self, session_id: i64) -> Result<()> {
        self.post_bgp_session_action(session_id, "stop")
    }

    /// Returns the account BGP summary.
    pub fn get_bgp_summary(&self) -> Result<BgpSummary> {
        self.request_json(Method::Get, "bgp/bgpsummary", None, None)
    }

    /// Returns BGP dashboard data, optionally filtered by group type and flap window.
    pub fn get_bgp_dashboard(&self, options: &BgpDashboardOptions) -> Result<BgpDashboard> {
        let mut pairs = Vec::new();
        push_query_opt_str(
            &mut pairs,
            "group_type",
            (!options.group_type.is_empty()).then_some(options.group_type.as_str()),
        );
        push_query_opt_i64(&mut pairs, "flap_window", options.flap_window);
        let path = query_path("bgp/dashboard", pairs);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Returns platform status for every monitored service, grouped by service and sorted by
    /// service name and location code.
    pub fn get_platform_status(&self) -> Result<Vec<PlatformStatusService>> {
        let payload = self.request_payload(Method::Get, "platform/status", None, None)?;
        decode_platform_status(&payload)
    }

    /// Lists every platform change log entry.
    pub fn get_platform_change_log(&self) -> Result<Vec<PlatformChangeLogEntry>> {
        self.request_json(Method::Get, "platform/change-log", None, None)
    }

    /// Gets one platform change log entry by id.
    pub fn get_platform_change_log_entry(&self, id: i64) -> Result<PlatformChangeLogEntry> {
        self.request_json(
            Method::Get,
            &format!("platform/change-log/{id}"),
            None,
            None,
        )
    }

    /// Lists the datacenters at a platform location.
    pub fn get_platform_datacenters(&self, location: &str) -> Result<Vec<Datacenter>> {
        let path = format!("platform/datacenters/{}", encode(location));
        self.request_json(Method::Get, &path, None, None)
    }

    /// Returns the datacenter id for an IATA airport code.
    pub fn get_datacenter_by_iata(&self, iata: &str) -> Result<i64> {
        let path = format!("platform/datacenters-by-iata/{}", encode(iata));
        let datacenter: Datacenter = self.request_json(Method::Get, &path, None, None)?;
        Ok(datacenter.id)
    }

    /// Returns the options and targets available for a looking glass query.
    pub fn get_platform_looking_glass_init(&self) -> Result<PlatformLookingGlassInit> {
        self.request_json(Method::Get, "platform/looking-glass/init", None, None)
    }

    /// Runs a looking glass action such as a ping or traceroute from a platform location.
    pub fn execute_platform_looking_glass(
        &self,
        options: &PlatformLookingGlassExecuteOptions,
    ) -> Result<PlatformLookingGlassResult> {
        let mut pairs = Vec::new();
        push_query_opt_str(&mut pairs, "action", options.action.as_deref());
        push_query_opt_str(&mut pairs, "target", options.target.as_deref());
        push_query_opt_str(&mut pairs, "location", options.location.as_deref());
        push_query_opt_i64(&mut pairs, "full", options.full);

        let path = query_path("platform/looking-glass/execute", pairs);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Gets detail for one platform maintenance record by id.
    pub fn get_platform_maintenance_info(&self, id: i64) -> Result<PlatformMaintenanceInfo> {
        self.request_json(
            Method::Get,
            &format!("platform/maintenance-info/{id}"),
            None,
            None,
        )
    }

    /// Lists active and upcoming incidents at a platform location.
    pub fn get_platform_incidents(&self, location: &str) -> Result<PlatformEvents> {
        let path = format!("platform/incidents/{}", encode(location));
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists historic incidents at a platform location.
    pub fn get_platform_incident_history(&self, location: &str) -> Result<PlatformEvents> {
        let path = format!("platform/incidents/history/{}", encode(location));
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists active and upcoming maintenance at a platform location.
    pub fn get_platform_maintenance(&self, location: &str) -> Result<PlatformEvents> {
        let path = format!("platform/maintenance/{}", encode(location));
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists historic maintenance at a platform location.
    pub fn get_platform_maintenance_history(&self, location: &str) -> Result<PlatformEvents> {
        let path = format!("platform/maintenance/history/{}", encode(location));
        self.request_json(Method::Get, &path, None, None)
    }

    /// Returns archived legacy support tickets.
    pub fn get_legacy_tickets(&self) -> Result<Vec<Ticket>> {
        self.request_json(Method::Get, "support/legacy-tickets", None, None)
    }

    /// Lists support tickets, optionally filtered by open state and whether to include stats.
    pub fn get_tickets(&self, options: &TicketListOptions) -> Result<Vec<Ticket>> {
        let path = ticket_list_path("support/tickets", options);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Opens a support ticket.
    pub fn create_ticket(&self, request: &CreateTicketRequest) -> Result<Ticket> {
        self.request_json(
            Method::Post,
            "support/tickets",
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Lists legacy support tickets, optionally filtered by open state and whether to include
    /// stats.
    pub fn get_old_tickets(&self, options: &TicketListOptions) -> Result<Vec<Ticket>> {
        let path = ticket_list_path("support/tickets-old", options);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Gets a legacy support ticket by id.
    pub fn get_old_ticket(&self, id: &str) -> Result<Ticket> {
        let path = format!("support/tickets-old/{}", encode(id));
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists the departments a support ticket can be opened against.
    pub fn get_ticket_departments(&self) -> Result<Vec<TicketDepartment>> {
        self.request_json(Method::Get, "support/tickets/departments", None, None)
    }

    /// Gets a support ticket by id.
    pub fn get_ticket(&self, id: &str) -> Result<Ticket> {
        let path = format!("support/tickets/{}", encode(id));
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists the replies posted to a support ticket.
    pub fn get_ticket_replies(&self, id: &str) -> Result<Vec<TicketReply>> {
        let path = format!("support/tickets/{}/replies", encode(id));
        self.request_json(Method::Get, &path, None, None)
    }

    /// Gets metadata for one ticket or reply attachment.
    ///
    /// `without_data` asks the platform to omit the base64-encoded content from the response
    /// when set to `1`.
    pub fn get_ticket_attachment(
        &self,
        id: &str,
        attachment_type: &str,
        rel_id: &str,
        index: i64,
        without_data: Option<i64>,
    ) -> Result<TicketAttachment> {
        let mut path = ticket_attachment_path(id, attachment_type, rel_id, index);
        if let Some(without_data) = without_data {
            path = format!("{path}?without_data={without_data}");
        }
        self.request_json(Method::Get, &path, None, None)
    }

    /// Downloads the binary content of a ticket or reply attachment.
    pub fn download_ticket_attachment(
        &self,
        id: &str,
        attachment_type: &str,
        rel_id: &str,
        index: i64,
    ) -> Result<Vec<u8>> {
        let path = format!(
            "{}/download",
            ticket_attachment_path(id, attachment_type, rel_id, index)
        );
        self.request_raw(Method::Get, &path)
    }

    /// Returns inline-preview content for a ticket or reply attachment.
    pub fn preview_ticket_attachment(
        &self,
        id: &str,
        attachment_type: &str,
        rel_id: &str,
        index: i64,
    ) -> Result<Vec<u8>> {
        let path = format!(
            "{}/preview",
            ticket_attachment_path(id, attachment_type, rel_id, index)
        );
        self.request_raw(Method::Get, &path)
    }

    /// Closes a support ticket.
    pub fn close_ticket(&self, id: &str) -> Result<()> {
        let path = format!("support/tickets/{}/close", encode(id));
        self.request_empty(
            Method::Post,
            &path,
            Some(Vec::new()),
            Some("application/json"),
        )
    }

    /// Closes a support ticket through the alternate close path.
    pub fn close_ticket_alias(&self, id: &str) -> Result<()> {
        let path = format!("support/tickets/close/{}", encode(id));
        self.request_empty(
            Method::Post,
            &path,
            Some(Vec::new()),
            Some("application/json"),
        )
    }

    /// Replies to a support ticket.
    pub fn reply_to_ticket(&self, id: &str, request: &ReplyTicketRequest) -> Result<TicketReply> {
        let path = format!("support/tickets/{}/reply", encode(id));
        self.request_json(
            Method::Post,
            &path,
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Replies to a support ticket through the alternate reply path.
    pub fn reply_to_ticket_alias(
        &self,
        id: &str,
        request: &ReplyTicketRequest,
    ) -> Result<TicketReply> {
        let path = format!("support/tickets/reply/{}", encode(id));
        self.request_json(
            Method::Post,
            &path,
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Lists every secret list on the account.
    pub fn get_secret_lists(&self) -> Result<Vec<SecretList>> {
        self.request_json(Method::Get, "secrets/lists", None, None)
    }

    /// Gets one secret list by id.
    pub fn get_secret_list(&self, id: i64) -> Result<SecretList> {
        self.request_json(Method::Get, &format!("secrets/lists/{id}"), None, None)
    }

    /// Creates a secret list.
    pub fn create_secret_list(&self, name: &str) -> Result<SecretList> {
        let body = form_encode(vec![("name".to_string(), name.to_string())]);
        self.request_json(
            Method::Post,
            "secrets/lists",
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Renames a secret list.
    pub fn update_secret_list(&self, id: i64, name: &str) -> Result<SecretList> {
        let body = form_encode(vec![("name".to_string(), name.to_string())]);
        self.request_json(
            Method::Post,
            &format!("secrets/lists/{id}"),
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Deletes a secret list. A list that is already gone is treated as success.
    pub fn delete_secret_list(&self, id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("secrets/lists/{id}"), None, None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists the values stored in a secret list.
    pub fn get_secret_list_values(&self, list_id: i64) -> Result<Vec<SecretListValue>> {
        let path = format!("secrets/lists/{list_id}/values");
        self.request_json(Method::Get, &path, None, None)
    }

    /// Returns every secret value visible to the account, across every list.
    pub fn get_all_secret_values(&self) -> Result<Vec<SecretListValue>> {
        self.request_json(Method::Get, "secrets/all-values", None, None)
    }

    /// Gets one secret value by id.
    pub fn get_secret_list_value(&self, list_id: i64, value_id: i64) -> Result<SecretListValue> {
        let path = format!("secrets/lists/{list_id}/values/{value_id}");
        self.request_json(Method::Get, &path, None, None)
    }

    /// Adds a key/value pair to a secret list.
    pub fn create_secret_list_value(
        &self,
        list_id: i64,
        key: &str,
        value: &str,
    ) -> Result<SecretListValue> {
        let body = secret_value_form(key, value);
        self.request_json(
            Method::Post,
            &format!("secrets/lists/{list_id}/values"),
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Updates a secret value's key and value.
    pub fn update_secret_list_value(
        &self,
        list_id: i64,
        value_id: i64,
        key: &str,
        value: &str,
    ) -> Result<SecretListValue> {
        let body = secret_value_form(key, value);
        self.request_json(
            Method::Post,
            &format!("secrets/lists/{list_id}/values/{value_id}"),
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Deletes a secret value. A value that is already gone is treated as success.
    pub fn delete_secret_list_value(&self, list_id: i64, value_id: i64) -> Result<()> {
        let path = format!("secrets/lists/{list_id}/values/{value_id}");
        match self.request_empty(Method::Delete, &path, None, None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists available dedicated devices, optionally narrowed by hardware and location filters.
    pub fn filter_dedicated_devices(
        &self,
        options: &DedicatedDeviceFilterOptions,
    ) -> Result<Vec<DedicatedDevice>> {
        let mut pairs = Vec::new();
        push_query_opt_i64(&mut pairs, "per_page", options.per_page);
        push_query_opt_str(&mut pairs, "nic", options.nic.as_deref());
        push_query_opt_str(&mut pairs, "cpu_type", options.cpu_type.as_deref());
        push_query_opt_str(&mut pairs, "gpu_type", options.gpu_type.as_deref());
        push_query_opt_str(&mut pairs, "disk_type", options.disk_type.as_deref());
        push_query_opt_str(&mut pairs, "cores", options.cores.as_deref());
        push_query_opt_str(&mut pairs, "ram_mb", options.ram_mb.as_deref());
        push_query_opt_str(&mut pairs, "disk_mib", options.disk_mib.as_deref());
        push_query_opt_str(&mut pairs, "dc_name", options.dc_name.as_deref());
        push_query_opt_str(&mut pairs, "region_name", options.region_name.as_deref());
        let path = query_path("dedicated/filter-dedicated-devices", pairs);

        let value: Value = self.request_json(Method::Get, &path, None, None)?;
        decode_dedicated_devices(value)
    }

    /// Lists locations that support dedicated servers.
    pub fn list_dedicated_locations(&self) -> Result<Vec<DedicatedLocation>> {
        self.request_json(Method::Get, "dedicated/locations", None, None)
    }

    /// Lists the OS profiles compatible with a dedicated device.
    pub fn list_dedicated_device_os_profiles(
        &self,
        device_id: i64,
        is_buyable: Option<bool>,
    ) -> Result<Vec<DedicatedOsProfile>> {
        let mut pairs = Vec::new();
        push_query_opt_bool(&mut pairs, "is_buyable", is_buyable);
        let path = query_path(&format!("dedicated/os/device/{device_id}"), pairs);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists every OS profile available for dedicated servers, independent of device.
    pub fn get_dedicated_os_profiles(&self) -> Result<Vec<DedicatedOsProfile>> {
        self.request_json(Method::Get, "dedicated/os", None, None)
    }

    /// Lists the rescue OS images available for dedicated servers.
    pub fn get_dedicated_rescue_os_profiles(&self) -> Result<Vec<DedicatedRescueOs>> {
        self.request_json(Method::Get, "dedicated/os/rescue-system-list", None, None)
    }

    /// Lists the disk layouts compatible with a dedicated OS profile.
    pub fn get_dedicated_disk_layouts(&self, os_id: i64) -> Result<Vec<DedicatedDiskLayout>> {
        self.request_json(
            Method::Get,
            &format!("dedicated/disklayouts/{os_id}"),
            None,
            None,
        )
    }

    /// Lists dedicated server plans available at a location.
    pub fn list_dedicated_plans(&self, location_id: i64) -> Result<Vec<DedicatedPlan>> {
        self.request_json(
            Method::Get,
            &format!("dedicated/plans/{location_id}"),
            None,
            None,
        )
    }

    /// Deploys an already purchased dedicated server package.
    pub fn deploy_dedicated_server(
        &self,
        mbpkgid: i64,
        request: &DedicatedServerBuildRequest,
    ) -> Result<DedicatedServerBuild> {
        self.request_json(
            Method::Post,
            &format!("dedicated/server/build/{mbpkgid}"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Purchases a dedicated device without deploying it.
    pub fn buy_dedicated_server(&self, device_id: i64) -> Result<DedicatedServerBuild> {
        self.request_json(
            Method::Post,
            &format!("dedicated/server/buy/{device_id}"),
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Purchases a dedicated device and deploys it in one call.
    pub fn buy_and_deploy_dedicated_server(
        &self,
        device_id: i64,
        request: &DedicatedServerBuildRequest,
    ) -> Result<DedicatedServerBuild> {
        self.request_json(
            Method::Post,
            &format!("dedicated/server/buy_build/{device_id}"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Updates the reverse DNS entry for a dedicated server's IPv4 address.
    pub fn update_dedicated_server_ipv4_reverse(
        &self,
        request: &DedicatedIpv4ReverseRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Put,
            "dedicated/server/ipv4_reverse",
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Asks the platform to soft reset a dedicated server.
    pub fn soft_reset_dedicated_server(&self, mbpkgid: i64) -> Result<()> {
        self.post_dedicated_action(mbpkgid, "soft-reset", None)
    }

    /// Deletes a dedicated server package.
    pub fn delete_dedicated_server(
        &self,
        mbpkgid: i64,
        request: Option<&DedicatedServerActionRequest>,
    ) -> Result<()> {
        self.post_dedicated_action(mbpkgid, "delete", request)
    }

    /// Asks the platform to reboot a dedicated server.
    pub fn reboot_dedicated_server(
        &self,
        mbpkgid: i64,
        request: Option<&DedicatedServerActionRequest>,
    ) -> Result<()> {
        self.post_dedicated_action(mbpkgid, "reboot", request)
    }

    /// Asks the platform to shut down a dedicated server.
    pub fn shutdown_dedicated_server(
        &self,
        mbpkgid: i64,
        request: Option<&DedicatedServerActionRequest>,
    ) -> Result<()> {
        self.post_dedicated_action(mbpkgid, "shutdown", request)
    }

    /// Asks the platform to start a dedicated server.
    pub fn start_dedicated_server(
        &self,
        mbpkgid: i64,
        request: Option<&DedicatedServerActionRequest>,
    ) -> Result<()> {
        self.post_dedicated_action(mbpkgid, "start", request)
    }

    /// Gets the power status of a dedicated server.
    pub fn get_dedicated_server_power_status(
        &self,
        mbpkgid: i64,
        request: Option<&DedicatedServerActionRequest>,
    ) -> Result<DedicatedPowerStatus> {
        self.post_dedicated_action_with_response(mbpkgid, "status", request)
    }

    /// Lists dedicated servers on the account.
    pub fn list_dedicated_servers(&self) -> Result<Vec<DedicatedServer>> {
        self.request_json(Method::Get, "dedicated/servers", None, None)
    }

    /// Lists images owned by the account.
    pub fn get_my_images(&self) -> Result<Vec<Image>> {
        self.request_json(Method::Get, "cloud/images/my", None, None)
    }

    /// Gets one image by id.
    pub fn get_image(&self, id: i64) -> Result<Image> {
        self.request_json(Method::Get, &format!("cloud/images/{id}"), None, None)
    }

    /// Creates a custom image from a built server.
    ///
    /// Returns the id of the queued imaging job; poll it with
    /// [`Client::get_image_queue_status`] or [`Client::wait_for_image_queue`].
    pub fn create_image(&self, request: &CreateImageRequest) -> Result<CreateImageResponse> {
        let body = request.to_form();
        self.request_json(
            Method::Post,
            "cloud/images/create",
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Renames an image and updates its description.
    pub fn edit_image(&self, id: i64, name: &str, description: &str) -> Result<()> {
        let body = serde_json::to_vec(&ImageEdit {
            os: name.to_string(),
            description: description.to_string(),
        })?;
        self.request_empty(
            Method::Patch,
            &format!("cloud/images/{id}/edit"),
            Some(body),
            Some("application/json"),
        )
    }

    /// Deletes a custom image. Returns the id of the queued deletion job.
    pub fn delete_image(&self, id: i64) -> Result<DeleteImageResponse> {
        self.request_json(
            Method::Delete,
            &format!("cloud/images/{id}/delete"),
            None,
            None,
        )
    }

    /// Gets the status of an image build or delete job.
    pub fn get_image_queue_status(&self, queue_id: i64) -> Result<ImageQueueStatus> {
        self.request_json(
            Method::Get,
            &format!("cloud/images/queue_status/{queue_id}"),
            None,
            None,
        )
    }

    /// Polls an image job every three seconds until it completes, fails, or thirty minutes
    /// pass.
    ///
    /// Returns [`Error::Api`] if the job settles into a failed state, and [`Error::Timeout`]
    /// if it is still running when the wait gives up.
    pub fn wait_for_image_queue(&self, queue_id: i64) -> Result<ImageQueueStatus> {
        let mut last = ImageQueueStatus::default();
        wait_for_ready(IMAGE_QUEUE_POLL_INTERVAL, IMAGE_QUEUE_TIMEOUT, || {
            let status = self.get_image_queue_status(queue_id)?;
            let complete = status.status == "Complete";
            let failed = status.status == "Failed";
            last = status;
            if failed {
                return Err(image_queue_failed_error(queue_id, &last.response));
            }
            Ok(complete)
        })?;
        Ok(last)
    }

    /// Returns raw bandwidth statistics for a server's billing package. When `date` is given it
    /// is sent as the platform's `date` query parameter; the shape of the returned data depends
    /// on the range the platform chooses, so it is not modelled further.
    pub fn get_bandwidth_stats(&self, mbpkg_id: i64, date: Option<&str>) -> Result<Value> {
        let mut pairs = Vec::new();
        push_query_opt_str(&mut pairs, "date", date);
        let path = query_path(&format!("cloud/bw_stats/{mbpkg_id}"), pairs);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Returns bandwidth statistics for a server's billing package over the platform's default
    /// range.
    pub fn get_bandwidth_stats_range(&self, mbpkg_id: i64) -> Result<Value> {
        self.request_json(
            Method::Get,
            &format!("cloud/bw_stats_range/{mbpkg_id}"),
            None,
            None,
        )
    }

    /// Returns the count of image provisioning jobs currently queued for the account.
    pub fn get_images_provisioning_jobs_count(&self) -> Result<Value> {
        self.request_json(
            Method::Get,
            "cloud/images-provisioning-jobs-count",
            None,
            None,
        )
    }

    /// Returns base images available for server deployment.
    pub fn get_base_images(&self) -> Result<Vec<Image>> {
        self.request_json(Method::Get, "cloud/images/base", None, None)
    }

    /// Returns private images available to the account.
    pub fn get_private_images(&self) -> Result<Vec<Image>> {
        self.request_json(Method::Get, "cloud/images/private", None, None)
    }

    /// Replaces image `id` with the image identified by `request.replace_id`.
    pub fn replace_image(&self, id: i64, request: &ReplaceImageRequest) -> Result<Value> {
        self.request_json(
            Method::Post,
            &format!("cloud/images/{id}/replace_image"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Returns IP address limits for a server's billing package.
    pub fn get_ip_limits(&self, mbpkg_id: i64) -> Result<Value> {
        self.request_json(
            Method::Get,
            &format!("cloud/iplimits/{mbpkg_id}"),
            None,
            None,
        )
    }

    /// Returns optional extras available for a server's billing package.
    pub fn get_cloud_extras(&self, mbpkg_id: i64) -> Result<Value> {
        self.request_json(Method::Get, &format!("cloud/extras/{mbpkg_id}"), None, None)
    }

    /// Updates reverse DNS for an IPv4 address by its address id.
    pub fn update_cloud_ipv4_reverse_dns(&self, id: i64, reverse: &str) -> Result<()> {
        self.request_empty(
            Method::Put,
            &format!("cloud/ipv4/{id}"),
            Some(serde_json::to_vec(&ReverseDnsUpdate {
                reverse: reverse.to_string(),
            })?),
            Some("application/json"),
        )
    }

    /// Updates reverse DNS for an IPv6 address by its address id.
    pub fn update_cloud_ipv6_reverse_dns(&self, id: i64, reverse: &str) -> Result<()> {
        self.request_empty(
            Method::Put,
            &format!("cloud/ipv6/{id}"),
            Some(serde_json::to_vec(&ReverseDnsUpdate {
                reverse: reverse.to_string(),
            })?),
            Some("application/json"),
        )
    }

    /// Returns boot kernels available for cloud servers.
    pub fn get_kernels(&self) -> Result<Vec<Kernel>> {
        self.request_json(Method::Get, "cloud/kernels", None, None)
    }

    /// Gets a single cloud deployment location by id.
    pub fn get_cloud_location(&self, id: i64) -> Result<CloudLocation> {
        self.request_json(Method::Get, &format!("cloud/locations/{id}"), None, None)
    }

    /// Gets a single capacity pool by id.
    pub fn get_cloud_pool(&self, cloud_pool_id: i64) -> Result<CloudPool> {
        self.request_json(
            Method::Get,
            &format!("cloud/pools/{cloud_pool_id}"),
            None,
            None,
        )
    }

    /// Returns scaling options for a server's billing package, optionally filtered.
    pub fn get_scaling_options(
        &self,
        mbpkg_id: i64,
        request: &ScalingOptionsRequest,
    ) -> Result<Value> {
        let mut pairs = Vec::new();
        if let Some(include_current_plan) = request.include_current_plan {
            pairs.push((
                "include_current_plan".to_string(),
                include_current_plan.to_string(),
            ));
        }
        push_query_opt_i64(&mut pairs, "min_ram", request.min_ram);
        push_query_opt_i64(&mut pairs, "max_ram", request.max_ram);
        push_query_opt_i64(&mut pairs, "min_cpus", request.min_cpus);
        push_query_opt_i64(&mut pairs, "max_cpus", request.max_cpus);
        let path = query_path(&format!("cloud/scaling/{mbpkg_id}"), pairs);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Returns the status of a server build.
    pub fn get_server_build_status(&self, build_id: i64) -> Result<ServerBuildStatus> {
        self.request_json(
            Method::Get,
            &format!("cloud/server/build_status/{build_id}"),
            None,
            None,
        )
    }

    /// Returns deployment metadata, optionally scoped to a contract type.
    pub fn get_server_deployment_info(&self, contract_type: Option<&str>) -> Result<Value> {
        let mut pairs = Vec::new();
        push_query_opt_str(&mut pairs, "contract_type", contract_type);
        let path = query_path("cloud/server/deploy/info", pairs);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Returns VNC status for a server.
    pub fn get_server_vnc_status(&self, mbpkg_id: i64) -> Result<Value> {
        self.request_json(
            Method::Get,
            &format!("cloud/server/vnc-status/{mbpkg_id}"),
            None,
            None,
        )
    }

    /// Updates mutable options for a server.
    pub fn update_server_options(
        &self,
        mbpkg_id: i64,
        request: &UpdateServerOptionsRequest,
    ) -> Result<Value> {
        self.request_json(
            Method::Put,
            &format!("cloud/options/{mbpkg_id}"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Deletes a server with full control over billing cancellation and password handling,
    /// returning the id of the deleted billing package.
    pub fn delete_server_with_options(
        &self,
        mbpkg_id: i64,
        request: &DeleteServerRequest,
    ) -> Result<DeleteServerResponse> {
        self.request_json(
            Method::Post,
            &format!("cloud/server/{mbpkg_id}/delete"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Starts a filesystem check for a server.
    pub fn run_server_fsck(&self, mbpkg_id: i64) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("cloud/server/{mbpkg_id}/fsck"),
            None,
            None,
        )
    }

    /// Returns IPv4 addresses attached to a server.
    pub fn get_server_ipv4(&self, mbpkg_id: i64) -> Result<Vec<ServerIpAddress>> {
        self.request_json(
            Method::Get,
            &format!("cloud/server/{mbpkg_id}/ipv4"),
            None,
            None,
        )
    }

    /// Returns IPv6 addresses attached to a server.
    pub fn get_server_ipv6(&self, mbpkg_id: i64) -> Result<Vec<ServerIpAddress>> {
        self.request_json(
            Method::Get,
            &format!("cloud/server/{mbpkg_id}/ipv6"),
            None,
            None,
        )
    }

    /// Lists queued jobs for a server.
    pub fn list_server_jobs(&self, mbpkg_id: i64) -> Result<Vec<JobStatus>> {
        self.request_json(
            Method::Get,
            &format!("cloud/server/{mbpkg_id}/jobs"),
            None,
            None,
        )
    }

    /// Gets a single queued job for a server.
    pub fn get_server_job(&self, mbpkg_id: i64, job_id: i64) -> Result<JobStatus> {
        self.request_json(
            Method::Get,
            &format!("cloud/server/{mbpkg_id}/jobs/{job_id}"),
            None,
            None,
        )
    }

    /// Starts network reconfiguration for a server.
    pub fn reconfigure_server_network(&self, mbpkg_id: i64) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("cloud/server/{mbpkg_id}/netconfig"),
            None,
            None,
        )
    }

    /// Returns the network IPs attached to a server.
    pub fn get_server_network_ips(&self, mbpkg_id: i64) -> Result<Value> {
        self.request_json(
            Method::Get,
            &format!("cloud/server/{mbpkg_id}/networkips"),
            None,
            None,
        )
    }

    /// Resets the root password for a server.
    pub fn reset_server_root_password(
        &self,
        mbpkg_id: i64,
        request: &ResetRootPasswordRequest,
    ) -> Result<Value> {
        self.request_json(
            Method::Post,
            &format!("cloud/server/{mbpkg_id}/password"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Reboots a server.
    pub fn reboot_server(&self, mbpkg_id: i64) -> Result<()> {
        self.reboot_server_with_options(mbpkg_id, None)
    }

    /// Reboots a server with optional force behaviour.
    pub fn reboot_server_with_options(
        &self,
        mbpkg_id: i64,
        request: Option<&ServerActionRequest>,
    ) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("cloud/server/{mbpkg_id}/reboot"),
            Some(server_action_body(request)?),
            Some("application/json"),
        )
    }

    /// Starts rescue mode for a server.
    pub fn start_server_rescue(
        &self,
        mbpkg_id: i64,
        request: &RescueStartRequest,
    ) -> Result<Value> {
        self.request_json(
            Method::Post,
            &format!("cloud/server/{mbpkg_id}/rescue_start"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Stops rescue mode for a server.
    pub fn stop_server_rescue(&self, mbpkg_id: i64) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("cloud/server/{mbpkg_id}/rescue_stop"),
            None,
            None,
        )
    }

    /// Returns BGP sessions for a server, optionally filtered by group type.
    pub fn get_server_bgp_sessions(
        &self,
        mbpkg_id: i64,
        group_type: Option<&str>,
    ) -> Result<Value> {
        let mut pairs = Vec::new();
        push_query_opt_str(&mut pairs, "group_type", group_type);
        let path = query_path(&format!("cloud/server/{mbpkg_id}/sessions"), pairs);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Shuts down a server with optional force behaviour.
    pub fn shutdown_server_with_options(
        &self,
        mbpkg_id: i64,
        request: Option<&ServerActionRequest>,
    ) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("cloud/server/{mbpkg_id}/shutdown"),
            Some(server_action_body(request)?),
            Some("application/json"),
        )
    }

    /// Starts a server with optional force behaviour.
    pub fn start_server_with_options(
        &self,
        mbpkg_id: i64,
        request: Option<&ServerActionRequest>,
    ) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("cloud/server/{mbpkg_id}/start"),
            Some(server_action_body(request)?),
            Some("application/json"),
        )
    }

    /// Returns status for a server.
    pub fn get_server_status(&self, mbpkg_id: i64) -> Result<ServerStatus> {
        self.request_json(
            Method::Get,
            &format!("cloud/server/{mbpkg_id}/status"),
            None,
            None,
        )
    }

    /// Starts a VNC session for a server.
    pub fn start_server_vnc(&self, mbpkg_id: i64) -> Result<Value> {
        self.request_json(
            Method::Post,
            &format!("cloud/server/{mbpkg_id}/vnc"),
            Some(b"null".to_vec()),
            Some("application/json"),
        )
    }

    /// Returns monthly bandwidth data for a server.
    pub fn get_server_monthly_bandwidth(&self, mbpkg_id: i64) -> Result<Value> {
        self.request_json(
            Method::Get,
            &format!("cloud/servermonthlybw/{mbpkg_id}"),
            None,
            None,
        )
    }

    /// Attempts an SSH login to a server, returning the platform's connection report.
    pub fn attempt_ssh_connection(&self, request: &AttemptSshRequest) -> Result<Value> {
        self.request_json(
            Method::Post,
            "cloud/servers/attempt-ssh",
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Returns the server associated with the caller's IP address.
    pub fn get_current_server(&self) -> Result<Server> {
        self.request_json(Method::Get, "cloud/servers/current", None, None)
    }

    /// Returns unprovisioned packages available to build.
    pub fn get_unprovisioned_packages(&self) -> Result<Value> {
        self.request_json(Method::Get, "cloud/servers/unprovisioned", None, None)
    }

    /// Returns usage statistics for cloud servers on the account.
    pub fn get_cloud_servers_usage_info(&self) -> Result<Value> {
        self.request_json(Method::Get, "cloud/servers/usage/info", None, None)
    }

    /// Returns contract usage data for a virtual server.
    pub fn get_virtual_server_contract(&self, mbpkg_id: i64) -> Result<ContractUsage> {
        self.request_json(
            Method::Get,
            &format!("cloud/servers/{mbpkg_id}/contract"),
            None,
            None,
        )
    }

    /// Returns account-level contract usage totals.
    pub fn get_contract_usage(&self) -> Result<ContractUsage> {
        self.request_json(Method::Get, "cloud/contract/usage", None, None)
    }

    /// Lists the boot profiles available for building cloud servers.
    pub fn get_boot_profiles(&self) -> Result<Vec<BootProfile>> {
        self.request_json(Method::Get, "cloud/boot-profiles", None, None)
    }

    /// Lists the disks attached to a cloud server's billing package.
    pub fn get_server_disks(&self, mbpkg_id: i64) -> Result<Vec<ServerDisk>> {
        self.request_json(Method::Get, &format!("cloud/disks/{mbpkg_id}"), None, None)
    }

    /// Returns summary data for a server.
    pub fn get_server_summary(&self, mbpkg_id: i64) -> Result<Value> {
        self.request_json(
            Method::Get,
            &format!("cloud/serversummary/{mbpkg_id}"),
            None,
            None,
        )
    }

    /// Returns the plan id for a plan name.
    pub fn get_plan_id(&self, plan_name: &str) -> Result<Value> {
        self.request_json(
            Method::Get,
            &format!("cloud/sizes/plan-id/{}", encode(plan_name)),
            None,
            None,
        )
    }

    /// Returns deployable sizes for a location, optionally filtered by minimum CPU or RAM.
    ///
    /// A location or filter that matches no plan answers with a 404 rather than an empty list;
    /// that is treated as zero available sizes rather than an error.
    pub fn get_deploy_sizes(
        &self,
        location: &str,
        request: &DeploySizesRequest,
    ) -> Result<Vec<Size>> {
        let mut pairs = Vec::new();
        push_query_opt_i64(&mut pairs, "min_cpu", request.min_cpu);
        push_query_opt_i64(&mut pairs, "min_ram", request.min_ram);
        let path = query_path(&format!("cloud/sizes/{}", encode(location)), pairs);
        self.list_or_empty(&path)
    }

    /// Returns all deployable sizes on the account, independent of location.
    pub fn get_sizes(&self) -> Result<Vec<Size>> {
        self.request_json(Method::Get, "cloud/sizes", None, None)
    }

    /// Returns all deployable plans on the account.
    ///
    /// This calls the same endpoint as [`Client::get_sizes`]; the platform does not distinguish
    /// a plan from a size at this endpoint.
    pub fn get_plans(&self) -> Result<Vec<Size>> {
        self.get_sizes()
    }

    /// Returns storage locations, optionally filtered to one capacity pool.
    pub fn get_storage_locations(&self, cloud_pool_id: Option<i64>) -> Result<Value> {
        let mut pairs = Vec::new();
        push_query_opt_i64(&mut pairs, "cloud_pool_id", cloud_pool_id);
        let path = query_path("cloud/storage-locations", pairs);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Binds a firewall set to a cloud server interface.
    pub fn bind_cloud_firewall_set(
        &self,
        mbpkg_id: i64,
        request: &BindFirewallSetRequest,
    ) -> Result<Value> {
        self.request_json(
            Method::Post,
            &format!("cloud/{mbpkg_id}/firewall-sets"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Unbinds a firewall set from a cloud server.
    pub fn unbind_cloud_firewall_set(&self, mbpkg_id: i64, firewall_set: &str) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!("cloud/{mbpkg_id}/firewall-sets/{}", encode(firewall_set)),
            None,
            None,
        )
    }

    /// Creates a usage contract for an account.
    pub fn create_usage_contract(
        &self,
        request: &CreateUsageContractRequest,
    ) -> Result<ContractUsage> {
        self.request_json(
            Method::Post,
            "cloud/contract/usage",
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Uploads and parses a cloud-init script, returning the platform's parsed representation.
    pub fn parse_cloud_init(&self, filename: &str, content: &[u8]) -> Result<Value> {
        let body = cloud_init_multipart_body(filename, content);
        self.request_json(
            Method::Post,
            "cloud/parse-cloud-init",
            Some(body),
            Some(&format!(
                "multipart/form-data; boundary={CLOUD_INIT_BOUNDARY}"
            )),
        )
    }

    /// Runs a GET request expecting a JSON array, treating a not-found response as an empty
    /// list rather than an error.
    ///
    /// A filtered listing that matches nothing answers 404 instead of `200` with `[]`; a caller
    /// asking "which groups have this type" wants zero groups back, not a hard error.
    fn list_or_empty<T>(&self, path: &str) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.request_json(Method::Get, path, None, None) {
            Err(err) if err.is_not_found() => Ok(Vec::new()),
            result => result,
        }
    }

    fn post_bgp_group_action(&self, group_id: i64, action: &str) -> Result<()> {
        let path = format!("bgp/bgpgroup/{group_id}/{action}");
        self.request_empty(
            Method::Post,
            &path,
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    fn post_bgp_session_action(&self, session_id: i64, action: &str) -> Result<()> {
        let path = format!("bgp/bgpsession/{session_id}/{action}");
        self.request_empty(
            Method::Post,
            &path,
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    fn post_dedicated_action(
        &self,
        mbpkgid: i64,
        action: &str,
        request: Option<&DedicatedServerActionRequest>,
    ) -> Result<()> {
        self.request_empty(
            Method::Post,
            &dedicated_action_path(mbpkgid, action),
            Some(dedicated_action_body(request)?),
            Some("application/json"),
        )
    }

    fn post_dedicated_action_with_response<T>(
        &self,
        mbpkgid: i64,
        action: &str,
        request: Option<&DedicatedServerActionRequest>,
    ) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.request_json(
            Method::Post,
            &dedicated_action_path(mbpkgid, action),
            Some(dedicated_action_body(request)?),
            Some("application/json"),
        )
    }

    fn request_json<T>(
        &self,
        method: Method,
        path: &str,
        body: Option<Vec<u8>>,
        content_type: Option<&str>,
    ) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let payload = self.request_payload(method, path, body, content_type)?;
        decode_required(&payload, path)
    }

    fn request_empty(
        &self,
        method: Method,
        path: &str,
        body: Option<Vec<u8>>,
        content_type: Option<&str>,
    ) -> Result<()> {
        self.request_payload(method, path, body, content_type)
            .map(|_| ())
    }

    fn request_payload(
        &self,
        method: Method,
        path: &str,
        body: Option<Vec<u8>>,
        content_type: Option<&str>,
    ) -> Result<Vec<u8>> {
        let url = build_url(&self.base_url, path, &self.api_key)?;
        let response = self.transport.send(Request {
            method,
            url: url.clone(),
            body,
            content_type: content_type.map(str::to_string),
        })?;
        let redacted_url = redact_url(&url);
        let envelope: V2Envelope = serde_json::from_slice(&response.body).map_err(|err| {
            if response.status == 404 {
                Error::NotFound {
                    method: method.as_str().to_string(),
                    url: redacted_url.clone(),
                    status_code: response.status,
                    api_code: 0,
                    message: "not found".to_string(),
                }
            } else {
                Error::Decode(format!("vAPI2 envelope: {err}"))
            }
        })?;

        classify_v2_error(method, &redacted_url, response.status, &envelope)?;
        let data = envelope.data.unwrap_or(Value::Null);
        let data = self.unwrap_v2_paginator(method, path, data)?;
        Ok(serde_json::to_vec(&data)?)
    }

    /// Sends a request and returns the raw response body, bypassing the vAPI2 envelope.
    ///
    /// The ticket attachment download and preview endpoints answer with the raw file content
    /// rather than a JSON envelope, so they cannot go through [`Client::request_payload`].
    fn request_raw(&self, method: Method, path: &str) -> Result<Vec<u8>> {
        let url = build_url(&self.base_url, path, &self.api_key)?;
        let response = self.transport.send(Request {
            method,
            url: url.clone(),
            body: None,
            content_type: None,
        })?;
        if !(200..300).contains(&response.status) {
            return Err(Error::Api {
                method: method.as_str().to_string(),
                url: redact_url(&url),
                status_code: response.status,
                api_code: 0,
                message: String::from_utf8_lossy(&response.body).into_owned(),
            });
        }
        Ok(response.body)
    }

    fn unwrap_v2_paginator(&self, method: Method, path: &str, data: Value) -> Result<Value> {
        let Some(paginator) = V2Paginator::from_value(&data)? else {
            return Ok(data);
        };
        if paginator.current_page <= 0 || paginator.last_page <= 0 {
            return Err(Error::Decode("invalid vAPI2 paginator".to_string()));
        }
        let mut rows = array_rows(paginator.data, "vAPI2 paginator data")?;
        if paginator.last_page == 1 {
            return Ok(Value::Array(rows));
        }
        if method != Method::Get {
            return Err(Error::Decode(
                "vAPI2 pagination is only supported for GET requests".to_string(),
            ));
        }
        for page in (paginator.current_page + 1)..=paginator.last_page {
            let next_path = page_path(path, page);
            let next_rows: Vec<Value> = self.request_json(method, &next_path, None, None)?;
            rows.extend(next_rows);
        }
        Ok(Value::Array(rows))
    }

    // --- DDoS ---

    /// Lists every recorded DDoS attack on the account.
    pub fn get_ddos_attacks(&self) -> Result<Vec<DdosAttack>> {
        self.request_json(Method::Get, "ddos/attacks", None, None)
    }

    /// Lists DDoS attacks currently in progress.
    pub fn get_ddos_active_attacks(&self) -> Result<Vec<DdosAttack>> {
        self.request_json(Method::Get, "ddos/attacks/active", None, None)
    }

    /// Returns DDoS dashboard summary data, optionally filtered by period, whether to include
    /// ended attacks, and the number of top attacks to return.
    pub fn get_ddos_dashboard(&self, options: &DdosDashboardOptions) -> Result<DdosDashboard> {
        let mut pairs = Vec::new();
        push_query_opt_i64(&mut pairs, "period", options.period);
        push_query_opt_bool(&mut pairs, "include_ended", options.include_ended);
        push_query_opt_i64(&mut pairs, "limit", options.limit);
        let path = query_path("ddos/dashboard", pairs);
        self.request_json(Method::Get, &path, None, None)
    }

    /// Lists every DDoS mitigation rule on the account.
    pub fn get_ddos_rules(&self) -> Result<Vec<DdosRule>> {
        self.request_json(Method::Get, "ddos/rules", None, None)
    }

    /// Gets a DDoS mitigation rule by id.
    pub fn get_ddos_rule(&self, id: i64) -> Result<DdosRule> {
        self.request_json(Method::Get, &format!("ddos/rule/{id}"), None, None)
    }

    // --- Access control subnets ---

    /// Lists the subnets authorized to reach the account's management API.
    pub fn get_access_control_subnets(&self) -> Result<Vec<AccessControlSubnet>> {
        self.request_json(
            Method::Get,
            "account/user-access-control-subnet-list",
            None,
            None,
        )
    }

    /// Gets an access control subnet by id.
    pub fn get_access_control_subnet(&self, id: i64) -> Result<AccessControlSubnet> {
        self.request_json(
            Method::Get,
            &format!("account/user-access-control-subnet/{id}"),
            None,
            None,
        )
    }

    /// Authorizes a new access control subnet.
    pub fn create_access_control_subnet(
        &self,
        request: &CreateAccessControlSubnetRequest,
    ) -> Result<AccessControlSubnet> {
        self.request_json(
            Method::Post,
            "account/user-access-control-subnet",
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Updates an access control subnet's label or address range.
    pub fn update_access_control_subnet(
        &self,
        id: i64,
        request: &UpdateAccessControlSubnetRequest,
    ) -> Result<AccessControlSubnet> {
        self.request_json(
            Method::Patch,
            &format!("account/user-access-control-subnet/{id}"),
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    /// Revokes an access control subnet. A subnet that is already gone is treated as success.
    pub fn delete_access_control_subnet(&self, id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("account/user-access-control-subnet/{id}"),
            None,
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    // --- Dedicated server builds (metal) ---

    /// Returns the status of a dedicated server build.
    pub fn get_dedicated_server_build_status(
        &self,
        build_id: i64,
    ) -> Result<DedicatedServerBuildStatus> {
        self.request_json(
            Method::Get,
            &format!("dedicated/server/build_status/{build_id}"),
            None,
            None,
        )
    }

    /// Purchases and builds a dedicated device in one call.
    pub fn buy_build_dedicated_server(
        &self,
        request: &BuyBuildDedicatedServerRequest,
    ) -> Result<ServerBuild> {
        let body = request.to_form();
        self.request_json(
            Method::Post,
            "dedicated/server/buy_build",
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Rebuilds an existing dedicated server package.
    pub fn rebuild_dedicated_server(
        &self,
        id: i64,
        request: &RebuildDedicatedServerRequest,
    ) -> Result<ServerBuild> {
        let body = request.to_form();
        self.request_json(
            Method::Post,
            &format!("dedicated/server/re_build/{id}"),
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Gets a dedicated server record by id.
    pub fn get_dedicated_server(&self, id: i64) -> Result<DedicatedServer> {
        self.request_json(Method::Get, &format!("dedicated/servers/{id}"), None, None)
    }

    // --- BGP sessions ---

    /// Gets a BGP session by id.
    pub fn get_bgp_session(&self, id: i64) -> Result<BgpSession> {
        self.request_json(Method::Get, &format!("bgp/bgpsession/{id}"), None, None)
    }

    /// Lists the BGP sessions belonging to a billing package.
    ///
    /// The platform has no endpoint that filters sessions by package directly, so this lists
    /// every session on the account and keeps only those whose customer peer address matches
    /// one of the package's own IPs.
    pub fn list_bgp_sessions(&self, mbpkg_id: i64) -> Result<Vec<BgpSession>> {
        let all_sessions: Vec<BgpSession> =
            self.request_json(Method::Get, "bgp/bgpsessions", None, None)?;
        if all_sessions.is_empty() {
            return Ok(Vec::new());
        }

        let ips = self.get_ips(mbpkg_id)?;
        if ips.ipv4.is_empty() && ips.ipv6.is_empty() {
            return Ok(Vec::new());
        }
        let families = ips.address_families();

        let mut sessions = Vec::new();
        for session in all_sessions {
            if families.contains_key(&session.customer_peer_ip) {
                sessions.push(self.get_bgp_session(session.id)?);
            }
        }
        Ok(sessions)
    }

    /// Creates a BGP session for a billing package on a BGP group.
    pub fn create_bgp_sessions(
        &self,
        mbpkg_id: i64,
        group_id: i64,
        is_ipv6: bool,
        redundant: bool,
    ) -> Result<BgpSession> {
        let mut pairs = vec![
            ("mbpkgid".to_string(), mbpkg_id.to_string()),
            ("group_id".to_string(), group_id.to_string()),
        ];
        if is_ipv6 {
            pairs.push(("ipv6".to_string(), "1".to_string()));
        }
        if redundant {
            pairs.push(("redundant".to_string(), "1".to_string()));
        }
        let body = form_encode(pairs);
        self.request_json(
            Method::Post,
            "bgp/bgpcreatesessions",
            Some(body.into_bytes()),
            Some("application/x-www-form-urlencoded"),
        )
    }

    /// Deletes a single BGP session by id. A session that is already gone is treated as
    /// success.
    pub fn delete_bgp_session(&self, session_id: i64) -> Result<()> {
        let path = format!("bgp/bgpsession/{session_id}/delete");
        match self.request_empty(
            Method::Post,
            &path,
            Some(Vec::new()),
            Some("application/x-www-form-urlencoded"),
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    // --- Capacity ---

    /// Lists the billing packages on the account.
    pub fn get_billing_packages(&self) -> Result<Vec<BillingPackage>> {
        self.request_json(Method::Get, "cloud/billing-packages", None, None)
    }

    /// Lists every capacity pool cloud servers can deploy into.
    pub fn list_cloud_pools(&self) -> Result<Vec<CloudPool>> {
        self.request_json(Method::Get, "cloud/pools", None, None)
    }

    /// Returns available cloud capacity for a package shape at a location.
    ///
    /// A pool and location combination with no capacity answers with a 404 rather than an
    /// empty list; that is treated as zero available packages rather than an error.
    pub fn get_cloud_capacity(
        &self,
        cloud_pool_id: i64,
        location_id: i64,
    ) -> Result<Vec<CloudCapacity>> {
        let path =
            format!("cloud/capacity?cloud_pool_id={cloud_pool_id}&location_id={location_id}");
        self.list_or_empty(&path)
    }

    /// Lists available dedicated server capacity across all locations.
    pub fn get_dedicated_capacity(&self) -> Result<Vec<DedicatedCapacity>> {
        self.request_json(Method::Get, "dedicated/capacity", None, None)
    }

    // --- Non-cloud packages ---

    /// Lists colocation packages on the account, sorted by billing package id.
    pub fn get_colocation_packages(&self) -> Result<Vec<ColocationPackage>> {
        let by_id: HashMap<String, ColocationPackage> =
            self.request_json(Method::Get, "colo/packages", None, None)?;
        let mut packages: Vec<ColocationPackage> = by_id.into_values().collect();
        packages.sort_by_key(|pkg| pkg.mbpkgid);
        Ok(packages)
    }

    /// Gets a colocation package by billing package id.
    pub fn get_colocation_package(&self, mbpkgid: i64) -> Result<ColocationPackage> {
        self.request_json(Method::Get, &format!("colo/package/{mbpkgid}"), None, None)
    }

    /// Lists transit packages on the account, sorted by billing package id.
    pub fn get_transit_packages(&self) -> Result<Vec<TransitPackage>> {
        let by_id: HashMap<String, TransitPackage> =
            self.request_json(Method::Get, "transit/packages", None, None)?;
        let mut packages: Vec<TransitPackage> = by_id.into_values().collect();
        packages.sort_by_key(|pkg| pkg.mbpkgid);
        Ok(packages)
    }

    /// Gets a transit package by billing package id.
    pub fn get_transit_package(&self, mbpkgid: i64) -> Result<TransitPackage> {
        self.request_json(
            Method::Get,
            &format!("transit/package/{mbpkgid}"),
            None,
            None,
        )
    }

    // --- Cloud packages ---

    /// Lists every purchased cloud package on the account.
    pub fn get_packages(&self) -> Result<Vec<Package>> {
        self.request_json(Method::Get, "cloud/packages", None, None)
    }

    /// Gets a purchased cloud package by billing package id.
    pub fn get_package(&self, id: i64) -> Result<Package> {
        self.request_json(Method::Get, &format!("cloud/package/{id}"), None, None)
    }

    /// Cancels a cloud package, returning the platform's raw response.
    pub fn cancel_package(&self, request: &CancelPackageRequest) -> Result<Value> {
        self.request_json(
            Method::Post,
            "cloud/package/cancel/",
            Some(serde_json::to_vec(request)?),
            Some("application/json"),
        )
    }

    // --- Longtail ---

    /// Returns the platform location detected for the caller's current IP address.
    pub fn get_location_by_current_ip(&self) -> Result<LocationByCurrentIp> {
        self.request_json(Method::Get, "location", None, None)
    }

    /// Returns graph data for a switch port over a time range.
    ///
    /// `time` must be one of `daily`, `weekly`, `monthly` or `yearly`.
    pub fn get_graph(&self, port: i64, time: &str) -> Result<Graph> {
        let path = format!("graphs/graph?port={port}&time={}", encode(time));
        self.request_json(Method::Get, &path, None, None)
    }

    // --- Locations ---

    /// Lists the available deployment locations.
    pub fn get_locations(&self) -> Result<Vec<Location>> {
        self.request_json(Method::Get, "cloud/locations", None, None)
    }

    // --- OS catalog ---

    /// Returns the OS catalog available for building cloud servers.
    ///
    /// This calls the same `cloud/images` endpoint used to enumerate base and custom images;
    /// the platform does not distinguish an OS entry from an image at this endpoint, so
    /// [`Image`] applies here too.
    pub fn get_oss(&self) -> Result<Vec<Image>> {
        self.request_json(Method::Get, "cloud/images", None, None)
    }

    // --- Network IPs ---

    /// Returns the IPv4 and IPv6 addresses on a billing package's network.
    pub fn get_ips(&self, mbpkg_id: i64) -> Result<NetworkIps> {
        self.request_json(
            Method::Get,
            &format!("cloud/networkips/{mbpkg_id}"),
            None,
            None,
        )
    }
}

/// Decodes a network interface response body.
///
/// The attach and update endpoints answer with either a single interface object or, on some
/// accounts, that same object wrapped in a one-element array. A null or empty-array body means
/// no interface was returned, which decodes to the zero value rather than an error.
fn server_nic_from_payload(payload: &[u8]) -> Result<ServerNic> {
    let value: Value = serde_json::from_slice(payload)?;
    Ok(match value {
        Value::Null => ServerNic::default(),
        Value::Array(mut rows) if rows.len() == 1 => serde_json::from_value(rows.remove(0))?,
        Value::Array(_) => ServerNic::default(),
        other => serde_json::from_value(other)?,
    })
}

/// Accepts a JSON null where a string is expected, yielding an empty string.
pub(crate) fn null_as_empty_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct V2Envelope {
    // The platform sends a null message on a successful call, so this cannot be a plain String:
    // serde rejects null for one and every successful response fails to decode.
    #[serde(default, deserialize_with = "crate::client::null_as_empty_string")]
    message: String,
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    code: i64,
    #[serde(default)]
    fields: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct V2Paginator {
    current_page: i64,
    last_page: i64,
    data: Value,
}

impl V2Paginator {
    fn from_value(value: &Value) -> Result<Option<Self>> {
        if !value.is_object() {
            return Ok(None);
        }
        if value.get("current_page").is_some() && value.get("last_page").is_some() {
            return Ok(Some(serde_json::from_value(value.clone())?));
        }
        if let Some(paginator) = value.get("paginator") {
            return Ok(Some(serde_json::from_value(paginator.clone())?));
        }
        Ok(None)
    }
}

/// Request body for creating a server.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateServerRequest {
    /// Package plan name.
    pub plan: String,
    /// Location id.
    pub location: i64,
    /// Image id.
    pub image: i64,
    /// Server FQDN.
    pub fqdn: String,
    /// SSH public key content.
    pub ssh_key: String,
    /// SSH key id.
    pub ssh_key_id: i64,
    /// Initial password.
    pub password: String,
    /// Billing mode.
    pub package_billing: String,
    /// Billing contract id.
    pub package_billing_contract_id: String,
    /// Cloud-init or script content.
    pub script_content: String,
    /// JSON params string.
    pub params: String,
    /// Tag name.
    pub tag: String,
    /// Tags to replace.
    pub tag_list: Vec<String>,
    /// Cloud pool id.
    pub cloud_pool_id: Option<i64>,
    /// VPC id.
    pub vpc_id: Option<i64>,
}

impl CreateServerRequest {
    fn to_form(&self) -> String {
        let mut pairs = Vec::new();
        push_str(&mut pairs, "plan", &self.plan);
        push_i64(&mut pairs, "location", self.location);
        push_i64(&mut pairs, "image", self.image);
        push_str(&mut pairs, "fqdn", &self.fqdn);
        push_str(&mut pairs, "ssh_key", &self.ssh_key);
        push_i64(&mut pairs, "ssh_key_id", self.ssh_key_id);
        push_str(&mut pairs, "password", &self.password);
        push_str(&mut pairs, "package_billing", &self.package_billing);
        push_str(
            &mut pairs,
            "package_billing_contract_id",
            &self.package_billing_contract_id,
        );
        push_str(&mut pairs, "script_content", &self.script_content);
        if !self.script_content.is_empty() {
            push_str(&mut pairs, "script_type", "user-data");
        }
        push_str(&mut pairs, "params", &self.params);
        push_str(&mut pairs, "tag", &self.tag);
        for tag in &self.tag_list {
            push_str(&mut pairs, "tag_list[]", tag);
        }
        push_opt_i64(&mut pairs, "cloud_pool_id", self.cloud_pool_id);
        push_opt_i64(&mut pairs, "vpc_id", self.vpc_id);
        form_encode(pairs)
    }
}

/// Request body for rebuilding a server.
pub type BuildServerRequest = CreateServerRequest;

/// Options for deleting a server.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeleteServerOptions {
    /// Whether to cancel billing with the delete.
    pub cancel_billing: bool,
}

impl DeleteServerOptions {
    fn to_form(self) -> String {
        if self.cancel_billing {
            form_encode(vec![("cancel_billing".to_string(), "1".to_string())])
        } else {
            String::new()
        }
    }
}

/// Request body for scaling a server to a different package.
///
/// Set either `pkg_name` or `pkg_id`, not both; an empty name and a zero id are both omitted
/// from the request.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScaleServerRequest {
    /// Target package name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub pkg_name: String,
    /// Target package id.
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub pkg_id: i64,
    /// Whether the platform may reboot the server to apply the scale.
    pub allow_reboot: bool,
}

fn is_zero_i64(value: &i64) -> bool {
    *value == 0
}

/// Request body for creating a DNS zone.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateDnsZoneRequest {
    /// Zone name.
    pub name: String,
    /// Zone type.
    pub zone_type: String,
    /// Optional IP value.
    pub ip: String,
}

impl CreateDnsZoneRequest {
    fn to_form(&self) -> String {
        let mut pairs = Vec::new();
        push_str(&mut pairs, "name", &self.name);
        push_str(&mut pairs, "type", &self.zone_type);
        push_str(&mut pairs, "ip", &self.ip);
        form_encode(pairs)
    }
}

/// Request body for creating a DNS record.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateDnsRecordRequest {
    /// Zone id.
    pub zone_id: i64,
    /// Record name.
    pub name: String,
    /// Record type.
    pub record_type: String,
    /// TTL.
    pub ttl: i64,
    /// Priority.
    pub priority: i64,
    /// Record content.
    pub record_content: String,
}

impl CreateDnsRecordRequest {
    fn to_form(&self) -> String {
        dns_record_form(None, self)
    }
}

/// Request body for updating a DNS record.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateDnsRecordRequest {
    /// Record id.
    pub id: i64,
    /// Zone id.
    pub zone_id: i64,
    /// Record name.
    pub name: String,
    /// Record type.
    pub record_type: String,
    /// TTL.
    pub ttl: i64,
    /// Priority.
    pub priority: i64,
    /// Record content.
    pub record_content: String,
}

impl UpdateDnsRecordRequest {
    fn to_form(&self) -> String {
        let create = CreateDnsRecordRequest {
            zone_id: self.zone_id,
            name: self.name.clone(),
            record_type: self.record_type.clone(),
            ttl: self.ttl,
            priority: self.priority,
            record_content: self.record_content.clone(),
        };
        dns_record_form(Some(self.id), &create)
    }
}

/// Request body for creating a tag.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateTagRequest {
    /// Tag name.
    pub name: String,
    /// Tag description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Icon identifier.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub icon: String,
    /// Display color.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub color: String,
}

/// Request body for updating a tag.
///
/// The flag fields are always sent; only `description`, `icon` and `color` are
/// optional.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateTagRequest {
    /// Tag name.
    pub name: String,
    /// Tag description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Icon identifier.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub icon: String,
    /// Display color.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub color: String,
    /// Whether this is the account's default tag.
    pub is_default: i64,
    /// Whether the tag is marked as a favorite.
    pub is_favorite: i64,
    /// Whether the tag is locked against deletion.
    pub is_locked: i64,
    /// Whether the tag is shown on the dashboard.
    pub show_dashboard: i64,
}

/// Resource reference sent to the tag assign/remove endpoints.
#[derive(Debug, Clone, Serialize)]
struct TagResourceRef {
    resource_name: String,
    // The request schema types identifier as a string even though it is stored and
    // returned as a number; send it as a string for spec compliance.
    identifier: String,
}

/// Body sent to the SSH key update endpoint.
#[derive(Debug, Clone, Serialize)]
struct SshKeyUpdate {
    name: String,
    ssh_key: String,
}

/// Request body for creating or updating a firewall rule.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateFirewallRuleRequest {
    /// IP version the rule applies to, such as `ipv4` or `ipv6`.
    pub ip_version: String,
    /// Traffic direction, `inbound` or `outbound`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub direction: String,
    /// Action taken on a match, such as `accept` or `drop`.
    pub action: String,
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Evaluation priority, lower values evaluated first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_priority: Option<i64>,
    /// Administrator comment.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub admin_comment: String,
    /// Match criteria.
    pub match_criteria: Option<FirewallMatchCriteria>,
}

/// Request body for moving a rule within a draft firewall set.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ReorderFirewallRulesRequest {
    /// Id of the rule being moved.
    pub move_id: i64,
    /// Id of the rule to place the moved rule after.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<i64>,
    /// Id of the rule to place the moved rule before.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<i64>,
}

/// Request body for attaching a network interface to a server.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ServerNicAttachRequest {
    /// Customer VLAN id to attach the new interface to.
    pub customer_vlan_id: i64,
}

/// Request body for updating a network interface.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ServerNicUpdateRequest {
    /// Server package id the interface belongs to.
    pub mbpkgid: i64,
    /// Customer VLAN id the interface is attached to.
    pub customer_vlan_id: i64,
    /// Attachment order among the server's interfaces.
    pub attach_order: i64,
}

/// Optional filters for the support ticket list endpoints.
#[derive(Debug, Clone, Default)]
pub struct TicketListOptions {
    /// Filters tickets by open state.
    pub open: Option<String>,
    /// Requests ticket statistics alongside the list.
    pub include_stats: Option<String>,
}

/// Request body for opening a support ticket.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateTicketRequest {
    /// Ticket subject.
    pub subject: String,
    /// Initial message body.
    pub message: String,
    /// Department id to route the ticket to.
    pub department: i64,
    /// Urgency level.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub urgency: String,
    /// References to files already uploaded as attachments.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

/// Request body for replying to a support ticket.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReplyTicketRequest {
    /// Reply message body.
    pub message: String,
    /// References to files already uploaded as attachments.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

/// Query options for [`Client::get_firewall_set_available_vms`].
#[derive(Debug, Clone, Copy, Default)]
pub struct FirewallAvailableVmOptions {
    /// Restricts results to VMs owned by this sub-account.
    pub extref_account_id: Option<i64>,
    /// Restricts results to VMs in this VPC.
    pub vpc_id: Option<i64>,
    /// Whether to include bandwidth details in the response.
    pub include_bandwidth: Option<bool>,
    /// Whether to include usage limit details in the response.
    pub include_ul: Option<bool>,
    /// Whether to check VPC membership when filtering.
    pub check_vpc: Option<bool>,
    /// Whether to disable filtering VMs by interface id.
    pub disable_interface_id_filter: Option<bool>,
}

/// Query options for [`Client::get_firewall_set_related_vms`].
#[derive(Debug, Clone, Copy, Default)]
pub struct FirewallRelatedSetOptions {
    /// Whether to disable filtering VMs by interface id.
    pub disable_interface_id_filter: Option<bool>,
}

/// Query options for [`Client::execute_platform_looking_glass`].
#[derive(Debug, Clone, Default)]
pub struct PlatformLookingGlassExecuteOptions {
    /// Looking glass action, such as `ping` or `traceroute`.
    pub action: Option<String>,
    /// Target host or address.
    pub target: Option<String>,
    /// Platform location to run the action from.
    pub location: Option<String>,
    /// Whether to request full, unabridged output.
    pub full: Option<i64>,
}

/// Request body for creating a BGP group.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateBgpGroupRequest {
    /// Group name.
    pub name: String,
    /// Group description.
    pub description: String,
    /// Group type, such as `bgp` or `anycast`. Omitted from the request when empty.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub group_type: String,
}

/// Request body for purchasing anycast BGP prefixes.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BuyBgpPrefixesRequest {
    /// Prefix name.
    pub name: String,
    /// Id of the group the prefix is created under.
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub group_id: i64,
    /// Id of the ASN the prefix is announced from.
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub asn_id: i64,
    /// Anycast profile id.
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub anycast_profile: i64,
    /// Id of the agreement the purchase is made under.
    pub agreement_id: i64,
}

/// Request body for binding a firewall set to an interface of a BGP group.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct BindBgpGroupFirewallSetRequest {
    /// Id of the binding, when updating an existing one.
    pub id: i64,
    /// Id of the firewall set to bind.
    pub firewall_set_id: i64,
    /// Interface number to bind the firewall set to.
    pub interface_number: i64,
    /// Evaluation priority among the sets bound to the same interface.
    pub set_priority: i64,
}

/// Query options for [`Client::get_bgp_dashboard`].
#[derive(Debug, Clone, Default)]
pub struct BgpDashboardOptions {
    /// Restricts the dashboard to one BGP group type.
    pub group_type: String,
    /// Restricts flap counts to this window, in seconds.
    pub flap_window: Option<i64>,
}

/// Body sent to the firewall set VM attach endpoint.
#[derive(Debug, Clone, Serialize)]
struct AttachFirewallSetVmRequest {
    vm_list: Vec<AttachFirewallSetVmEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct AttachFirewallSetVmEntry {
    mbpkgid: i64,
    interface_id: i64,
    set_priority: i64,
}

/// Optional filters for [`Client::filter_dedicated_devices`].
#[derive(Debug, Clone, Default)]
pub struct DedicatedDeviceFilterOptions {
    /// Number of results per page.
    pub per_page: Option<i64>,
    /// Restricts results to devices with this network interface.
    pub nic: Option<String>,
    /// Restricts results to devices with this CPU type.
    pub cpu_type: Option<String>,
    /// Restricts results to devices with this GPU type.
    pub gpu_type: Option<String>,
    /// Restricts results to devices with this disk type.
    pub disk_type: Option<String>,
    /// Restricts results to devices with this core count.
    pub cores: Option<String>,
    /// Restricts results to devices with this amount of RAM, in megabytes.
    pub ram_mb: Option<String>,
    /// Restricts results to devices with this amount of disk, in mebibytes.
    pub disk_mib: Option<String>,
    /// Restricts results to devices in this datacenter.
    pub dc_name: Option<String>,
    /// Restricts results to devices in this region.
    pub region_name: Option<String>,
}

/// Request body for deploying or rebuilding a dedicated server.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DedicatedServerBuildRequest {
    /// Hostname to assign to the server.
    pub fqdn: String,
    /// OS profile id to build.
    pub profile: i64,
    /// Disk layout id, when the profile offers more than one.
    #[serde(rename = "disklayout", skip_serializing_if = "Option::is_none")]
    pub disk_layout: Option<i64>,
    /// Root password to set on the built server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_password: Option<String>,
    /// SSH public key content to install.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key: Option<String>,
    /// Id of an account SSH key to install.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_id: Option<i64>,
    /// Cloud-init or shell script to run on first boot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_script: Option<String>,
}

/// Request body for [`Client::buy_build_dedicated_server`]. Every field is optional; the
/// platform applies its own default when a field is left unset.
#[derive(Debug, Clone, Default)]
pub struct BuyBuildDedicatedServerRequest {
    /// Location id to buy in.
    pub location: i64,
    /// Specific device id to buy, when narrowing to one.
    pub device_id: i64,
    /// SSH public key content to install.
    pub ssh_key: String,
    /// Id of an account SSH key to install.
    pub ssh_key_id: i64,
    /// Root password to set on the built server.
    pub root_password: String,
    /// Cloud-init or shell script to run on first boot.
    pub build_script: String,
    /// Disk layout id, when the profile offers more than one.
    pub disk_layout: i64,
    /// OS profile id to build.
    pub profile: i64,
    /// Hostname to assign to the server.
    pub hostname: String,
}

impl BuyBuildDedicatedServerRequest {
    fn to_form(&self) -> String {
        let mut pairs = Vec::new();
        push_i64(&mut pairs, "location", self.location);
        push_i64(&mut pairs, "device_id", self.device_id);
        push_str(&mut pairs, "ssh_key", &self.ssh_key);
        push_i64(&mut pairs, "ssh_key_id", self.ssh_key_id);
        push_str(&mut pairs, "root_password", &self.root_password);
        push_str(&mut pairs, "build_script", &self.build_script);
        push_i64(&mut pairs, "disklayout", self.disk_layout);
        push_i64(&mut pairs, "profile", self.profile);
        push_str(&mut pairs, "fqdn", &self.hostname);
        form_encode(pairs)
    }
}

/// Request body for [`Client::rebuild_dedicated_server`]. Every field but `mbpkgid` is
/// optional; the platform applies its own default when a field is left unset.
#[derive(Debug, Clone, Default)]
pub struct RebuildDedicatedServerRequest {
    /// Billing package id of the server to rebuild.
    pub mbpkgid: i64,
    /// SSH public key content to install.
    pub ssh_key: String,
    /// Id of an account SSH key to install.
    pub ssh_key_id: i64,
    /// Root password to set on the rebuilt server.
    pub root_password: String,
    /// Cloud-init or shell script to run on first boot.
    pub build_script: String,
    /// Disk layout id, when the profile offers more than one.
    pub disk_layout: i64,
    /// OS profile id to build.
    pub profile: i64,
    /// Hostname to assign to the server.
    pub hostname: String,
}

impl RebuildDedicatedServerRequest {
    fn to_form(&self) -> String {
        let mut pairs = vec![("mbpkgid".to_string(), self.mbpkgid.to_string())];
        push_str(&mut pairs, "ssh_key", &self.ssh_key);
        push_i64(&mut pairs, "ssh_key_id", self.ssh_key_id);
        push_str(&mut pairs, "root_password", &self.root_password);
        push_str(&mut pairs, "build_script", &self.build_script);
        push_i64(&mut pairs, "disklayout", self.disk_layout);
        push_i64(&mut pairs, "profile", self.profile);
        push_str(&mut pairs, "fqdn", &self.hostname);
        form_encode(pairs)
    }
}

/// Optional fields for the dedicated server power and delete actions.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DedicatedServerActionRequest {
    /// Forces the action even if the platform would otherwise refuse it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    /// Password required by the action, when the platform demands one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Request body for updating a dedicated server's IPv4 reverse DNS entry.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DedicatedIpv4ReverseRequest {
    /// Billing package id of the dedicated server.
    pub mbpkgid: i64,
    /// IP address id to update.
    pub id: i64,
    /// Reverse DNS value.
    pub reverse: String,
}

/// Request body for creating a custom image from a built server.
#[derive(Debug, Clone, Default)]
pub struct CreateImageRequest {
    /// Package id of the server to image.
    pub mbpkgid: i64,
    /// Name for the new image.
    pub image_name: String,
    /// Description for the new image.
    pub image_description: String,
    /// Whether to preserve SSH user home directories in the image.
    pub keep_ssh_userdirs: bool,
}

impl CreateImageRequest {
    fn to_form(&self) -> String {
        let mut pairs = vec![
            ("mbpkgid".to_string(), self.mbpkgid.to_string()),
            ("image_name".to_string(), self.image_name.clone()),
        ];
        if !self.image_description.is_empty() {
            pairs.push((
                "image_description".to_string(),
                self.image_description.clone(),
            ));
        }
        if self.keep_ssh_userdirs {
            pairs.push(("keep_ssh_userdirs".to_string(), "1".to_string()));
        }
        form_encode(pairs)
    }
}

/// Body sent to the image edit endpoint.
#[derive(Debug, Clone, Serialize)]
struct ImageEdit {
    os: String,
    description: String,
}

/// Response returned when an image build is enqueued.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct CreateImageResponse {
    /// Id of the queued imaging job.
    #[serde(default)]
    pub queue_id: i64,
}

/// Response returned when an image deletion is enqueued.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DeleteImageResponse {
    /// Id of the queued deletion job.
    #[serde(default)]
    pub queue_id: i64,
}

/// Request body for replacing an image with another.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ReplaceImageRequest {
    /// Id of the image that replaces the target image.
    pub replace_id: i64,
}

/// Body sent to the IPv4 and IPv6 reverse DNS update endpoints.
#[derive(Debug, Clone, Serialize)]
struct ReverseDnsUpdate {
    reverse: String,
}

/// Filters for a server's scaling options.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScalingOptionsRequest {
    /// Whether to include the server's current plan among the options.
    pub include_current_plan: Option<bool>,
    /// Minimum RAM to consider.
    pub min_ram: Option<i64>,
    /// Maximum RAM to consider.
    pub max_ram: Option<i64>,
    /// Minimum vCPUs to consider.
    pub min_cpus: Option<i64>,
    /// Maximum vCPUs to consider.
    pub max_cpus: Option<i64>,
}

/// Request body for updating mutable server options.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateServerOptionsRequest {
    /// New fully qualified domain name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    /// Autorescue setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autorescue: Option<i64>,
    /// New description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New vCPU count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcpus: Option<i64>,
    /// New boot mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot: Option<String>,
    /// New boot kernel id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_id: Option<i64>,
}

/// Request body for deleting a server with full control over billing and password handling.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DeleteServerRequest {
    /// Whether to cancel billing along with the delete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_billing: Option<bool>,
    /// Password required to force the delete through, when the account requires one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_password: Option<String>,
    /// Account password confirming the delete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Request body for resetting a server's root password.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ResetRootPasswordRequest {
    /// New root password.
    #[serde(rename = "rootpass")]
    pub root_pass: String,
    /// Account password confirming the reset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Optional fields for a cloud server power action such as reboot, start or shutdown.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ServerActionRequest {
    /// Whether to force the action through.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

/// Request body for starting rescue mode on a server.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RescueStartRequest {
    /// Password to use for the rescue environment.
    pub rescue_pass: String,
    /// Account password confirming the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Request body for testing SSH access to a server.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AttemptSshRequest {
    /// Billing package id of the server to test.
    pub mbpkgid: i64,
    /// Username to authenticate as.
    pub username: String,
    /// Password to authenticate with.
    pub password: String,
}

/// Filters for a deploy sizes lookup.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeploySizesRequest {
    /// Minimum vCPU count to consider.
    pub min_cpu: Option<i64>,
    /// Minimum RAM to consider.
    pub min_ram: Option<i64>,
}

/// Request body for binding a firewall set to a cloud server interface.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct BindFirewallSetRequest {
    /// Firewall set id to bind.
    pub firewall_set_id: i64,
    /// Server interface id to bind the set to.
    pub interface_id: i64,
    /// Priority the set is evaluated at on the interface.
    pub set_priority: i64,
}

/// Request body for creating a usage contract.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct CreateUsageContractRequest {
    /// Account mb_id the contract is created for.
    pub mb_id: i64,
}

fn firewall_set_form(name: &str, description: &str, enabled: bool) -> String {
    // Unlike push_str elsewhere in this file, name and description are sent even when
    // empty: the platform treats them as required fields on this endpoint, not optional ones.
    form_encode(vec![
        ("name".to_string(), name.to_string()),
        ("description".to_string(), description.to_string()),
        (
            "enabled".to_string(),
            if enabled { "1" } else { "0" }.to_string(),
        ),
    ])
}

fn query_path(path: &str, pairs: Vec<(String, String)>) -> String {
    let encoded = form_encode(pairs);
    if encoded.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{encoded}")
    }
}

fn push_query_opt_i64(pairs: &mut Vec<(String, String)>, key: &str, value: Option<i64>) {
    if let Some(value) = value {
        pairs.push((key.to_string(), value.to_string()));
    }
}

fn push_query_opt_bool(pairs: &mut Vec<(String, String)>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        pairs.push((key.to_string(), if value { "1" } else { "0" }.to_string()));
    }
}

fn push_query_opt_str(pairs: &mut Vec<(String, String)>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        pairs.push((key.to_string(), value.to_string()));
    }
}

/// Appends an optional integer filter to a services listing path as a query parameter.
fn service_list_path(path: &str, name: &str, value: Option<i64>) -> String {
    match value {
        Some(value) => format!("{path}?{name}={value}"),
        None => path.to_string(),
    }
}

/// Appends an optional group type filter to a BGP listing path as a query parameter.
fn bgp_group_type_path(path: &str, group_type: &str) -> String {
    if group_type.is_empty() {
        path.to_string()
    } else {
        format!("{path}?group_type={}", encode(group_type))
    }
}

fn ticket_list_path(path: &str, options: &TicketListOptions) -> String {
    let mut pairs = Vec::new();
    push_query_opt_str(&mut pairs, "open", options.open.as_deref());
    push_query_opt_str(
        &mut pairs,
        "include_stats",
        options.include_stats.as_deref(),
    );
    query_path(path, pairs)
}

fn ticket_attachment_path(id: &str, attachment_type: &str, rel_id: &str, index: i64) -> String {
    format!(
        "support/tickets/{}/attachment/{}/{}/{index}",
        encode(id),
        encode(attachment_type),
        encode(rel_id),
    )
}

fn secret_value_form(key: &str, value: &str) -> String {
    form_encode(vec![
        ("secret_key".to_string(), key.to_string()),
        ("secret_value".to_string(), value.to_string()),
    ])
}

fn classify_v2_error(
    method: Method,
    redacted_url: &str,
    status_code: u16,
    envelope: &V2Envelope,
) -> Result<()> {
    let api_code = envelope.code;
    if status_code == 404 || api_code == 404 || v2_field_not_found(envelope.fields.as_ref()) {
        return Err(Error::NotFound {
            method: method.as_str().to_string(),
            url: redacted_url.to_string(),
            status_code,
            api_code,
            message: message_from_fields(envelope).unwrap_or_else(|| envelope.message.clone()),
        });
    }
    if status_code == 412 || api_code == 412 {
        return Err(Error::Contract {
            method: method.as_str().to_string(),
            url: redacted_url.to_string(),
            status_code,
            api_code,
            message: envelope.message.clone(),
        });
    }
    if !(200..300).contains(&status_code) || (api_code != 0 && !(200..300).contains(&api_code)) {
        return Err(Error::Api {
            method: method.as_str().to_string(),
            url: redacted_url.to_string(),
            status_code,
            api_code,
            message: message_from_fields(envelope).unwrap_or_else(|| envelope.message.clone()),
        });
    }
    Ok(())
}

fn v2_field_not_found(fields: Option<&Value>) -> bool {
    let Some(Value::Object(fields)) = fields else {
        return false;
    };
    let wants = [
        ("id", "must be a valid zone id"),
        ("id", "must be a valid record id"),
        ("mbpkgid", "must be a valid mbpkgid"),
        ("mbpkgid", "must be a valid package"),
    ];
    wants.iter().any(|(field, needle)| {
        fields
            .get(*field)
            .and_then(Value::as_array)
            .is_some_and(|messages| {
                messages
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|message| message.contains(needle))
            })
    })
}

fn message_from_fields(envelope: &V2Envelope) -> Option<String> {
    let fields = envelope.fields.as_ref()?.as_object()?;
    let mut parts = Vec::new();
    for (field, messages) in fields {
        if let Some(messages) = messages.as_array() {
            for message in messages.iter().filter_map(Value::as_str) {
                parts.push(format!("{field}: {message}"));
            }
        }
    }
    (!parts.is_empty()).then(|| parts.join(", "))
}

fn array_rows(value: Value, context: &str) -> Result<Vec<Value>> {
    match value {
        Value::Array(rows) => Ok(rows),
        _ => Err(Error::Decode(format!("{context} was not an array"))),
    }
}

fn page_path(path: &str, page: i64) -> String {
    let separator = if path.contains('?') { '&' } else { '?' };
    format!("{path}{separator}page={page}")
}

fn dns_record_form(id: Option<i64>, request: &CreateDnsRecordRequest) -> String {
    let mut pairs = Vec::new();
    push_opt_i64(&mut pairs, "id", id);
    push_i64(&mut pairs, "domain_id", request.zone_id);
    push_str(&mut pairs, "name", &request.name);
    push_str(&mut pairs, "type", &request.record_type);
    push_str(&mut pairs, "record_content", &request.record_content);
    push_i64(&mut pairs, "ttl", request.ttl);
    push_i64(&mut pairs, "prio", request.priority);
    form_encode(pairs)
}

fn push_str(pairs: &mut Vec<(String, String)>, key: &str, value: &str) {
    if !value.is_empty() {
        pairs.push((key.to_string(), value.to_string()));
    }
}

fn push_i64(pairs: &mut Vec<(String, String)>, key: &str, value: i64) {
    if value != 0 {
        pairs.push((key.to_string(), value.to_string()));
    }
}

fn push_opt_i64(pairs: &mut Vec<(String, String)>, key: &str, value: Option<i64>) {
    if let Some(value) = value {
        push_i64(pairs, key, value);
    }
}

fn form_encode(pairs: Vec<(String, String)>) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in pairs {
        serializer.append_pair(&key, &value);
    }
    serializer.finish()
}

fn encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn dedicated_action_path(mbpkgid: i64, action: &str) -> String {
    if action == "soft-reset" {
        format!("dedicated/server/soft-reset/{mbpkgid}")
    } else {
        format!("dedicated/server/{mbpkgid}/{action}")
    }
}

fn dedicated_action_body(request: Option<&DedicatedServerActionRequest>) -> Result<Vec<u8>> {
    match request {
        Some(request) => Ok(serde_json::to_vec(request)?),
        None => Ok(Vec::new()),
    }
}

/// Encodes an optional server power action request, matching the platform's own encoding of a
/// null pointer as a literal JSON `null` body rather than an empty one.
fn server_action_body(request: Option<&ServerActionRequest>) -> Result<Vec<u8>> {
    match request {
        Some(request) => Ok(serde_json::to_vec(request)?),
        None => Ok(b"null".to_vec()),
    }
}

const CLOUD_INIT_BOUNDARY: &str = "nars-cloud-init-2f6a1c8e9b4d";

/// Builds the multipart/form-data body for a cloud-init upload.
fn cloud_init_multipart_body(filename: &str, content: &[u8]) -> Vec<u8> {
    let safe_filename = filename.replace('"', "'");
    let mut body = Vec::with_capacity(content.len() + 256);
    body.extend_from_slice(format!("--{CLOUD_INIT_BOUNDARY}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{safe_filename}\"\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(content);
    body.extend_from_slice(format!("\r\n--{CLOUD_INIT_BOUNDARY}--\r\n").as_bytes());
    body
}

/// Decodes the filter-dedicated-devices response, which the platform answers either as a bare
/// array or as a paginated list nested under `devices.paginator.data` alongside column metadata
/// the portal uses for its filter UI.
fn decode_dedicated_devices(value: Value) -> Result<Vec<DedicatedDevice>> {
    if let Ok(devices) = serde_json::from_value::<Vec<DedicatedDevice>>(value.clone()) {
        return Ok(devices);
    }

    #[derive(Deserialize, Default)]
    struct Envelope {
        #[serde(default)]
        devices: DevicesWrapper,
    }
    #[derive(Deserialize, Default)]
    struct DevicesWrapper {
        #[serde(default)]
        paginator: Paginator,
    }
    #[derive(Deserialize, Default)]
    struct Paginator {
        #[serde(default)]
        data: Vec<DedicatedDevice>,
    }

    let envelope: Envelope = serde_json::from_value(value)
        .map_err(|err| Error::Decode(format!("filter dedicated devices: {err}")))?;
    Ok(envelope.devices.paginator.data)
}

fn image_queue_failed_error(queue_id: i64, response: &str) -> Error {
    Error::Api {
        method: "GET".to_string(),
        url: format!("cloud/images/queue_status/{queue_id}"),
        status_code: 0,
        api_code: 0,
        message: format!("image job {queue_id} failed: {response}"),
    }
}
