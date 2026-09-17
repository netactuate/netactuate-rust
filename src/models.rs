use crate::error::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

/// A vAPI2 cloud server.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct Server {
    /// Server fully qualified domain name.
    #[serde(default, alias = "fqdn")]
    pub name: String,
    /// Billing package id.
    #[serde(default, alias = "mbpkgid")]
    pub id: i64,
    /// Operating system name.
    #[serde(default, alias = "os")]
    pub os: String,
    /// Operating system id.
    #[serde(default)]
    pub os_id: i64,
    /// Primary IPv4 address.
    #[serde(default, alias = "ip")]
    pub primary_ipv4: String,
    /// Primary IPv6 address.
    #[serde(default, alias = "ipv6")]
    pub primary_ipv6: String,
    /// Plan id.
    #[serde(default)]
    pub plan_id: i64,
    /// Package name.
    #[serde(default)]
    pub package: String,
    /// Location display name.
    #[serde(default, alias = "city")]
    pub location: String,
    /// Location id.
    #[serde(default)]
    pub location_id: i64,
    /// Server lifecycle status.
    #[serde(default, alias = "status")]
    pub server_status: String,
    /// Power state.
    #[serde(default, alias = "state")]
    pub power_status: String,
    /// Installation state.
    #[serde(default)]
    pub installed: i64,
    /// VPC id when the server belongs to a VPC.
    #[serde(default)]
    pub vpc_id: Option<i64>,
}

/// Response returned when a server build is enqueued.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ServerBuild {
    /// Server package id.
    #[serde(default, rename = "mbpkgid")]
    pub server_id: i64,
    /// Build status text.
    #[serde(default)]
    pub status: String,
    /// Build job id.
    #[serde(default)]
    pub build: i64,
}

/// DNS TTL value returned as either a JSON string or number.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct FlexibleTtl(pub String);

impl FlexibleTtl {
    /// Returns the TTL as an integer, or zero if it is absent or unparseable.
    pub fn as_i64(&self) -> i64 {
        self.0.parse().unwrap_or(0)
    }
}

impl<'de> Deserialize<'de> for FlexibleTtl {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(value) => Ok(Self(value)),
            Value::Number(value) => Ok(Self(value.to_string())),
            Value::Null => Ok(Self(String::new())),
            other => Err(serde::de::Error::custom(format!(
                "ttl must be a string or number, got {other}"
            ))),
        }
    }
}

/// DNS zone start-of-authority data.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DnsZoneSoa {
    /// Primary nameserver.
    #[serde(default)]
    pub primary: String,
    /// Hostmaster mailbox.
    #[serde(default)]
    pub hostmaster: String,
    /// Zone serial.
    #[serde(default)]
    pub serial: String,
    /// Refresh interval.
    #[serde(default)]
    pub refresh: String,
    /// Retry interval.
    #[serde(default)]
    pub retry: String,
    /// Expiry interval.
    #[serde(default)]
    pub expire: String,
    /// Default TTL.
    #[serde(default)]
    pub default_ttl: String,
}

/// A DNS zone.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DnsZone {
    /// Zone id.
    #[serde(default)]
    pub id: i64,
    /// Zone name.
    #[serde(default)]
    pub name: String,
    /// Zone type.
    #[serde(default, rename = "type")]
    pub zone_type: String,
    /// Master IP for reverse zones.
    #[serde(default)]
    pub ip: String,
    /// Master zone value present on list responses.
    #[serde(default)]
    pub master: String,
    /// Nameserver records present on single get responses.
    #[serde(default)]
    pub ns: Vec<DnsRecord>,
    /// Start-of-authority block present on single get responses.
    #[serde(default)]
    pub soa: Option<DnsZoneSoa>,
    /// Zone TTL.
    #[serde(default)]
    pub ttl: FlexibleTtl,
    /// Records returned with a single zone response.
    #[serde(default)]
    pub records: Vec<DnsRecord>,
}

/// A DNS record.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DnsRecord {
    /// Record id.
    #[serde(default)]
    pub id: i64,
    /// Zone id.
    #[serde(default, rename = "domain_id")]
    pub zone_id: i64,
    /// Record name.
    #[serde(default)]
    pub name: String,
    /// Record type.
    #[serde(default, rename = "type")]
    pub record_type: String,
    /// Record content.
    #[serde(default)]
    pub content: String,
    /// Record TTL.
    #[serde(default)]
    pub ttl: FlexibleTtl,
    /// MX/SRV priority.
    #[serde(default, rename = "prio")]
    pub priority: i64,
}

/// vAPI3 location descriptor.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct V3Location {
    /// Location id.
    #[serde(default)]
    pub id: i64,
    /// Location name.
    #[serde(default)]
    pub name: String,
    /// Flag code.
    #[serde(default)]
    pub flag: String,
}

/// vAPI3 package descriptor.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct V3Package {
    /// Package id.
    #[serde(default)]
    pub id: i64,
    /// Package name.
    #[serde(default)]
    pub name: String,
}

/// vAPI3 storage or network capacity descriptor.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct V3Capacity {
    /// Whether autoscaling is enabled.
    #[serde(default)]
    pub autoscaling: bool,
    /// Requested capacity in GB.
    #[serde(default, rename = "requestedGB")]
    pub requested_gb: Option<i64>,
    /// Total capacity in GB.
    #[serde(default, rename = "totalGB")]
    pub total_gb: i64,
}

/// Hardware class used by a storage resource.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct StorageHardwareClass {
    /// Hardware class id.
    #[serde(default)]
    pub id: i64,
    /// Hardware class name.
    #[serde(default)]
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: String,
}

/// Credentials for S3-compatible storage resources.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct StorageS3Credentials {
    /// Endpoint URLs.
    #[serde(default)]
    pub endpoints: Vec<String>,
    /// Access key.
    #[serde(default, rename = "accessKey")]
    pub access_key: String,
    /// Secret key.
    #[serde(default, rename = "secretKey")]
    pub secret_key: String,
    /// User key.
    #[serde(default, rename = "userKey")]
    pub user_key: String,
}

/// Metadata for a storage bucket.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct StorageBucketMetadata {
    /// Bucket id.
    #[serde(default, rename = "bucketId")]
    pub bucket_id: i64,
    /// Label.
    #[serde(default)]
    pub label: String,
    /// Readiness flag.
    #[serde(default)]
    pub ready: bool,
    /// Privacy flag.
    #[serde(default)]
    pub private: bool,
    /// Assignment timestamp.
    #[serde(default, rename = "assignedOn")]
    pub assigned_on: String,
    /// Location.
    #[serde(default)]
    pub location: V3Location,
    /// Capacity.
    #[serde(default)]
    pub capacity: V3Capacity,
    /// Hardware class.
    #[serde(default, rename = "hardwareClass")]
    pub hardware_class: StorageHardwareClass,
}

/// Storage bucket, decoded from both flat list rows and nested single-get responses.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct StorageBucket {
    /// Credentials present on single get responses.
    pub credentials: StorageS3Credentials,
    /// Bucket metadata.
    pub metadata: StorageBucketMetadata,
}

impl<'de> Deserialize<'de> for StorageBucket {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("storage bucket must be an object"))?;
        if object.contains_key("metadata") {
            #[derive(Deserialize)]
            struct Nested {
                #[serde(default)]
                credentials: StorageS3Credentials,
                metadata: StorageBucketMetadata,
            }
            let nested = Nested::deserialize(value).map_err(serde::de::Error::custom)?;
            ensure_non_empty_id(nested.metadata.bucket_id, "bucketId")
                .map_err(serde::de::Error::custom)?;
            return Ok(Self {
                credentials: nested.credentials,
                metadata: nested.metadata,
            });
        }
        let metadata =
            StorageBucketMetadata::deserialize(value).map_err(serde::de::Error::custom)?;
        ensure_non_empty_id(metadata.bucket_id, "bucketId").map_err(serde::de::Error::custom)?;
        Ok(Self {
            credentials: StorageS3Credentials::default(),
            metadata,
        })
    }
}

/// Metadata for a VPC.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcMetadata {
    /// Creation timestamp.
    #[serde(default, rename = "createdOn")]
    pub created_on: String,
    /// Label.
    #[serde(default)]
    pub label: String,
    /// Description.
    #[serde(default)]
    pub description: String,
    /// Ready timestamp.
    #[serde(default, rename = "readyOn")]
    pub ready_on: String,
    /// Status.
    #[serde(default)]
    pub status: String,
    /// Uptime in seconds.
    #[serde(default, rename = "uptimeSeconds")]
    pub uptime_seconds: i64,
}

/// VPC floating IPs.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcFloatingIps {
    /// IPv4 floating IPs.
    #[serde(default)]
    pub ipv4: Vec<String>,
    /// IPv6 floating IPs.
    #[serde(default)]
    pub ipv6: Vec<String>,
}

/// VPC bastion address set.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcBastionAddresses {
    /// IPv4 address.
    #[serde(default)]
    pub ipv4: String,
    /// IPv6 address.
    #[serde(default)]
    pub ipv6: String,
}

/// VPC bastion settings.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcBastion {
    /// Whether bastion is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Bastion addresses.
    #[serde(default)]
    pub addresses: VpcBastionAddresses,
    /// SSH port.
    #[serde(default)]
    pub port: Option<i64>,
}

/// VPC decoded from both flat list rows and nested single-get responses.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct Vpc {
    /// VPC id.
    pub vpc_id: i64,
    /// VPC metadata.
    pub metadata: VpcMetadata,
    /// Location.
    pub location: V3Location,
    /// Bastion settings.
    pub bastion: VpcBastion,
    /// Floating IPs.
    pub floating_ips: VpcFloatingIps,
}

impl<'de> Deserialize<'de> for Vpc {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut value = Value::deserialize(deserializer)?;
        let object = value
            .as_object_mut()
            .ok_or_else(|| serde::de::Error::custom("VPC must be an object"))?;
        if !object.contains_key("metadata") {
            let id = object.get("id").and_then(Value::as_i64).unwrap_or_default();
            object.insert("vpcId".to_string(), Value::Number(id.into()));
            let metadata = metadata_from_object(object);
            object.insert("metadata".to_string(), Value::Object(metadata));
        }
        #[derive(Deserialize)]
        struct RawVpc {
            #[serde(default, rename = "vpcId")]
            vpc_id: i64,
            #[serde(default)]
            metadata: VpcMetadata,
            #[serde(default)]
            location: V3Location,
            #[serde(default)]
            bastion: VpcBastion,
            #[serde(default, rename = "floatingIps")]
            floating_ips: VpcFloatingIps,
        }
        let raw = RawVpc::deserialize(value).map_err(serde::de::Error::custom)?;
        ensure_non_empty_id(raw.vpc_id, "vpcId").map_err(serde::de::Error::custom)?;
        Ok(Self {
            vpc_id: raw.vpc_id,
            metadata: raw.metadata,
            location: raw.location,
            bastion: raw.bastion,
            floating_ips: raw.floating_ips,
        })
    }
}

/// NKE tag.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeTag {
    /// Tag name.
    #[serde(default)]
    pub name: String,
    /// Tag description.
    #[serde(default)]
    pub description: String,
    /// Optional icon.
    #[serde(default)]
    pub icon: String,
    /// Optional color.
    #[serde(default)]
    pub color: String,
}

/// NKE cluster status block.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeClusterStatus {
    /// Cluster lifecycle status.
    #[serde(default)]
    pub cluster: String,
    /// Scaling status.
    #[serde(default)]
    pub scaling: String,
}

/// NKE cluster version block.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeClusterVersion {
    /// Active Kubernetes version.
    #[serde(default)]
    pub active: String,
    /// Requested Kubernetes version.
    #[serde(default)]
    pub requested: Option<String>,
}

/// NKE cluster node summary.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeClusterNodes {
    /// Total nodes.
    #[serde(default)]
    pub total: i64,
    /// Ready nodes.
    #[serde(default)]
    pub ready: i64,
    /// Building nodes.
    #[serde(default)]
    pub building: i64,
    /// Minimum node count.
    #[serde(default)]
    pub minimum: i64,
    /// Maximum node count.
    #[serde(default)]
    pub maximum: Option<i64>,
    /// Outdated nodes.
    #[serde(default)]
    pub outdated: i64,
    /// Deleting nodes.
    #[serde(default)]
    pub deleting: i64,
    /// Unevictable nodes.
    #[serde(default)]
    pub unevictable: i64,
}

/// NKE cluster, decoded from current vAPI3 cluster responses.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeCluster {
    /// Cluster id.
    #[serde(default, rename = "clusterId")]
    pub cluster_id: i64,
    /// Cluster name.
    #[serde(default)]
    pub name: String,
    /// Contract id.
    #[serde(default, rename = "contractId")]
    pub contract_id: i64,
    /// VPC id.
    #[serde(default, rename = "vpcId")]
    pub vpc_id: i64,
    /// Replica count.
    #[serde(default)]
    pub replicas: i64,
    /// Autoscaling flag, returned as 0 or 1 by the API.
    #[serde(default, rename = "doAutoscaling")]
    pub do_autoscaling: i64,
    /// Dual-stack flag.
    #[serde(default, rename = "isDualStack")]
    pub is_dual_stack: bool,
    /// High-availability flag. The JSON spelling preserves the API typo.
    #[serde(default, rename = "hasHighAvailabity")]
    pub has_high_availability: bool,
    /// Status block.
    #[serde(default)]
    pub status: NkeClusterStatus,
    /// Version block.
    #[serde(default)]
    pub version: NkeClusterVersion,
    /// Location.
    #[serde(default)]
    pub location: V3Location,
    /// Package.
    #[serde(default)]
    pub package: V3Package,
    /// Node summary.
    #[serde(default)]
    pub nodes: NkeClusterNodes,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<NkeTag>,
}

/// NKE worker node status.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeWorkerNodeStatus {
    /// Whether the node is ready.
    #[serde(default)]
    pub ready: bool,
}

/// NKE worker node.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeWorkerNode {
    /// Worker node id.
    #[serde(default, rename = "workerNodeId")]
    pub worker_node_id: i64,
    /// Cluster id.
    #[serde(default, rename = "clusterId")]
    pub cluster_id: i64,
    /// Node name.
    #[serde(default)]
    pub name: String,
    /// Server package id.
    #[serde(default, rename = "mbpkgid")]
    pub mbpkgid: i64,
    /// Location id.
    #[serde(default, rename = "locationId")]
    pub location_id: i64,
    /// Node status.
    #[serde(default)]
    pub status: NkeWorkerNodeStatus,
    /// Package.
    #[serde(default)]
    pub package: V3Package,
    /// Location.
    #[serde(default)]
    pub location: V3Location,
}

/// NKE log entry.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeLogEntry {
    /// Recording timestamp.
    #[serde(default, rename = "recordedOn")]
    pub recorded_on: String,
    /// Message.
    #[serde(default)]
    pub message: String,
}

/// NKE access URLs.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeAccessUrls {
    /// Kubernetes API URL.
    #[serde(default)]
    pub api: String,
    /// Prometheus URL.
    #[serde(default)]
    pub prometheus: String,
    /// Kubernetes dashboard URL.
    #[serde(default, rename = "kubernetesDashboard")]
    pub kubernetes_dashboard: String,
}

pub(crate) fn decode_required<T>(raw: &[u8], context: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_slice(raw).map_err(|err| Error::Decode(format!("{context}: {err}")))
}

fn metadata_from_object(object: &Map<String, Value>) -> Map<String, Value> {
    let mut metadata = Map::new();
    for key in [
        "createdOn",
        "label",
        "description",
        "readyOn",
        "status",
        "uptimeSeconds",
    ] {
        if let Some(value) = object.get(key) {
            metadata.insert(key.to_string(), value.clone());
        }
    }
    metadata
}

fn ensure_non_empty_id(id: i64, name: &str) -> std::result::Result<(), String> {
    if id == 0 {
        Err(format!("{name} is missing or zero"))
    } else {
        Ok(())
    }
}
