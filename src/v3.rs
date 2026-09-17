use crate::error::{Error, Result};
use crate::models::{
    decode_required, NkeAccessUrls, NkeCluster, NkeLogEntry, NkeWorkerNode, StorageBucket,
    V3Location, Vpc,
};
#[cfg(feature = "blocking")]
use crate::transport::ReqwestTransport;
use crate::transport::{api_key_from_env, build_url, redact_url, DynTransport, Method, Request};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use url::Url;

const V3_BASE_ENDPOINT: &str = "https://vapi3.netactuate.com";

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

#[derive(Debug, Deserialize)]
struct StorageCreateResponse {
    #[serde(default, rename = "bucketId")]
    bucket_id: i64,
    #[serde(default, rename = "objectStoreId")]
    object_store_id: i64,
}

#[derive(Debug, Deserialize)]
struct NkeCreateResponse {
    #[serde(default, rename = "clusterId")]
    cluster_id: i64,
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

fn message_or_body(envelope: &V3Envelope, body: &[u8]) -> String {
    if !envelope.message.is_empty() {
        return envelope.message.clone();
    }
    String::from_utf8_lossy(body).into_owned()
}

fn is_zero(value: &i64) -> bool {
    *value == 0
}
