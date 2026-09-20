use crate::error::{Error, Result};
use crate::models::{
    decode_required, flexible_bool, parse_metric_names, AccountLimit, BgpNeighborAsn,
    BgpNeighborEnabledIpVersion, BgpNeighborRouteMap, BgpNeighborSource, CloudFloatingIpv4,
    CloudFloatingIpv4Vm, CloudNetworkingLocation, HttpLbGroup, HttpLbGroupBackend,
    HttpLbGroupHealthCheck, HttpLbGroupMatch, HttpLbGroupRule, MagicMesh, MeshRouter, MetricNames,
    NkeAccessUrls, NkeAddon, NkeAddonCatalogEntry, NkeCluster, NkeClusterDnsZone, NkeLogEntry,
    NkeWorkerNode, NlbGroup, NlbGroupBackend, NlbGroupHealthCheck, NlbGroupMatch, NlbGroupRule,
    OidcClient, OidcClientAuthLog, OidcClientBareMetalServer, OidcClientChangeLog, OidcClientKey,
    OidcClientVm, PrefixListRule, Router, RouterConfig, RouterDhcpRange, RouterDhcpServer,
    RouterDhcpStaticRoute, RouterIpSecConfig, RouterIpSecEspGroup, RouterIpSecIkeGroup,
    RouterNtpConfig, RouterNtpUpstream, RouterPrefixList, RouterStaticRoute, RouterVrfBgpConfig,
    RouterVrfBgpNeighbor, RouterVrfBgpNetwork, RouterVrfBgpUpdateResult, RouterVrfConfig,
    RouterVrfDhcpConfig, RouterVrfDnatMatch, RouterVrfDnatPriority, RouterVrfDnatRule,
    RouterVrfDnatTranslation, RouterVrfInterface, RouterVrfInterfaceWireguardPeer,
    RouterVrfIpSecOverlayNetwork, RouterVrfIpSecPeer, RouterVrfSnatMatch, RouterVrfSnatPriority,
    RouterVrfSnatRule, RouterVrfSnatTranslation, RouterVrfTunnel, SslCertificate, StaticRouteVia,
    StatisticResult, StorageBlockNamespace, StorageBlockVolume, StorageBucket, StorageLocation,
    StorageObjectStore, StorageType, V3Location, Vpc, VpcBackend, VpcBackendTemplate, VpcDnatMatch,
    VpcDnatPriority, VpcDnatRule, VpcDnatTranslation, VpcFirewallRule, VpcFloatingIp,
    VpcIpReservations, VpcPortRange, VpcSnatMatch, VpcSnatPriority, VpcSnatRule,
    VpcSnatTranslation, VpcSshKey, VpcSshSettings, WireguardPeerAllowedIp,
};
#[cfg(feature = "blocking")]
use crate::transport::ReqwestTransport;
use crate::transport::{api_key_from_env, build_url, redact_url, DynTransport, Method, Request};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

const V3_BASE_ENDPOINT: &str = "https://vapi3.netactuate.com";

/// Total attempts (including the first) made by a call that retries on a transient failure.
const V3_RETRY_ATTEMPTS: u32 = 7;

/// Delay between retry attempts.
const V3_RETRY_DELAY: Duration = Duration::from_secs(10);

/// Interval between polls while waiting for a storage resource to become ready.
const STORAGE_WAIT_INTERVAL: Duration = Duration::from_secs(10);

/// Maximum total time to wait for a storage resource to become ready.
const STORAGE_WAIT_TIMEOUT: Duration = Duration::from_secs(120);

/// Interval between polls while waiting on an NKE cluster or its worker nodes.
const NKE_WAIT_INTERVAL: Duration = Duration::from_secs(60);

/// Maximum total time to wait on an NKE cluster or its worker nodes.
const NKE_WAIT_TIMEOUT: Duration = Duration::from_secs(900);

/// Interval between polls while waiting for a cloud router to finish provisioning.
const ROUTER_WAIT_INTERVAL: Duration = Duration::from_secs(10);

/// Maximum total time to wait for a cloud router to finish provisioning.
const ROUTER_WAIT_TIMEOUT: Duration = Duration::from_secs(600);

/// How long a cloud router build may make no progress before it is treated as stalled. A
/// healthy build completes seven steps in about five minutes, so five minutes without a
/// single step completing means it is stuck, not slow.
const ROUTER_STALL_AFTER: Duration = Duration::from_secs(300);

/// vAPI3 client.
#[derive(Clone)]
pub struct V3Client {
    base_url: Url,
    api_key: String,
    transport: DynTransport,
}

impl V3Client {
    /// Creates a production vAPI3 client with an explicit API key.
    #[cfg(feature = "blocking")]
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_base_url(api_key, "")
    }

    /// Creates a production vAPI3 client from `NETACTUATE_API_KEY`.
    #[cfg(feature = "blocking")]
    pub fn from_env() -> Result<Self> {
        Self::new(api_key_from_env()?)
    }

    /// Creates a vAPI3 client with an explicit base URL. An empty base URL selects production.
    #[cfg(feature = "blocking")]
    pub fn with_base_url(api_key: impl Into<String>, base_url: &str) -> Result<Self> {
        let base = if base_url.is_empty() {
            V3_BASE_ENDPOINT
        } else {
            base_url
        };
        Self::with_transport(api_key, base, Arc::new(ReqwestTransport::new()))
    }

    /// Creates a vAPI3 client with a caller-supplied transport.
    pub fn with_transport(
        api_key: impl Into<String>,
        base_url: &str,
        transport: DynTransport,
    ) -> Result<Self> {
        let base = if base_url.is_empty() {
            V3_BASE_ENDPOINT
        } else {
            base_url
        };
        Ok(Self {
            base_url: Url::parse(base)?,
            api_key: api_key.into(),
            transport,
        })
    }

    /// Lists all VPCs visible to the account.
    pub fn list_vpcs(&self) -> Result<Vec<Vpc>> {
        self.request_list("/vpcs?limit=1000", None)
    }

    /// Lists locations where VPCs can be created.
    pub fn list_vpc_locations(&self) -> Result<Vec<V3Location>> {
        self.request_json(Method::Get, "/vpcs/locations", None)
    }

    /// Gets one VPC by id.
    pub fn get_vpc(&self, id: i64) -> Result<Vpc> {
        self.request_json(Method::Get, &format!("/vpcs/{id}"), None)
    }

    /// Creates a VPC.
    pub fn create_vpc(&self, request: &CreateVpcRequest) -> Result<Vpc> {
        self.request_json(Method::Post, "/vpcs", Some(serde_json::to_vec(request)?))
    }

    /// Updates a VPC.
    pub fn update_vpc(&self, id: i64, request: &UpdateVpcRequest) -> Result<Vpc> {
        self.request_json(
            Method::Patch,
            &format!("/vpcs/{id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a VPC. A VPC that is already gone is treated as success.
    pub fn delete_vpc(&self, id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("/vpcs/{id}"), None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Adds a redundant standby gateway to a VPC.
    pub fn add_vpc_standby_gateway(&self, id: i64) -> Result<()> {
        self.request_empty(Method::Post, &format!("/vpcs/{id}/gateway/standby"), None)
    }

    /// Removes the redundant standby gateway from a VPC. A VPC with no standby gateway is
    /// treated as success.
    pub fn delete_vpc_standby_gateway(&self, id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("/vpcs/{id}/gateway/standby"), None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Returns the gateway, interface and VM IP reservations for a VPC.
    pub fn get_vpc_ip_reservations(&self, id: i64) -> Result<VpcIpReservations> {
        self.request_json(Method::Get, &format!("/vpcs/{id}/ip-reservations"), None)
    }

    /// Returns the DHCP nameservers announced by a VPC.
    ///
    /// The `/vpcs/{id}/dhcp/nameservers` path is PUT only; GET on it answers with a 405. The
    /// current nameservers are read from the VPC object instead, where they arrive under
    /// `dhcp.nameservers` as plain string lists.
    pub fn get_vpc_nameservers(&self, vpc_id: i64) -> Result<VpcNameservers> {
        let value = self.request_value(Method::Get, &format!("/vpcs/{vpc_id}"), None)?;
        Ok(vpc_nameservers_from_dhcp(&value))
    }

    /// Replaces the DHCP nameservers announced by a VPC.
    pub fn replace_vpc_nameservers(
        &self,
        vpc_id: i64,
        request: &ReplaceVpcNameserversRequest,
    ) -> Result<ReplaceVpcNameserversResponse> {
        let path = format!("/vpcs/{vpc_id}/dhcp/nameservers");
        let value = self.request_value(Method::Put, &path, Some(serde_json::to_vec(request)?))?;
        decode_optional_or_default(value, &path)
    }

    /// Updates the DHCP nameservers announced by a VPC.
    pub fn update_vpc_nameservers(
        &self,
        vpc_id: i64,
        request: &VpcNameservers,
    ) -> Result<VpcNameservers> {
        let path = format!("/vpcs/{vpc_id}/dhcp/nameservers");
        let value = self.request_value(Method::Patch, &path, Some(serde_json::to_vec(request)?))?;
        decode_optional_or_default(value, &path)
    }

    /// Returns the bastion SSH settings for a VPC.
    pub fn get_vpc_ssh_settings(&self, id: i64) -> Result<VpcSshSettings> {
        self.request_json(Method::Get, &format!("/vpcs/{id}/ssh"), None)
    }

    /// Updates the bastion SSH settings for a VPC.
    pub fn update_vpc_ssh_settings(
        &self,
        id: i64,
        request: &UpdateVpcSshSettingsRequest,
    ) -> Result<VpcSshSettings> {
        self.request_json(
            Method::Patch,
            &format!("/vpcs/{id}/ssh"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Lists the SSH keys authorized for bastion access to a VPC.
    pub fn list_vpc_ssh_keys(&self, vpc_id: i64) -> Result<Vec<VpcSshKey>> {
        self.request_list(&format!("/vpcs/{vpc_id}/ssh/keys"), None)
    }

    /// Gets one SSH key authorized for bastion access to a VPC.
    ///
    /// There is no single-key endpoint, so this filters the full key list. Returns
    /// [`Error::NotFound`] when no key has this id.
    pub fn get_vpc_ssh_key(&self, vpc_id: i64, ssh_key_id: i64) -> Result<VpcSshKey> {
        let path = format!("/vpcs/{vpc_id}/ssh/keys/{ssh_key_id}");
        self.list_vpc_ssh_keys(vpc_id)?
            .into_iter()
            .find(|key| effective_ssh_key_id(key) == ssh_key_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!("SSH key {ssh_key_id} not found in VPC {vpc_id}"),
            })
    }

    /// Enables or disables an SSH key for bastion access to a VPC.
    pub fn enable_vpc_ssh_key(&self, vpc_id: i64, ssh_key_id: i64, enabled: bool) -> Result<()> {
        let request = EnableVpcSshKeyRequest { enabled };
        self.request_empty(
            Method::Patch,
            &format!("/vpcs/{vpc_id}/ssh/keys/{ssh_key_id}"),
            Some(serde_json::to_vec(&request)?),
        )
    }

    /// Revokes an SSH key's bastion access to a VPC.
    pub fn delete_vpc_ssh_key(&self, vpc_id: i64, ssh_key_id: i64) -> Result<()> {
        self.enable_vpc_ssh_key(vpc_id, ssh_key_id, false)
    }

    /// Creates a floating IP on a VPC and returns its id.
    ///
    /// VPC child services can still be initializing immediately after the VPC itself is
    /// created, so this retries automatically on a transient server error or a "VPC not
    /// ready" response.
    pub fn create_vpc_floating_ip(
        &self,
        vpc_id: i64,
        request: &CreateVpcFloatingIpRequest,
    ) -> Result<i64> {
        let path = format!("/vpcs/{vpc_id}/floating-ips");
        let body = serde_json::to_vec(request)?;
        let response: FloatingIpCreateResponse = retry_while(
            V3_RETRY_ATTEMPTS,
            V3_RETRY_DELAY,
            |err| is_transient_server_error(err) || is_vpc_not_ready_error(err),
            || self.request_json(Method::Post, &path, Some(body.clone())),
        )?;
        Ok(response.floating_ip_id)
    }

    /// Lists the floating IPs assigned to a VPC.
    pub fn list_vpc_floating_ips(&self, vpc_id: i64) -> Result<Vec<VpcFloatingIp>> {
        self.request_list(&format!("/vpcs/{vpc_id}/floating-ips"), None)
    }

    /// Gets one floating IP assigned to a VPC.
    ///
    /// There is no single-floating-IP endpoint, so this filters the full list. Returns
    /// [`Error::NotFound`] when no floating IP has this id.
    pub fn get_vpc_floating_ip(&self, vpc_id: i64, floating_ip_id: i64) -> Result<VpcFloatingIp> {
        let path = format!("/vpcs/{vpc_id}/floating-ips/{floating_ip_id}");
        self.list_vpc_floating_ips(vpc_id)?
            .into_iter()
            .find(|floating_ip| floating_ip.floating_ip_id == floating_ip_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!("floating IP {floating_ip_id} not found in VPC {vpc_id}"),
            })
    }

    /// Updates a floating IP's reverse DNS record.
    pub fn update_vpc_floating_ip(
        &self,
        vpc_id: i64,
        floating_ip_id: i64,
        request: &UpdateVpcFloatingIpRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Patch,
            &format!("/vpcs/{vpc_id}/floating-ips/{floating_ip_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a floating IP. A floating IP that is already gone is treated as success.
    pub fn delete_vpc_floating_ip(&self, vpc_id: i64, floating_ip_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/vpcs/{vpc_id}/floating-ips/{floating_ip_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Creates a firewall rule on a VPC gateway and returns its id.
    pub fn create_vpc_firewall_rule(
        &self,
        vpc_id: i64,
        request: &CreateVpcFirewallRuleRequest,
    ) -> Result<i64> {
        let response: FirewallRuleCreateResponse = self.request_json(
            Method::Post,
            &format!("/vpcs/{vpc_id}/gateway/rules/firewall"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.firewall_rule_id)
    }

    /// Lists every firewall rule on a VPC gateway, across both IP versions.
    pub fn list_vpc_firewall_rules_all(&self, vpc_id: i64) -> Result<Vec<VpcFirewallRule>> {
        self.request_list(&format!("/vpcs/{vpc_id}/gateway/rules/firewall"), None)
    }

    /// Lists the firewall rules on a VPC gateway for one IP version.
    pub fn list_vpc_firewall_rules(
        &self,
        vpc_id: i64,
        ip_version: i64,
    ) -> Result<Vec<VpcFirewallRule>> {
        self.request_list(
            &format!("/vpcs/{vpc_id}/gateway/rules/firewall/ipv{ip_version}"),
            None,
        )
    }

    /// Gets one firewall rule on a VPC gateway.
    ///
    /// There is no single-rule endpoint, so this filters the rule list for the given IP
    /// version. Returns [`Error::NotFound`] when no rule has this id.
    pub fn get_vpc_firewall_rule(
        &self,
        vpc_id: i64,
        rule_id: i64,
        ip_version: i64,
    ) -> Result<VpcFirewallRule> {
        let path = format!("/vpcs/{vpc_id}/gateway/rules/firewall/ipv{ip_version}/{rule_id}");
        self.list_vpc_firewall_rules(vpc_id, ip_version)?
            .into_iter()
            .find(|rule| rule.firewall_rule_id == rule_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!("firewall rule {rule_id} not found in VPC {vpc_id}"),
            })
    }

    /// Updates a firewall rule on a VPC gateway.
    pub fn update_vpc_firewall_rule(
        &self,
        vpc_id: i64,
        rule_id: i64,
        request: &UpdateVpcFirewallRuleRequest,
    ) -> Result<VpcFirewallRule> {
        self.request_json(
            Method::Patch,
            &format!("/vpcs/{vpc_id}/gateway/rules/firewall/{rule_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a firewall rule. A rule that is already gone is treated as success.
    pub fn delete_vpc_firewall_rule(&self, vpc_id: i64, rule_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/vpcs/{vpc_id}/gateway/rules/firewall/{rule_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Applies pending firewall rule changes to a VPC gateway.
    ///
    /// The gateway VM can be briefly unavailable right after VPC creation, so this retries
    /// automatically on a transient server error.
    pub fn apply_vpc_firewall_changes(&self, vpc_id: i64) -> Result<()> {
        let path = format!("/vpcs/{vpc_id}/gateway/rules/firewall/apply-changes");
        retry_while(
            V3_RETRY_ATTEMPTS,
            V3_RETRY_DELAY,
            is_transient_server_error,
            || self.request_empty(Method::Post, &path, None),
        )
    }

    /// Creates a SNAT rule on a VPC gateway.
    pub fn create_vpc_snat_rule(
        &self,
        vpc_id: i64,
        request: &CreateVpcSnatRuleRequest,
    ) -> Result<VpcSnatRule> {
        self.request_json(
            Method::Post,
            &format!("/vpcs/{vpc_id}/gateway/rules/snat"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Lists every SNAT rule on a VPC gateway, across both IP versions.
    pub fn list_vpc_snat_rules_all(&self, vpc_id: i64) -> Result<Vec<VpcSnatRule>> {
        self.request_list(&format!("/vpcs/{vpc_id}/gateway/rules/snat"), None)
    }

    /// Lists the SNAT rules on a VPC gateway for one IP version.
    pub fn list_vpc_snat_rules(&self, vpc_id: i64, ip_version: i64) -> Result<Vec<VpcSnatRule>> {
        self.request_list(
            &format!("/vpcs/{vpc_id}/gateway/rules/snat/ipv{ip_version}"),
            None,
        )
    }

    /// Gets one SNAT rule on a VPC gateway.
    ///
    /// There is no single-rule endpoint, so this filters the rule list for the given IP
    /// version. Returns [`Error::NotFound`] when no rule has this id.
    pub fn get_vpc_snat_rule(
        &self,
        vpc_id: i64,
        rule_id: i64,
        ip_version: i64,
    ) -> Result<VpcSnatRule> {
        let path = format!("/vpcs/{vpc_id}/gateway/rules/snat/ipv{ip_version}/{rule_id}");
        self.list_vpc_snat_rules(vpc_id, ip_version)?
            .into_iter()
            .find(|rule| rule.snat_rule_id == rule_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!("SNAT rule {rule_id} not found in VPC {vpc_id}"),
            })
    }

    /// Updates a SNAT rule on a VPC gateway.
    pub fn update_vpc_snat_rule(
        &self,
        vpc_id: i64,
        rule_id: i64,
        request: &UpdateVpcSnatRuleRequest,
    ) -> Result<VpcSnatRule> {
        self.request_json(
            Method::Patch,
            &format!("/vpcs/{vpc_id}/gateway/rules/snat/{rule_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a SNAT rule. A rule that is already gone is treated as success.
    pub fn delete_vpc_snat_rule(&self, vpc_id: i64, rule_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/vpcs/{vpc_id}/gateway/rules/snat/{rule_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Applies pending SNAT rule changes to a VPC gateway.
    ///
    /// The gateway VM can be briefly unavailable right after VPC creation, so this retries
    /// automatically on a transient server error.
    pub fn apply_vpc_snat_changes(&self, vpc_id: i64) -> Result<()> {
        let path = format!("/vpcs/{vpc_id}/gateway/rules/snat/apply-changes");
        retry_while(
            V3_RETRY_ATTEMPTS,
            V3_RETRY_DELAY,
            is_transient_server_error,
            || self.request_empty(Method::Post, &path, None),
        )
    }

    /// Creates a DNAT rule on a VPC gateway.
    pub fn create_vpc_dnat_rule(
        &self,
        vpc_id: i64,
        request: &CreateVpcDnatRuleRequest,
    ) -> Result<VpcDnatRule> {
        self.request_json(
            Method::Post,
            &format!("/vpcs/{vpc_id}/gateway/rules/dnat"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Lists every DNAT rule on a VPC gateway, across both IP versions.
    pub fn list_vpc_dnat_rules_all(&self, vpc_id: i64) -> Result<Vec<VpcDnatRule>> {
        self.request_list(&format!("/vpcs/{vpc_id}/gateway/rules/dnat"), None)
    }

    /// Lists the DNAT rules on a VPC gateway for one IP version.
    pub fn list_vpc_dnat_rules(&self, vpc_id: i64, ip_version: i64) -> Result<Vec<VpcDnatRule>> {
        self.request_list(
            &format!("/vpcs/{vpc_id}/gateway/rules/dnat/ipv{ip_version}"),
            None,
        )
    }

    /// Gets one DNAT rule on a VPC gateway.
    ///
    /// There is no single-rule endpoint, so this filters the rule list for the given IP
    /// version. Returns [`Error::NotFound`] when no rule has this id.
    pub fn get_vpc_dnat_rule(
        &self,
        vpc_id: i64,
        rule_id: i64,
        ip_version: i64,
    ) -> Result<VpcDnatRule> {
        let path = format!("/vpcs/{vpc_id}/gateway/rules/dnat/ipv{ip_version}/{rule_id}");
        self.list_vpc_dnat_rules(vpc_id, ip_version)?
            .into_iter()
            .find(|rule| rule.dnat_rule_id == rule_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!("DNAT rule {rule_id} not found in VPC {vpc_id}"),
            })
    }

    /// Updates a DNAT rule on a VPC gateway.
    pub fn update_vpc_dnat_rule(
        &self,
        vpc_id: i64,
        rule_id: i64,
        request: &UpdateVpcDnatRuleRequest,
    ) -> Result<VpcDnatRule> {
        self.request_json(
            Method::Patch,
            &format!("/vpcs/{vpc_id}/gateway/rules/dnat/{rule_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a DNAT rule. A rule that is already gone is treated as success.
    pub fn delete_vpc_dnat_rule(&self, vpc_id: i64, rule_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/vpcs/{vpc_id}/gateway/rules/dnat/{rule_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Applies pending DNAT rule changes to a VPC gateway.
    ///
    /// The gateway VM can be briefly unavailable right after VPC creation, so this retries
    /// automatically on a transient server error.
    pub fn apply_vpc_dnat_changes(&self, vpc_id: i64) -> Result<()> {
        let path = format!("/vpcs/{vpc_id}/gateway/rules/dnat/apply-changes");
        retry_while(
            V3_RETRY_ATTEMPTS,
            V3_RETRY_DELAY,
            is_transient_server_error,
            || self.request_empty(Method::Post, &path, None),
        )
    }

    /// Creates a backend template for a VPC.
    pub fn create_vpc_backend_template(
        &self,
        vpc_id: i64,
        request: &CreateVpcBackendTemplateRequest,
    ) -> Result<VpcBackendTemplate> {
        self.request_json(
            Method::Post,
            &format!("/vpcs/{vpc_id}/backend-templates"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Gets one backend template for a VPC.
    pub fn get_vpc_backend_template(
        &self,
        vpc_id: i64,
        template_id: i64,
    ) -> Result<VpcBackendTemplate> {
        self.request_json(
            Method::Get,
            &format!("/vpcs/{vpc_id}/backend-templates/{template_id}"),
            None,
        )
    }

    /// Lists backend templates for a VPC.
    pub fn list_vpc_backend_templates(&self, vpc_id: i64) -> Result<Vec<VpcBackendTemplate>> {
        self.request_list(&format!("/vpcs/{vpc_id}/backend-templates"), None)
    }

    /// Updates a backend template's name or description.
    pub fn update_vpc_backend_template(
        &self,
        vpc_id: i64,
        template_id: i64,
        request: &UpdateVpcBackendTemplateRequest,
    ) -> Result<VpcBackendTemplate> {
        self.request_json(
            Method::Patch,
            &format!("/vpcs/{vpc_id}/backend-templates/{template_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Replaces a backend template's name, description and backend list in one call.
    pub fn replace_vpc_backend_template(
        &self,
        vpc_id: i64,
        template_id: i64,
        request: &ReplaceVpcBackendTemplateRequest,
    ) -> Result<VpcBackendTemplate> {
        self.request_json(
            Method::Put,
            &format!("/vpcs/{vpc_id}/backend-templates/{template_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a backend template. A template that is already gone is treated as success.
    pub fn delete_vpc_backend_template(&self, vpc_id: i64, template_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/vpcs/{vpc_id}/backend-templates/{template_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Adds one backend to a VPC backend template.
    pub fn create_vpc_backend(
        &self,
        vpc_id: i64,
        template_id: i64,
        request: &CreateVpcBackendRequest,
    ) -> Result<VpcBackend> {
        self.request_json(
            Method::Post,
            &format!("/vpcs/{vpc_id}/backend-templates/{template_id}/backends"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Replaces every backend registered under a VPC backend template.
    ///
    /// The endpoint answers with an object carrying the backend list under `backendHosts`,
    /// not a bare array, so that wrapper is decoded explicitly here rather than through the
    /// generic list machinery.
    pub fn replace_vpc_backends(
        &self,
        vpc_id: i64,
        template_id: i64,
        request: &ReplaceVpcBackendsRequest,
    ) -> Result<Vec<VpcBackend>> {
        let wrapper: VpcBackendsWrapper = self.request_json(
            Method::Put,
            &format!("/vpcs/{vpc_id}/backend-templates/{template_id}/backends"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(wrapper.backend_hosts)
    }

    /// Lists the backends registered under a VPC backend template.
    pub fn list_vpc_backends(&self, vpc_id: i64, template_id: i64) -> Result<Vec<VpcBackend>> {
        self.request_list(
            &format!("/vpcs/{vpc_id}/backend-templates/{template_id}/backends"),
            None,
        )
    }

    /// Gets one backend from a VPC backend template.
    ///
    /// There is no single-backend endpoint, so this filters the full backend list. Returns
    /// [`Error::NotFound`] when no backend has this id.
    pub fn get_vpc_backend(
        &self,
        vpc_id: i64,
        template_id: i64,
        backend_id: i64,
    ) -> Result<VpcBackend> {
        let path = format!("/vpcs/{vpc_id}/backend-templates/{template_id}/backends/{backend_id}");
        self.list_vpc_backends(vpc_id, template_id)?
            .into_iter()
            .find(|backend| backend.backend_host_id == backend_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!(
                    "backend {backend_id} not found in template {template_id} VPC {vpc_id}"
                ),
            })
    }

    /// Updates one backend's name or address.
    pub fn update_vpc_backend(
        &self,
        vpc_id: i64,
        template_id: i64,
        backend_id: i64,
        request: &UpdateVpcBackendRequest,
    ) -> Result<VpcBackend> {
        self.request_json(
            Method::Patch,
            &format!("/vpcs/{vpc_id}/backend-templates/{template_id}/backends/{backend_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes one backend. A backend that is already gone is treated as success.
    pub fn delete_vpc_backend(&self, vpc_id: i64, template_id: i64, backend_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/vpcs/{vpc_id}/backend-templates/{template_id}/backends/{backend_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists the storage types available to the account.
    ///
    /// The endpoint answers with a plain array, a paginated envelope, or a map keyed by type
    /// code depending on the account and platform version, so this normalizes all three
    /// shapes rather than committing to one.
    pub fn list_storage_types(&self) -> Result<Vec<StorageType>> {
        let value = self.request_value(Method::Get, "/storage", None)?;
        decode_storage_types(value)
    }

    /// Lists storage buckets.
    pub fn list_storage_buckets(&self) -> Result<Vec<StorageBucket>> {
        self.request_list("/storage/buckets?limit=1000", None)
    }

    /// Gets a storage bucket.
    pub fn get_storage_bucket(&self, id: i64) -> Result<StorageBucket> {
        self.request_json(Method::Get, &format!("/storage/buckets/{id}"), None)
    }

    /// Creates a storage bucket and returns the new bucket id.
    pub fn create_storage_bucket(&self, request: &CreateStorageBucketRequest) -> Result<i64> {
        let response: StorageCreateResponse = self.request_json(
            Method::Post,
            "/storage/buckets",
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.bucket_id)
    }

    /// Updates a storage bucket.
    pub fn update_storage_bucket(
        &self,
        id: i64,
        request: &UpdateStorageBucketRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Patch,
            &format!("/storage/buckets/{id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a storage bucket. A bucket that is already gone is treated as success.
    pub fn delete_storage_bucket(&self, id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("/storage/buckets/{id}"), None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Converts a bucket into an object store and returns the object store id.
    pub fn convert_storage_bucket_to_store(&self, id: i64) -> Result<i64> {
        let response: StorageCreateResponse = self.request_json(
            Method::Post,
            &format!("/storage/buckets/{id}/convert-to-store"),
            None,
        )?;
        Ok(response.object_store_id)
    }

    /// Polls a storage bucket until it reports ready.
    pub fn wait_for_storage_bucket_ready(&self, id: i64) -> Result<()> {
        wait_for_ready(STORAGE_WAIT_INTERVAL, STORAGE_WAIT_TIMEOUT, || {
            Ok(self.get_storage_bucket(id)?.metadata.ready)
        })
    }

    /// Creates an object store and returns its id.
    pub fn create_storage_object_store(
        &self,
        request: &CreateStorageObjectStoreRequest,
    ) -> Result<i64> {
        let response: StorageCreateResponse = self.request_json(
            Method::Post,
            "/storage/object-stores",
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.object_store_id)
    }

    /// Lists object stores.
    pub fn list_storage_object_stores(&self) -> Result<Vec<StorageObjectStore>> {
        self.request_list("/storage/object-stores?limit=1000", None)
    }

    /// Gets an object store.
    pub fn get_storage_object_store(&self, id: i64) -> Result<StorageObjectStore> {
        self.request_json(Method::Get, &format!("/storage/object-stores/{id}"), None)
    }

    /// Updates an object store.
    pub fn update_storage_object_store(
        &self,
        id: i64,
        request: &UpdateStorageObjectStoreRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Patch,
            &format!("/storage/object-stores/{id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes an object store. A store that is already gone is treated as success.
    pub fn delete_storage_object_store(&self, id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/storage/object-stores/{id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Polls an object store until it reports ready.
    pub fn wait_for_storage_object_store_ready(&self, id: i64) -> Result<()> {
        wait_for_ready(STORAGE_WAIT_INTERVAL, STORAGE_WAIT_TIMEOUT, || {
            Ok(self.get_storage_object_store(id)?.metadata.ready)
        })
    }

    /// Creates a block storage namespace and returns its id.
    pub fn create_storage_block_namespace(
        &self,
        request: &CreateStorageBlockNamespaceRequest,
    ) -> Result<i64> {
        let response: StorageCreateResponse = self.request_json(
            Method::Post,
            "/storage/block-namespaces",
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.block_namespace_id)
    }

    /// Lists block storage namespaces.
    pub fn list_storage_block_namespaces(&self) -> Result<Vec<StorageBlockNamespace>> {
        self.request_list("/storage/block-namespaces?limit=1000", None)
    }

    /// Gets a block storage namespace.
    pub fn get_storage_block_namespace(&self, id: i64) -> Result<StorageBlockNamespace> {
        self.request_json(
            Method::Get,
            &format!("/storage/block-namespaces/{id}"),
            None,
        )
    }

    /// Updates a block storage namespace.
    pub fn update_storage_block_namespace(
        &self,
        id: i64,
        request: &UpdateStorageBlockNamespaceRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Patch,
            &format!("/storage/block-namespaces/{id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a block storage namespace. A namespace that is already gone is treated as
    /// success.
    pub fn delete_storage_block_namespace(&self, id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/storage/block-namespaces/{id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Polls a block storage namespace until it reports ready.
    pub fn wait_for_storage_block_namespace_ready(&self, id: i64) -> Result<()> {
        wait_for_ready(STORAGE_WAIT_INTERVAL, STORAGE_WAIT_TIMEOUT, || {
            Ok(self.get_storage_block_namespace(id)?.metadata.ready)
        })
    }

    /// Creates a block volume and returns its id.
    pub fn create_storage_block_volume(
        &self,
        request: &CreateStorageBlockVolumeRequest,
    ) -> Result<i64> {
        let response: StorageCreateResponse = self.request_json(
            Method::Post,
            "/storage/block-volumes",
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.block_volume_id)
    }

    /// Lists every block volume on the account.
    pub fn list_storage_block_volumes(&self) -> Result<Vec<StorageBlockVolume>> {
        self.request_list("/storage/block-volumes", None)
    }

    /// Gets one block volume.
    ///
    /// A platform defect on this endpoint can return object-store shaped metadata, with no
    /// `blockVolumeId` at all, for an id that otherwise resolves correctly (the label and
    /// location in the response match the volume that was asked for). When that happens this
    /// fills in the id that was requested rather than surfacing a zero id to the caller. Once
    /// the platform fixes the endpoint this fallback simply stops firing.
    pub fn get_storage_block_volume(&self, id: i64) -> Result<StorageBlockVolume> {
        let mut volume: StorageBlockVolume =
            self.request_json(Method::Get, &format!("/storage/block-volumes/{id}"), None)?;
        if volume.metadata.block_volume_id == 0 {
            volume.metadata.block_volume_id = id;
        }
        Ok(volume)
    }

    /// Updates a block volume.
    pub fn update_storage_block_volume(
        &self,
        id: i64,
        request: &UpdateStorageBlockVolumeRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Patch,
            &format!("/storage/block-volumes/{id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a block volume. A volume that is already gone is treated as success.
    pub fn delete_storage_block_volume(&self, id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/storage/block-volumes/{id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Polls a block volume until it reports ready.
    pub fn wait_for_storage_block_volume_ready(&self, id: i64) -> Result<()> {
        wait_for_ready(STORAGE_WAIT_INTERVAL, STORAGE_WAIT_TIMEOUT, || {
            Ok(self.get_storage_block_volume(id)?.metadata.ready)
        })
    }

    /// Lists the locations where storage resources can be created, with the hardware class
    /// offered at each one.
    pub fn list_storage_locations(&self) -> Result<Vec<StorageLocation>> {
        self.request_json(Method::Get, "/storage/locations", None)
    }

    /// Lists available NKE versions.
    pub fn list_nke_versions(&self) -> Result<Vec<String>> {
        self.request_json(Method::Get, "/nke/versions", None)
    }

    /// Lists NKE clusters.
    pub fn list_nke_clusters(&self) -> Result<Vec<NkeCluster>> {
        self.request_list("/nke/clusters", None)
    }

    /// Gets an NKE cluster.
    pub fn get_nke_cluster(&self, id: i64) -> Result<NkeCluster> {
        self.request_json(Method::Get, &format!("/nke/clusters/{id}"), None)
    }

    /// Creates an NKE cluster and returns the new cluster id.
    pub fn create_nke_cluster(&self, request: &CreateNkeClusterRequest) -> Result<i64> {
        let response: NkeCreateResponse = self.request_json(
            Method::Post,
            "/nke/clusters",
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.cluster_id)
    }

    /// Updates an NKE cluster.
    pub fn update_nke_cluster(&self, id: i64, request: &UpdateNkeClusterRequest) -> Result<()> {
        self.request_empty(
            Method::Patch,
            &format!("/nke/clusters/{id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes an NKE cluster. A cluster that is already gone is treated as success.
    pub fn delete_nke_cluster(&self, id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("/nke/clusters/{id}"), None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Generates a kubeconfig for an NKE cluster.
    pub fn generate_nke_kubeconfig(&self, id: i64, expiration_seconds: i64) -> Result<String> {
        let body = serde_json::json!({ "expirationSeconds": expiration_seconds });
        let raw = self.request_value(
            Method::Post,
            &format!("/nke/clusters/{id}/kubeconfig"),
            Some(serde_json::to_vec(&body)?),
        )?;
        Ok(raw
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| raw.to_string()))
    }

    /// Creates access URLs for an NKE cluster.
    pub fn create_nke_access_urls(&self, id: i64) -> Result<NkeAccessUrls> {
        self.request_json(
            Method::Post,
            &format!("/nke/clusters/{id}/create-access-urls"),
            None,
        )
    }

    /// Lists logs for an NKE cluster.
    pub fn list_nke_cluster_logs(&self, id: i64) -> Result<Vec<NkeLogEntry>> {
        self.request_list(&format!("/nke/clusters/{id}/logs"), None)
    }

    /// Lists worker nodes for an NKE cluster.
    pub fn list_nke_worker_nodes(&self, cluster_id: i64) -> Result<Vec<NkeWorkerNode>> {
        self.request_list(&format!("/nke/clusters/{cluster_id}/worker-nodes"), None)
    }

    /// Gets a worker node.
    pub fn get_nke_worker_node(&self, cluster_id: i64, node_id: i64) -> Result<NkeWorkerNode> {
        self.request_json(
            Method::Get,
            &format!("/nke/clusters/{cluster_id}/worker-nodes/{node_id}"),
            None,
        )
    }

    /// Updates worker node metadata.
    pub fn update_nke_worker_node(
        &self,
        cluster_id: i64,
        node_id: i64,
        request: &UpdateNkeWorkerNodeRequest,
    ) -> Result<NkeWorkerNode> {
        self.request_json(
            Method::Patch,
            &format!("/nke/clusters/{cluster_id}/worker-nodes/{node_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a worker node. A node that is already gone is treated as success.
    pub fn delete_nke_worker_node(&self, cluster_id: i64, node_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/nke/clusters/{cluster_id}/worker-nodes/{node_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Waits until an NKE cluster reports the `Healthy` status.
    ///
    /// Returns an error as soon as the cluster reports `Failed` or `Error` rather than waiting
    /// out the full timeout on a cluster that has already given up.
    pub fn wait_for_nke_cluster_healthy(&self, id: i64) -> Result<()> {
        wait_for_ready(NKE_WAIT_INTERVAL, NKE_WAIT_TIMEOUT, || {
            let cluster = self.get_nke_cluster(id)?;
            match cluster.status.cluster.as_str() {
                "Failed" | "Error" => Err(nke_cluster_failed_error(id, &cluster.status.cluster)),
                "Healthy" => Ok(true),
                _ => Ok(false),
            }
        })
    }

    /// Waits until an NKE cluster lists at least `minimum` worker nodes.
    ///
    /// A cluster reports `Healthy` before its worker nodes appear in the worker node listing,
    /// so a caller that reads the nodes as soon as creation returns can see an empty list for a
    /// cluster that is about to have several. Waiting on the nodes themselves closes that
    /// window. A non-positive `minimum` returns immediately.
    pub fn wait_for_nke_worker_nodes(&self, cluster_id: i64, minimum: i64) -> Result<()> {
        if minimum <= 0 {
            return Ok(());
        }
        wait_for_ready(NKE_WAIT_INTERVAL, NKE_WAIT_TIMEOUT, || {
            let nodes = self.list_nke_worker_nodes(cluster_id)?;
            Ok(nodes.len() as i64 >= minimum)
        })
    }

    /// Lists the add-ons available to install on NKE clusters.
    pub fn list_nke_addon_catalog(&self) -> Result<Vec<NkeAddonCatalogEntry>> {
        self.request_list("/nke/addons", None)
    }

    /// Lists the add-ons installed on an NKE cluster.
    pub fn list_nke_cluster_addons(&self, cluster_id: i64) -> Result<Vec<NkeAddon>> {
        self.request_list(&format!("/nke/clusters/{cluster_id}/addons"), None)
    }

    /// Gets one add-on installed on an NKE cluster.
    pub fn get_nke_cluster_addon(&self, cluster_id: i64, addon_type: &str) -> Result<NkeAddon> {
        self.request_json(
            Method::Get,
            &format!("/nke/clusters/{cluster_id}/addons/{addon_type}"),
            None,
        )
    }

    /// Installs an add-on on an NKE cluster.
    pub fn create_nke_cluster_addon(
        &self,
        cluster_id: i64,
        request: &CreateNkeAddonRequest,
    ) -> Result<NkeAddon> {
        self.request_json(
            Method::Post,
            &format!("/nke/clusters/{cluster_id}/addons"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Updates an add-on installed on an NKE cluster.
    pub fn update_nke_cluster_addon(
        &self,
        cluster_id: i64,
        addon_type: &str,
        request: &UpdateNkeAddonRequest,
    ) -> Result<NkeAddon> {
        self.request_json(
            Method::Patch,
            &format!("/nke/clusters/{cluster_id}/addons/{addon_type}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Removes an add-on from an NKE cluster. An add-on that is already gone is treated as
    /// success.
    pub fn delete_nke_cluster_addon(&self, cluster_id: i64, addon_type: &str) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/nke/clusters/{cluster_id}/addons/{addon_type}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists the DNS zones attached to an NKE cluster through the `netactuate-dns` add-on.
    pub fn list_nke_cluster_dns_zones(&self, cluster_id: i64) -> Result<Vec<NkeClusterDnsZone>> {
        self.request_list(&format!("/nke/clusters/{cluster_id}/dns-zones"), None)
    }

    /// Creates an OIDC client and returns its id.
    pub fn create_oidc_client(&self, request: &CreateOidcClientRequest) -> Result<i64> {
        let response: OidcClientCreateResponse = self.request_json(
            Method::Post,
            "/oidc/clients",
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.client_id)
    }

    /// Lists the OIDC clients configured on the account.
    pub fn list_oidc_clients(&self) -> Result<Vec<OidcClient>> {
        self.list_oidc_clients_page("/oidc/clients?limit=1000")
    }

    fn list_oidc_clients_page(&self, path: &str) -> Result<Vec<OidcClient>> {
        let value = self.request_value(Method::Get, path, None)?;
        let envelope: OidcClientsEnvelope = serde_json::from_value(value)
            .map_err(|err| Error::Decode(format!("OIDC clients response: {err}")))?;
        let tenant = tenant_id_as_string(&envelope.tenant);
        let page = parse_v3_list(envelope.clients)?;
        let rows: Vec<OidcClientRow> = serde_json::from_value(Value::Array(page.rows))
            .map_err(|err| Error::Decode(format!("OIDC clients rows: {err}")))?;
        let mut clients: Vec<OidcClient> = rows
            .into_iter()
            .map(|row| row.into_client(&tenant))
            .collect();

        if page.meta.total > 0
            && page.meta.limit > 0
            && page.meta.offset + page.meta.limit < page.meta.total
        {
            let next_path =
                v3_list_page_path(path, page.meta.offset + page.meta.limit, page.meta.limit);
            clients.extend(self.list_oidc_clients_page(&next_path)?);
        }
        Ok(clients)
    }

    /// Gets one OIDC client by id, including its keys, authentication logs and change logs.
    ///
    /// The single-client endpoint does not return the account-default, audience, TTL, allow
    /// list enforcement or tenant fields, so this also fetches the full client list to fill
    /// those in. Returns [`Error::NotFound`] when no client in the list has this id.
    pub fn get_oidc_client(&self, client_id: i64) -> Result<OidcClient> {
        let path = format!("/oidc/clients/{client_id}");
        let value = self.request_value(Method::Get, &path, None)?;
        let detail: OidcClientDetail = serde_json::from_value(value)
            .map_err(|err| Error::Decode(format!("OIDC client {client_id} response: {err}")))?;

        let mut client = self
            .list_oidc_clients()?
            .into_iter()
            .find(|client| client.client_id == client_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!("OIDC client {client_id} not found in client list"),
            })?;

        client.created_on = detail.metadata.created_on;
        client.last_used_on = detail.metadata.last_used_on;
        client.label = detail.metadata.label;
        client.description = detail.metadata.description;
        client.jwks_uri = detail.metadata.jwks_uri;
        client.keys = decode_oidc_sub_list(detail.keys, client_id, "keys")?;
        client.auth_logs = decode_oidc_sub_list(detail.logs.auth, client_id, "auth logs")?;
        client.change_logs = decode_oidc_sub_list(detail.logs.changes, client_id, "change logs")?;
        Ok(client)
    }

    /// Updates an OIDC client's configuration.
    pub fn update_oidc_client(
        &self,
        client_id: i64,
        request: &UpdateOidcClientRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Patch,
            &format!("/oidc/clients/{client_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes an OIDC client. A client that is already gone is treated as success.
    pub fn delete_oidc_client(&self, client_id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("/oidc/clients/{client_id}"), None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Adds one or more public keys to an OIDC client.
    pub fn create_oidc_client_keys(
        &self,
        client_id: i64,
        keys: &[CreateOidcClientKeyRequest],
    ) -> Result<Vec<OidcClientKey>> {
        let body = CreateOidcClientKeysBody { keys };
        let response: CreateOidcClientKeysResponse = self.request_json(
            Method::Post,
            &format!("/oidc/clients/{client_id}/keys"),
            Some(serde_json::to_vec(&body)?),
        )?;
        Ok(response.keys)
    }

    /// Lists the public keys registered on an OIDC client.
    pub fn list_oidc_client_keys(&self, client_id: i64) -> Result<Vec<OidcClientKey>> {
        self.request_list(
            &format!("/oidc/clients/{client_id}/keys?limit=1000"),
            Some("keys"),
        )
    }

    /// Updates a public key's label and description on an OIDC client.
    pub fn update_oidc_client_key(
        &self,
        client_id: i64,
        key_id: i64,
        request: &UpdateOidcClientKeyRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Patch,
            &format!("/oidc/clients/{client_id}/keys/{key_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Revokes a public key on an OIDC client.
    ///
    /// Deleting a key revokes it. Revoking an already-revoked key answers with a 400 saying so;
    /// this treats that response the same as not found, so the delete is idempotent.
    pub fn delete_oidc_client_key(&self, client_id: i64, key_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/oidc/clients/{client_id}/keys/{key_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() || is_oidc_key_already_revoked_error(&err) => Ok(()),
            result => result,
        }
    }

    /// Grants virtual machines access to an OIDC client's allow list.
    pub fn add_oidc_client_vms(&self, client_id: i64, mbpkgids: &[i64]) -> Result<()> {
        let body = AddOidcClientVmsBody {
            vms: mbpkgids
                .iter()
                .map(|&mbpkgid| OidcClientMbpkgRef { mbpkgid })
                .collect(),
        };
        self.request_empty(
            Method::Post,
            &format!("/oidc/clients/{client_id}/allow-list/vms"),
            Some(serde_json::to_vec(&body)?),
        )
    }

    /// Grants bare metal servers access to an OIDC client's allow list.
    pub fn add_oidc_client_bare_metal_servers(
        &self,
        client_id: i64,
        mbpkgids: &[i64],
    ) -> Result<()> {
        let body = AddOidcClientBareMetalServersBody {
            servers: mbpkgids
                .iter()
                .map(|&mbpkgid| OidcClientMbpkgRef { mbpkgid })
                .collect(),
        };
        self.request_empty(
            Method::Post,
            &format!("/oidc/clients/{client_id}/allow-list/bare-metal"),
            Some(serde_json::to_vec(&body)?),
        )
    }

    /// Revokes a virtual machine's access to an OIDC client's allow list. A VM that is already
    /// off the allow list is treated as success.
    pub fn remove_oidc_client_vm(&self, client_id: i64, mbpkgid: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/oidc/clients/{client_id}/allow-list/vms/{mbpkgid}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Revokes a bare metal server's access to an OIDC client's allow list. A server that is
    /// already off the allow list is treated as success.
    pub fn remove_oidc_client_bare_metal_server(&self, client_id: i64, mbpkgid: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/oidc/clients/{client_id}/allow-list/bare-metal/{mbpkgid}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists the virtual machines allowed to reach an OIDC client.
    pub fn list_oidc_client_vms(&self, client_id: i64) -> Result<Vec<OidcClientVm>> {
        let value = self.request_value(
            Method::Get,
            &format!("/oidc/clients/{client_id}/allow-list/vms"),
            None,
        )?;
        let wrapped: OidcClientVmsResponse = serde_json::from_value(value)
            .map_err(|err| Error::Decode(format!("OIDC client {client_id} VMs: {err}")))?;
        Ok(wrapped.vms)
    }

    /// Lists the bare metal servers allowed to reach an OIDC client.
    pub fn list_oidc_client_bare_metal_servers(
        &self,
        client_id: i64,
    ) -> Result<Vec<OidcClientBareMetalServer>> {
        let value = self.request_value(
            Method::Get,
            &format!("/oidc/clients/{client_id}/allow-list/bare-metal"),
            None,
        )?;
        let wrapped: OidcClientBareMetalServersResponse =
            serde_json::from_value(value).map_err(|err| {
                Error::Decode(format!("OIDC client {client_id} bare metal servers: {err}"))
            })?;
        Ok(wrapped.servers)
    }

    /// Lists the authentication log entries recorded for an OIDC client.
    pub fn list_oidc_client_auth_logs(&self, client_id: i64) -> Result<Vec<OidcClientAuthLog>> {
        self.request_list(
            &format!("/oidc/clients/{client_id}/auth-logs?limit=1000"),
            Some("logs"),
        )
    }

    /// Lists the change log entries recorded for an OIDC client.
    pub fn list_oidc_client_change_logs(&self, client_id: i64) -> Result<Vec<OidcClientChangeLog>> {
        self.request_list(
            &format!("/oidc/clients/{client_id}/change-logs?limit=1000"),
            Some("logs"),
        )
    }

    /// Lists the floating IPv4 addresses allocated to the account.
    pub fn list_cloud_floating_ipv4(&self) -> Result<Vec<CloudFloatingIpv4>> {
        self.request_list("/cloud/networking/floating-ips/ipv4", None)
    }

    /// Allocates a floating IPv4 address to the account.
    pub fn create_cloud_floating_ipv4(
        &self,
        request: &CreateCloudFloatingIpv4Request,
    ) -> Result<CloudFloatingIpv4> {
        self.request_json(
            Method::Post,
            "/cloud/networking/floating-ips/ipv4",
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a floating IPv4 address. An address that is already gone is treated as success.
    pub fn delete_cloud_floating_ipv4(&self, floating_ipv4_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/cloud/networking/floating-ips/ipv4/{floating_ipv4_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists the virtual machines allowed to use a floating IPv4 address.
    pub fn list_cloud_floating_ipv4_vms(
        &self,
        floating_ipv4_id: i64,
    ) -> Result<Vec<CloudFloatingIpv4Vm>> {
        self.request_list(
            &format!("/cloud/networking/floating-ips/ipv4/{floating_ipv4_id}/vms"),
            None,
        )
    }

    /// Grants virtual machines access to a floating IPv4 address.
    pub fn grant_cloud_floating_ipv4_vms(
        &self,
        floating_ipv4_id: i64,
        request: &GrantCloudFloatingIpv4VmsRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("/cloud/networking/floating-ips/ipv4/{floating_ipv4_id}/vms/mass-grant"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Revokes virtual machines' access to a floating IPv4 address.
    pub fn revoke_cloud_floating_ipv4_vms(
        &self,
        floating_ipv4_id: i64,
        request: &RevokeCloudFloatingIpv4VmsRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("/cloud/networking/floating-ips/ipv4/{floating_ipv4_id}/vms/mass-revoke"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Lists cloud locations mapped to their datacenters, for networking purposes.
    pub fn list_cloud_networking_locations(&self) -> Result<Vec<CloudNetworkingLocation>> {
        self.request_json(Method::Get, "/cloud/networking/locations", None)
    }

    /// Lists all magic meshes visible to the account.
    pub fn list_magic_meshes(&self) -> Result<Vec<MagicMesh>> {
        self.request_list("/cloud-routing/meshes?limit=1000", None)
    }

    /// Creates a magic mesh and returns the new mesh's id.
    pub fn create_magic_mesh(&self, request: &CreateMagicMeshRequest) -> Result<i64> {
        let response: MeshCreateResponse = self.request_json(
            Method::Post,
            "/cloud-routing/meshes",
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.mesh_id)
    }

    /// Gets a magic mesh by id.
    pub fn get_magic_mesh(&self, mesh_id: i64) -> Result<MagicMesh> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/meshes/{mesh_id}"),
            None,
        )
    }

    /// Updates a magic mesh's name or description.
    pub fn update_magic_mesh(&self, mesh_id: i64, request: &UpdateMagicMeshRequest) -> Result<()> {
        self.request_empty(
            Method::Patch,
            &format!("/cloud-routing/meshes/{mesh_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a magic mesh. A mesh that is already gone is treated as success.
    pub fn delete_magic_mesh(&self, mesh_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/cloud-routing/meshes/{mesh_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists the routers attached to a magic mesh.
    pub fn list_mesh_routers(&self, mesh_id: i64) -> Result<Vec<MeshRouter>> {
        self.request_list(&format!("/cloud-routing/meshes/{mesh_id}/routers"), None)
    }

    /// Adds a router to a magic mesh.
    pub fn add_router_to_mesh(&self, mesh_id: i64, request: &AddMeshRouterRequest) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("/cloud-routing/meshes/{mesh_id}/routers"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Removes a router from a magic mesh. A router already removed is treated as success.
    pub fn remove_router_from_mesh(&self, mesh_id: i64, router_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/cloud-routing/meshes/{mesh_id}/routers/{router_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Lists the cloud routers visible to the account.
    pub fn list_routers(&self) -> Result<Vec<Router>> {
        self.request_list("/cloud-routing/routers?limit=1000", None)
    }

    /// Gets a cloud router by id.
    pub fn get_router(&self, router_id: i64) -> Result<Router> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}"),
            None,
        )
    }

    /// Gets a cloud router's full configuration, including its VRFs.
    pub fn get_router_config(&self, router_id: i64) -> Result<RouterConfig> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}/config"),
            None,
        )
    }

    /// Lists a router's interface configuration.
    ///
    /// The interface schema is not yet stable, so this returns the raw decoded response.
    pub fn list_router_config_interfaces(&self, router_id: i64) -> Result<Value> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}/config/interfaces"),
            None,
        )
    }

    /// Marks the platform's cached configuration for a router as stale, forcing the next
    /// read to recompute it.
    pub fn invalidate_router_config_cache(&self, router_id: i64) -> Result<()> {
        self.request_empty(
            Method::Post,
            &format!("/cloud-routing/routers/{router_id}/config/invalidate-cache"),
            None,
        )
    }

    /// Creates a cloud router and returns its id.
    ///
    /// The new router is not immediately usable; poll [`Self::wait_for_router_ready`] before
    /// configuring it further.
    pub fn create_router(&self, request: &CreateRouterRequest) -> Result<i64> {
        let response: RouterCreateResponse = self.request_json(
            Method::Post,
            "/cloud-routing/routers",
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.router_id)
    }

    /// Updates a cloud router's name or description.
    pub fn update_router(&self, router_id: i64, request: &UpdateRouterRequest) -> Result<Router> {
        self.request_json(
            Method::Patch,
            &format!("/cloud-routing/routers/{router_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a cloud router.
    pub fn delete_router(&self, router_id: i64) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!("/cloud-routing/routers/{router_id}"),
            None,
        )
    }

    /// Waits for a cloud router to finish provisioning, using the default timeout.
    pub fn wait_for_router_ready(&self, router_id: i64) -> Result<()> {
        self.wait_for_router_ready_timeout(router_id, ROUTER_WAIT_TIMEOUT)
    }

    /// Waits up to `timeout` for a cloud router to finish provisioning. A non-positive
    /// timeout falls back to the default.
    ///
    /// A cloud router build reports a fixed sequence of timestamped steps, and a healthy
    /// build completes all of them in about five minutes. A stalled build never fails and
    /// never sets its ready timestamp: the remaining steps simply keep no completion date,
    /// which makes a stalled build indistinguishable from a slow one to anything that only
    /// watches the clock. This tracks how many steps have completed and gives up naming the
    /// stuck step once none complete within five minutes, rather than waiting out
    /// the full timeout on a build that has already given up.
    pub fn wait_for_router_ready_timeout(&self, router_id: i64, timeout: Duration) -> Result<()> {
        let timeout = if timeout.is_zero() {
            ROUTER_WAIT_TIMEOUT
        } else {
            timeout
        };
        let deadline = std::time::Instant::now() + timeout;
        let mut steps_done = 0usize;
        let mut last_progress = std::time::Instant::now();

        loop {
            let router = self.get_router(router_id)?;
            if router.ready_on.is_some() {
                return Ok(());
            }

            let mut done = 0usize;
            let mut pending = String::new();
            for event in &router.build {
                if event.date.is_some() {
                    done += 1;
                } else if pending.is_empty() {
                    pending = event.text.clone();
                }
            }
            if done > steps_done {
                steps_done = done;
                last_progress = std::time::Instant::now();
            }

            let stalled = last_progress.elapsed();
            if stalled > ROUTER_STALL_AFTER {
                return Err(Error::Timeout(format!(
                    "router {router_id} build has made no progress for {stalled:?}: {done} of \
                     {total} steps complete, stuck on {pending:?}. A healthy build finishes in \
                     about five minutes, so this is a stalled build rather than a slow one",
                    total = router.build.len(),
                )));
            }
            if std::time::Instant::now() >= deadline {
                return Err(Error::Timeout(format!(
                    "router {router_id} did not become ready within {timeout:?}"
                )));
            }

            std::thread::sleep(ROUTER_WAIT_INTERVAL);
        }
    }

    /// Lists a router's VRFs, keyed by VRF id.
    pub fn list_router_vrfs(&self, router_id: i64) -> Result<BTreeMap<String, RouterVrfConfig>> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs"),
            None,
        )
    }

    /// Creates a VRF on a router and returns its id.
    pub fn create_router_vrf(
        &self,
        router_id: i64,
        request: &CreateRouterVrfRequest,
    ) -> Result<i64> {
        let response: RouterVrfIdResponse = self.request_json(
            Method::Post,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.vrf_id)
    }

    /// Gets one VRF on a router.
    pub fn get_router_vrf(&self, router_id: i64, vrf_id: i64) -> Result<RouterVrfConfig> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}"),
            None,
        )
    }

    /// Updates a VRF's name or description and returns its id.
    pub fn update_router_vrf(
        &self,
        router_id: i64,
        vrf_id: i64,
        request: &UpdateRouterVrfRequest,
    ) -> Result<i64> {
        let response: RouterVrfIdResponse = self.request_json(
            Method::Put,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.vrf_id)
    }

    /// Deletes a VRF from a router.
    pub fn delete_router_vrf(&self, router_id: i64, vrf_id: i64) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}"),
            None,
        )
    }

    /// Gets a VRF's BGP configuration.
    pub fn get_router_vrf_bgp(&self, router_id: i64, vrf_id: i64) -> Result<RouterVrfBgpConfig> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/bgp"),
            None,
        )
    }

    /// Updates a VRF's BGP configuration.
    pub fn update_router_vrf_bgp(
        &self,
        router_id: i64,
        vrf_id: i64,
        request: &UpdateRouterVrfBgpRequest,
    ) -> Result<RouterVrfBgpUpdateResult> {
        self.request_json(
            Method::Put,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/bgp"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Lists the BGP neighbors configured on a router VRF.
    pub fn list_router_vrf_bgp_neighbors(
        &self,
        router_id: i64,
        vrf_id: i64,
    ) -> Result<Vec<RouterVrfBgpNeighbor>> {
        self.request_list(
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/bgp/neighbors"),
            None,
        )
    }

    /// Creates a BGP neighbor on a router VRF and returns its id.
    pub fn create_router_vrf_bgp_neighbor(
        &self,
        router_id: i64,
        vrf_id: i64,
        request: &CreateRouterVrfBgpNeighborRequest,
    ) -> Result<i64> {
        let response: RouterVrfBgpNeighborIdResponse = self.request_json(
            Method::Post,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/bgp/neighbors"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.neighbor_id)
    }

    /// Gets one BGP neighbor configured on a router VRF.
    pub fn get_router_vrf_bgp_neighbor(
        &self,
        router_id: i64,
        vrf_id: i64,
        neighbor_id: i64,
    ) -> Result<RouterVrfBgpNeighbor> {
        self.request_json(
            Method::Get,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/bgp/neighbors/{neighbor_id}"
            ),
            None,
        )
    }

    /// Updates a BGP neighbor configured on a router VRF and returns its id.
    pub fn update_router_vrf_bgp_neighbor(
        &self,
        router_id: i64,
        vrf_id: i64,
        neighbor_id: i64,
        request: &UpdateRouterVrfBgpNeighborRequest,
    ) -> Result<i64> {
        let response: RouterVrfBgpNeighborIdResponse = self.request_json(
            Method::Put,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/bgp/neighbors/{neighbor_id}"
            ),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.neighbor_id)
    }

    /// Deletes a BGP neighbor from a router VRF.
    pub fn delete_router_vrf_bgp_neighbor(
        &self,
        router_id: i64,
        vrf_id: i64,
        neighbor_id: i64,
    ) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/bgp/neighbors/{neighbor_id}"
            ),
            None,
        )
    }

    /// Lists the static routes configured on a router VRF.
    pub fn list_router_static_routes(
        &self,
        router_id: i64,
        vrf_id: i64,
    ) -> Result<Vec<RouterStaticRoute>> {
        self.request_list(
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/static-routes"),
            None,
        )
    }

    /// Gets one static route configured on a router VRF.
    ///
    /// There is no single-route endpoint, so this filters the full route list. Returns
    /// [`Error::NotFound`] when no route has this id.
    pub fn get_router_static_route(
        &self,
        router_id: i64,
        vrf_id: i64,
        route_id: i64,
    ) -> Result<RouterStaticRoute> {
        let path = format!(
            "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/static-routes/{route_id}"
        );
        self.list_router_static_routes(router_id, vrf_id)?
            .into_iter()
            .find(|route| route.route_id == route_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!(
                    "static route {route_id} not found for VRF {vrf_id} on router {router_id}"
                ),
            })
    }

    /// Creates a static route on a router VRF and returns its id.
    pub fn create_router_static_route(
        &self,
        router_id: i64,
        vrf_id: i64,
        request: &CreateRouterStaticRouteRequest,
    ) -> Result<i64> {
        let response: RouterStaticRouteIdResponse = self.request_json(
            Method::Post,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/static-routes"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.route_id)
    }

    /// Updates a static route on a router VRF and returns its id.
    pub fn update_router_static_route(
        &self,
        router_id: i64,
        vrf_id: i64,
        route_id: i64,
        request: &UpdateRouterStaticRouteRequest,
    ) -> Result<i64> {
        let response: RouterStaticRouteIdResponse = self.request_json(
            Method::Put,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/static-routes/{route_id}"
            ),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.route_id)
    }

    /// Deletes a static route from a router VRF.
    pub fn delete_router_static_route(
        &self,
        router_id: i64,
        vrf_id: i64,
        route_id: i64,
    ) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/static-routes/{route_id}"
            ),
            None,
        )
    }

    /// Creates a prefix list on a router and returns its id.
    pub fn create_router_prefix_list(
        &self,
        router_id: i64,
        request: &CreateRouterPrefixListRequest,
    ) -> Result<i64> {
        let response: RouterPrefixListIdResponse = self.request_json(
            Method::Post,
            &format!("/cloud-routing/routers/{router_id}/config/prefix-lists"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.prefix_list_id)
    }

    /// Lists the prefix lists configured on a router.
    pub fn list_router_prefix_lists(&self, router_id: i64) -> Result<Vec<RouterPrefixList>> {
        self.request_list(
            &format!("/cloud-routing/routers/{router_id}/config/prefix-lists"),
            None,
        )
    }

    /// Gets one prefix list configured on a router.
    ///
    /// There is no single-list endpoint, so this filters the full prefix list listing.
    /// Returns [`Error::NotFound`] when no prefix list has this id.
    pub fn get_router_prefix_list(
        &self,
        router_id: i64,
        prefix_list_id: i64,
    ) -> Result<RouterPrefixList> {
        let path =
            format!("/cloud-routing/routers/{router_id}/config/prefix-lists/{prefix_list_id}");
        self.list_router_prefix_lists(router_id)?
            .into_iter()
            .find(|list| list.prefix_list_id == prefix_list_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!("prefix list {prefix_list_id} not found on router {router_id}"),
            })
    }

    /// Updates a prefix list on a router and returns its id.
    pub fn update_router_prefix_list(
        &self,
        router_id: i64,
        prefix_list_id: i64,
        request: &UpdateRouterPrefixListRequest,
    ) -> Result<i64> {
        let response: RouterPrefixListIdResponse = self.request_json(
            Method::Put,
            &format!("/cloud-routing/routers/{router_id}/config/prefix-lists/{prefix_list_id}"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.prefix_list_id)
    }

    /// Deletes a prefix list from a router.
    pub fn delete_router_prefix_list(&self, router_id: i64, prefix_list_id: i64) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!("/cloud-routing/routers/{router_id}/config/prefix-lists/{prefix_list_id}"),
            None,
        )
    }

    /// Gets a router's NTP configuration.
    pub fn get_router_ntp_config(&self, router_id: i64) -> Result<RouterNtpConfig> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}/config/services/ntp"),
            None,
        )
    }

    /// Updates a router's NTP configuration and returns the router's id.
    pub fn update_router_ntp_config(
        &self,
        router_id: i64,
        request: &UpdateRouterNtpConfigRequest,
    ) -> Result<i64> {
        let response: RouterIdResponse = self.request_json(
            Method::Put,
            &format!("/cloud-routing/routers/{router_id}/config/services/ntp"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.router_id)
    }

    /// Gets the requested live routing views for a router VRF.
    ///
    /// The routing view schema is not yet stable, so this returns the raw decoded response.
    pub fn get_router_routing_views(
        &self,
        router_id: i64,
        vrf_id: i64,
        request: &RouterRoutingViewRequest,
    ) -> Result<Value> {
        self.request_json(
            Method::Post,
            &format!("/cloud-routing/routers/{router_id}/view/routing/{vrf_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Gets the live routing overview for a router VRF.
    ///
    /// The routing overview schema is not yet stable, so this returns the raw decoded
    /// response.
    pub fn get_router_routing_overview(&self, router_id: i64, vrf_id: i64) -> Result<Value> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}/view/routing/{vrf_id}/overview"),
            None,
        )
    }

    /// Gets a router's IPsec IKE and ESP configuration.
    pub fn get_router_ipsec_config(&self, router_id: i64) -> Result<RouterIpSecConfig> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}/config/ipSec"),
            None,
        )
    }

    /// Updates a router's IPsec IKE and ESP configuration.
    pub fn update_router_ipsec_config(
        &self,
        router_id: i64,
        request: &UpdateRouterIpSecConfigRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Put,
            &format!("/cloud-routing/routers/{router_id}/config/ipSec"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Lists the IPsec peers configured on a router VRF.
    pub fn list_router_vrf_ipsec_peers(
        &self,
        router_id: i64,
        vrf_id: i64,
    ) -> Result<Vec<RouterVrfIpSecPeer>> {
        self.request_list(
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/ipSec/peers"),
            None,
        )
    }

    /// Gets one IPsec peer configured on a router VRF.
    ///
    /// There is no single-peer endpoint, so this filters the full peer list. Returns
    /// [`Error::NotFound`] when no peer has this id.
    pub fn get_router_vrf_ipsec_peer(
        &self,
        router_id: i64,
        vrf_id: i64,
        peer_id: i64,
    ) -> Result<RouterVrfIpSecPeer> {
        let path = format!(
            "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/ipSec/peers/{peer_id}"
        );
        self.list_router_vrf_ipsec_peers(router_id, vrf_id)?
            .into_iter()
            .find(|peer| peer.ip_sec_peer_id == peer_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!(
                    "IPsec peer {peer_id} not found in router {router_id} VRF {vrf_id}"
                ),
            })
    }

    /// Creates an IPsec peer on a router VRF and returns its id.
    pub fn create_router_vrf_ipsec_peer(
        &self,
        router_id: i64,
        vrf_id: i64,
        request: &CreateRouterVrfIpSecPeerRequest,
    ) -> Result<i64> {
        let response: RouterVrfIpSecPeerIdResponse = self.request_json(
            Method::Post,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/ipSec/peers"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.peer_id)
    }

    /// Updates an IPsec peer on a router VRF and returns its id.
    pub fn update_router_vrf_ipsec_peer(
        &self,
        router_id: i64,
        vrf_id: i64,
        peer_id: i64,
        request: &UpdateRouterVrfIpSecPeerRequest,
    ) -> Result<i64> {
        let response: RouterVrfIpSecPeerIdResponse = self.request_json(
            Method::Put,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/ipSec/peers/{peer_id}"
            ),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.peer_id)
    }

    /// Deletes an IPsec peer from a router VRF.
    pub fn delete_router_vrf_ipsec_peer(
        &self,
        router_id: i64,
        vrf_id: i64,
        peer_id: i64,
    ) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/ipSec/peers/{peer_id}"
            ),
            None,
        )
    }

    /// Lists the interfaces configured on a router VRF, keyed by interface id.
    pub fn list_router_vrf_interfaces(
        &self,
        router_id: i64,
        vrf_id: i64,
    ) -> Result<BTreeMap<String, RouterVrfInterface>> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/interfaces"),
            None,
        )
    }

    /// Creates an interface on a router VRF and returns its id.
    pub fn create_router_vrf_interface(
        &self,
        router_id: i64,
        vrf_id: i64,
        request: &CreateRouterVrfInterfaceRequest,
    ) -> Result<i64> {
        let response: RouterVrfInterfaceIdResponse = self.request_json(
            Method::Post,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/interfaces"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.interface_id)
    }

    /// Gets one interface configured on a router VRF.
    pub fn get_router_vrf_interface(
        &self,
        router_id: i64,
        vrf_id: i64,
        interface_id: i64,
    ) -> Result<RouterVrfInterface> {
        self.request_json(
            Method::Get,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/interfaces/{interface_id}"
            ),
            None,
        )
    }

    /// Updates an interface on a router VRF and returns its id.
    pub fn update_router_vrf_interface(
        &self,
        router_id: i64,
        vrf_id: i64,
        interface_id: i64,
        request: &UpdateRouterVrfInterfaceRequest,
    ) -> Result<i64> {
        let response: RouterVrfInterfaceIdResponse = self.request_json(
            Method::Put,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/interfaces/{interface_id}"
            ),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.interface_id)
    }

    /// Deletes an interface from a router VRF.
    pub fn delete_router_vrf_interface(
        &self,
        router_id: i64,
        vrf_id: i64,
        interface_id: i64,
    ) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/interfaces/{interface_id}"
            ),
            None,
        )
    }

    /// Creates a wireguard peer on a router VRF interface and returns its id.
    pub fn create_router_vrf_interface_wireguard_peer(
        &self,
        router_id: i64,
        vrf_id: i64,
        interface_id: i64,
        request: &CreateRouterVrfInterfaceWireguardPeerRequest,
    ) -> Result<i64> {
        let response: RouterVrfInterfaceWireguardPeerIdResponse = self.request_json(
            Method::Post,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/interfaces/{interface_id}/wireguard-peers"
            ),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.wireguard_peer_id)
    }

    /// Gets one wireguard peer configured on a router VRF interface.
    pub fn get_router_vrf_interface_wireguard_peer(
        &self,
        router_id: i64,
        vrf_id: i64,
        interface_id: i64,
        wireguard_peer_id: i64,
    ) -> Result<RouterVrfInterfaceWireguardPeer> {
        self.request_json(
            Method::Get,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/interfaces/{interface_id}/wireguard-peers/{wireguard_peer_id}"
            ),
            None,
        )
    }

    /// Deletes a wireguard peer from a router VRF interface.
    pub fn delete_router_vrf_interface_wireguard_peer(
        &self,
        router_id: i64,
        vrf_id: i64,
        interface_id: i64,
        wireguard_peer_id: i64,
    ) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/interfaces/{interface_id}/wireguard-peers/{wireguard_peer_id}"
            ),
            None,
        )
    }

    /// Creates a SNAT rule on a router VRF and returns its id.
    pub fn create_router_vrf_snat_rule(
        &self,
        router_id: i64,
        vrf_id: i64,
        request: &CreateRouterVrfSnatRuleRequest,
    ) -> Result<i64> {
        let response: RouterVrfSnatRuleIdResponse = self.request_json(
            Method::Post,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/snat-rules"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.snat_rule_id)
    }

    /// Lists the SNAT rules configured on a router VRF.
    pub fn list_router_vrf_snat_rules(
        &self,
        router_id: i64,
        vrf_id: i64,
    ) -> Result<Vec<RouterVrfSnatRule>> {
        self.request_list(
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/snat-rules"),
            None,
        )
    }

    /// Gets one SNAT rule configured on a router VRF.
    ///
    /// There is no single-rule endpoint, so this filters the full rule list. Returns
    /// [`Error::NotFound`] when no rule has this id.
    pub fn get_router_vrf_snat_rule(
        &self,
        router_id: i64,
        vrf_id: i64,
        snat_rule_id: i64,
    ) -> Result<RouterVrfSnatRule> {
        let path = format!(
            "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/snat-rules/{snat_rule_id}"
        );
        self.list_router_vrf_snat_rules(router_id, vrf_id)?
            .into_iter()
            .find(|rule| rule.snat_rule_id == snat_rule_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!(
                    "SNAT rule {snat_rule_id} not found in router {router_id} VRF {vrf_id}"
                ),
            })
    }

    /// Updates a SNAT rule on a router VRF and returns its id.
    pub fn update_router_vrf_snat_rule(
        &self,
        router_id: i64,
        vrf_id: i64,
        snat_rule_id: i64,
        request: &UpdateRouterVrfSnatRuleRequest,
    ) -> Result<i64> {
        let response: RouterVrfSnatRuleIdResponse = self.request_json(
            Method::Put,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/snat-rules/{snat_rule_id}"
            ),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.snat_rule_id)
    }

    /// Deletes a SNAT rule from a router VRF.
    pub fn delete_router_vrf_snat_rule(
        &self,
        router_id: i64,
        vrf_id: i64,
        snat_rule_id: i64,
    ) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/snat-rules/{snat_rule_id}"
            ),
            None,
        )
    }

    /// Creates a DNAT rule on a router VRF and returns its id.
    pub fn create_router_vrf_dnat_rule(
        &self,
        router_id: i64,
        vrf_id: i64,
        request: &CreateRouterVrfDnatRuleRequest,
    ) -> Result<i64> {
        let response: RouterVrfDnatRuleIdResponse = self.request_json(
            Method::Post,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/dnat-rules"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.dnat_rule_id)
    }

    /// Lists the DNAT rules configured on a router VRF.
    pub fn list_router_vrf_dnat_rules(
        &self,
        router_id: i64,
        vrf_id: i64,
    ) -> Result<Vec<RouterVrfDnatRule>> {
        self.request_list(
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/dnat-rules"),
            None,
        )
    }

    /// Gets one DNAT rule configured on a router VRF.
    ///
    /// There is no single-rule endpoint, so this filters the full rule list. Returns
    /// [`Error::NotFound`] when no rule has this id.
    pub fn get_router_vrf_dnat_rule(
        &self,
        router_id: i64,
        vrf_id: i64,
        dnat_rule_id: i64,
    ) -> Result<RouterVrfDnatRule> {
        let path = format!(
            "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/dnat-rules/{dnat_rule_id}"
        );
        self.list_router_vrf_dnat_rules(router_id, vrf_id)?
            .into_iter()
            .find(|rule| rule.dnat_rule_id == dnat_rule_id)
            .ok_or_else(|| Error::NotFound {
                method: "GET".to_string(),
                url: path,
                status_code: 404,
                api_code: 0,
                message: format!(
                    "DNAT rule {dnat_rule_id} not found in router {router_id} VRF {vrf_id}"
                ),
            })
    }

    /// Updates a DNAT rule on a router VRF and returns its id.
    pub fn update_router_vrf_dnat_rule(
        &self,
        router_id: i64,
        vrf_id: i64,
        dnat_rule_id: i64,
        request: &UpdateRouterVrfDnatRuleRequest,
    ) -> Result<i64> {
        let response: RouterVrfDnatRuleIdResponse = self.request_json(
            Method::Put,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/dnat-rules/{dnat_rule_id}"
            ),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.dnat_rule_id)
    }

    /// Deletes a DNAT rule from a router VRF.
    pub fn delete_router_vrf_dnat_rule(
        &self,
        router_id: i64,
        vrf_id: i64,
        dnat_rule_id: i64,
    ) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!(
                "/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/dnat-rules/{dnat_rule_id}"
            ),
            None,
        )
    }

    /// Lists the tunnels configured on a router VRF.
    pub fn list_router_vrf_tunnels(
        &self,
        router_id: i64,
        vrf_id: i64,
    ) -> Result<Vec<RouterVrfTunnel>> {
        self.request_list(
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/tunnels"),
            None,
        )
    }

    /// Gets one tunnel configured on a router VRF.
    pub fn get_router_vrf_tunnel(
        &self,
        router_id: i64,
        vrf_id: i64,
        tunnel_id: i64,
    ) -> Result<RouterVrfTunnel> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/tunnels/{tunnel_id}"),
            None,
        )
    }

    /// Creates a tunnel on a router VRF and returns its id.
    pub fn create_router_vrf_tunnel(
        &self,
        router_id: i64,
        vrf_id: i64,
        request: &CreateRouterVrfTunnelRequest,
    ) -> Result<i64> {
        let response: RouterVrfTunnelIdResponse = self.request_json(
            Method::Post,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/tunnels"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.tunnel_id)
    }

    /// Updates a tunnel on a router VRF and returns its id.
    pub fn update_router_vrf_tunnel(
        &self,
        router_id: i64,
        vrf_id: i64,
        tunnel_id: i64,
        request: &UpdateRouterVrfTunnelRequest,
    ) -> Result<i64> {
        let response: RouterVrfTunnelIdResponse = self.request_json(
            Method::Put,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/tunnels/{tunnel_id}"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.tunnel_id)
    }

    /// Deletes a tunnel from a router VRF.
    pub fn delete_router_vrf_tunnel(
        &self,
        router_id: i64,
        vrf_id: i64,
        tunnel_id: i64,
    ) -> Result<()> {
        self.request_empty(
            Method::Delete,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/tunnels/{tunnel_id}"),
            None,
        )
    }

    /// Gets a router VRF's DHCP configuration.
    pub fn get_router_vrf_dhcp(&self, router_id: i64, vrf_id: i64) -> Result<RouterVrfDhcpConfig> {
        self.request_json(
            Method::Get,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/services/dhcp"),
            None,
        )
    }

    /// Updates a router VRF's DHCP configuration and returns the router's id.
    pub fn update_router_vrf_dhcp(
        &self,
        router_id: i64,
        vrf_id: i64,
        request: &UpdateRouterVrfDhcpRequest,
    ) -> Result<i64> {
        let response: RouterIdResponse = self.request_json(
            Method::Put,
            &format!("/cloud-routing/routers/{router_id}/config/vrfs/{vrf_id}/services/dhcp"),
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.router_id)
    }

    /// Creates an SSL certificate and returns its id.
    pub fn create_ssl_certificate(&self, request: &CreateSslCertificateRequest) -> Result<i64> {
        let response: CreateSslCertificateResponse = self.request_json(
            Method::Post,
            "/ssl-certificates",
            Some(serde_json::to_vec(request)?),
        )?;
        Ok(response.ssl_certificate_id)
    }

    /// Lists the SSL certificates on the account.
    pub fn list_ssl_certificates(&self) -> Result<Vec<SslCertificate>> {
        self.request_list("/ssl-certificates", None)
    }

    /// Gets an SSL certificate by id.
    pub fn get_ssl_certificate(&self, id: i64) -> Result<SslCertificate> {
        self.request_json(Method::Get, &format!("/ssl-certificates/{id}"), None)
    }

    /// Updates an SSL certificate.
    pub fn update_ssl_certificate(
        &self,
        id: i64,
        request: &UpdateSslCertificateRequest,
    ) -> Result<()> {
        self.request_empty(
            Method::Patch,
            &format!("/ssl-certificates/{id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes an SSL certificate. A certificate that is already gone is treated as success.
    pub fn delete_ssl_certificate(&self, id: i64) -> Result<()> {
        match self.request_empty(Method::Delete, &format!("/ssl-certificates/{id}"), None) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Creates a group on a network load balancer.
    pub fn create_nlb_group(
        &self,
        nlb_id: i64,
        request: &CreateNlbGroupRequest,
    ) -> Result<NlbGroup> {
        self.request_json(
            Method::Post,
            &format!("/network-loadbalancers/{nlb_id}/groups"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Gets one group on a network load balancer.
    pub fn get_nlb_group(&self, nlb_id: i64, group_id: i64) -> Result<NlbGroup> {
        self.request_json(
            Method::Get,
            &format!("/network-loadbalancers/{nlb_id}/groups/{group_id}"),
            None,
        )
    }

    /// Lists the groups on a network load balancer.
    pub fn list_nlb_groups(&self, nlb_id: i64) -> Result<Vec<NlbGroup>> {
        self.request_list(&format!("/network-loadbalancers/{nlb_id}/groups"), None)
    }

    /// Replaces a group on a network load balancer.
    pub fn replace_nlb_group(
        &self,
        nlb_id: i64,
        group_id: i64,
        request: &ReplaceNlbGroupRequest,
    ) -> Result<NlbGroup> {
        self.request_json(
            Method::Put,
            &format!("/network-loadbalancers/{nlb_id}/groups/{group_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a group from a network load balancer. A group that is already gone is treated
    /// as success.
    pub fn delete_nlb_group(&self, nlb_id: i64, group_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/network-loadbalancers/{nlb_id}/groups/{group_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Creates a group on an HTTP load balancer.
    pub fn create_http_lb_group(
        &self,
        http_lb_id: i64,
        request: &CreateHttpLbGroupRequest,
    ) -> Result<HttpLbGroup> {
        self.request_json(
            Method::Post,
            &format!("/http-loadbalancers/{http_lb_id}/groups"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Gets one group on an HTTP load balancer.
    pub fn get_http_lb_group(&self, http_lb_id: i64, group_id: i64) -> Result<HttpLbGroup> {
        self.request_json(
            Method::Get,
            &format!("/http-loadbalancers/{http_lb_id}/groups/{group_id}"),
            None,
        )
    }

    /// Lists the groups on an HTTP load balancer.
    pub fn list_http_lb_groups(&self, http_lb_id: i64) -> Result<Vec<HttpLbGroup>> {
        self.request_list(&format!("/http-loadbalancers/{http_lb_id}/groups"), None)
    }

    /// Replaces a group on an HTTP load balancer.
    pub fn replace_http_lb_group(
        &self,
        http_lb_id: i64,
        group_id: i64,
        request: &ReplaceHttpLbGroupRequest,
    ) -> Result<HttpLbGroup> {
        self.request_json(
            Method::Put,
            &format!("/http-loadbalancers/{http_lb_id}/groups/{group_id}"),
            Some(serde_json::to_vec(request)?),
        )
    }

    /// Deletes a group from an HTTP load balancer. A group that is already gone is treated as
    /// success.
    pub fn delete_http_lb_group(&self, http_lb_id: i64, group_id: i64) -> Result<()> {
        match self.request_empty(
            Method::Delete,
            &format!("/http-loadbalancers/{http_lb_id}/groups/{group_id}"),
            None,
        ) {
            Err(err) if err.is_not_found() => Ok(()),
            result => result,
        }
    }

    /// Returns usage and ceilings for every account-level limit.
    pub fn get_account_limits(&self) -> Result<BTreeMap<String, AccountLimit>> {
        self.request_json(Method::Get, "/account-limits", None)
    }

    /// Queries a named set of cloud statistics.
    pub fn query_statistics(&self, metrics: &[String]) -> Result<Vec<StatisticResult>> {
        self.query_statistics_at("/cloud/statistics", metrics)
    }

    /// Queries a named set of cloud networking statistics.
    pub fn query_networking_statistics(&self, metrics: &[String]) -> Result<Vec<StatisticResult>> {
        self.query_statistics_at("/cloud/networking/statistics", metrics)
    }

    /// Queries a named set of anycast networking statistics.
    pub fn query_anycast_statistics(&self, metrics: &[String]) -> Result<Vec<StatisticResult>> {
        self.query_statistics_at("/cloud/networking/anycast/statistics", metrics)
    }

    fn query_statistics_at(&self, path: &str, metrics: &[String]) -> Result<Vec<StatisticResult>> {
        let body = build_statistics_request(metrics);
        self.request_json(Method::Post, path, Some(serde_json::to_vec(&body)?))
    }

    /// Returns every metric the account can query, alongside the time window the platform
    /// evaluated them over.
    pub fn get_metric_names(&self) -> Result<MetricNames> {
        let value = self.request_value(
            Method::Post,
            "/cloud/statistics/views/all-metrics",
            Some(b"{}".to_vec()),
        )?;
        parse_metric_names(value)
    }

    fn request_json<T>(&self, method: Method, path: &str, body: Option<Vec<u8>>) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let value = self.request_value(method, path, body)?;
        let bytes = serde_json::to_vec(&value)?;
        decode_required(&bytes, path)
    }

    fn request_empty(&self, method: Method, path: &str, body: Option<Vec<u8>>) -> Result<()> {
        self.request_value(method, path, body).map(|_| ())
    }

    pub(crate) fn request_list<T>(&self, path: &str, key: Option<&str>) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut list = self.get_list_page(path, key)?;
        let mut rows = list.rows;
        while list.meta.total > 0
            && list.meta.limit > 0
            && list.meta.offset + list.meta.limit < list.meta.total
        {
            let next_offset = list.meta.offset + list.meta.limit;
            let next_path = v3_list_page_path(path, next_offset, list.meta.limit);
            let next = self.get_list_page(&next_path, key)?;
            if next.meta.offset <= list.meta.offset || next.rows.is_empty() {
                break;
            }
            list = next;
            rows.extend(list.rows.clone());
        }
        let bytes = serde_json::to_vec(&rows)?;
        decode_required(&bytes, path)
    }

    fn get_list_page(&self, path: &str, key: Option<&str>) -> Result<V3ListPage> {
        let data = self.request_value(Method::Get, path, None)?;
        unwrap_v3_list(data, key)
    }

    fn request_value(&self, method: Method, path: &str, body: Option<Vec<u8>>) -> Result<Value> {
        let url = build_url(&self.base_url, path, &self.api_key)?;
        let response = self.transport.send(Request {
            method,
            url: url.clone(),
            body,
            content_type: Some("application/json".to_string()),
        })?;
        let redacted_url = redact_url(&url);
        if response.status == 204 {
            return Ok(Value::Null);
        }
        let envelope: V3Envelope = serde_json::from_slice(&response.body).map_err(|err| {
            if response.status == 404 || response.status == 410 {
                Error::NotFound {
                    method: method.as_str().to_string(),
                    url: redacted_url.clone(),
                    status_code: response.status,
                    api_code: 0,
                    message: "not found".to_string(),
                }
            } else {
                Error::Decode(format!("vAPI3 envelope: {err}"))
            }
        })?;
        classify_v3_error(
            method,
            &redacted_url,
            response.status,
            &envelope,
            &response.body,
        )?;
        Ok(envelope.data.unwrap_or(Value::Null))
    }
}

#[derive(Debug, Deserialize)]
struct V3Envelope {
    #[serde(default)]
    code: i64,
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    message: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct V3Meta {
    #[serde(default)]
    limit: i64,
    #[serde(default)]
    offset: i64,
    #[serde(default)]
    total: i64,
}

#[derive(Debug, Clone)]
struct V3ListPage {
    rows: Vec<Value>,
    meta: V3Meta,
}

#[derive(Debug, Deserialize)]
struct OidcClientCreateResponse {
    #[serde(default, rename = "clientId")]
    client_id: i64,
}

#[derive(Debug, Default, Deserialize)]
struct OidcClientsEnvelope {
    #[serde(default)]
    tenant: Value,
    #[serde(default)]
    clients: Value,
}

/// Renders the tenant id carried in an OIDC clients response as a string, whether the platform
/// sent it as a JSON number or a string.
fn tenant_id_as_string(tenant: &Value) -> String {
    match tenant {
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        _ => String::new(),
    }
}

/// A single row from the OIDC client list, before the tenant id is threaded in.
#[derive(Debug, Clone, Default, Deserialize)]
struct OidcClientRow {
    #[serde(default, rename = "clientId")]
    client_id: i64,
    #[serde(default, rename = "createdOn")]
    created_on: String,
    #[serde(default, rename = "lastUsedOn")]
    last_used_on: Option<String>,
    #[serde(default)]
    label: String,
    #[serde(default)]
    description: String,
    #[serde(default, rename = "jwksUri")]
    jwks_uri: Option<String>,
    // The list endpoint returns the same value as jwksHttpsUrl instead of jwksUri; both are
    // kept so a read does not drift depending on which key the platform used.
    #[serde(default, rename = "jwksHttpsUrl")]
    jwks_https_url: Option<String>,
    #[serde(default, rename = "accountDefault")]
    account_default: bool,
    #[serde(default, rename = "defaultAudience")]
    default_audience: String,
    #[serde(default)]
    ttl: i64,
    #[serde(
        default,
        rename = "enforceAllowList",
        deserialize_with = "flexible_bool"
    )]
    enforce_allow_list: bool,
}

impl OidcClientRow {
    fn into_client(self, tenant: &str) -> OidcClient {
        OidcClient {
            client_id: self.client_id,
            created_on: self.created_on,
            last_used_on: self.last_used_on,
            label: self.label,
            description: self.description,
            jwks_uri: self.jwks_uri.or(self.jwks_https_url),
            account_default: self.account_default,
            default_audience: self.default_audience,
            ttl: self.ttl,
            enforce_allow_list: self.enforce_allow_list,
            tenant: tenant.to_string(),
            keys: Vec::new(),
            auth_logs: Vec::new(),
            change_logs: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct OidcClientDetail {
    #[serde(default)]
    metadata: OidcClientMetadata,
    #[serde(default)]
    keys: Value,
    #[serde(default)]
    logs: OidcClientLogs,
}

#[derive(Debug, Default, Deserialize)]
struct OidcClientMetadata {
    #[serde(default, rename = "createdOn")]
    created_on: String,
    #[serde(default, rename = "lastUsedOn")]
    last_used_on: Option<String>,
    #[serde(default)]
    label: String,
    #[serde(default)]
    description: String,
    #[serde(default, rename = "jwksUri")]
    jwks_uri: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct OidcClientLogs {
    #[serde(default)]
    changes: Value,
    #[serde(default)]
    auth: Value,
}

/// Decodes a keys or logs sub-list nested in an OIDC client detail response.
///
/// Absent and malformed must not look alike: a missing or empty sub-list is normal, so a null
/// value is skipped, but anything present that fails to decode is returned as an error rather
/// than silently discarded.
fn decode_oidc_sub_list<T>(value: Value, client_id: i64, label: &str) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    if value.is_null() {
        return Ok(Vec::new());
    }
    let page = parse_v3_list(value)?;
    if page.rows.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_value(Value::Array(page.rows))
        .map_err(|err| Error::Decode(format!("OIDC client {client_id} {label}: {err}")))
}

#[derive(Debug, Serialize)]
struct CreateOidcClientKeysBody<'a> {
    keys: &'a [CreateOidcClientKeyRequest],
}

#[derive(Debug, Default, Deserialize)]
struct CreateOidcClientKeysResponse {
    #[serde(default)]
    keys: Vec<OidcClientKey>,
}

#[derive(Debug, Serialize)]
struct OidcClientMbpkgRef {
    mbpkgid: i64,
}

#[derive(Debug, Serialize)]
struct AddOidcClientVmsBody {
    vms: Vec<OidcClientMbpkgRef>,
}

#[derive(Debug, Serialize)]
struct AddOidcClientBareMetalServersBody {
    servers: Vec<OidcClientMbpkgRef>,
}

#[derive(Debug, Default, Deserialize)]
struct OidcClientVmsResponse {
    #[serde(default)]
    vms: Vec<OidcClientVm>,
}

#[derive(Debug, Default, Deserialize)]
struct OidcClientBareMetalServersResponse {
    #[serde(default)]
    servers: Vec<OidcClientBareMetalServer>,
}

/// Returns true when the API reports a 400 because the OIDC client key targeted for deletion
/// was already revoked.
fn is_oidc_key_already_revoked_error(err: &Error) -> bool {
    match err {
        Error::Api {
            status_code: 400,
            message,
            ..
        } => message.to_lowercase().contains("the key is revoked"),
        _ => false,
    }
}

/// Request body for creating an OIDC client.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateOidcClientRequest {
    /// Display label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// JSON Web Key Set URI used to validate client-presented keys.
    #[serde(rename = "jwksUri", skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,
    /// Whether this is the account's default OIDC client.
    #[serde(rename = "accountDefault")]
    pub account_default: bool,
    /// Whether the VM and bare metal allow list is enforced.
    #[serde(rename = "enforceAllowList")]
    pub enforce_allow_list: bool,
    /// Token time-to-live in seconds.
    #[serde(skip_serializing_if = "is_zero")]
    pub ttl: i64,
    /// Default audience issued in tokens from this client.
    #[serde(rename = "defaultAudience", skip_serializing_if = "String::is_empty")]
    pub default_audience: String,
}

/// Request body for updating an OIDC client. Every field is optional; only the fields set are
/// changed.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateOidcClientRequest {
    /// Display label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// JSON Web Key Set URI used to validate client-presented keys.
    #[serde(rename = "jwksUri", skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,
    /// Whether this is the account's default OIDC client.
    #[serde(rename = "accountDefault", skip_serializing_if = "Option::is_none")]
    pub account_default: Option<bool>,
    /// Whether the VM and bare metal allow list is enforced.
    #[serde(rename = "enforceAllowList", skip_serializing_if = "Option::is_none")]
    pub enforce_allow_list: Option<bool>,
    /// Token time-to-live in seconds.
    #[serde(skip_serializing_if = "is_zero")]
    pub ttl: i64,
    /// Default audience issued in tokens from this client.
    #[serde(rename = "defaultAudience", skip_serializing_if = "String::is_empty")]
    pub default_audience: String,
}

/// Request body for adding a public key to an OIDC client.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateOidcClientKeyRequest {
    /// Display label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Public key content.
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

/// Request body for updating a public key's label and description on an OIDC client.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateOidcClientKeyRequest {
    /// Display label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

/// Request body for creating a floating IPv4 address.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateCloudFloatingIpv4Request {
    /// Reverse DNS domain to assign to the address.
    #[serde(rename = "ptrDomain", skip_serializing_if = "Option::is_none")]
    pub ptr_domain: Option<String>,
    /// VLAN id to bind the address to.
    #[serde(rename = "vlanId", skip_serializing_if = "Option::is_none")]
    pub vlan_id: Option<i64>,
}

/// A virtual machine reference used by floating IPv4 grant and revoke requests.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct CloudFloatingIpv4VmRef {
    /// Server package id.
    pub mbpkgid: i64,
}

/// Request body for granting virtual machines access to a floating IPv4 address.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GrantCloudFloatingIpv4VmsRequest {
    /// Whether to revoke every VM currently granted before applying this grant.
    #[serde(rename = "revokeExisting", skip_serializing_if = "Option::is_none")]
    pub revoke_existing: Option<bool>,
    /// Virtual machines to grant.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vms: Vec<CloudFloatingIpv4VmRef>,
}

/// Request body for revoking virtual machines' access to a floating IPv4 address.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RevokeCloudFloatingIpv4VmsRequest {
    /// Virtual machines to revoke.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vms: Vec<CloudFloatingIpv4VmRef>,
}

/// One router to attach when creating a magic mesh.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct MeshRouterEntry {
    /// Router id.
    #[serde(rename = "routerId")]
    pub router_id: i64,
}

/// Request body for creating a magic mesh.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateMagicMeshRequest {
    /// Mesh name.
    pub name: String,
    /// Mesh description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Routers to attach to the mesh on creation.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub routers: Vec<MeshRouterEntry>,
}

/// Request body for updating a magic mesh. Only the fields set are changed.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateMagicMeshRequest {
    /// New mesh name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New mesh description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Request body for attaching a router to a magic mesh.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct AddMeshRouterRequest {
    /// Router id.
    #[serde(rename = "routerId")]
    pub router_id: i64,
}

/// Request body for VPC creation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateVpcRequest {
    /// Label.
    pub label: String,
    /// Description.
    pub description: String,
    /// Location id.
    #[serde(rename = "location_id")]
    pub location_id: i64,
    /// Network overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<VpcNetwork>,
    /// Nameservers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameservers: Option<VpcNameservers>,
    /// Firewall defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firewalls: Option<VpcFirewalls>,
    /// Default behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defaults: Option<VpcDefaults>,
}

/// Request body for VPC updates.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateVpcRequest {
    /// Label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Firewall settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firewalls: Option<VpcFirewalls>,
}

/// VPC network request block.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VpcNetwork {
    /// IPv4 CIDR.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub ipv4: String,
    /// IPv6 CIDR.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub ipv6: String,
}

/// VPC nameservers request block.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VpcNameservers {
    /// IPv4 nameservers.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ipv4: Vec<VpcNameserver>,
    /// IPv6 nameservers.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ipv6: Vec<VpcNameserver>,
}

/// VPC nameserver.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VpcNameserver {
    /// Nameserver address.
    pub server: String,
}

/// Request body for [`V3Client::replace_vpc_nameservers`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplaceVpcNameserversRequest {
    /// Nameservers to announce, replacing the current set.
    pub nameservers: Vec<VpcNameserver>,
}

/// Response returned by [`V3Client::replace_vpc_nameservers`].
///
/// Shares [`ReplaceVpcNameserversRequest`]'s shape; the platform answers with the same body it
/// accepts.
pub type ReplaceVpcNameserversResponse = ReplaceVpcNameserversRequest;

/// VPC firewall settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VpcFirewalls {
    /// IPv4 directions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<VpcFirewallDirections>,
    /// IPv6 directions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<VpcFirewallDirections>,
}

/// Firewall direction toggles.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VpcFirewallDirections {
    /// Inbound enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbound: Option<bool>,
    /// Outbound enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbound: Option<bool>,
}

/// VPC default behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VpcDefaults {
    /// Whether a default SNAT rule is created.
    #[serde(
        rename = "enableDefaultSnatRule",
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_default_snat_rule: Option<bool>,
}

/// Request body for creating a VPC backend template.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateVpcBackendTemplateRequest {
    /// Template name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Template description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Backends to create with the template.
    #[serde(
        rename = "backendHosts",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub backend_hosts: Vec<VpcBackend>,
}

/// Request body for updating a VPC backend template's name or description.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateVpcBackendTemplateRequest {
    /// Template name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Template description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

/// Request body for replacing a VPC backend template.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReplaceVpcBackendTemplateRequest {
    /// Template name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Template description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Complete backend list the template is replaced with.
    #[serde(rename = "backendHosts")]
    pub backend_hosts: Vec<VpcBackend>,
}

/// Request body for creating one VPC backend.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateVpcBackendRequest {
    /// Backend label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Backend address.
    pub address: String,
}

/// Request body for updating one VPC backend.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateVpcBackendRequest {
    /// Backend label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Backend address.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub address: String,
}

/// Request body for replacing every backend under a VPC backend template.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReplaceVpcBackendsRequest {
    /// Complete backend list to replace the template's backends with.
    #[serde(rename = "backendHosts")]
    pub backend_hosts: Vec<VpcBackend>,
}

/// Wrapper the API answers `PUT` backend-replacement calls with, carrying the backend list
/// under `backendHosts` rather than as a bare array.
#[derive(Debug, Deserialize)]
struct VpcBackendsWrapper {
    #[serde(default, rename = "backendHosts")]
    backend_hosts: Vec<VpcBackend>,
}

/// Request body for updating a VPC's bastion SSH settings.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct UpdateVpcSshSettingsRequest {
    /// SSH port. `None` resets bastion access to the default port.
    pub port: Option<i64>,
}

/// Request body for enabling or disabling an SSH key on a VPC's bastion.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct EnableVpcSshKeyRequest {
    /// Whether the key is enabled for bastion access.
    pub enabled: bool,
}

/// Request body for creating a floating IP on a VPC.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateVpcFloatingIpRequest {
    /// IP version to allocate, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// Reverse DNS record.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ptr: String,
}

/// Request body for updating a floating IP's reverse DNS record.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateVpcFloatingIpRequest {
    /// Reverse DNS record.
    pub ptr: String,
}

#[derive(Debug, Deserialize)]
struct FloatingIpCreateResponse {
    #[serde(default, rename = "floatingIpId")]
    floating_ip_id: i64,
}

/// Request body for creating a firewall rule on a VPC gateway.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateVpcFirewallRuleRequest {
    /// IP version the rule applies to, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// Traffic direction, "inbound" or "outbound".
    pub direction: String,
    /// Protocol the rule matches.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub protocol: String,
    /// Rule description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Network the rule applies to, in CIDR notation.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub network: String,
    /// Port range the rule matches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<VpcPortRange>,
}

/// Request body for updating a firewall rule.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateVpcFirewallRuleRequest {
    /// Traffic direction, "inbound" or "outbound".
    #[serde(skip_serializing_if = "String::is_empty")]
    pub direction: String,
    /// IP version the rule applies to, 4 or 6.
    #[serde(rename = "ipVersion", skip_serializing_if = "is_zero")]
    pub ip_version: i64,
    /// Protocol the rule matches.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub protocol: String,
    /// Rule description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Network the rule applies to, in CIDR notation.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub network: String,
    /// Port range the rule matches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<VpcPortRange>,
}

#[derive(Debug, Deserialize)]
struct FirewallRuleCreateResponse {
    #[serde(default, rename = "firewallRuleId")]
    firewall_rule_id: i64,
}

/// Request body for creating a SNAT rule on a VPC gateway.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateVpcSnatRuleRequest {
    /// IP version the rule applies to, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// Protocol the rule matches.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub protocol: String,
    /// Rule description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Match criteria.
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_criteria: Option<VpcSnatMatch>,
    /// Translation applied to matching traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<VpcSnatTranslation>,
    /// Placement of this rule relative to the others on the gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<VpcSnatPriority>,
}

/// Request body for updating a SNAT rule.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateVpcSnatRuleRequest {
    /// Protocol the rule matches.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub protocol: String,
    /// Rule description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Match criteria.
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_criteria: Option<VpcSnatMatch>,
    /// Translation applied to matching traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<VpcSnatTranslation>,
    /// Placement of this rule relative to the others on the gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<VpcSnatPriority>,
}

/// Request body for creating a DNAT rule on a VPC gateway.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateVpcDnatRuleRequest {
    /// IP version the rule applies to, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// Protocol the rule matches.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub protocol: String,
    /// Rule description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Match criteria.
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_criteria: Option<VpcDnatMatch>,
    /// Translation applied to matching traffic. Always sent, even when absent, because the
    /// API requires the field to be present.
    pub translation: Option<VpcDnatTranslation>,
    /// Placement of this rule relative to the others on the gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<VpcDnatPriority>,
}

/// Request body for updating a DNAT rule.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateVpcDnatRuleRequest {
    /// Protocol the rule matches.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub protocol: String,
    /// Rule description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Match criteria.
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_criteria: Option<VpcDnatMatch>,
    /// Translation applied to matching traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<VpcDnatTranslation>,
    /// Placement of this rule relative to the others on the gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<VpcDnatPriority>,
}

/// Request body for creating an SSL certificate.
#[derive(Clone, Default, Serialize)]
pub struct CreateSslCertificateRequest {
    /// Certificate name.
    pub name: String,
    /// Certificate description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// PEM-encoded certificate content.
    pub certificate: String,
    /// PEM-encoded private key content.
    #[serde(rename = "privateKey")]
    pub private_key: String,
}

// credential fields omitted from Debug
crate::redacted_debug!(
    CreateSslCertificateRequest;
    name, description, certificate;
    private_key
);

/// Request body for updating an SSL certificate. Fields left empty are left unchanged.
#[derive(Clone, Default, Serialize)]
pub struct UpdateSslCertificateRequest {
    /// Certificate name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Certificate description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// PEM-encoded certificate content.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub certificate: String,
    /// PEM-encoded private key content.
    #[serde(rename = "privateKey", skip_serializing_if = "String::is_empty")]
    pub private_key: String,
}

// credential fields omitted from Debug
crate::redacted_debug!(
    UpdateSslCertificateRequest;
    name, description, certificate;
    private_key
);

#[derive(Debug, Deserialize)]
struct CreateSslCertificateResponse {
    #[serde(default, rename = "sslCertificateId")]
    ssl_certificate_id: i64,
}

/// Request body for creating a group on a network load balancer.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateNlbGroupRequest {
    /// Group name.
    pub name: String,
    /// Group description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// IP version the group balances, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// Load balancing algorithm.
    pub algorithm: String,
    /// Address the group matches incoming traffic against.
    #[serde(rename = "match")]
    pub matcher: NlbGroupMatch,
    /// Health check configuration for the group's backends.
    #[serde(rename = "healthCheck")]
    pub health_check: NlbGroupHealthCheck,
    /// Traffic rules for the group.
    pub rules: Vec<NlbGroupRule>,
    /// Backends the group balances across.
    pub backends: Vec<NlbGroupBackend>,
}

/// Request body for replacing a group on a network load balancer.
///
/// Shares [`CreateNlbGroupRequest`]'s fields; the platform accepts the same body for both.
pub type ReplaceNlbGroupRequest = CreateNlbGroupRequest;

/// Request body for creating a group on an HTTP load balancer.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateHttpLbGroupRequest {
    /// Group name.
    pub name: String,
    /// Group description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Load balancing algorithm.
    pub algorithm: String,
    /// Whether sticky sessions are enabled.
    #[serde(rename = "stickySessionsEnabled")]
    pub sticky_sessions_enabled: bool,
    /// Whether TLS is used from the load balancer to the backends.
    #[serde(rename = "sslToBackendEnabled")]
    pub ssl_to_backend_enabled: bool,
    /// Port the backends listen on.
    #[serde(rename = "internalPort")]
    pub internal_port: i64,
    /// Address and ports the group matches incoming traffic against.
    #[serde(rename = "match")]
    pub matcher: HttpLbGroupMatch,
    /// Health check configuration for the group's backends.
    #[serde(rename = "healthCheck")]
    pub health_check: HttpLbGroupHealthCheck,
    /// Traffic rules for the group.
    pub rules: Vec<HttpLbGroupRule>,
    /// Backends the group balances across.
    pub backends: Vec<HttpLbGroupBackend>,
}

/// Request body for replacing a group on an HTTP load balancer.
///
/// Shares [`CreateHttpLbGroupRequest`]'s fields; the platform accepts the same body for both.
pub type ReplaceHttpLbGroupRequest = CreateHttpLbGroupRequest;

/// Request body for storage bucket creation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateStorageBucketRequest {
    /// Location id.
    #[serde(rename = "locationId")]
    pub location_id: i64,
    /// Label.
    pub label: String,
    /// Capacity in GB.
    #[serde(skip_serializing_if = "is_zero")]
    pub capacity: i64,
    /// Private flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    /// Autoscaling flag.
    #[serde(rename = "enableAutoScaling", skip_serializing_if = "Option::is_none")]
    pub enable_auto_scaling: Option<bool>,
}

/// Request body for storage bucket updates.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateStorageBucketRequest {
    /// Label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Capacity in GB.
    #[serde(skip_serializing_if = "is_zero")]
    pub capacity: i64,
    /// Private flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    /// Autoscaling flag.
    #[serde(rename = "enableAutoScaling", skip_serializing_if = "Option::is_none")]
    pub enable_auto_scaling: Option<bool>,
}

/// Request body for object store creation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateStorageObjectStoreRequest {
    /// Location id.
    #[serde(rename = "locationId")]
    pub location_id: i64,
    /// Label.
    pub label: String,
    /// Capacity in GB.
    #[serde(skip_serializing_if = "is_zero")]
    pub capacity: i64,
    /// Autoscaling flag.
    #[serde(rename = "enableAutoScaling", skip_serializing_if = "Option::is_none")]
    pub enable_auto_scaling: Option<bool>,
}

/// Request body for object store updates.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateStorageObjectStoreRequest {
    /// Label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Capacity in GB.
    #[serde(skip_serializing_if = "is_zero")]
    pub capacity: i64,
    /// Autoscaling flag.
    #[serde(rename = "enableAutoScaling", skip_serializing_if = "Option::is_none")]
    pub enable_auto_scaling: Option<bool>,
}

/// Request body for block storage namespace creation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateStorageBlockNamespaceRequest {
    /// Location id.
    #[serde(rename = "locationId")]
    pub location_id: i64,
    /// Label.
    pub label: String,
    /// Capacity in GB.
    #[serde(skip_serializing_if = "is_zero")]
    pub capacity: i64,
    /// Autoscaling flag.
    #[serde(rename = "enableAutoScaling", skip_serializing_if = "Option::is_none")]
    pub enable_auto_scaling: Option<bool>,
}

/// Request body for block storage namespace updates.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateStorageBlockNamespaceRequest {
    /// Label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Capacity in GB.
    #[serde(skip_serializing_if = "is_zero")]
    pub capacity: i64,
    /// Autoscaling flag.
    #[serde(rename = "enableAutoScaling", skip_serializing_if = "Option::is_none")]
    pub enable_auto_scaling: Option<bool>,
}

/// Request body for block volume creation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateStorageBlockVolumeRequest {
    /// Location id.
    #[serde(rename = "locationId")]
    pub location_id: i64,
    /// Label.
    pub label: String,
    /// Capacity in GB.
    #[serde(skip_serializing_if = "is_zero")]
    pub capacity: i64,
}

/// Request body for block volume updates.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateStorageBlockVolumeRequest {
    /// Label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Capacity in GB.
    #[serde(skip_serializing_if = "is_zero")]
    pub capacity: i64,
}

/// Tag reference for NKE cluster requests.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct NkeClusterTagInput {
    /// Tag id.
    #[serde(rename = "tagId")]
    pub tag_id: i64,
}

/// NKE add-ons requested at cluster creation.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct NkeAddons {
    /// Kubernetes dashboard add-on.
    #[serde(rename = "kubernetesDashboard")]
    pub kubernetes_dashboard: bool,
}

/// NKE billing block.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct NkeBilling {
    /// Package id.
    #[serde(rename = "packageId")]
    pub package_id: i64,
    /// Location id.
    #[serde(rename = "locationId")]
    pub location_id: i64,
    /// Contract id.
    #[serde(rename = "contractId", skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<i64>,
}

/// NKE cluster networking block.
#[derive(Debug, Clone, Default, Serialize)]
pub struct NkeClusterNetwork {
    /// Existing VPC id.
    #[serde(rename = "vpcId", skip_serializing_if = "is_zero")]
    pub vpc_id: i64,
    /// Pod CIDR.
    #[serde(rename = "podCidr", skip_serializing_if = "String::is_empty")]
    pub pod_cidr: String,
    /// Service CIDR.
    #[serde(rename = "serviceCidr", skip_serializing_if = "String::is_empty")]
    pub service_cidr: String,
}

/// Request body for NKE cluster creation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateNkeClusterRequest {
    /// Cluster name.
    pub name: String,
    /// Kubernetes version.
    pub version: String,
    /// Initial replica count.
    pub replicas: i64,
    /// Minimum worker nodes.
    #[serde(rename = "minimumNodes")]
    pub minimum_nodes: i64,
    /// Maximum worker nodes.
    #[serde(rename = "maximumNodes")]
    pub maximum_nodes: i64,
    /// Autoscaling flag.
    #[serde(rename = "doAutoscaling")]
    pub do_autoscaling: bool,
    /// Dual-stack flag.
    #[serde(rename = "doDualStack")]
    pub do_dual_stack: bool,
    /// Billing.
    pub billing: NkeBilling,
    /// Networking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networking: Option<NkeClusterNetwork>,
    /// Add-ons to install.
    #[serde(rename = "addonsToInstall", skip_serializing_if = "Option::is_none")]
    pub addons_to_install: Option<NkeAddons>,
    /// Tags.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NkeClusterTagInput>,
}

/// Node autoscaling update block.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct NkeUpdateNodes {
    /// Minimum worker nodes.
    pub minimum: i64,
    /// Maximum worker nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
}

/// NKE billing update block.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct NkeUpdateBilling {
    /// Package id.
    #[serde(rename = "packageId", skip_serializing_if = "is_zero")]
    pub package_id: i64,
}

/// Request body for NKE cluster updates.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateNkeClusterRequest {
    /// Cluster name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Kubernetes version.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub version: String,
    /// Autoscaling flag.
    #[serde(rename = "doAutoscaling", skip_serializing_if = "Option::is_none")]
    pub do_autoscaling: Option<bool>,
    /// Billing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing: Option<NkeUpdateBilling>,
    /// Nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<NkeUpdateNodes>,
    /// Tags.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NkeClusterTagInput>,
}

/// Request body for NKE worker node updates.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateNkeWorkerNodeRequest {
    /// Node label.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// Tags.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NkeClusterTagInput>,
}

/// Request body for installing an NKE add-on.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateNkeAddonRequest {
    /// Add-on type, such as `netactuate-dns` or `storage`.
    #[serde(rename = "addonType")]
    pub addon_type: String,
    /// Version to install. An empty value installs the catalog default.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub version: String,
    /// Release channel to install from.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub channel: String,
    /// Add-on specific config, such as [`NkeDnsAddonWriteConfig`] or
    /// [`NkeStorageAddonWriteConfig`] serialized to JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
}

/// Request body for updating an installed NKE add-on.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateNkeAddonRequest {
    /// Version to move to.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub version: String,
    /// Release channel to move to.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub channel: String,
    /// Add-on specific config, such as [`NkeDnsAddonWriteConfig`] or
    /// [`NkeStorageAddonWriteConfig`] serialized to JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
}

/// Write-side config for the `netactuate-dns` NKE add-on.
///
/// The add-on reports a different shape back; see [`crate::NkeDnsAddonZone`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct NkeDnsAddonWriteConfig {
    /// DNS zone to bind.
    pub zone: String,
    /// Zone mode.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mode: String,
}

/// Write-side config for the `storage` NKE add-on.
///
/// The add-on reports a different shape back, with `makeDefault` becoming `isDefaultClass`; see
/// [`crate::NkeStorageAddonIntegration`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct NkeStorageAddonWriteConfig {
    /// Block storage namespace id to provision from.
    #[serde(rename = "blockNamespaceId", skip_serializing_if = "is_zero")]
    pub block_namespace_id: i64,
    /// Storage pool label.
    #[serde(rename = "poolLabel", skip_serializing_if = "String::is_empty")]
    pub pool_label: String,
    /// Capacity in GB.
    #[serde(skip_serializing_if = "is_zero")]
    pub capacity: i64,
    /// Autoscaling flag.
    #[serde(rename = "enableAutoScaling", skip_serializing_if = "Option::is_none")]
    pub enable_auto_scaling: Option<bool>,
    /// StorageClass name to create.
    #[serde(rename = "storageClassName", skip_serializing_if = "String::is_empty")]
    pub storage_class_name: String,
    /// Whether this becomes the cluster's default StorageClass.
    #[serde(rename = "makeDefault", skip_serializing_if = "Option::is_none")]
    pub make_default: Option<bool>,
    /// Reclaim policy applied to volumes created from this class.
    #[serde(rename = "reclaimPolicy", skip_serializing_if = "String::is_empty")]
    pub reclaim_policy: String,
}

#[derive(Debug, Deserialize)]
struct StorageCreateResponse {
    #[serde(default, rename = "bucketId")]
    bucket_id: i64,
    #[serde(default, rename = "objectStoreId")]
    object_store_id: i64,
    #[serde(default, rename = "blockNamespaceId")]
    block_namespace_id: i64,
    #[serde(default, rename = "blockVolumeId")]
    block_volume_id: i64,
}

#[derive(Debug, Deserialize)]
struct NkeCreateResponse {
    #[serde(default, rename = "clusterId")]
    cluster_id: i64,
}

#[derive(Debug, Deserialize)]
struct MeshCreateResponse {
    #[serde(default, rename = "meshId")]
    mesh_id: i64,
}

/// Request body for creating a cloud router.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRouterRequest {
    /// Billing package id to provision the router under.
    #[serde(rename = "packageId")]
    pub package_id: i64,
    /// Location id to provision the router in.
    #[serde(rename = "locationId")]
    pub location_id: i64,
    /// Router name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Router description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RouterCreateResponse {
    #[serde(default, rename = "routerId")]
    router_id: i64,
}

/// Request body for updating a cloud router. Only the fields set are changed.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterRequest {
    /// New router name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New router description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RouterIdResponse {
    #[serde(default, rename = "routerId")]
    router_id: i64,
}

/// Request body for creating a VRF on a router.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRouterVrfRequest {
    /// VRF name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// VRF description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Request body for updating a VRF's name or description.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterVrfRequest {
    /// New VRF name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New VRF description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RouterVrfIdResponse {
    #[serde(default, rename = "vrfId")]
    vrf_id: i64,
}

/// Request body for updating a VRF's BGP configuration.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterVrfBgpRequest {
    /// Networks to advertise over BGP.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub networks: Vec<RouterVrfBgpNetwork>,
    /// Local ASN to set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asn: Option<RouterVrfBgpAsn>,
}

/// A VRF's local ASN, as sent when updating BGP configuration.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RouterVrfBgpAsn {
    /// Local ASN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<String>,
}

/// Request body for creating a BGP neighbor on a router VRF.
#[derive(Clone, Default, Serialize)]
pub struct CreateRouterVrfBgpNeighborRequest {
    /// Neighbor address.
    pub address: String,
    /// Whether the neighbor session starts administratively shut down.
    #[serde(rename = "isShutdown")]
    pub is_shutdown: bool,
    /// Whether to override the AS path with the local ASN.
    #[serde(rename = "doAsOverride")]
    pub do_as_override: bool,
    /// Whether to set the next hop to this router on advertised routes.
    #[serde(rename = "doNextHelpSelf")]
    pub do_next_help_self: bool,
    /// Source address override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<BgpNeighborSource>,
    /// IP versions to enable for the session.
    #[serde(rename = "enabledIpVersion")]
    pub enabled_ip_version: BgpNeighborEnabledIpVersion,
    /// eBGP multihop count.
    #[serde(rename = "ebgpMultihop", skip_serializing_if = "Option::is_none")]
    pub ebgp_multihop: Option<i64>,
    /// Remote ASN.
    pub asn: BgpNeighborAsn,
    /// MD5 authentication secret.
    #[serde(rename = "md5Secret", skip_serializing_if = "String::is_empty")]
    pub md5_secret: String,
    /// Import route map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<BgpNeighborRouteMap>,
    /// Export route map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<BgpNeighborRouteMap>,
    /// Neighbor name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Neighbor description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

// credential fields omitted from Debug
crate::redacted_debug!(
    CreateRouterVrfBgpNeighborRequest;
    address, is_shutdown, do_as_override, do_next_help_self, source, enabled_ip_version,
    ebgp_multihop, asn, import, export, name, description;
    md5_secret
);

/// Request body for updating a BGP neighbor on a router VRF.
#[derive(Clone, Default, Serialize)]
pub struct UpdateRouterVrfBgpNeighborRequest {
    /// Neighbor address.
    pub address: String,
    /// Whether the neighbor session is administratively shut down.
    #[serde(rename = "isShutdown")]
    pub is_shutdown: bool,
    /// Whether to override the AS path with the local ASN.
    #[serde(rename = "doAsOverride")]
    pub do_as_override: bool,
    /// Whether to set the next hop to this router on advertised routes.
    #[serde(rename = "doNextHelpSelf")]
    pub do_next_help_self: bool,
    /// Source address override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<BgpNeighborSource>,
    /// IP versions to enable for the session.
    #[serde(rename = "enabledIpVersion")]
    pub enabled_ip_version: BgpNeighborEnabledIpVersion,
    /// eBGP multihop count.
    #[serde(rename = "ebgpMultihop", skip_serializing_if = "Option::is_none")]
    pub ebgp_multihop: Option<i64>,
    /// Remote ASN.
    pub asn: BgpNeighborAsn,
    /// MD5 authentication secret.
    #[serde(rename = "md5Secret", skip_serializing_if = "String::is_empty")]
    pub md5_secret: String,
    /// Import route map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<BgpNeighborRouteMap>,
    /// Export route map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<BgpNeighborRouteMap>,
    /// Neighbor name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Neighbor description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

// credential fields omitted from Debug
crate::redacted_debug!(
    UpdateRouterVrfBgpNeighborRequest;
    address, is_shutdown, do_as_override, do_next_help_self, source, enabled_ip_version,
    ebgp_multihop, asn, import, export, name, description;
    md5_secret
);

#[derive(Debug, Deserialize)]
struct RouterVrfBgpNeighborIdResponse {
    #[serde(default, rename = "neighborId")]
    neighbor_id: i64,
}

/// Request body for creating or updating a static route on a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRouterStaticRouteRequest {
    /// Destination network in CIDR notation.
    pub network: String,
    /// Next hop for the route.
    pub via: StaticRouteVia,
    /// Route description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Administrative distance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<i64>,
}

/// Request body for updating a static route on a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterStaticRouteRequest {
    /// Destination network in CIDR notation.
    pub network: String,
    /// Next hop for the route.
    pub via: StaticRouteVia,
    /// Route description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Administrative distance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct RouterStaticRouteIdResponse {
    #[serde(default, rename = "staticRouteId")]
    route_id: i64,
}

/// Request body for creating a prefix list on a router.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRouterPrefixListRequest {
    /// Prefix list name.
    pub name: String,
    /// IP version the list matches, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// Prefix list description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Ordered match rules.
    pub rules: Vec<PrefixListRule>,
}

/// Request body for updating a prefix list on a router.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterPrefixListRequest {
    /// Prefix list name.
    pub name: String,
    /// IP version the list matches, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// Prefix list description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Ordered match rules.
    pub rules: Vec<PrefixListRule>,
}

#[derive(Debug, Deserialize)]
struct RouterPrefixListIdResponse {
    #[serde(default, rename = "prefixListId")]
    prefix_list_id: i64,
}

/// Request body for updating a router's NTP configuration.
///
/// The platform expects every field on every update, so `enabled` and `interface_id` are
/// always sent, including as JSON `null` when absent.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterNtpConfigRequest {
    /// Whether NTP should be enabled.
    pub enabled: Option<bool>,
    /// Interface id NTP should listen on.
    #[serde(rename = "interfaceId")]
    pub interface_id: Option<i64>,
    /// Upstream NTP servers.
    pub upstreams: Vec<RouterNtpUpstream>,
}

/// Request body for fetching live routing views for a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RouterRoutingViewRequest {
    /// Routing views to fetch.
    pub views: Vec<RouterRoutingViewSelector>,
}

/// One routing view to fetch, with an optional filter.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RouterRoutingViewSelector {
    /// View id, when selecting a previously requested view.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// IP version the view covers, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// View name.
    pub name: String,
    /// Filter expression to narrow the view.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub filter: String,
}

/// Request body for updating a router's IPsec IKE and ESP configuration.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterIpSecConfigRequest {
    /// IKE phase configuration.
    #[serde(rename = "ikeGroup")]
    pub ike_group: RouterIpSecIkeGroup,
    /// ESP phase configuration.
    #[serde(rename = "espGroup")]
    pub esp_group: RouterIpSecEspGroup,
}

/// Request body for creating an IPsec peer on a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRouterVrfIpSecPeerRequest {
    /// Peer name.
    pub name: String,
    /// Peer description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Remote peer identifier.
    #[serde(rename = "remoteId")]
    pub remote_id: String,
    /// Pre-shared key secret.
    #[serde(rename = "pskSecret")]
    pub psk_secret: String,
    /// Whether this side initiates the connection.
    #[serde(rename = "doInitiateConnection")]
    pub do_initiate_connection: bool,
    /// Remote peer address.
    #[serde(rename = "peerAddress", skip_serializing_if = "String::is_empty")]
    pub peer_address: String,
    /// Overlay network addressing for the tunnel.
    #[serde(rename = "overlayNetwork")]
    pub overlay_network: RouterVrfIpSecOverlayNetwork,
}

/// Request body for updating an IPsec peer on a router VRF.
///
/// Shares [`CreateRouterVrfIpSecPeerRequest`]'s shape; the platform requires the same fields
/// on both create and update.
pub type UpdateRouterVrfIpSecPeerRequest = CreateRouterVrfIpSecPeerRequest;

#[derive(Debug, Deserialize)]
struct RouterVrfIpSecPeerIdResponse {
    #[serde(default, rename = "ipSecPeerId")]
    peer_id: i64,
}

/// Request body for creating an interface on a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRouterVrfInterfaceRequest {
    /// Interface type.
    #[serde(rename = "type")]
    pub interface_type: String,
    /// Interface name.
    pub name: String,
    /// Interface description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// IPv4 address in CIDR notation.
    #[serde(rename = "ipv4Cidr", skip_serializing_if = "Option::is_none")]
    pub ipv4_cidr: Option<String>,
    /// IPv6 address in CIDR notation.
    #[serde(rename = "ipv6Cidr", skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr: Option<String>,
    /// Hardware id of the ethernet interface being attached, for ethernet interfaces.
    #[serde(rename = "ethernetHardwareId", skip_serializing_if = "Option::is_none")]
    pub ethernet_hardware_id: Option<String>,
    /// Wireguard listen port, for wireguard interfaces.
    #[serde(rename = "wireguardPort", skip_serializing_if = "Option::is_none")]
    pub wireguard_port: Option<i64>,
}

/// Request body for updating an interface on a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterVrfInterfaceRequest {
    /// Interface type.
    #[serde(rename = "type")]
    pub interface_type: String,
    /// Interface name.
    pub name: String,
    /// Interface description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// IPv4 address in CIDR notation.
    #[serde(rename = "ipv4Cidr", skip_serializing_if = "Option::is_none")]
    pub ipv4_cidr: Option<String>,
    /// IPv6 address in CIDR notation.
    #[serde(rename = "ipv6Cidr", skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr: Option<String>,
    /// Hardware id of the ethernet interface being attached, for ethernet interfaces.
    #[serde(rename = "ethernetHardwareId", skip_serializing_if = "Option::is_none")]
    pub ethernet_hardware_id: Option<String>,
    /// Wireguard listen port, for wireguard interfaces.
    #[serde(rename = "wireguardPort", skip_serializing_if = "Option::is_none")]
    pub wireguard_port: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct RouterVrfInterfaceIdResponse {
    #[serde(default, rename = "interfaceId")]
    interface_id: i64,
}

/// Request body for creating a wireguard peer on a router VRF interface.
#[derive(Clone, Default, Serialize)]
pub struct CreateRouterVrfInterfaceWireguardPeerRequest {
    /// Networks routed to this peer.
    #[serde(rename = "allowedIps")]
    pub allowed_ips: Vec<WireguardPeerAllowedIp>,
    /// Peer's public key. Generated by the platform when absent.
    #[serde(rename = "publicKey")]
    pub public_key: Option<String>,
    /// Pre-shared key shared with the peer.
    #[serde(rename = "preSharedKey")]
    pub pre_shared_key: Option<String>,
    /// Peer name.
    pub name: Option<String>,
    /// Peer description.
    pub description: Option<String>,
    /// Peer's remote endpoint address.
    pub remote: Option<String>,
}

// credential fields omitted from Debug
crate::redacted_debug!(
    CreateRouterVrfInterfaceWireguardPeerRequest;
    allowed_ips, public_key, name, description, remote;
    pre_shared_key
);

#[derive(Debug, Deserialize)]
struct RouterVrfInterfaceWireguardPeerIdResponse {
    #[serde(default, rename = "wireguardPeerId")]
    wireguard_peer_id: i64,
}

/// Request body for creating a SNAT rule on a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRouterVrfSnatRuleRequest {
    /// IP version the rule applies to, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// Protocol the rule matches.
    pub protocol: String,
    /// Rule description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Match criteria. Always sent, even as null, because the API requires the field to be
    /// present.
    #[serde(rename = "match")]
    pub match_criteria: Option<RouterVrfSnatMatch>,
    /// Translation applied to matching traffic. Always sent, even as null, for the same
    /// reason as `match_criteria`.
    pub translation: Option<RouterVrfSnatTranslation>,
    /// Placement of this rule relative to the others on the VRF.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<RouterVrfSnatPriority>,
}

/// Request body for updating a SNAT rule on a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterVrfSnatRuleRequest {
    /// IP version the rule applies to, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// Protocol the rule matches.
    pub protocol: String,
    /// Rule description. Always sent, even empty.
    pub description: String,
    /// Match criteria.
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_criteria: Option<RouterVrfSnatMatch>,
    /// Translation applied to matching traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<RouterVrfSnatTranslation>,
    /// Placement of this rule relative to the others on the VRF.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<RouterVrfSnatPriority>,
}

#[derive(Debug, Deserialize)]
struct RouterVrfSnatRuleIdResponse {
    #[serde(default, rename = "snatRuleId")]
    snat_rule_id: i64,
}

/// Request body for creating a DNAT rule on a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRouterVrfDnatRuleRequest {
    /// IP version the rule applies to, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// Protocol the rule matches.
    pub protocol: String,
    /// Rule description.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Match criteria. Always sent, even as null, because the API requires the field to be
    /// present.
    #[serde(rename = "match")]
    pub match_criteria: Option<RouterVrfDnatMatch>,
    /// Translation applied to matching traffic. Always sent, even as null, for the same
    /// reason as `match_criteria`.
    pub translation: Option<RouterVrfDnatTranslation>,
    /// Placement of this rule relative to the others on the VRF.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<RouterVrfDnatPriority>,
}

/// Request body for updating a DNAT rule on a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterVrfDnatRuleRequest {
    /// IP version the rule applies to, 4 or 6.
    #[serde(rename = "ipVersion")]
    pub ip_version: i64,
    /// Protocol the rule matches.
    pub protocol: String,
    /// Rule description. Always sent, even empty.
    pub description: String,
    /// Match criteria.
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_criteria: Option<RouterVrfDnatMatch>,
    /// Translation applied to matching traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<RouterVrfDnatTranslation>,
    /// Placement of this rule relative to the others on the VRF.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<RouterVrfDnatPriority>,
}

#[derive(Debug, Deserialize)]
struct RouterVrfDnatRuleIdResponse {
    #[serde(default, rename = "dnatRuleId")]
    dnat_rule_id: i64,
}

/// A tunnel's remote endpoint address, as sent when creating or updating a tunnel.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RouterVrfTunnelRemoteEndpoint {
    /// Remote endpoint address.
    pub remote: String,
}

/// Request body for creating a tunnel on a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRouterVrfTunnelRequest {
    /// GRE key distinguishing this tunnel from others to the same remote.
    #[serde(rename = "ipKey")]
    pub ip_key: i64,
    /// Tunnel name.
    pub name: String,
    /// Tunnel description.
    pub description: Option<String>,
    /// Tunnel MTU.
    pub mtu: i64,
    /// IPv4 address in CIDR notation.
    #[serde(rename = "ipv4Cidr")]
    pub ipv4_cidr: Option<String>,
    /// IPv6 address in CIDR notation.
    #[serde(rename = "ipv6Cidr")]
    pub ipv6_cidr: Option<String>,
    /// Remote tunnel endpoint.
    #[serde(rename = "endpointAddress")]
    pub endpoint_address: RouterVrfTunnelRemoteEndpoint,
}

/// Request body for updating a tunnel on a router VRF.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterVrfTunnelRequest {
    /// GRE key distinguishing this tunnel from others to the same remote.
    #[serde(rename = "ipKey")]
    pub ip_key: i64,
    /// Tunnel name.
    pub name: String,
    /// Tunnel description.
    pub description: Option<String>,
    /// Tunnel MTU.
    pub mtu: i64,
    /// IPv4 address in CIDR notation.
    #[serde(rename = "ipv4Cidr")]
    pub ipv4_cidr: Option<String>,
    /// IPv6 address in CIDR notation.
    #[serde(rename = "ipv6Cidr")]
    pub ipv6_cidr: Option<String>,
    /// Remote tunnel endpoint.
    #[serde(rename = "endpointAddress")]
    pub endpoint_address: RouterVrfTunnelRemoteEndpoint,
}

#[derive(Debug, Deserialize)]
struct RouterVrfTunnelIdResponse {
    #[serde(default, rename = "tunnelId")]
    tunnel_id: i64,
}

/// Request body for updating a router VRF's DHCP configuration.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRouterVrfDhcpRequest {
    /// Whether the DHCP service is enabled.
    pub enabled: bool,
    /// Interface id the DHCP service listens on.
    #[serde(rename = "interfaceId")]
    pub interface_id: i64,
    /// Subnet DHCP serves addresses on, in CIDR notation.
    pub subnet: String,
    /// Default router address handed to clients.
    #[serde(
        rename = "defaultRouterAddress",
        skip_serializing_if = "String::is_empty"
    )]
    pub default_router_address: String,
    /// Domain name handed to clients.
    #[serde(rename = "clientDomainName", skip_serializing_if = "String::is_empty")]
    pub client_domain_name: String,
    /// Lease timeout in seconds.
    #[serde(rename = "leaseTimeout")]
    pub lease_timeout: i64,
    /// Whether the server pings an address before leasing it.
    #[serde(rename = "doPingCheck")]
    pub do_ping_check: bool,
    /// Address range leased to clients.
    pub range: Option<RouterDhcpRange>,
    /// DNS servers handed to clients.
    #[serde(rename = "domainNameServers")]
    pub domain_name_servers: Vec<RouterDhcpServer>,
    /// NTP servers handed to clients.
    #[serde(rename = "ntpServers")]
    pub ntp_servers: Vec<RouterDhcpServer>,
    /// Static routes handed to clients.
    #[serde(rename = "staticRoutes")]
    pub static_routes: Vec<RouterDhcpStaticRoute>,
}

fn classify_v3_error(
    method: Method,
    redacted_url: &str,
    status_code: u16,
    envelope: &V3Envelope,
    body: &[u8],
) -> Result<()> {
    let body_text = String::from_utf8_lossy(body).to_lowercase();
    if status_code == 404
        || status_code == 410
        || body_text.contains("does not exist")
        || body_text.contains("not associated with your account")
        || body_text.contains("not found")
    {
        return Err(Error::NotFound {
            method: method.as_str().to_string(),
            url: redacted_url.to_string(),
            status_code,
            api_code: envelope.code,
            message: message_or_body(envelope, body),
        });
    }
    if status_code == 412 || envelope.code == 412 {
        return Err(Error::Contract {
            method: method.as_str().to_string(),
            url: redacted_url.to_string(),
            status_code,
            api_code: envelope.code,
            message: message_or_body(envelope, body),
        });
    }
    if !(200..300).contains(&status_code)
        || (envelope.code != 0 && !(200..300).contains(&envelope.code))
    {
        return Err(Error::Api {
            method: method.as_str().to_string(),
            url: redacted_url.to_string(),
            status_code,
            api_code: envelope.code,
            message: message_or_body(envelope, body),
        });
    }
    Ok(())
}

fn unwrap_v3_list(mut value: Value, key: Option<&str>) -> Result<V3ListPage> {
    if let Some(key) = key {
        value = value
            .get(key)
            .cloned()
            .ok_or_else(|| Error::Decode(format!("vAPI3 list response has no {key} key")))?;
    }
    parse_v3_list(value)
}

fn parse_v3_list(value: Value) -> Result<V3ListPage> {
    match value {
        Value::Array(rows) => Ok(V3ListPage {
            rows,
            meta: V3Meta::default(),
        }),
        Value::Object(mut object) => {
            if let Some(Value::Object(mut paginator)) = object
                .get_mut("data")
                .and_then(Value::as_object_mut)
                .and_then(|data| data.remove("paginator"))
            {
                let data = paginator
                    .remove("data")
                    .ok_or_else(|| Error::Decode("vAPI3 paginator has no data".to_string()))?;
                return parse_v3_list(data);
            }
            if let Some(data) = object.remove("data") {
                if data.is_array() {
                    let meta = object
                        .remove("meta")
                        .map(serde_json::from_value)
                        .transpose()?
                        .unwrap_or_default();
                    return Ok(V3ListPage {
                        rows: data.as_array().cloned().unwrap_or_default(),
                        meta,
                    });
                }
                if data.is_object() {
                    if let Ok(page) = parse_v3_list(data) {
                        return Ok(page);
                    }
                }
            }
            for (nested_key, nested) in object {
                if nested_key == "meta" {
                    continue;
                }
                if let Ok(page) = parse_v3_list(nested) {
                    return Ok(page);
                }
            }
            Err(Error::Decode(
                "vAPI3 list response has no list data".to_string(),
            ))
        }
        _ => Err(Error::Decode(
            "vAPI3 list response is not a list".to_string(),
        )),
    }
}

fn v3_list_page_path(path: &str, offset: i64, limit: i64) -> String {
    let Ok(mut url) = Url::parse(&format!("http://placeholder{}", path)) else {
        return path.to_string();
    };
    url.query_pairs_mut()
        .append_pair("offset", &offset.to_string())
        .append_pair("limit", &limit.to_string());
    let mut out = url.path().to_string();
    if let Some(query) = url.query() {
        out.push('?');
        out.push_str(query);
    }
    out
}

/// Extracts the DHCP nameservers embedded in a raw VPC response, under `dhcp.nameservers`.
///
/// A VPC with no DHCP nameservers configured, or with no `dhcp` block at all, yields an empty
/// [`VpcNameservers`] rather than an error.
fn vpc_nameservers_from_dhcp(vpc: &Value) -> VpcNameservers {
    let mut result = VpcNameservers::default();
    let Some(nameservers) = vpc.get("dhcp").and_then(|dhcp| dhcp.get("nameservers")) else {
        return result;
    };
    if let Some(list) = nameservers.get("ipv4").and_then(Value::as_array) {
        result.ipv4 = list
            .iter()
            .filter_map(Value::as_str)
            .map(|server| VpcNameserver {
                server: server.to_string(),
            })
            .collect();
    }
    if let Some(list) = nameservers.get("ipv6").and_then(Value::as_array) {
        result.ipv6 = list
            .iter()
            .filter_map(Value::as_str)
            .map(|server| VpcNameserver {
                server: server.to_string(),
            })
            .collect();
    }
    result
}

/// Decodes a response body that the platform sometimes answers empty on success, yielding the
/// default value rather than a decode error when there is no payload.
fn decode_optional_or_default<T>(value: Value, context: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de> + Default,
{
    if value.is_null() {
        return Ok(T::default());
    }
    serde_json::from_value(value).map_err(|err| Error::Decode(format!("{context}: {err}")))
}

fn message_or_body(envelope: &V3Envelope, body: &[u8]) -> String {
    if !envelope.message.is_empty() {
        return envelope.message.clone();
    }
    String::from_utf8_lossy(body).into_owned()
}

fn is_zero(value: &i64) -> bool {
    *value == 0
}

/// Request body for a statistics query: each metric name paired with an empty selector object,
/// matching the shape the platform expects on `/cloud/statistics` and its sibling endpoints.
#[derive(Debug, Serialize)]
struct StatisticsRequest {
    metrics: Vec<StatisticMetricRequest>,
}

#[derive(Debug, Serialize)]
struct StatisticMetricRequest {
    metric: BTreeMap<String, Value>,
}

fn build_statistics_request(metrics: &[String]) -> StatisticsRequest {
    let metrics = metrics
        .iter()
        .map(|metric| {
            let mut selector = BTreeMap::new();
            selector.insert(metric.clone(), Value::Object(Map::new()));
            StatisticMetricRequest { metric: selector }
        })
        .collect();
    StatisticsRequest { metrics }
}

/// Returns the id an SSH key is addressed by: the key's own id when set, otherwise the
/// underlying account SSH key id it was created from.
fn effective_ssh_key_id(key: &VpcSshKey) -> i64 {
    if key.id != 0 {
        key.id
    } else {
        key.ssh_key_id
    }
}

/// Returns true for a 5xx response, which is often transient right after VPC creation while
/// the gateway VM is still initializing.
fn is_transient_server_error(err: &Error) -> bool {
    matches!(err, Error::Api { status_code, .. } if matches!(status_code, 500 | 502 | 503 | 504))
}

/// Returns true when the API reports a 400 because a VPC exists but its child services are
/// still initializing.
fn is_vpc_not_ready_error(err: &Error) -> bool {
    match err {
        Error::Api {
            status_code: 400,
            message,
            ..
        } => {
            let lower = message.to_lowercase();
            lower.contains("vpc") && lower.contains("not ready")
        }
        _ => false,
    }
}

/// Builds the error returned when an NKE cluster settles into a failed state while
/// [`V3Client::wait_for_nke_cluster_healthy`] is polling it.
fn nke_cluster_failed_error(id: i64, status: &str) -> Error {
    Error::Api {
        method: "GET".to_string(),
        url: format!("/nke/clusters/{id}"),
        status_code: 0,
        api_code: 0,
        message: format!("NKE cluster {id} entered failed state: {status}"),
    }
}

/// Calls `attempt` up to `attempts` times, waiting `delay` between tries, as long as the
/// error it returns satisfies `should_retry`.
fn retry_while<T>(
    attempts: u32,
    delay: Duration,
    should_retry: impl Fn(&Error) -> bool,
    mut attempt: impl FnMut() -> Result<T>,
) -> Result<T> {
    let mut result = attempt();
    for _ in 1..attempts {
        match &result {
            Err(err) if should_retry(err) => {
                std::thread::sleep(delay);
                result = attempt();
            }
            _ => break,
        }
    }
    result
}

/// Calls `ready` immediately and then on every `interval` until it reports true or `timeout`
/// elapses, whichever comes first. Returns [`Error::Timeout`] when the deadline passes with
/// the condition never satisfied, and propagates any error `ready` itself returns.
pub(crate) fn wait_for_ready(
    interval: Duration,
    timeout: Duration,
    mut ready: impl FnMut() -> Result<bool>,
) -> Result<()> {
    if ready()? {
        return Ok(());
    }
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Timeout(format!(
                "condition not met after {timeout:?}"
            )));
        }
        std::thread::sleep(interval);
        if ready()? {
            return Ok(());
        }
    }
}

/// Decodes the storage types listing, which the platform answers as a plain array, a
/// paginated envelope, or a map keyed by type code depending on account and platform
/// version.
fn decode_storage_types(value: Value) -> Result<Vec<StorageType>> {
    if let Ok(types) = serde_json::from_value::<Vec<StorageType>>(value.clone()) {
        return Ok(types);
    }

    if let Ok(list) = parse_v3_list(value.clone()) {
        let types: std::result::Result<Vec<StorageType>, _> =
            list.rows.into_iter().map(serde_json::from_value).collect();
        if let Ok(types) = types {
            return Ok(types);
        }
    }

    let object = value
        .as_object()
        .ok_or_else(|| Error::Decode("storage types response is not an object".to_string()))?;

    let mut keys: Vec<&String> = object
        .keys()
        .filter(|key| *key != "meta" && *key != "data")
        .collect();
    keys.sort();

    let mut types = Vec::with_capacity(keys.len());
    for key in keys {
        let mut storage_type: StorageType = serde_json::from_value(object[key].clone())?;
        if storage_type.type_code.is_empty() {
            storage_type.type_code = key.clone();
        }
        types.push(storage_type);
    }
    Ok(types)
}
