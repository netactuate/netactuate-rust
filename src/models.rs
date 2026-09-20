use crate::error::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

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

/// A dedicated device available for purchase, as returned by
/// [`crate::Client::filter_dedicated_devices`].
///
/// The set of fields varies with the filters applied and with the columns the portal's own
/// filter UI requests, so this is left as an untyped map rather than a fixed struct.
pub type DedicatedDevice = Map<String, Value>;

/// A dedicated server plan available at a location, as returned by
/// [`crate::Client::list_dedicated_plans`].
///
/// Left as an untyped map because the plan schema is not fully specified and varies by
/// location and device class.
pub type DedicatedPlan = Map<String, Value>;

/// The power status of a dedicated server, as returned by
/// [`crate::Client::get_dedicated_server_power_status`].
pub type DedicatedPowerStatus = Map<String, Value>;

/// A location where dedicated servers can be deployed.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DedicatedLocation {
    /// Short location code.
    #[serde(default)]
    pub short_name: String,
    /// Public description of the location.
    #[serde(default)]
    pub pub_description: String,
    /// Location id.
    #[serde(default)]
    pub location_id: i64,
}

/// An id/name pair nested within a [`DedicatedOsProfile`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DedicatedIdName {
    /// Numeric id.
    #[serde(default, rename = "ID")]
    pub id: i64,
    /// Display name.
    #[serde(default, rename = "Name")]
    pub name: String,
}

/// An operating system profile compatible with a dedicated device.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DedicatedOsProfile {
    /// OS id.
    #[serde(default, rename = "OSID")]
    pub os_id: i64,
    /// OS name.
    #[serde(default, rename = "Name")]
    pub name: String,
    /// OS group name.
    #[serde(default, rename = "GroupName")]
    pub group_name: String,
    /// Descriptive tags.
    #[serde(default, rename = "Tags")]
    pub tags: Vec<String>,
    /// Disk layouts compatible with this profile.
    #[serde(default, rename = "DiskLayouts")]
    pub disk_layouts: Vec<DedicatedIdName>,
    /// Build scripts compatible with this profile.
    #[serde(default, rename = "Scripts")]
    pub scripts: Vec<DedicatedIdName>,
    /// Default disk layout id.
    #[serde(default, rename = "DefaultDiskLayout")]
    pub default_disk_layout: i64,
    /// Default build script ids.
    #[serde(default, rename = "DefaultScripts")]
    pub default_scripts: Vec<i64>,
    /// Whether SSH keys can be injected at build time.
    #[serde(default, rename = "AllowSSHKeys")]
    pub allow_ssh_keys: i64,
    /// Whether a root password can be set at build time.
    #[serde(default, rename = "SetRootPassword")]
    pub set_root_password: i64,
    /// Whether this profile is a rescue image.
    #[serde(default, rename = "RescueImage")]
    pub rescue_image: i64,
    /// Whether the profile is publicly available.
    #[serde(default, rename = "Public")]
    pub public: i64,
    /// Whether the profile is enabled.
    #[serde(default, rename = "Enabled")]
    pub enabled: i64,
    /// Creation timestamp.
    #[serde(default, rename = "Created")]
    pub created: String,
    /// Last update timestamp.
    #[serde(default, rename = "LastUpdated")]
    pub last_updated: String,
    /// Profile id.
    #[serde(default, rename = "ProfileID")]
    pub profile_id: i64,
    /// CPU architecture.
    #[serde(default, rename = "Arch")]
    pub arch: String,
    /// Profile flavor.
    #[serde(default, rename = "Flavor")]
    pub flavor: String,
    /// Location id restricting this profile, when scoped to a single location.
    #[serde(default, rename = "LocationID")]
    pub location_id: Option<i64>,
}

/// Response returned when a dedicated server build or purchase is enqueued.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DedicatedServerBuild {
    /// Billing package id.
    #[serde(default)]
    pub mbpkgid: i64,
    /// Build status text.
    #[serde(default)]
    pub status: String,
    /// Build job id.
    #[serde(default)]
    pub build: i64,
}

/// A dedicated server package on the account, as returned by
/// [`crate::Client::list_dedicated_servers`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct DedicatedServer {
    /// Server record id.
    #[serde(default)]
    pub id: i64,
    /// Datacenter id.
    #[serde(default)]
    pub datacenter_id: i64,
    /// Nonzero while the server is being canceled.
    #[serde(default)]
    pub canceling: i64,
    /// Billing package id.
    #[serde(default)]
    pub mbpkgid: i64,
    /// Recurring price.
    #[serde(default)]
    pub price: String,
    /// Server hostname.
    #[serde(default)]
    pub hostname: String,
    /// Primary interface MAC address.
    #[serde(default)]
    pub eth0_mac: String,
    /// Secondary interface MAC address, when present.
    #[serde(default)]
    pub eth1_mac: Option<String>,
    /// IPMI interface MAC address.
    #[serde(default)]
    pub ipmi_mac: String,
    /// Primary IPv4 address.
    #[serde(default)]
    pub primary_ip: String,
    /// Primary IPv6 address, when assigned.
    #[serde(default)]
    pub primary_ipv6: Option<String>,
    /// Whether the netactuate provisioning system is installed.
    #[serde(default)]
    pub nps_installed: i64,
    /// Operating system reported by the provisioning system.
    #[serde(default)]
    pub nps_os: String,
    /// Motherboard model, when known.
    #[serde(default)]
    pub mb_model: Option<String>,
    /// Primary CPU model.
    #[serde(default)]
    pub cpu0_model: String,
    /// Secondary CPU model, when present.
    #[serde(default)]
    pub cpu1_model: Option<String>,
    /// Total RAM in megabytes.
    #[serde(default)]
    pub total_ram: i64,
    /// Public IPMI address.
    #[serde(default)]
    pub ipmi_pubip: String,
    /// IPMI console username, when set.
    #[serde(default)]
    pub ipmi_cxuser: Option<String>,
    /// IPMI console password, when set.
    #[serde(default)]
    pub ipmi_cxpass: Option<String>,
    /// IPMI status code.
    #[serde(default)]
    pub ipmi_status: i64,
    /// Nonzero while the server is locked.
    #[serde(default)]
    pub locked: i64,
    /// Lock reason, when locked.
    #[serde(default)]
    pub locked_msg: Option<String>,
    /// Timestamp of the last IPMI status change.
    #[serde(default)]
    pub ipmi_status_time: String,
    /// Out-of-band management id.
    ///
    /// The platform sends this as either a JSON number or a numeric string depending on
    /// account, so it is normalized to a string.
    #[serde(default, deserialize_with = "flexible_string")]
    pub ob_id: String,
    /// Free-form info text.
    #[serde(default)]
    pub info: String,
    /// Display title.
    #[serde(default)]
    pub title: String,
    /// Whether automatic IPMI refresh is enabled.
    #[serde(default)]
    pub ipmi_refresh_enabled: i64,
    /// Location display name.
    #[serde(default)]
    pub location: String,
    /// IP subnet id.
    #[serde(default)]
    pub ip_subnet_id: i64,
    /// IP subnet name.
    #[serde(default)]
    pub ip_subnet_name: String,
    /// Package lifecycle status.
    #[serde(default)]
    pub package_status: String,
    /// Build record.
    ///
    /// The platform sends an object while a build is in progress and null otherwise, so this
    /// carries the raw value rather than a fixed shape.
    #[serde(default)]
    pub building: Value,
}

/// A custom or base image, as returned by [`crate::Client::get_my_images`] and
/// [`crate::Client::get_image`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Image {
    /// Image id.
    #[serde(default)]
    pub id: i64,
    /// Image name.
    #[serde(default, rename = "os")]
    pub name: String,
    /// Image description, when set.
    #[serde(default)]
    pub description: Option<String>,
    /// Image type.
    #[serde(default, rename = "type")]
    pub image_type: String,
    /// Image subtype.
    #[serde(default)]
    pub subtype: String,
    /// Architecture bit width.
    #[serde(default)]
    pub bits: String,
    /// Underlying virtualization technology.
    #[serde(default)]
    pub tech: String,
    /// Image size.
    #[serde(default)]
    pub size: String,
    /// Image category.
    #[serde(default)]
    pub category: String,
    /// Whether the image is enabled, when the platform reports it.
    #[serde(default, rename = "os_enabled")]
    pub enabled: Option<i64>,
    /// Whether a bash build script can be attached.
    #[serde(default)]
    pub script_bash: i64,
    /// Whether a cloud-init build script can be attached.
    #[serde(default)]
    pub script_cloudinit: i64,
    /// Creation timestamp.
    #[serde(default)]
    pub created: String,
    /// Last update timestamp.
    #[serde(default)]
    pub updated: String,
    /// The image's currently running build job, when one is in progress.
    #[serde(default)]
    pub active_build: Option<ImageBuild>,
}

/// A queued or completed image build or delete job.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ImageBuild {
    /// Job id.
    #[serde(default)]
    pub id: i64,
    /// Job status code.
    #[serde(default)]
    pub status: i64,
    /// Command the job runs.
    #[serde(default)]
    pub command: String,
    /// Timestamp the job was inserted.
    #[serde(default)]
    pub ts_insert: String,
    /// Id of the server the job operates on.
    #[serde(default)]
    pub mb_id: i64,
    /// Billing package id of the server the job operates on.
    #[serde(default)]
    pub mb_pkgid: i64,
    /// Job parameters.
    #[serde(default)]
    pub params: String,
    /// Raw build packet sent to the provisioning system.
    #[serde(default)]
    pub build_packet: String,
    /// Raw response from the provisioning system.
    #[serde(default)]
    pub response: String,
    /// Creation timestamp.
    #[serde(default)]
    pub created: String,
    /// Last update timestamp.
    #[serde(default)]
    pub last_updated: String,
}

/// The status of a queued image build or delete job, as returned by
/// [`crate::Client::get_image_queue_status`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ImageQueueStatus {
    /// Job status text, such as `Complete` or `Failed`.
    #[serde(default)]
    pub status: String,
    /// Completion percentage.
    #[serde(default)]
    pub percent: i64,
    /// Raw response from the provisioning system.
    #[serde(default)]
    pub response: String,
    /// Id of the resulting image.
    #[serde(default)]
    pub image_id: i64,
    /// Name of the resulting image.
    #[serde(default)]
    pub image_name: String,
    /// Help text associated with the job.
    #[serde(default)]
    pub image_help: String,
    /// Location the job is running in.
    #[serde(default)]
    pub location: String,
    /// Billing package id of the server the job operates on.
    #[serde(default)]
    pub mb_pkgid: i64,
    /// Hostname of the server the job operates on.
    #[serde(default)]
    pub fqdn: String,
    /// Operating system name.
    #[serde(default)]
    pub os: String,
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

/// An SSH key stored on the account.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct SshKey {
    /// Key id.
    pub id: i64,
    /// Key label.
    pub name: String,
    /// Public key content.
    #[serde(rename = "ssh_key")]
    pub key: String,
    /// Key fingerprint.
    pub fingerprint: String,
}

impl<'de> Deserialize<'de> for SshKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // A key that no longer exists does not 404: the API answers 200 with a null
        // body. Treating null as the zero value here lets the caller detect absence
        // by id rather than failing to decode.
        let value = Value::deserialize(deserializer)?;
        if value.is_null() {
            return Ok(Self::default());
        }
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            name: String,
            #[serde(default, rename = "ssh_key")]
            key: String,
            #[serde(default)]
            fingerprint: String,
        }
        let raw = Raw::deserialize(value).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            name: raw.name,
            key: raw.key,
            fingerprint: raw.fingerprint,
        })
    }
}

/// A tag, together with the resources it is assigned to.
///
/// Tags are global and shared across the account: the same tag can be attached to
/// many resources, such as servers or NKE clusters.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Tag {
    /// Tag id.
    #[serde(default)]
    pub id: i64,
    /// Tag name.
    #[serde(default)]
    pub name: String,
    /// Tag description.
    #[serde(default)]
    pub description: String,
    /// Icon identifier.
    #[serde(default)]
    pub icon: String,
    /// Display color.
    #[serde(default)]
    pub color: String,
    /// Whether this is the account's default tag.
    #[serde(default)]
    pub is_default: i64,
    /// Whether the tag is marked as a favorite.
    #[serde(default)]
    pub is_favorite: i64,
    /// Whether the tag is locked against deletion.
    #[serde(default)]
    pub is_locked: i64,
    /// Whether the tag is shown on the dashboard.
    #[serde(default)]
    pub show_dashboard: i64,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: String,
    /// Owning account id.
    #[serde(default)]
    pub mb_id: i64,
    /// Count of resources assigned to this tag.
    #[serde(default)]
    pub resources_count: i64,
    /// Resources currently assigned to this tag.
    #[serde(default)]
    pub resources: Vec<TagResource>,
}

/// A single tag-to-resource assignment.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct TagResource {
    /// Assignment row id.
    #[serde(default)]
    pub id: i64,
    /// Id of the tag this assignment belongs to.
    #[serde(default)]
    pub resource_tag_id: i64,
    /// Resource type name, such as `server` or `nke_cluster`.
    #[serde(default)]
    pub resource_name: String,
    /// Primary numeric id of the assigned resource.
    #[serde(default)]
    pub identifier: i64,
    /// Assignment timestamp.
    #[serde(default)]
    pub created_at: String,
}

/// A log entry recorded for a tag.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct TagLog {
    /// Log entry id.
    #[serde(default)]
    pub id: i64,
    /// Id of the tag this entry belongs to.
    #[serde(default)]
    pub tag_id: i64,
    /// Action performed.
    #[serde(default)]
    pub action: String,
    /// Human readable log message.
    #[serde(default)]
    pub message: String,
    /// Timestamp the entry was recorded.
    #[serde(default)]
    pub created_at: String,
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
#[derive(Clone, Serialize, PartialEq, Eq, Default)]
pub struct StorageBucket {
    /// Credentials present on single get responses.
    pub credentials: StorageS3Credentials,
    /// Bucket metadata.
    pub metadata: StorageBucketMetadata,
}

// credential fields omitted from Debug
crate::redacted_debug!(StorageBucket; metadata; credentials);

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

/// Credentials for block storage resources.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct StorageBlockCredentials {
    /// Endpoint URLs.
    #[serde(default)]
    pub endpoints: Vec<String>,
    /// User key.
    #[serde(default, rename = "userKey")]
    pub user_key: String,
    /// Secret key.
    #[serde(default, rename = "secretKey")]
    pub secret_key: String,
    /// Ceph pool backing the namespace or volume.
    #[serde(default)]
    pub pool: String,
    /// Ceph namespace.
    #[serde(default)]
    pub namespace: String,
    /// Ceph cluster id.
    #[serde(default, rename = "clusterId")]
    pub cluster_id: String,
    /// RBD image name, present on a volume.
    #[serde(default, rename = "imageName")]
    pub image_name: String,
}

/// A storage type available to the account.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct StorageType {
    /// Type code, such as "bucket" or "block".
    #[serde(default, rename = "type")]
    pub type_code: String,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: String,
}

/// A location where storage resources can be created, with the hardware class offered there.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct StorageLocation {
    /// Location.
    #[serde(default)]
    pub location: V3Location,
    /// Hardware class offered at this location.
    #[serde(default)]
    pub hardware: StorageHardwareClass,
}

/// Metadata for an object store.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct StorageObjectStoreMetadata {
    /// Object store id.
    #[serde(default, rename = "objectStoreId")]
    pub object_store_id: i64,
    /// Label.
    #[serde(default)]
    pub label: String,
    /// Readiness flag.
    #[serde(default)]
    pub ready: bool,
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

/// Object store, decoded from both flat list rows and nested single-get responses.
#[derive(Clone, Serialize, PartialEq, Eq, Default)]
pub struct StorageObjectStore {
    /// Credentials present on single get responses.
    pub credentials: StorageS3Credentials,
    /// Object store metadata.
    pub metadata: StorageObjectStoreMetadata,
}

// credential fields omitted from Debug
crate::redacted_debug!(StorageObjectStore; metadata; credentials);

impl<'de> Deserialize<'de> for StorageObjectStore {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("object store must be an object"))?;
        if object.contains_key("metadata") {
            #[derive(Deserialize)]
            struct Nested {
                #[serde(default)]
                credentials: StorageS3Credentials,
                metadata: StorageObjectStoreMetadata,
            }
            let nested = Nested::deserialize(value).map_err(serde::de::Error::custom)?;
            ensure_non_empty_id(nested.metadata.object_store_id, "objectStoreId")
                .map_err(serde::de::Error::custom)?;
            return Ok(Self {
                credentials: nested.credentials,
                metadata: nested.metadata,
            });
        }
        let metadata =
            StorageObjectStoreMetadata::deserialize(value).map_err(serde::de::Error::custom)?;
        ensure_non_empty_id(metadata.object_store_id, "objectStoreId")
            .map_err(serde::de::Error::custom)?;
        Ok(Self {
            credentials: StorageS3Credentials::default(),
            metadata,
        })
    }
}

/// Metadata for a block storage namespace.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct StorageBlockNamespaceMetadata {
    /// Block namespace id.
    #[serde(default, rename = "blockNamespaceId")]
    pub block_namespace_id: i64,
    /// Label.
    #[serde(default)]
    pub label: String,
    /// Readiness flag.
    #[serde(default)]
    pub ready: bool,
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

/// Block storage namespace, decoded from both flat list rows and nested single-get responses.
#[derive(Clone, Serialize, PartialEq, Eq, Default)]
pub struct StorageBlockNamespace {
    /// Credentials present on single get responses.
    pub credentials: StorageBlockCredentials,
    /// Namespace metadata.
    pub metadata: StorageBlockNamespaceMetadata,
}

// credential fields omitted from Debug
crate::redacted_debug!(StorageBlockNamespace; metadata; credentials);

impl<'de> Deserialize<'de> for StorageBlockNamespace {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("block namespace must be an object"))?;
        if object.contains_key("metadata") {
            #[derive(Deserialize)]
            struct Nested {
                #[serde(default)]
                credentials: StorageBlockCredentials,
                metadata: StorageBlockNamespaceMetadata,
            }
            let nested = Nested::deserialize(value).map_err(serde::de::Error::custom)?;
            ensure_non_empty_id(nested.metadata.block_namespace_id, "blockNamespaceId")
                .map_err(serde::de::Error::custom)?;
            return Ok(Self {
                credentials: nested.credentials,
                metadata: nested.metadata,
            });
        }
        let metadata =
            StorageBlockNamespaceMetadata::deserialize(value).map_err(serde::de::Error::custom)?;
        ensure_non_empty_id(metadata.block_namespace_id, "blockNamespaceId")
            .map_err(serde::de::Error::custom)?;
        Ok(Self {
            credentials: StorageBlockCredentials::default(),
            metadata,
        })
    }
}

/// Metadata for a block volume.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct StorageBlockVolumeMetadata {
    /// Block volume id.
    #[serde(default, rename = "blockVolumeId")]
    pub block_volume_id: i64,
    /// Label.
    #[serde(default)]
    pub label: String,
    /// Readiness flag.
    #[serde(default)]
    pub ready: bool,
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

/// Block volume, decoded from both flat list rows and nested single-get responses.
///
/// A platform defect on the single-volume GET can return object-store shaped metadata with
/// no `blockVolumeId`. This type decodes leniently rather than rejecting that shape, because
/// recovering the id is the job of the caller that knows which id it asked for; see
/// `V3Client::get_storage_block_volume`.
#[derive(Clone, Serialize, PartialEq, Eq, Default)]
pub struct StorageBlockVolume {
    /// Credentials present on single get responses.
    pub credentials: StorageBlockCredentials,
    /// Volume metadata.
    pub metadata: StorageBlockVolumeMetadata,
}

// credential fields omitted from Debug
crate::redacted_debug!(StorageBlockVolume; metadata; credentials);

impl<'de> Deserialize<'de> for StorageBlockVolume {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("block volume must be an object"))?;
        if object.contains_key("metadata") {
            #[derive(Deserialize)]
            struct Nested {
                #[serde(default)]
                credentials: StorageBlockCredentials,
                metadata: StorageBlockVolumeMetadata,
            }
            let nested = Nested::deserialize(value).map_err(serde::de::Error::custom)?;
            return Ok(Self {
                credentials: nested.credentials,
                metadata: nested.metadata,
            });
        }
        let metadata =
            StorageBlockVolumeMetadata::deserialize(value).map_err(serde::de::Error::custom)?;
        Ok(Self {
            credentials: StorageBlockCredentials::default(),
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

/// IP reservations held by a VPC, grouped by consumer.
///
/// Each field is kept as raw JSON because the reservation rows vary in shape across accounts;
/// a caller decodes the shape it expects rather than this crate guessing one.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct VpcIpReservations {
    /// Gateway IP reservations.
    #[serde(default)]
    pub gateways: Value,
    /// Interface IP reservations.
    #[serde(default)]
    pub interfaces: Value,
    /// VM IP reservations.
    #[serde(default)]
    pub vms: Value,
}

/// Bastion addresses exposed for SSH access to a VPC.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcSshBastion {
    /// IPv4 address.
    #[serde(default)]
    pub ipv4: String,
    /// IPv6 address.
    #[serde(default)]
    pub ipv6: String,
}

/// Enablement dates for an SSH key authorized on a VPC.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcSshKeyDates {
    /// Timestamp the key was added.
    #[serde(default)]
    pub created: String,
    /// Timestamp the key was enabled, when it is.
    #[serde(default)]
    pub enabled: Option<String>,
}

/// An SSH key authorized for bastion access to a VPC.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcSshKey {
    /// Key id.
    #[serde(default)]
    pub id: i64,
    /// Underlying account SSH key id.
    #[serde(default, rename = "sshKeyId")]
    pub ssh_key_id: i64,
    /// Key label.
    #[serde(default)]
    pub name: String,
    /// Key fingerprint.
    #[serde(default)]
    pub fingerprint: String,
    /// Public key content.
    #[serde(default, rename = "publicKey")]
    pub public_key: String,
    /// Enablement dates, when present.
    #[serde(default)]
    pub dates: Option<VpcSshKeyDates>,
}

/// Bastion SSH settings for a VPC.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct VpcSshSettings {
    /// SSH port, when a non-default one is configured.
    pub port: Option<i64>,
    /// Whether bastion SSH access is enabled.
    pub enabled: bool,
    /// Authorized keys, keyed by key id.
    pub keys: std::collections::HashMap<String, VpcSshKey>,
    /// Bastion addresses.
    pub bastion: Option<VpcSshBastion>,
}

impl<'de> Deserialize<'de> for VpcSshSettings {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // The port arrives as a number, a numeric string, or an object nesting the number
        // under port/value/number. The keys arrive either as a map of key id to key, or
        // wrapped in the {"data": [...]} list envelope this API uses elsewhere. Both cases
        // are normalized here rather than left to two different call sites.
        let value = Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("VPC SSH settings must be an object"))?;

        let enabled = object
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let bastion = object
            .get("bastion")
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(serde::de::Error::custom)?;
        let port = match object.get("port") {
            None | Some(Value::Null) => None,
            Some(raw) => Some(decode_vpc_ssh_port(raw).map_err(serde::de::Error::custom)?),
        };
        let keys = match object.get("keys") {
            None | Some(Value::Null) => std::collections::HashMap::new(),
            Some(Value::Object(map)) => {
                if let Some(Value::Array(rows)) = map.get("data") {
                    let mut keys = std::collections::HashMap::with_capacity(rows.len());
                    for row in rows {
                        let key: VpcSshKey = serde_json::from_value(row.clone())
                            .map_err(serde::de::Error::custom)?;
                        keys.insert(key.id.to_string(), key);
                    }
                    keys
                } else {
                    let mut keys = std::collections::HashMap::with_capacity(map.len());
                    for (id, raw) in map {
                        let key: VpcSshKey = serde_json::from_value(raw.clone())
                            .map_err(serde::de::Error::custom)?;
                        keys.insert(id.clone(), key);
                    }
                    keys
                }
            }
            Some(other) => {
                return Err(serde::de::Error::custom(format!(
                    "VPC SSH keys must be an object, got {other}"
                )))
            }
        };
        Ok(Self {
            port,
            enabled,
            keys,
            bastion,
        })
    }
}

fn decode_vpc_ssh_port(raw: &Value) -> std::result::Result<i64, String> {
    match raw {
        Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| format!("VPC SSH port {number} is not an integer")),
        Value::String(text) => text
            .parse()
            .map_err(|_| format!("VPC SSH port {text:?} is not an integer")),
        Value::Object(fields) => {
            for key in ["port", "value", "number"] {
                if let Some(value) = fields.get(key) {
                    return decode_vpc_ssh_port(value);
                }
            }
            Err("VPC SSH port object has no port value".to_string())
        }
        other => Err(format!("VPC SSH port has unexpected shape: {other}")),
    }
}

/// A backend registered under a VPC backend template.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcBackend {
    /// Backend id.
    #[serde(default, rename = "backendHostId")]
    pub backend_host_id: i64,
    /// Backend label.
    #[serde(default)]
    pub name: String,
    /// Public address.
    #[serde(default)]
    pub address: String,
    /// Internal address.
    #[serde(default, rename = "internalAddress")]
    pub internal_address: String,
}

/// A VPC backend template, grouping a set of backends behind one name.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcBackendTemplate {
    /// Template id.
    #[serde(default, rename = "backendTemplateId")]
    pub backend_template_id: i64,
    /// Template name.
    #[serde(default)]
    pub name: String,
    /// Template description.
    #[serde(default)]
    pub description: String,
    /// Backends registered under this template.
    #[serde(default, rename = "backendHosts")]
    pub backend_hosts: Vec<VpcBackend>,
}

/// A TCP or UDP port range used by a VPC gateway rule.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcPortRange {
    /// First port in the range.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub start: i64,
    /// Last port in the range.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub end: i64,
}

/// A floating IP assigned to a VPC.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcFloatingIp {
    /// Floating IP id.
    #[serde(default, rename = "floatingIpId")]
    pub floating_ip_id: i64,
    /// IP address.
    #[serde(default)]
    pub address: String,
    /// IP version, 4 or 6.
    #[serde(default, rename = "ipVersion")]
    pub ip_version: i64,
    /// Reverse DNS record.
    #[serde(default)]
    pub ptr: String,
    /// Whether this is the VPC's primary floating IP.
    #[serde(default, rename = "isPrimary")]
    pub is_primary: bool,
}

/// A firewall rule on a VPC gateway.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcFirewallRule {
    /// Rule id.
    #[serde(default, rename = "firewallRuleId")]
    pub firewall_rule_id: i64,
    /// IP version the rule applies to, 4 or 6.
    #[serde(default, rename = "ipVersion")]
    pub ip_version: i64,
    /// Traffic direction, "inbound" or "outbound".
    #[serde(default)]
    pub direction: String,
    /// Protocol the rule matches, for example "tcp" or "udp".
    #[serde(default)]
    pub protocol: String,
    /// Rule description.
    #[serde(default)]
    pub description: String,
    /// Network the rule applies to, in CIDR notation.
    #[serde(default)]
    pub network: String,
    /// Matched address, when the rule targets a single address rather than a network.
    #[serde(default)]
    pub address: String,
    /// Network prefix length, when the rule targets a network.
    #[serde(default, rename = "prefixLength")]
    pub prefix_length: i64,
    /// Port range the rule matches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<VpcPortRange>,
}

/// SNAT rule match criteria, matching by internal (private) network CIDR.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcSnatMatch {
    /// Internal CIDR block this rule applies to.
    #[serde(
        default,
        rename = "internalCidr",
        skip_serializing_if = "String::is_empty"
    )]
    pub internal_cidr: String,
}

/// Address range a SNAT rule translates matching traffic to.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcSnatTranslationAddress {
    /// First address in the translation pool.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub start: String,
    /// Last address in the translation pool.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub end: String,
}

/// Translation applied by a SNAT rule.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcSnatTranslation {
    /// Address pool traffic is translated to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<VpcSnatTranslationAddress>,
    /// Port range traffic is translated to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<VpcPortRange>,
}

/// Placement of a SNAT rule relative to the others on a VPC gateway.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcSnatPriority {
    /// Placement location, for example "first", "last" or "after".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub location: String,
    /// Rule id to place this rule after, when location is "after".
    #[serde(default, rename = "afterSnatRuleId", skip_serializing_if = "is_zero")]
    pub after_snat_rule_id: i64,
}

/// A SNAT rule on a VPC gateway.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcSnatRule {
    /// Rule id.
    #[serde(default, rename = "snatRuleId")]
    pub snat_rule_id: i64,
    /// IP version the rule applies to, 4 or 6.
    #[serde(default, rename = "ipVersion")]
    pub ip_version: i64,
    /// Protocol the rule matches, for example "tcp" or "udp".
    #[serde(default)]
    pub protocol: String,
    /// Rule description.
    #[serde(default)]
    pub description: String,
    /// Match criteria.
    #[serde(default, rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_criteria: Option<VpcSnatMatch>,
    /// Translation applied to matching traffic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translation: Option<VpcSnatTranslation>,
}

/// DNAT rule match criteria, matching by destination address and port.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcDnatMatch {
    /// Destination address to match.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub address: String,
    /// Destination port range to match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<VpcPortRange>,
}

/// Translation applied by a DNAT rule.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcDnatTranslation {
    /// Address traffic is forwarded to.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub address: String,
    /// Port range traffic is forwarded to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<VpcPortRange>,
}

/// Placement of a DNAT rule relative to the others on a VPC gateway.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcDnatPriority {
    /// Placement location, for example "first", "last" or "after".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub location: String,
    /// Rule id to place this rule after, when location is "after".
    #[serde(default, rename = "afterDnatRuleId", skip_serializing_if = "is_zero")]
    pub after_dnat_rule_id: i64,
}

/// A DNAT rule on a VPC gateway.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct VpcDnatRule {
    /// Rule id.
    #[serde(default, rename = "dnatRuleId")]
    pub dnat_rule_id: i64,
    /// IP version the rule applies to, 4 or 6.
    #[serde(default, rename = "ipVersion")]
    pub ip_version: i64,
    /// Protocol the rule matches, for example "tcp" or "udp".
    #[serde(default)]
    pub protocol: String,
    /// Rule description.
    #[serde(default)]
    pub description: String,
    /// Match criteria.
    #[serde(default, rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_criteria: Option<VpcDnatMatch>,
    /// Translation applied to matching traffic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translation: Option<VpcDnatTranslation>,
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

/// NKE add-on catalog entry, describing an installable add-on and the cluster versions it
/// supports.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeAddonCatalogEntry {
    /// Add-on id.
    #[serde(default, rename = "addonId")]
    pub addon_id: i64,
    /// Add-on type, such as `netactuate-dns` or `storage`.
    #[serde(default, rename = "addonType")]
    pub addon_type: String,
    /// Add-on version.
    #[serde(default)]
    pub version: String,
    /// Release channel.
    #[serde(default)]
    pub channel: String,
    /// Human readable name.
    #[serde(default, rename = "displayName")]
    pub display_name: String,
    /// Minimum Kubernetes version this add-on supports.
    #[serde(default, rename = "minKubernetesVersion")]
    pub min_kubernetes_version: String,
    /// Maximum Kubernetes version this add-on supports.
    #[serde(default, rename = "maxKubernetesVersion")]
    pub max_kubernetes_version: String,
    /// Whether the platform installs this add-on by default.
    #[serde(default, rename = "isDefault")]
    pub is_default: bool,
    /// Whether the add-on requires the cluster to run inside a VPC.
    #[serde(default, rename = "requiresVpc")]
    pub requires_vpc: bool,
}

/// Condition reported on an NKE add-on's health or workload health.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeAddonCondition {
    /// Condition type.
    #[serde(default, rename = "type")]
    pub condition_type: String,
    /// Condition status.
    #[serde(default)]
    pub status: String,
    /// Machine readable reason.
    #[serde(default)]
    pub reason: String,
    /// Human readable message.
    #[serde(default)]
    pub message: String,
}

/// Catalog metadata carried on an installed add-on.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeAddonCatalog {
    /// Default version the catalog currently publishes for this add-on.
    #[serde(default, rename = "defaultVersion")]
    pub default_version: String,
}

/// Health as reported by the add-on itself.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeAddonHealth {
    /// Timestamp of the last heartbeat.
    #[serde(default, rename = "lastHeartbeatOn")]
    pub last_heartbeat_on: String,
    /// Version the add-on last reported running.
    #[serde(default, rename = "observedVersion")]
    pub observed_version: String,
    /// Reported conditions.
    #[serde(default)]
    pub conditions: Vec<NkeAddonCondition>,
    /// Timestamp the conditions were last reported.
    #[serde(default, rename = "conditionsReportedOn")]
    pub conditions_reported_on: String,
    /// Summary status string.
    #[serde(default)]
    pub summary: String,
}

/// Health of the workloads an add-on installs.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeAddonWorkloadHealth {
    /// Workload state.
    #[serde(default)]
    pub state: String,
    /// Reported conditions.
    #[serde(default)]
    pub conditions: Vec<NkeAddonCondition>,
}

/// Timestamps tracked across an add-on's lifecycle.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeAddonTimestamps {
    /// When the add-on was requested.
    #[serde(default, rename = "requestedOn")]
    pub requested_on: String,
    /// When the add-on finished installing.
    #[serde(default, rename = "installedOn")]
    pub installed_on: String,
    /// When the add-on was last updated.
    #[serde(default, rename = "updatedOn")]
    pub updated_on: String,
    /// When the add-on was deleted.
    #[serde(default, rename = "deletedOn")]
    pub deleted_on: String,
}

/// DNS zone bound by the `netactuate-dns` add-on, as the add-on reports it back.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeDnsAddonZone {
    /// DNS zone id.
    #[serde(default, rename = "dnsZoneId")]
    pub dns_zone_id: i64,
    /// Zone name.
    #[serde(default)]
    pub zone: String,
    /// Zone mode.
    #[serde(default)]
    pub mode: String,
}

/// StorageClass binding installed by the `storage` add-on, as the add-on reports it back.
///
/// The field names differ from the write side: `makeDefault` is sent when writing, and
/// `isDefaultClass` comes back on read, alongside ids the write side never sees.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeStorageAddonIntegration {
    /// Storage integration id.
    #[serde(default, rename = "storageIntegrationId")]
    pub storage_integration_id: i64,
    /// Block storage namespace id backing this integration.
    #[serde(default, rename = "blockNamespaceId")]
    pub block_namespace_id: i64,
    /// StorageClass name installed in the cluster.
    #[serde(default, rename = "storageClassName")]
    pub storage_class_name: String,
    /// VolumeSnapshotClass name installed in the cluster.
    #[serde(default, rename = "volumeSnapshotClassName")]
    pub volume_snapshot_class_name: String,
    /// Whether this is the cluster's default StorageClass.
    #[serde(default, rename = "isDefaultClass")]
    pub is_default_class: bool,
    /// Reclaim policy applied to volumes created from this class.
    #[serde(default, rename = "reclaimPolicy")]
    pub reclaim_policy: String,
}

/// Config an NKE add-on reports back, which is not the shape it was written with.
///
/// Both known add-on types answer with a list keyed by their own concern: the
/// `netactuate-dns` add-on returns `zones`, and the `storage` add-on returns `integrations`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeAddonReadConfig {
    /// DNS zones bound by the `netactuate-dns` add-on.
    #[serde(default)]
    pub zones: Vec<NkeDnsAddonZone>,
    /// StorageClass integrations installed by the `storage` add-on.
    #[serde(default)]
    pub integrations: Vec<NkeStorageAddonIntegration>,
}

/// An add-on installed on an NKE cluster.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NkeAddon {
    /// Installation id.
    #[serde(default)]
    pub id: i64,
    /// Catalog add-on id.
    #[serde(default, rename = "addonId")]
    pub addon_id: i64,
    /// Cluster id.
    #[serde(default, rename = "clusterId")]
    pub cluster_id: i64,
    /// Add-on type, such as `netactuate-dns` or `storage`.
    #[serde(default, rename = "addonType")]
    pub addon_type: String,
    /// Installed version.
    #[serde(default)]
    pub version: String,
    /// Release channel.
    #[serde(default)]
    pub channel: String,
    /// Human readable name.
    #[serde(default, rename = "displayName")]
    pub display_name: String,
    /// Installation state.
    #[serde(default)]
    pub state: String,
    /// Whether a newer version is available.
    #[serde(default, rename = "updateAvailable")]
    pub update_available: bool,
    /// Catalog metadata for the installed add-on.
    #[serde(default)]
    pub catalog: NkeAddonCatalog,
    /// Health as reported by the add-on.
    #[serde(default)]
    pub health: NkeAddonHealth,
    /// Health of the workloads the add-on installs.
    #[serde(default, rename = "workloadHealth")]
    pub workload_health: NkeAddonWorkloadHealth,
    /// Config the add-on reports back.
    #[serde(default)]
    pub config: NkeAddonReadConfig,
    /// Lifecycle timestamps.
    #[serde(default)]
    pub timestamps: NkeAddonTimestamps,
    /// Reason the last install or update failed, when it did.
    #[serde(default, rename = "failureReason")]
    pub failure_reason: String,
    /// When the platform will retry a failed install.
    #[serde(default, rename = "installRetryOn")]
    pub install_retry_on: String,
    /// Number of consecutive install failures.
    #[serde(default, rename = "installFailureCt")]
    pub install_failure_ct: i64,
}

/// DNS zone attached to an NKE cluster through the `netactuate-dns` add-on.
///
/// `health` and `timestamps` are kept as raw JSON because their shape is not yet stable.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct NkeClusterDnsZone {
    /// DNS zone id.
    #[serde(default, rename = "dnsZoneId")]
    pub dns_zone_id: i64,
    /// Cluster id.
    #[serde(default, rename = "clusterId")]
    pub cluster_id: i64,
    /// Zone name.
    #[serde(default)]
    pub zone: String,
    /// Zone mode.
    #[serde(default)]
    pub mode: String,
    /// Reason the zone failed to attach, when it did.
    #[serde(default, rename = "failureReason")]
    pub failure_reason: String,
    /// Attachment state.
    #[serde(default)]
    pub state: String,
    /// Health payload.
    #[serde(default)]
    pub health: Value,
    /// Timestamps payload.
    #[serde(default)]
    pub timestamps: Value,
}

/// Coerces a boolean field the platform sends inconsistently as a JSON boolean, a number
/// (`1` for true), or a string (`"1"` or `"true"` for true).
pub(crate) fn flexible_bool<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(match value {
        Value::Bool(flag) => flag,
        Value::Number(number) => number.as_f64() == Some(1.0),
        Value::String(text) => text == "1" || text == "true",
        _ => false,
    })
}

/// Coerces an integer field the platform sends inconsistently as a JSON number or a numeric
/// string. An unparseable or absent value decodes to zero.
fn flexible_int<'de, D>(deserializer: D) -> std::result::Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(match value {
        Value::Number(number) => number
            .as_i64()
            .or_else(|| number.as_f64().map(|value| value as i64))
            .unwrap_or_default(),
        Value::String(text) => text.parse().unwrap_or_default(),
        _ => 0,
    })
}

/// A cloud firewall set: a named, orderable collection of firewall rules that can be attached
/// to one or more VMs.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct FirewallSet {
    /// Set id.
    #[serde(default)]
    pub id: i64,
    /// Set name.
    #[serde(default)]
    pub name: String,
    /// Set description.
    #[serde(default)]
    pub description: String,
    /// Whether the set is enabled.
    #[serde(default, deserialize_with = "flexible_bool")]
    pub enabled: bool,
    /// Whether this is an unpublished draft of another set.
    #[serde(default, deserialize_with = "flexible_bool")]
    pub is_draft: bool,
    /// Id of the draft set derived from this one, when one has been created.
    #[serde(default)]
    pub draft_firewall_set_id: Option<i64>,
    /// Creation timestamp.
    #[serde(default)]
    pub created: String,
    /// Last update timestamp.
    #[serde(default)]
    pub last_updated: String,
}

/// Extra match options nested under [`FirewallMatchCriteria`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct FirewallMatchOptions {
    /// ICMP type to match, when the rule's protocol is ICMP.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub icmp_type: String,
}

/// Match criteria for a firewall rule.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct FirewallMatchCriteria {
    /// Protocol to match, such as `tcp`, `udp` or `icmp`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub protocol: String,
    /// Source networks in CIDR notation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_net: Vec<String>,
    /// Destination networks in CIDR notation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub destination_net: Vec<String>,
    /// First port in the matched source range.
    #[serde(default)]
    pub source_port_start: Option<i64>,
    /// Last port in the matched source range.
    #[serde(default)]
    pub source_port_end: Option<i64>,
    /// First port in the matched destination range.
    #[serde(default)]
    pub destination_port_start: Option<i64>,
    /// Last port in the matched destination range.
    #[serde(default)]
    pub destination_port_end: Option<i64>,
    /// IP version the rule applies to, 4 or 6.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_version_number: Option<i64>,
    /// Additional match options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<FirewallMatchOptions>,
}

/// A rule belonging to a [`FirewallSet`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct FirewallRule {
    /// Rule id.
    #[serde(default)]
    pub id: i64,
    /// Id of the set this rule belongs to.
    #[serde(default, deserialize_with = "flexible_int")]
    pub firewall_set_id: i64,
    /// IP version the rule applies to, such as `ipv4` or `ipv6`.
    #[serde(default)]
    pub ip_version: String,
    /// Traffic direction, `inbound` or `outbound`.
    #[serde(default)]
    pub direction: String,
    /// Action taken on a match, such as `accept` or `drop`.
    #[serde(default)]
    pub action: String,
    /// Whether the rule is enabled.
    #[serde(default, deserialize_with = "flexible_bool")]
    pub enabled: bool,
    /// Match criteria.
    #[serde(default)]
    pub match_criteria: Option<FirewallMatchCriteria>,
    /// Administrator comment.
    #[serde(default)]
    pub admin_comment: String,
    /// Evaluation priority, lower values evaluated first.
    #[serde(default)]
    pub rule_priority: i64,
    /// Creation timestamp.
    #[serde(default)]
    pub created: String,
    /// Last update timestamp.
    #[serde(default)]
    pub last_updated: String,
}

/// An external IP set that can be referenced from firewall rules.
///
/// The platform's shape for this resource is not fully specified, so [`Self::raw`] carries
/// the complete decoded response for callers that need a field this type does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct FirewallExternalIpSet {
    /// Set id.
    pub id: i64,
    /// Set name.
    pub name: String,
    /// Set description.
    pub description: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for FirewallExternalIpSet {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            name: String,
            #[serde(default)]
            description: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            name: raw.name,
            description: raw.description,
            raw: value,
        })
    }
}

/// Whether cloud firewall management is available to the account.
///
/// The platform's shape for this resource is not fully specified, so [`Self::raw`] carries
/// the complete decoded response for callers that need a field this type does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct FirewallManageEnabled {
    /// Whether firewall management is available.
    pub enabled: bool,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for FirewallManageEnabled {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            enabled: bool,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            enabled: raw.enabled,
            raw: value,
        })
    }
}

/// A VM attached to a [`FirewallSet`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct FirewallSetVm {
    /// Attachment id.
    #[serde(default, deserialize_with = "flexible_int")]
    pub id: i64,
    /// Server package id.
    #[serde(default, deserialize_with = "flexible_int")]
    pub mbpkgid: i64,
    /// Network interface id the set is attached to.
    #[serde(default, deserialize_with = "flexible_int")]
    pub interface_id: i64,
    /// Id of the attached firewall set.
    #[serde(default, deserialize_with = "flexible_int")]
    pub firewall_set_id: i64,
    /// Priority of this set relative to other sets attached to the same interface.
    #[serde(default, deserialize_with = "flexible_int")]
    pub set_priority: i64,
    /// Creation timestamp.
    #[serde(default)]
    pub created: String,
    /// Last update timestamp.
    #[serde(default)]
    pub last_updated: String,
    /// IATA code of the VM's location.
    #[serde(default)]
    pub iata_code: String,
    /// VM's location name.
    #[serde(default)]
    pub location: String,
    /// VM hostname.
    #[serde(default)]
    pub hostname: String,
}

/// A VLAN provisioned on the account.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Vlan {
    /// VLAN id.
    #[serde(default)]
    pub id: i64,
    /// Server package id the VLAN belongs to.
    #[serde(default)]
    pub mbid: i64,
    /// Whether the VLAN is private.
    #[serde(default)]
    pub private: i64,
    /// Whether SR-IOV is allowed on the VLAN.
    #[serde(default)]
    pub allow_sriov: i64,
    /// Display name.
    #[serde(default)]
    pub display_name: String,
    /// Description.
    #[serde(default)]
    pub description: String,
    /// Last update timestamp.
    #[serde(default)]
    pub last_updated: String,
    /// Creation timestamp.
    #[serde(default)]
    pub created: String,
    /// Locations where the VLAN is provisioned.
    #[serde(default)]
    pub provisioned_locations: Vec<ProvisionedLocation>,
}

/// A location where a VLAN is or can be provisioned.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ProvisionedLocation {
    /// Whether the VLAN is provisioned at this location.
    #[serde(default)]
    pub provisioned: bool,
    /// Location name.
    #[serde(default)]
    pub name: String,
    /// Location id.
    #[serde(default)]
    pub location_id: i64,
    /// Location flag or country code.
    #[serde(default)]
    pub flag: String,
    /// IATA airport code for the location.
    #[serde(default)]
    pub iata_code: String,
}

/// A network interface attached to a server.
///
/// The platform reports these fields under varying keys across endpoints (`nic_id` or `id` for
/// the interface id) and sometimes as numeric strings, so this decodes leniently rather than
/// committing to one shape.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct ServerNic {
    /// Network interface id.
    pub nic_id: i64,
    /// Server package id the interface belongs to.
    pub mbpkg_id: i64,
    /// Customer VLAN id the interface is attached to.
    pub customer_vlan_id: i64,
    /// Attachment order among the server's interfaces.
    pub attach_order: i64,
}

impl<'de> Deserialize<'de> for ServerNic {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            nic_id: Option<Value>,
            #[serde(default)]
            id: Option<Value>,
            #[serde(default)]
            mbpkgid: Option<Value>,
            #[serde(default)]
            customer_vlan_id: Option<Value>,
            #[serde(default)]
            attach_order: Option<Value>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let nic_id = value_as_i64(raw.nic_id.as_ref());
        let nic_id = if nic_id == 0 {
            value_as_i64(raw.id.as_ref())
        } else {
            nic_id
        };
        Ok(Self {
            nic_id,
            mbpkg_id: value_as_i64(raw.mbpkgid.as_ref()),
            customer_vlan_id: value_as_i64(raw.customer_vlan_id.as_ref()),
            attach_order: value_as_i64(raw.attach_order.as_ref()),
        })
    }
}

/// Coerces an optional JSON value carrying a number or a numeric string into an integer,
/// yielding zero when absent, null or unparseable.
fn value_as_i64(value: Option<&Value>) -> i64 {
    match value {
        Some(Value::Number(number)) => number
            .as_i64()
            .or_else(|| number.as_f64().map(|value| value as i64))
            .unwrap_or_default(),
        Some(Value::String(text)) => text.parse().unwrap_or_default(),
        _ => 0,
    }
}

/// A floating IPv4 address allocated to the account.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct CloudFloatingIpv4 {
    /// Floating IPv4 id.
    #[serde(default, rename = "floatingIpv4Id")]
    pub floating_ipv4_id: i64,
    /// Timestamp the address was assigned.
    #[serde(default, rename = "AssignedOn")]
    pub assigned_on: String,
    /// IPv4 address.
    #[serde(default)]
    pub address: String,
    /// VLAN id the address is bound to.
    #[serde(default, rename = "vlanId")]
    pub vlan_id: i64,
    /// Reverse DNS domain.
    #[serde(default, rename = "ptrDomain")]
    pub ptr_domain: Option<String>,
    /// Location of the address.
    #[serde(default)]
    pub location: Option<FloatingIpLocation>,
}

/// Location of a floating IPv4 address.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct FloatingIpLocation {
    /// Location id.
    #[serde(default)]
    pub id: i64,
    /// Location name.
    #[serde(default)]
    pub name: String,
    /// Location flag or country code.
    #[serde(default)]
    pub flag: String,
    /// Latitude.
    #[serde(default)]
    pub latitude: String,
    /// Longitude.
    #[serde(default)]
    pub longitude: String,
}

/// A cloud location mapped to a datacenter, for networking purposes.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct CloudNetworkingLocation {
    /// Location id.
    #[serde(default, rename = "locationId")]
    pub location_id: i64,
    /// Datacenter id.
    #[serde(default, rename = "datacenterId")]
    pub datacenter_id: i64,
}

/// A magic mesh, connecting a set of routers into a full-mesh network overlay.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct MagicMesh {
    /// Mesh id.
    #[serde(default, rename = "meshId")]
    pub mesh_id: i64,
    /// Mesh name.
    #[serde(default)]
    pub name: String,
    /// Mesh description.
    #[serde(default)]
    pub description: Option<String>,
}

/// A router attached to a [`MagicMesh`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct MeshRouter {
    /// Router id.
    #[serde(default, rename = "routerId")]
    pub router_id: i64,
    /// Router name.
    #[serde(default)]
    pub name: String,
    /// Router description.
    #[serde(default)]
    pub description: String,
    /// Router IPv4 address.
    #[serde(default, rename = "ipv4Address")]
    pub ipv4_address: String,
}

/// A virtual machine allowed to use a floating IPv4 address.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct CloudFloatingIpv4Vm {
    /// Server package id.
    #[serde(default)]
    pub mbpkgid: i64,
    /// Server fully qualified domain name.
    #[serde(default)]
    pub fqdn: String,
    /// Server IP address.
    #[serde(default)]
    pub ip: String,
}

/// An OIDC client configured on the account.
///
/// This is assembled from two endpoints: the client list, which carries the account-default,
/// audience, TTL, allow list enforcement and tenant fields, and the single-client endpoint,
/// which carries the current label, description, JWKS URI and usage timestamps plus the keys
/// and log entries. See [`crate::V3Client::get_oidc_client`].
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct OidcClient {
    /// Client id.
    pub client_id: i64,
    /// Creation timestamp.
    pub created_on: String,
    /// Timestamp the client last issued a token, when it has ever been used.
    pub last_used_on: Option<String>,
    /// Display label.
    pub label: String,
    /// Description.
    pub description: String,
    /// JSON Web Key Set URI used to validate client-presented keys.
    pub jwks_uri: Option<String>,
    /// Whether this is the account's default OIDC client.
    pub account_default: bool,
    /// Default audience issued in tokens from this client.
    pub default_audience: String,
    /// Token time-to-live in seconds.
    pub ttl: i64,
    /// Whether the VM and bare metal allow list is enforced.
    pub enforce_allow_list: bool,
    /// Id of the tenant the client belongs to.
    pub tenant: String,
    /// Public keys registered on the client.
    pub keys: Vec<OidcClientKey>,
    /// Authentication log entries recorded for the client.
    pub auth_logs: Vec<OidcClientAuthLog>,
    /// Change log entries recorded for the client.
    pub change_logs: Vec<OidcClientChangeLog>,
}

/// A public key registered on an OIDC client.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct OidcClientKey {
    /// Key id.
    #[serde(default, rename = "keyId")]
    pub key_id: i64,
    /// Display label.
    #[serde(default)]
    pub label: String,
    /// Description.
    #[serde(default)]
    pub description: String,
    /// Timestamp the key was provided.
    #[serde(default, rename = "providedOn")]
    pub provided_on: String,
    /// Timestamp the key was revoked, empty while the key is still active.
    #[serde(default, rename = "revokedOn")]
    pub revoked_on: String,
    /// Key type.
    #[serde(default, rename = "type")]
    pub key_type: String,
    /// Key value.
    #[serde(default)]
    pub value: String,
    /// Public key content.
    #[serde(default, rename = "publicKey")]
    pub public_key: String,
}

/// An authentication log entry recorded for an OIDC client.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct OidcClientAuthLog {
    /// Log entry id.
    #[serde(default, rename = "id")]
    pub log_id: i64,
    /// Timestamp the token was issued.
    #[serde(default, rename = "issuedOn")]
    pub issued_on: String,
    /// Timestamp the token expires.
    #[serde(default, rename = "expiresOn")]
    pub expires_on: String,
    /// Token JWT id.
    #[serde(default)]
    pub jti: String,
}

/// A change log entry recorded for an OIDC client.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct OidcClientChangeLog {
    /// Id of the key the change applies to.
    #[serde(default, rename = "keyId")]
    pub key_id: i64,
    /// Timestamp the change was recorded.
    #[serde(default, rename = "recordedOn")]
    pub recorded_on: String,
    /// Change type.
    #[serde(default, rename = "type")]
    pub change_type: String,
}

/// A virtual machine allowed to reach an OIDC client through its allow list.
///
/// The platform returns each entry as either an object with an `mbpkgid` field or a bare
/// integer id, so [`Self::raw`] carries the complete decoded response for callers that need a
/// field this type does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct OidcClientVm {
    /// Server package id.
    pub mbpkgid: i64,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for OidcClientVm {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(Self {
            mbpkgid: mbpkgid_from_value(&value),
            raw: value,
        })
    }
}

/// A bare metal server allowed to reach an OIDC client through its allow list.
///
/// The platform returns each entry as either an object with an `mbpkgid` field or a bare
/// integer id, so [`Self::raw`] carries the complete decoded response for callers that need a
/// field this type does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct OidcClientBareMetalServer {
    /// Server package id.
    pub mbpkgid: i64,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for OidcClientBareMetalServer {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(Self {
            mbpkgid: mbpkgid_from_value(&value),
            raw: value,
        })
    }
}

/// Extracts `mbpkgid` from a value that is either a bare integer or an object carrying that
/// field.
fn mbpkgid_from_value(value: &Value) -> i64 {
    match value {
        Value::Number(number) => number.as_i64().unwrap_or_default(),
        Value::Object(object) => object.get("mbpkgid").and_then(Value::as_i64).unwrap_or(0),
        _ => 0,
    }
}

/// A queued or completed job on the account, polled by command name and job id.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct JobStatus {
    /// Job id.
    #[serde(default)]
    pub id: i64,
    /// Timestamp the job was inserted.
    #[serde(default)]
    pub ts_insert: String,
    /// Command the job runs.
    #[serde(default)]
    pub command: String,
    /// Job status code.
    #[serde(default)]
    pub status: i64,
}

/// A top-level account service record.
///
/// The platform's service schema varies by service type and is not fully specified, so
/// [`Self::raw`] carries the complete decoded response for callers that need a field this type
/// does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct Service {
    /// Service id.
    pub id: i64,
    /// Service description.
    pub description: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for Service {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            description: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            description: raw.description,
            raw: value,
        })
    }
}

/// A colocation service record.
///
/// The platform's shape for this resource is not fully specified, so [`Self::raw`] carries
/// the complete decoded response for callers that need a field this type does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct ColocationService {
    /// Service record id.
    pub id: i64,
    /// Id of the parent [`Service`].
    pub service_id: i64,
    /// Datacenter id.
    pub datacenter_id: i64,
    /// Rack identifier.
    pub rack_identifier: String,
    /// Power details.
    pub power_details: String,
    /// Service description.
    pub description: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for ColocationService {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            service_id: i64,
            #[serde(default)]
            datacenter_id: i64,
            #[serde(default)]
            rack_identifier: String,
            #[serde(default)]
            power_details: String,
            #[serde(default)]
            description: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            service_id: raw.service_id,
            datacenter_id: raw.datacenter_id,
            rack_identifier: raw.rack_identifier,
            power_details: raw.power_details,
            description: raw.description,
            raw: value,
        })
    }
}

/// An IP transit service record.
///
/// The platform's shape for this resource is not fully specified, so [`Self::raw`] carries
/// the complete decoded response for callers that need a field this type does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct IpTransitService {
    /// Service record id.
    pub id: i64,
    /// Id of the parent [`Service`].
    pub service_id: i64,
    /// Datacenter id.
    pub datacenter_id: i64,
    /// BGP group id.
    pub bgp_group_id: i64,
    /// Service description.
    pub description: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for IpTransitService {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            service_id: i64,
            #[serde(default)]
            datacenter_id: i64,
            #[serde(default)]
            bgp_group_id: i64,
            #[serde(default)]
            description: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            service_id: raw.service_id,
            datacenter_id: raw.datacenter_id,
            bgp_group_id: raw.bgp_group_id,
            description: raw.description,
            raw: value,
        })
    }
}

/// An IP address assigned to an IP transit service.
///
/// The platform's shape for this resource is not fully specified, so [`Self::raw`] carries
/// the complete decoded response for callers that need a field this type does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct IpTransitIpAddress {
    /// Address record id.
    pub id: i64,
    /// Id of the owning IP transit service.
    pub service_iptransit_id: i64,
    /// IP address.
    pub ip: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for IpTransitIpAddress {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            service_iptransit_id: i64,
            #[serde(default)]
            ip: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            service_iptransit_id: raw.service_iptransit_id,
            ip: raw.ip,
            raw: value,
        })
    }
}

/// A port assigned to an IP transit service.
///
/// The platform's shape for this resource is not fully specified, so [`Self::raw`] carries
/// the complete decoded response for callers that need a field this type does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct IpTransitPort {
    /// Port record id.
    pub id: i64,
    /// Id of the owning IP transit service.
    pub service_iptransit_id: i64,
    /// Port name.
    pub name: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for IpTransitPort {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            service_iptransit_id: i64,
            #[serde(default)]
            name: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            service_iptransit_id: raw.service_iptransit_id,
            name: raw.name,
            raw: value,
        })
    }
}

/// A transport service record.
///
/// The platform's shape for this resource is not fully specified, so [`Self::raw`] carries
/// the complete decoded response for callers that need a field this type does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct TransportService {
    /// Service record id.
    pub id: i64,
    /// Id of the parent [`Service`].
    pub service_id: i64,
    /// Datacenter id.
    pub datacenter_id: i64,
    /// Service description.
    pub description: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for TransportService {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            service_id: i64,
            #[serde(default)]
            datacenter_id: i64,
            #[serde(default)]
            description: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            service_id: raw.service_id,
            datacenter_id: raw.datacenter_id,
            description: raw.description,
            raw: value,
        })
    }
}

/// A port assigned to a transport service.
///
/// The platform's shape for this resource is not fully specified, so [`Self::raw`] carries
/// the complete decoded response for callers that need a field this type does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct TransportPort {
    /// Port record id.
    pub id: i64,
    /// Id of the owning transport service.
    pub service_transport_id: i64,
    /// Port name.
    pub name: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for TransportPort {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            service_transport_id: i64,
            #[serde(default)]
            name: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            service_transport_id: raw.service_transport_id,
            name: raw.name,
            raw: value,
        })
    }
}

/// An account BGP group.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct BgpGroup {
    /// Group id.
    #[serde(default)]
    pub id: i64,
    /// Group name.
    #[serde(default)]
    pub name: String,
    /// Group description.
    #[serde(default)]
    pub description: String,
    /// Group type, such as `bgp` or `anycast`.
    #[serde(default)]
    pub group_type: String,
}

/// An account BGP prefix.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct BgpPrefix {
    /// Prefix id.
    #[serde(default)]
    pub id: i64,
    /// Prefix name.
    #[serde(default)]
    pub name: String,
    /// Prefix in CIDR notation.
    #[serde(default)]
    pub prefix: String,
    /// Id of the owning [`BgpGroup`].
    #[serde(default)]
    pub group_id: i64,
    /// Id of the ASN the prefix is announced from.
    #[serde(default)]
    pub asn_id: i64,
    /// Anycast profile id.
    #[serde(default)]
    pub anycast_profile: i64,
    /// Id of the agreement the purchase was made under.
    #[serde(default)]
    pub agreement_id: i64,
}

/// An account ASN.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct BgpAsn {
    /// ASN record id.
    #[serde(default)]
    pub id: i64,
    /// The autonomous system number.
    #[serde(default)]
    pub asn: i64,
    /// ASN name.
    #[serde(default)]
    pub name: String,
    /// Group type, such as `bgp` or `anycast`.
    #[serde(default)]
    pub group_type: String,
}

/// A firewall set bound to an interface of a BGP group.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct BgpGroupFirewallSetBinding {
    /// Binding id.
    #[serde(default)]
    pub id: i64,
    /// Id of the bound BGP group.
    #[serde(default, rename = "bgp2_group_id")]
    pub bgp_group_id: i64,
    /// Id of the bound firewall set.
    #[serde(default)]
    pub firewall_set_id: i64,
    /// Interface number the firewall set is bound to.
    #[serde(default)]
    pub interface_number: i64,
    /// Evaluation priority among the sets bound to the same interface.
    #[serde(default)]
    pub set_priority: i64,
}

/// Account BGP summary counters, keyed by the platform's own field names.
///
/// The shape of this payload is not fully specified, so it is carried as a raw map rather
/// than a typed struct.
pub type BgpSummary = Map<String, Value>;

/// Account BGP dashboard data, keyed by the platform's own field names.
///
/// The shape of this payload is not fully specified, so it is carried as a raw map rather
/// than a typed struct.
pub type BgpDashboard = Map<String, Value>;

/// A legal agreement available to the account.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct AccountAgreement {
    /// Agreement id.
    #[serde(default)]
    pub id: i64,
    /// Agreement name.
    #[serde(default)]
    pub name: String,
    /// Agreement title.
    #[serde(default)]
    pub title: String,
    /// Agreement description.
    #[serde(default)]
    pub description: String,
    /// Agreement version.
    #[serde(default)]
    pub version: String,
}

/// A datacenter within a platform location.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Datacenter {
    /// Datacenter id.
    #[serde(default)]
    pub id: i64,
    /// Datacenter name.
    #[serde(default)]
    pub name: String,
    /// IATA airport code for the datacenter's location.
    #[serde(default)]
    pub iata: String,
}

/// Platform status for one monitored location within a service.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct PlatformStatusLocation {
    /// Location code, taken from the wire response's map key.
    #[serde(default)]
    pub location: String,
    /// Container id.
    #[serde(default)]
    pub container_id: String,
    /// Status text.
    #[serde(default)]
    pub status: String,
    /// Timestamp the status was last updated.
    #[serde(default)]
    pub last_updated: String,
}

/// Platform status for one monitored service, aggregated across locations.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct PlatformStatusService {
    /// Service name, taken from the wire response's map key.
    #[serde(default)]
    pub service: String,
    /// Component id.
    #[serde(default)]
    pub component_id: String,
    /// Status for every monitored location, sorted by location code.
    #[serde(default)]
    pub locations: Vec<PlatformStatusLocation>,
}

#[derive(Debug, Deserialize)]
struct PlatformStatusLocationWire {
    #[serde(default)]
    container_id: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    last_updated: String,
}

#[derive(Debug, Deserialize)]
struct PlatformStatusServiceWire {
    #[serde(default)]
    component_id: String,
    #[serde(default)]
    locations: std::collections::BTreeMap<String, PlatformStatusLocationWire>,
}

/// Decodes the platform status response.
///
/// The wire shape is a map of service name to an object whose locations are themselves keyed
/// by location code. This flattens both map layers into their key, sorted, giving the stable
/// ordering the portal renders.
pub(crate) fn decode_platform_status(raw: &[u8]) -> Result<Vec<PlatformStatusService>> {
    let wire: std::collections::BTreeMap<String, PlatformStatusServiceWire> =
        decode_required(raw, "platform status")?;
    Ok(wire
        .into_iter()
        .map(|(service, service_wire)| PlatformStatusService {
            service,
            component_id: service_wire.component_id,
            locations: service_wire
                .locations
                .into_iter()
                .map(|(location, location_wire)| PlatformStatusLocation {
                    location,
                    container_id: location_wire.container_id,
                    status: location_wire.status,
                    last_updated: location_wire.last_updated,
                })
                .collect(),
        })
        .collect())
}

/// Coerces a field the platform can send as either a string or a JSON number into a string. A
/// null or missing value decodes to an empty string.
pub(crate) fn flexible_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(text) => Ok(text),
        Value::Number(number) => Ok(number.to_string()),
        Value::Null => Ok(String::new()),
        other => Err(serde::de::Error::custom(format!(
            "expected a string or number, got {other}"
        ))),
    }
}

/// Coerces a router config's IPv4 address, which the platform can send as a dotted-quad
/// string or as a packed 32-bit integer, into its dotted-quad string form.
pub(crate) fn flexible_ipv4<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(text) => Ok(text),
        Value::Number(number) => {
            let packed = number.as_u64().ok_or_else(|| {
                serde::de::Error::custom(format!("IPv4 address {number} out of range"))
            })? as u32;
            Ok(std::net::Ipv4Addr::from(packed).to_string())
        }
        other => Err(serde::de::Error::custom(format!(
            "IPv4 address must be a string or number, got {other}"
        ))),
    }
}

/// A platform change log entry.
///
/// The platform can send `id`, `title`, `short_description` and `status` as either a string or
/// a JSON number, and the schema is not otherwise fully specified, so [`Self::raw`] carries the
/// complete decoded response for callers that need a field this type does not expose.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct PlatformChangeLogEntry {
    /// Change log entry id.
    pub change_log_id: String,
    /// Entry title.
    pub title: String,
    /// Short description.
    pub short_description: String,
    /// Entry status.
    pub status: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for PlatformChangeLogEntry {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default, rename = "id", deserialize_with = "flexible_string")]
            change_log_id: String,
            #[serde(default, deserialize_with = "flexible_string")]
            title: String,
            #[serde(default, deserialize_with = "flexible_string")]
            short_description: String,
            #[serde(default, deserialize_with = "flexible_string")]
            status: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            change_log_id: raw.change_log_id,
            title: raw.title,
            short_description: raw.short_description,
            status: raw.status,
            raw: value,
        })
    }
}

/// The options and targets available for a looking glass query, in the platform's native
/// shape.
///
/// The response schema is not specified, so this wraps the complete decoded payload.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(transparent)]
pub struct PlatformLookingGlassInit {
    /// Complete decoded response.
    pub raw: Value,
}

/// The output of a looking glass action, in the platform's native shape.
///
/// The response schema depends on the action requested, so this wraps the complete decoded
/// payload.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(transparent)]
pub struct PlatformLookingGlassResult {
    /// Complete decoded response.
    pub raw: Value,
}

/// Detail for one platform maintenance record, in the platform's native shape.
///
/// The response schema is not specified, so this wraps the complete decoded payload.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(transparent)]
pub struct PlatformMaintenanceInfo {
    /// Complete decoded response.
    pub raw: Value,
}

/// Platform incidents or maintenance events of one kind, grouped by lifecycle.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct PlatformEvents {
    /// Events currently in progress.
    #[serde(default)]
    pub active: Vec<PlatformEvent>,
    /// Events scheduled to start in the future.
    #[serde(default)]
    pub upcoming: Vec<PlatformEvent>,
    /// Events that have concluded.
    #[serde(default)]
    pub historic: Vec<PlatformEvent>,
}

/// A single platform incident or maintenance event.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct PlatformEvent {
    /// Event id.
    #[serde(default)]
    pub event_id: String,
    /// Event type.
    #[serde(default, rename = "type")]
    pub event_type: String,
    /// Event name.
    #[serde(default)]
    pub name: String,
    /// Event status.
    #[serde(default)]
    pub status: String,
    /// Start time.
    #[serde(default)]
    pub start_time: String,
    /// End time.
    #[serde(default)]
    pub end_time: String,
    /// Platform components affected by the event.
    #[serde(default)]
    pub components: Vec<String>,
    /// Containers affected by the event.
    #[serde(default)]
    pub containers: Vec<String>,
}

/// A support ticket.
///
/// The schema carries fields beyond what this type exposes, so [`Self::raw`] keeps the
/// complete decoded response.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct Ticket {
    /// Ticket id.
    pub id: String,
    /// Ticket subject.
    pub subject: String,
    /// Ticket status.
    pub status: String,
    /// Owning department.
    pub department: String,
    /// Urgency level.
    pub urgency: String,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for Ticket {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: String,
            #[serde(default)]
            subject: String,
            #[serde(default)]
            status: String,
            #[serde(default)]
            department: String,
            #[serde(default)]
            urgency: String,
            #[serde(default)]
            created_at: String,
            #[serde(default)]
            updated_at: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            subject: raw.subject,
            status: raw.status,
            department: raw.department,
            urgency: raw.urgency,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
            raw: value,
        })
    }
}

/// A reply posted to a support ticket.
///
/// The schema carries fields beyond what this type exposes, so [`Self::raw`] keeps the
/// complete decoded response.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct TicketReply {
    /// Reply id.
    pub id: String,
    /// Reply message body.
    pub message: String,
    /// Creation timestamp.
    pub created_at: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for TicketReply {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: String,
            #[serde(default)]
            message: String,
            #[serde(default)]
            created_at: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            message: raw.message,
            created_at: raw.created_at,
            raw: value,
        })
    }
}

/// A department a support ticket can be routed to.
///
/// The schema carries fields beyond what this type exposes, so [`Self::raw`] keeps the
/// complete decoded response.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct TicketDepartment {
    /// Department id.
    pub id: i64,
    /// Department name.
    pub name: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for TicketDepartment {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            name: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: raw.id,
            name: raw.name,
            raw: value,
        })
    }
}

/// Metadata for a support ticket or ticket reply attachment.
///
/// The schema carries fields beyond what this type exposes, so [`Self::raw`] keeps the
/// complete decoded response.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct TicketAttachment {
    /// Attachment file name.
    pub name: String,
    /// Attachment MIME type.
    pub content_type: String,
    /// Attachment size in bytes.
    pub size: i64,
    /// Base64-encoded attachment content, omitted when fetched with `without_data`.
    pub data: String,
    /// Complete decoded response.
    #[serde(skip)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for TicketAttachment {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            name: String,
            #[serde(default)]
            content_type: String,
            #[serde(default)]
            size: i64,
            #[serde(default)]
            data: String,
        }
        let raw = Raw::deserialize(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            name: raw.name,
            content_type: raw.content_type,
            size: raw.size,
            data: raw.data,
            raw: value,
        })
    }
}

/// A named list of secret key/value pairs.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct SecretList {
    /// List id.
    #[serde(default)]
    pub id: i64,
    /// List name.
    #[serde(default)]
    pub name: String,
}

/// One key/value pair stored in a [`SecretList`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct SecretListValue {
    /// Value id.
    #[serde(default)]
    pub id: i64,
    /// Id of the secret list this value belongs to.
    #[serde(default, deserialize_with = "flexible_int")]
    pub secret_list_id: i64,
    /// Secret key.
    #[serde(default)]
    pub secret_key: String,
    /// Secret value.
    #[serde(default)]
    pub secret_value: String,
}

/// A cloud deployment location, as returned by [`crate::Client::get_cloud_location`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct CloudLocation {
    /// Location id.
    #[serde(default)]
    pub id: i64,
    /// Location code.
    #[serde(default)]
    pub name: String,
    /// Location display name.
    #[serde(default)]
    pub location: String,
    /// City the location is in.
    #[serde(default)]
    pub city: String,
    /// Country the location is in.
    #[serde(default)]
    pub country: String,
    /// IATA airport code for the location.
    #[serde(default)]
    pub iata_code: String,
    /// URL of the country flag icon.
    #[serde(default)]
    pub flag: String,
    /// Latitude, when the platform reports it.
    #[serde(default)]
    pub latitude: Option<String>,
    /// Longitude, when the platform reports it.
    #[serde(default)]
    pub longitude: Option<String>,
}

/// A boot kernel option, as returned by [`crate::Client::get_kernels`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct Kernel {
    /// Kernel id.
    #[serde(default)]
    pub id: i64,
    /// Kernel name.
    #[serde(default)]
    pub name: String,
    /// Kernel description, when set.
    #[serde(default)]
    pub description: Option<String>,
}

/// A capacity pool cloud servers deploy into, as returned by [`crate::Client::get_cloud_pool`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct CloudPool {
    /// Pool id.
    #[serde(default)]
    pub id: i64,
    /// Pool name.
    #[serde(default)]
    pub name: String,
    /// Pool description.
    #[serde(default)]
    pub description: String,
    /// Required CPU model such as `EPYC-Milan`, when the pool constrains it.
    #[serde(default)]
    pub required_vcpu: Option<String>,
    /// Capabilities every server on the pool must have.
    #[serde(default)]
    pub hard_capabilities: Vec<String>,
    /// Capabilities the pool prefers but does not require.
    #[serde(default)]
    pub soft_capabilities: Vec<String>,
    /// Whether the pool is private to the account.
    #[serde(default)]
    pub private: i64,
    /// Id of the pool used as an overflow backup, when set.
    #[serde(default)]
    pub backup_cloud_pool_id: Option<i64>,
    /// Default RAM price for the pool.
    #[serde(default)]
    pub default_ram_price: String,
    /// Default CPU price for the pool.
    #[serde(default)]
    pub default_cpu_price: String,
    /// Default disk price for the pool.
    #[serde(default)]
    pub default_disk_price: String,
    /// Timestamp of the last update to the pool.
    #[serde(default)]
    pub last_updated: String,
    /// Id of the contract the pool is scoped to, when it is private to one.
    #[serde(default)]
    pub contract_id: Option<i64>,
}

/// The status of an asynchronous server build, as returned by
/// [`crate::Client::get_server_build_status`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ServerBuildStatus {
    /// Build id.
    #[serde(default)]
    pub id: i64,
    /// Build status text.
    #[serde(default)]
    pub status: String,
    /// Build completion percentage.
    #[serde(default)]
    pub percent: i64,
    /// Build log or response text.
    #[serde(default)]
    pub response: String,
}

/// An IPv4 or IPv6 address attached to a server, as returned by
/// [`crate::Client::get_server_ipv4`] and [`crate::Client::get_server_ipv6`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ServerIpAddress {
    /// Address id.
    #[serde(default)]
    pub id: i64,
    /// The IP address.
    #[serde(default)]
    pub ip: String,
    /// Reverse DNS entry, when set.
    #[serde(default)]
    pub reverse: Option<String>,
    /// Netmask, when applicable.
    #[serde(default)]
    pub netmask: Option<String>,
    /// Gateway address, when applicable.
    #[serde(default)]
    pub gateway: Option<String>,
    /// Address type such as `public` or `private`.
    #[serde(default, rename = "type")]
    pub address_type: Option<String>,
    /// Whether this is the server's primary address.
    #[serde(default)]
    pub primary: Option<i64>,
}

/// A server's status, as returned by [`crate::Client::get_server_status`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ServerStatus {
    /// Lifecycle status such as `active`.
    #[serde(default)]
    pub status: String,
    /// Power state such as `running`.
    #[serde(default)]
    pub state: String,
}

/// Usage contract data for an account or a virtual server, as returned by
/// [`crate::Client::get_virtual_server_contract`] and
/// [`crate::Client::create_usage_contract`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ContractUsage {
    /// Contract id.
    #[serde(default)]
    pub id: Option<i64>,
    /// Billing package id the contract is attached to.
    #[serde(default)]
    pub contract_mbpkgid: Option<i64>,
    /// Id of the parent contract, when this contract rolls up into another.
    #[serde(default)]
    pub parent_contract_id: Option<i64>,
    /// Contract brand.
    #[serde(default)]
    pub brand: Option<String>,
    /// Account mb_id the contract belongs to.
    #[serde(default)]
    pub mb_id: Option<i64>,
    /// Contract type.
    #[serde(default)]
    pub contract_type: Option<String>,
    /// Whether the contract is free.
    #[serde(default)]
    pub is_free: Option<i64>,
    /// Whether bandwidth is included in the contract.
    #[serde(default)]
    pub include_bandwidth: Option<i64>,
    /// Customer purchase order reference.
    #[serde(default)]
    pub customer_po: Option<String>,
    /// Customer description for the contract.
    #[serde(default)]
    pub customer_description: Option<String>,
    /// Monthly purchase order limit.
    #[serde(default)]
    pub po_monthly_limit: Option<i64>,
    /// Monthly discount applied to the contract.
    #[serde(default)]
    pub monthly_discount: Option<i64>,
    /// Hourly discount applied to the contract.
    #[serde(default)]
    pub hourly_discount: Option<i64>,
    /// Maximum vCPUs allowed under the contract.
    #[serde(default)]
    pub max_cpus: Option<i64>,
    /// Maximum RAM allowed under the contract.
    #[serde(default)]
    pub max_ram: Option<i64>,
    /// Maximum disk allowed under the contract.
    #[serde(default)]
    pub max_disk: Option<i64>,
    /// Whether usage past the contract limits is allowed.
    #[serde(default)]
    pub allow_overage: Option<i64>,
}

/// A deployable server size, as returned by [`crate::Client::get_deploy_sizes`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct Size {
    /// Plan id.
    #[serde(default)]
    pub plan_id: i64,
    /// Plan name.
    #[serde(default)]
    pub plan: String,
    /// RAM allotment, formatted by the platform.
    #[serde(default)]
    pub ram: String,
    /// Disk allotment, formatted by the platform.
    #[serde(default)]
    pub disk: String,
    /// Transfer allotment, formatted by the platform.
    #[serde(default)]
    pub transfer: String,
    /// Formatted plan price.
    #[serde(default)]
    pub price: String,
    /// vCPU count.
    #[serde(default)]
    pub cpu: i64,
    /// Formatted port speed.
    #[serde(default)]
    pub port: String,
    /// Units of this size currently available at the queried location.
    #[serde(default)]
    pub available: f64,
}

/// The response to deleting a server with options, as returned by
/// [`crate::Client::delete_server_with_options`].
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DeleteServerResponse {
    /// Id of the deleted server's billing package.
    #[serde(default)]
    pub id: i64,
}

/// A cloud router, as returned by [`crate::V3Client::get_router`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Router {
    /// Router name.
    #[serde(default)]
    pub name: String,
    /// Router description.
    #[serde(default)]
    pub description: Option<String>,
    /// Timestamp the router finished provisioning, absent while a build is still in
    /// progress.
    #[serde(default, rename = "readyOn")]
    pub ready_on: Option<String>,
    /// Whether the router has a default VRF.
    #[serde(default, rename = "hasDefaultVrf")]
    pub has_default_vrf: bool,
    /// Whether the router can join a magic mesh.
    #[serde(default, rename = "canJoinMagicMesh")]
    pub can_join_magic_mesh: bool,
    /// Id of the magic mesh the router belongs to, when it belongs to one.
    #[serde(default, rename = "meshId")]
    pub mesh_id: Option<i64>,
    /// Provisioning steps reported for the router's build.
    #[serde(default)]
    pub build: Vec<RouterBuildEvent>,
}

/// One timestamped step in a router's provisioning build.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterBuildEvent {
    /// Step description.
    #[serde(default)]
    pub text: String,
    /// Completion timestamp, absent while the step is still pending.
    #[serde(default)]
    pub date: Option<String>,
}

/// The full configuration of a cloud router.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct RouterConfig {
    /// Id of the router's default VRF.
    #[serde(default, rename = "defaultVrfId")]
    pub default_vrf_id: i64,
    /// Router-wide service configuration.
    #[serde(default)]
    pub service: RouterService,
    /// Prefix lists configured on the router, kept opaque pending a stable schema.
    #[serde(default, rename = "prefixLists")]
    pub prefix_lists: Value,
    /// VRFs configured on the router, keyed by VRF id.
    #[serde(default)]
    pub vrf: BTreeMap<String, RouterVrfConfig>,
    /// IPsec configuration, kept opaque pending a stable schema. Use
    /// [`crate::V3Client::get_router_ipsec_config`] for the typed config.
    #[serde(default, rename = "ipSec")]
    pub ip_sec: Value,
    /// Router status and identity metadata.
    #[serde(default)]
    pub metadata: RouterConfigMetadata,
}

/// Router-wide service configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterService {
    /// NTP service configuration.
    #[serde(default)]
    pub ntp: RouterNtpConfig,
}

/// Status and identity metadata for a router's configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterConfigMetadata {
    /// Router status.
    #[serde(default)]
    pub status: String,
    /// Router name.
    #[serde(default)]
    pub name: String,
    /// Timestamp the configuration was last updated.
    #[serde(default, rename = "updatedOn")]
    pub updated_on: Option<String>,
    /// Configuration version number.
    #[serde(default)]
    pub version: i64,
    /// Router's primary IPv4 address. The platform sends this as either a dotted-quad
    /// string or a packed 32-bit integer.
    #[serde(default, rename = "ipv4Address", deserialize_with = "flexible_ipv4")]
    pub ipv4_address: String,
    /// Router location.
    #[serde(default)]
    pub location: Option<RouterLocation>,
    /// Whether the router has a default VRF.
    #[serde(default, rename = "hasDefaultVrf")]
    pub has_default_vrf: bool,
    /// Id of the magic mesh the router belongs to, when it belongs to one.
    #[serde(default, rename = "meshId")]
    pub mesh_id: Option<i64>,
    /// Whether the router can join a magic mesh.
    #[serde(default, rename = "canJoinMagicMesh")]
    pub can_join_magic_mesh: bool,
}

/// A router's datacenter location.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterLocation {
    /// Location id.
    #[serde(default)]
    pub id: i64,
    /// Location name.
    #[serde(default)]
    pub name: String,
    /// Location flag, when the platform provides one.
    #[serde(default)]
    pub flag: Option<String>,
}

/// One VRF's configuration on a cloud router.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct RouterVrfConfig {
    /// VRF id.
    #[serde(default, rename = "vrfId")]
    pub vrf_id: i64,
    /// VRF name.
    #[serde(default)]
    pub name: String,
    /// VRF description.
    #[serde(default)]
    pub description: String,
    /// DNAT rules, kept opaque pending a stable schema. Use
    /// [`crate::V3Client::list_router_vrf_dnat_rules`] for the typed listing.
    #[serde(default, rename = "dnatRules")]
    pub dnat_rules: Value,
    /// SNAT rules, kept opaque pending a stable schema. Use
    /// [`crate::V3Client::list_router_vrf_snat_rules`] for the typed listing.
    #[serde(default, rename = "snatRules")]
    pub snat_rules: Value,
    /// VRF service configuration.
    #[serde(default)]
    pub services: RouterVrfServices,
    /// Tunnels configured on the VRF, kept opaque pending a stable schema. Use
    /// [`crate::V3Client::list_router_vrf_tunnels`] for the typed listing.
    #[serde(default)]
    pub tunnels: Value,
    /// BGP configuration for the VRF.
    #[serde(default)]
    pub bgp: RouterVrfBgpConfig,
    /// Routing configuration for the VRF.
    #[serde(default)]
    pub routes: RouterVrfRoutesConfig,
    /// Interfaces attached to the VRF, kept opaque pending a stable schema. Use
    /// [`crate::V3Client::list_router_vrf_interfaces`] for the typed listing.
    #[serde(default)]
    pub interfaces: Value,
    /// IPsec configuration for the VRF.
    #[serde(default, rename = "ipSec")]
    pub ip_sec: RouterVrfIpSecConfig,
}

/// A VRF's service configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct RouterVrfServices {
    /// DHCP configuration, kept opaque pending a stable schema. Use
    /// [`crate::V3Client::get_router_vrf_dhcp`] for the typed config.
    #[serde(default)]
    pub dhcp: Value,
}

/// A VRF's static routing configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct RouterVrfRoutesConfig {
    /// Static routes, kept opaque pending a stable schema. Use
    /// [`crate::V3Client::list_router_static_routes`] for the typed listing.
    #[serde(default, rename = "static")]
    pub static_routes: Value,
}

/// A VRF's IPsec configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct RouterVrfIpSecConfig {
    /// IPsec peers, kept opaque pending a stable schema. Use
    /// [`crate::V3Client::list_router_vrf_ipsec_peers`] for the typed listing.
    #[serde(default)]
    pub peers: Value,
}

/// A VRF's BGP configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct RouterVrfBgpConfig {
    /// Local ASN, when configured.
    #[serde(default, rename = "localAsn")]
    pub local_asn: Option<String>,
    /// Router id advertised in BGP updates.
    #[serde(default, rename = "routerId")]
    pub router_id: String,
    /// Networks advertised over BGP.
    #[serde(default)]
    pub networks: Vec<RouterVrfBgpNetwork>,
    /// Configured BGP neighbors.
    #[serde(default)]
    pub neighbors: Vec<RouterVrfBgpNeighbor>,
}

/// One network advertised over BGP.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfBgpNetwork {
    /// Subnet in CIDR notation.
    #[serde(default)]
    pub subnet: String,
}

/// The result of updating a VRF's BGP configuration.
///
/// The platform returns `routerId` here as a number, unlike the string form carried on
/// [`RouterVrfBgpConfig`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct RouterVrfBgpUpdateResult {
    /// Local ASN, when configured.
    #[serde(default, rename = "localAsn")]
    pub local_asn: Option<String>,
    /// Router id, as a number.
    #[serde(default, rename = "routerId")]
    pub router_id: i64,
    /// Networks advertised over BGP.
    #[serde(default)]
    pub networks: Vec<RouterVrfBgpNetwork>,
    /// Configured BGP neighbors.
    #[serde(default)]
    pub neighbors: Vec<RouterVrfBgpNeighbor>,
}

/// A BGP neighbor's source address override.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct BgpNeighborSource {
    /// Source address to peer from.
    #[serde(default)]
    pub address: String,
}

/// Which IP versions are enabled for a BGP neighbor.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct BgpNeighborEnabledIpVersion {
    /// Whether IPv4 is enabled.
    #[serde(default)]
    pub ipv4: bool,
    /// Whether IPv6 is enabled.
    #[serde(default)]
    pub ipv6: bool,
}

/// A BGP neighbor's remote ASN.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct BgpNeighborAsn {
    /// Remote ASN.
    #[serde(default)]
    pub remote: i64,
}

/// One rule in a BGP neighbor's route map.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct BgpNeighborRouteMapRule {
    /// Prefix list the rule matches against.
    #[serde(default, rename = "prefixListId")]
    pub prefix_list_id: i64,
    /// Action to take on a match: `permit`, `deny`, or `next`.
    #[serde(default)]
    pub action: String,
    /// Local preference to set on a match.
    #[serde(default, rename = "setLocalPreference")]
    pub set_local_preference: Option<i64>,
    /// Number of times to prepend the last ASN on a match.
    #[serde(default, rename = "prependLastAsn")]
    pub prepend_last_asn: Option<i64>,
}

/// A BGP neighbor's import or export route map.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct BgpNeighborRouteMap {
    /// Whether routes not matched by any rule are dropped by default.
    #[serde(default, rename = "doDefaultDrop")]
    pub do_default_drop: bool,
    /// Ordered route map rules.
    #[serde(default)]
    pub rules: Vec<BgpNeighborRouteMapRule>,
}

/// A BGP neighbor configured on a router VRF.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfBgpNeighbor {
    /// Neighbor id.
    #[serde(default, rename = "neighborId")]
    pub neighbor_id: i64,
    /// Neighbor address.
    #[serde(default)]
    pub address: String,
    /// Whether the neighbor session is administratively shut down.
    #[serde(default, rename = "isShutdown")]
    pub is_shutdown: bool,
    /// Whether to override the AS path with the local ASN.
    #[serde(default, rename = "doAsOverride")]
    pub do_as_override: bool,
    /// Whether to set the next hop to this router on advertised routes.
    #[serde(default, rename = "doNextHelpSelf")]
    pub do_next_help_self: bool,
    /// Source address override, when set.
    #[serde(default)]
    pub source: Option<BgpNeighborSource>,
    /// IP versions enabled for the session.
    #[serde(default, rename = "enabledIpVersion")]
    pub enabled_ip_version: BgpNeighborEnabledIpVersion,
    /// eBGP multihop count, when configured.
    #[serde(default, rename = "ebgpMultihop")]
    pub ebgp_multihop: Option<i64>,
    /// Remote ASN.
    #[serde(default)]
    pub asn: BgpNeighborAsn,
    /// MD5 authentication secret. Never returned by the platform on reads.
    #[serde(default, rename = "md5Secret")]
    pub md5_secret: String,
    /// Import route map, when configured.
    #[serde(default)]
    pub import: Option<BgpNeighborRouteMap>,
    /// Export route map, when configured.
    #[serde(default)]
    pub export: Option<BgpNeighborRouteMap>,
    /// Neighbor name.
    #[serde(default)]
    pub name: String,
    /// Neighbor description.
    #[serde(default)]
    pub description: String,
}

// credential fields omitted from Debug
crate::redacted_debug!(
    RouterVrfBgpNeighbor;
    neighbor_id, address, is_shutdown, do_as_override, do_next_help_self, source,
    enabled_ip_version, ebgp_multihop, asn, import, export, name, description;
    md5_secret
);

/// A static route configured on a router VRF.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterStaticRoute {
    /// Static route id.
    #[serde(default, rename = "staticRouteId")]
    pub route_id: i64,
    /// Destination network in CIDR notation.
    #[serde(default)]
    pub network: String,
    /// Next hop for the route.
    #[serde(default)]
    pub via: StaticRouteVia,
    /// Route description.
    #[serde(default)]
    pub description: String,
    /// Administrative distance, when set.
    #[serde(default)]
    pub distance: Option<i64>,
}

/// A static route's next hop.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct StaticRouteVia {
    /// Next hop IP address, when routing to an address.
    #[serde(default, rename = "nextHop")]
    pub next_hop: String,
    /// Interface id, when routing out an interface.
    #[serde(default, rename = "interfaceId")]
    pub interface_id: Option<i64>,
    /// Tunnel id, when routing over a tunnel.
    #[serde(default, rename = "tunnelId")]
    pub tunnel_id: Option<i64>,
    /// IPsec peer id, when routing over an IPsec tunnel.
    #[serde(default, rename = "ipSecPeerId")]
    pub ip_sec_peer_id: Option<i64>,
}

/// A prefix list configured on a router.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterPrefixList {
    /// Prefix list id.
    #[serde(default, rename = "prefixListId")]
    pub prefix_list_id: i64,
    /// Prefix list name.
    #[serde(default)]
    pub name: String,
    /// IP version the list matches, 4 or 6.
    #[serde(default, rename = "ipVersion")]
    pub ip_version: i64,
    /// Prefix list description.
    #[serde(default)]
    pub description: String,
    /// Ordered match rules.
    #[serde(default)]
    pub rules: Vec<PrefixListRule>,
}

/// One rule in a prefix list.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct PrefixListRule {
    /// Action to take on a match: `permit` or `deny`.
    #[serde(default)]
    pub action: String,
    /// Prefix in CIDR notation.
    #[serde(default)]
    pub prefix: String,
}

/// A router's NTP configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterNtpConfig {
    /// Whether NTP is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Interface id NTP listens on, when set.
    #[serde(default, rename = "interfaceId")]
    pub interface_id: Option<i64>,
    /// Upstream NTP servers.
    #[serde(default)]
    pub upstreams: Vec<RouterNtpUpstream>,
}

/// One upstream NTP server.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterNtpUpstream {
    /// Upstream domain.
    #[serde(default)]
    pub domain: String,
}

/// A router's IPsec IKE and ESP configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterIpSecConfig {
    /// IKE phase configuration.
    #[serde(default, rename = "ikeGroup")]
    pub ike_group: RouterIpSecIkeGroup,
    /// ESP phase configuration.
    #[serde(default, rename = "espGroup")]
    pub esp_group: RouterIpSecEspGroup,
}

/// IKE phase configuration for a router's IPsec service.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterIpSecIkeGroup {
    /// Whether the tunnel automatically renegotiates before its key lifetime expires.
    #[serde(default, rename = "doAutoRenegotiation")]
    pub do_auto_renegotiation: bool,
    /// IKE protocol version, 1 or 2.
    #[serde(default, rename = "keyExchangeVersion")]
    pub key_exchange_version: i64,
    /// Key lifetime in seconds.
    #[serde(default, rename = "lifetimeSeconds")]
    pub lifetime_seconds: i64,
    /// Diffie-Hellman group number.
    #[serde(default, rename = "dhGroupNumber")]
    pub dh_group_number: i64,
    /// Encryption algorithm.
    #[serde(default)]
    pub encryption: String,
    /// Hash algorithm.
    #[serde(default)]
    pub hash: String,
    /// Pseudo-random function.
    #[serde(default)]
    pub prf: String,
}

/// ESP phase configuration for a router's IPsec service.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterIpSecEspGroup {
    /// Key lifetime in seconds.
    #[serde(default, rename = "lifetimeSeconds")]
    pub lifetime_seconds: i64,
    /// Encryption algorithm.
    #[serde(default)]
    pub encryption: String,
    /// Hash algorithm.
    #[serde(default)]
    pub hash: String,
}

/// An IPsec peer configured on a router VRF.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfIpSecPeer {
    /// Peer id.
    #[serde(default, rename = "ipSecPeerId")]
    pub ip_sec_peer_id: i64,
    /// Peer name.
    #[serde(default)]
    pub name: String,
    /// Peer description.
    #[serde(default)]
    pub description: Option<String>,
    /// Remote peer identifier.
    #[serde(default, rename = "remoteId")]
    pub remote_id: String,
    /// Pre-shared key secret.
    #[serde(default, rename = "pskSecret")]
    pub psk_secret: String,
    /// Whether this side initiates the connection.
    #[serde(default, rename = "doInitiateConnection")]
    pub do_initiate_connection: bool,
    /// Remote peer address.
    #[serde(default, rename = "peerAddress")]
    pub peer_address: String,
    /// Local peer identifier.
    #[serde(default, rename = "localId")]
    pub local_id: String,
    /// Overlay network addressing for the tunnel.
    #[serde(default, rename = "overlayNetwork")]
    pub overlay_network: RouterVrfIpSecOverlayNetwork,
}

/// Overlay network addressing for an IPsec tunnel.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfIpSecOverlayNetwork {
    /// Overlay IPv4 address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<String>,
    /// Overlay IPv6 address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<String>,
}

/// A network routed to a wireguard peer.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct WireguardPeerAllowedIp {
    /// Network in CIDR notation.
    #[serde(default)]
    pub network: String,
}

/// A wireguard peer configured on a router VRF interface.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfInterfaceWireguardPeer {
    /// Peer id.
    #[serde(default, rename = "wireguardPeerId")]
    pub wireguard_peer_id: i64,
    /// Networks routed to this peer.
    #[serde(default, rename = "allowedIps")]
    pub allowed_ips: Vec<WireguardPeerAllowedIp>,
    /// Peer's public key.
    #[serde(default, rename = "publicKey")]
    pub public_key: String,
    /// This side's private key.
    #[serde(default, rename = "privateKey")]
    pub private_key: String,
    /// Pre-shared key shared with the peer.
    #[serde(default, rename = "preSharedKey")]
    pub pre_shared_key: Option<String>,
    /// Peer's remote endpoint address.
    #[serde(default)]
    pub remote: Option<String>,
    /// Peer name.
    #[serde(default)]
    pub name: Option<String>,
    /// Peer description.
    #[serde(default)]
    pub description: Option<String>,
}

// credential fields omitted from Debug
crate::redacted_debug!(
    RouterVrfInterfaceWireguardPeer;
    wireguard_peer_id, allowed_ips, public_key, remote, name, description;
    private_key, pre_shared_key
);

/// An interface configured on a router VRF.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct RouterVrfInterface {
    /// Interface id.
    #[serde(default, rename = "interfaceId")]
    pub interface_id: i64,
    /// Id of the VRF the interface belongs to.
    #[serde(default, rename = "vrfId")]
    pub vrf_id: i64,
    /// Interface type.
    #[serde(default, rename = "type")]
    pub interface_type: String,
    /// Interface name.
    #[serde(default)]
    pub name: String,
    /// Interface description.
    #[serde(default)]
    pub description: Option<String>,
    /// IPv4 address in CIDR notation.
    #[serde(default, rename = "ipv4Cidr")]
    pub ipv4_cidr: Option<String>,
    /// IPv6 address in CIDR notation.
    #[serde(default, rename = "ipv6Cidr")]
    pub ipv6_cidr: Option<String>,
    /// Hardware id of the attached ethernet interface, for ethernet interfaces.
    #[serde(default, rename = "ethernetHardwareId")]
    pub ethernet_hardware_id: Option<String>,
    /// Wireguard listen port, for wireguard interfaces.
    #[serde(default, rename = "wireguardPort")]
    pub wireguard_port: Option<i64>,
    /// This side's wireguard public key, for wireguard interfaces.
    #[serde(default, rename = "publicKey")]
    pub public_key: Option<String>,
    /// Static routes attached to this interface, kept opaque pending a stable schema.
    #[serde(default, rename = "staticRoutes")]
    pub static_routes: Value,
    /// Wireguard peers configured on this interface.
    #[serde(default)]
    pub peers: Vec<RouterVrfInterfaceWireguardPeer>,
}

/// SNAT rule match criteria on a router VRF, matching by ingress interface and source network.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfSnatMatch {
    /// Id of the interface traffic must arrive on to match.
    #[serde(default, rename = "interfaceId")]
    pub interface_id: i64,
    /// Source network this rule applies to, in CIDR notation.
    #[serde(default)]
    pub network: String,
    /// Port range the rule matches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<VpcPortRange>,
}

/// Translation applied by a router VRF SNAT rule.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfSnatTranslation {
    /// Network traffic is translated to, in CIDR notation.
    #[serde(default)]
    pub network: String,
    /// Port range traffic is translated to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<VpcPortRange>,
}

/// Placement of a router VRF SNAT rule relative to the others on the VRF.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfSnatPriority {
    /// Placement location, for example "first", "last" or "after".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub location: String,
    /// Rule id to place this rule after, when location is "after".
    #[serde(
        default,
        rename = "afterSnatRuleId",
        skip_serializing_if = "Option::is_none"
    )]
    pub after_snat_rule_id: Option<i64>,
}

/// A SNAT rule configured on a router VRF.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfSnatRule {
    /// Rule id.
    #[serde(default, rename = "snatRuleId")]
    pub snat_rule_id: i64,
    /// IP version the rule applies to, 4 or 6.
    #[serde(default, rename = "ipVersion")]
    pub ip_version: i64,
    /// Protocol the rule matches.
    #[serde(default)]
    pub protocol: String,
    /// Rule name.
    #[serde(default)]
    pub name: String,
    /// Rule description.
    #[serde(default)]
    pub description: String,
    /// Match criteria.
    #[serde(default, rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_criteria: Option<RouterVrfSnatMatch>,
    /// Translation applied to matching traffic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translation: Option<RouterVrfSnatTranslation>,
    /// Placement of this rule relative to the others on the VRF.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<RouterVrfSnatPriority>,
}

/// DNAT rule match criteria on a router VRF, matching by ingress interface and destination
/// network.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfDnatMatch {
    /// Id of the interface traffic must arrive on to match.
    #[serde(default, rename = "interfaceId")]
    pub interface_id: i64,
    /// Destination network this rule applies to, in CIDR notation.
    #[serde(default)]
    pub network: String,
    /// Port range the rule matches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<VpcPortRange>,
}

/// Translation applied by a router VRF DNAT rule.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfDnatTranslation {
    /// Network traffic is forwarded to, in CIDR notation.
    #[serde(default)]
    pub network: String,
    /// Port range traffic is forwarded to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<VpcPortRange>,
}

/// Placement of a router VRF DNAT rule relative to the others on the VRF.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfDnatPriority {
    /// Placement location, for example "first", "last" or "after".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub location: String,
    /// Rule id to place this rule after, when location is "after".
    #[serde(
        default,
        rename = "afterDnatRuleId",
        skip_serializing_if = "Option::is_none"
    )]
    pub after_dnat_rule_id: Option<i64>,
}

/// A DNAT rule configured on a router VRF.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfDnatRule {
    /// Rule id.
    #[serde(default, rename = "dnatRuleId")]
    pub dnat_rule_id: i64,
    /// IP version the rule applies to, 4 or 6.
    #[serde(default, rename = "ipVersion")]
    pub ip_version: i64,
    /// Protocol the rule matches.
    #[serde(default)]
    pub protocol: String,
    /// Rule name.
    #[serde(default)]
    pub name: String,
    /// Rule description.
    #[serde(default)]
    pub description: String,
    /// Match criteria.
    #[serde(default, rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_criteria: Option<RouterVrfDnatMatch>,
    /// Translation applied to matching traffic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translation: Option<RouterVrfDnatTranslation>,
    /// Placement of this rule relative to the others on the VRF.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<RouterVrfDnatPriority>,
}

/// A tunnel's endpoint addresses, as returned by the platform.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfTunnelEndpoint {
    /// This side's address.
    #[serde(default)]
    pub source: String,
    /// Remote endpoint address.
    #[serde(default)]
    pub remote: String,
}

/// A tunnel configured on a router VRF.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfTunnel {
    /// Tunnel id.
    #[serde(default, rename = "tunnelId")]
    pub tunnel_id: i64,
    /// Tunnel name.
    #[serde(default)]
    pub name: String,
    /// Tunnel description.
    #[serde(default)]
    pub description: Option<String>,
    /// GRE key distinguishing this tunnel from others to the same remote.
    #[serde(default, rename = "ipKey")]
    pub ip_key: i64,
    /// Tunnel MTU, as reported by the platform.
    #[serde(default)]
    pub mtu: String,
    /// IPv4 address in CIDR notation.
    #[serde(default, rename = "ipv4Cidr")]
    pub ipv4_cidr: Option<String>,
    /// IPv6 address in CIDR notation.
    #[serde(default, rename = "ipv6Cidr")]
    pub ipv6_cidr: Option<String>,
    /// IP version the tunnel carries, 4 or 6.
    #[serde(default, rename = "ipVersion")]
    pub ip_version: i64,
    /// Tunnel endpoint addresses.
    #[serde(default, rename = "endpointAddress")]
    pub endpoint_address: RouterVrfTunnelEndpoint,
}

/// An address range a router VRF DHCP service leases from.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterDhcpRange {
    /// First address in the range.
    #[serde(default, rename = "firstAddress")]
    pub first_address: String,
    /// Last address in the range.
    #[serde(default, rename = "lastAddress")]
    pub last_address: String,
}

/// A server address handed to DHCP clients.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterDhcpServer {
    /// Server address.
    #[serde(default)]
    pub address: String,
}

/// A static route handed to DHCP clients.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterDhcpStaticRoute {
    /// Destination network in CIDR notation.
    #[serde(default)]
    pub network: String,
    /// Next hop for the route.
    #[serde(default, rename = "nextHop")]
    pub next_hop: String,
}

/// A router VRF's DHCP configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RouterVrfDhcpConfig {
    /// Whether the DHCP service is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Interface id the DHCP service listens on.
    #[serde(default, rename = "interfaceId")]
    pub interface_id: i64,
    /// Subnet DHCP serves addresses on, in CIDR notation.
    #[serde(default)]
    pub subnet: String,
    /// Default router address handed to clients.
    #[serde(default, rename = "defaultRouterAddress")]
    pub default_router_address: String,
    /// Domain name handed to clients.
    #[serde(default, rename = "clientDomainName")]
    pub client_domain_name: String,
    /// Lease timeout in seconds.
    #[serde(default, rename = "leaseTimeout")]
    pub lease_timeout: i64,
    /// Whether the server pings an address before leasing it.
    #[serde(default, rename = "doPingCheck")]
    pub do_ping_check: bool,
    /// Address range leased to clients.
    #[serde(default)]
    pub range: Option<RouterDhcpRange>,
    /// DNS servers handed to clients.
    #[serde(default, rename = "domainNameServers")]
    pub domain_name_servers: Vec<RouterDhcpServer>,
    /// NTP servers handed to clients.
    #[serde(default, rename = "ntpServers")]
    pub ntp_servers: Vec<RouterDhcpServer>,
    /// Static routes handed to clients.
    #[serde(default, rename = "staticRoutes")]
    pub static_routes: Vec<RouterDhcpStaticRoute>,
}

/// A boot profile available for building a cloud server, as returned by
/// [`crate::Client::get_boot_profiles`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct BootProfile {
    /// Boot profile id.
    #[serde(default, rename = "id")]
    pub boot_profile_id: i64,
    /// Profile name.
    #[serde(default)]
    pub name: String,
    /// Profile type.
    #[serde(default, rename = "type")]
    pub profile_type: String,
    /// Profile description.
    #[serde(default)]
    pub description: String,
    /// Builder used to construct the profile.
    #[serde(default)]
    pub builder: String,
    /// Kernel the profile boots.
    #[serde(default)]
    pub kernel: String,
    /// Boot mode.
    #[serde(default)]
    pub boot: String,
    /// Serial console configuration.
    #[serde(default)]
    pub serial: String,
    /// How the profile's disk is represented to the hypervisor.
    #[serde(default)]
    pub disk_represent: String,
    /// Image template id backing the profile, or zero when the profile has none.
    #[serde(default)]
    pub image_template: i64,
    /// Timestamp the profile was last updated.
    #[serde(default)]
    pub last_updated: String,
    /// Free-form extra configuration.
    #[serde(default)]
    pub extra: Option<String>,
    /// VNC display number.
    #[serde(default, rename = "vncdisplay")]
    pub vnc_display: Option<String>,
    /// Root disk device.
    #[serde(default)]
    pub disk_root: Option<String>,
    /// Bootloader used by the profile.
    #[serde(default)]
    pub bootloader: Option<String>,
    /// Ramdisk image path.
    #[serde(default)]
    pub ramdisk: Option<String>,
    /// Initrd image path.
    #[serde(default)]
    pub initrd: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created: Option<String>,
    /// Whether PAE is enabled.
    #[serde(default)]
    pub pae: i64,
    /// Whether ACPI is enabled.
    #[serde(default)]
    pub acpi: i64,
    /// Whether APIC is enabled.
    #[serde(default)]
    pub apic: i64,
    /// Whether the guest clock runs in local time.
    #[serde(default)]
    pub xlocaltime: i64,
    /// Whether an SDL console is enabled.
    #[serde(default)]
    pub sdl: i64,
    /// Whether a VNC console is enabled.
    #[serde(default)]
    pub vnc: i64,
    /// Whether the VNC console is enabled.
    #[serde(default, rename = "vncconsole")]
    pub vnc_console: i64,
    /// Whether the VNC port is left unused.
    #[serde(default, rename = "vncunused")]
    pub vnc_unused: i64,
    /// Whether the profile is hidden from selection.
    #[serde(default)]
    pub hide: i64,
    /// Whether the profile requires KVM.
    #[serde(default)]
    pub kvm: i64,
}

/// A disk attached to a cloud server's billing package, as returned by
/// [`crate::Client::get_server_disks`].
///
/// The platform does not document a stable field set for this endpoint, so each element
/// decodes successfully regardless of its content and exposes no fields.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ServerDisk {}

/// A disk layout compatible with a dedicated OS profile, as returned by
/// [`crate::Client::get_dedicated_disk_layouts`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DedicatedDiskLayout {
    /// Layout id.
    #[serde(default, rename = "id")]
    pub layout_id: i64,
    /// Layout name.
    #[serde(default)]
    pub name: String,
    /// Profile the layout applies to.
    #[serde(default)]
    pub profile: String,
    /// Minimum number of disks the layout requires.
    #[serde(default)]
    pub min_disks: i64,
}

/// A rescue OS image compatible with dedicated servers, as returned by
/// [`crate::Client::get_dedicated_rescue_os_profiles`].
///
/// Shares [`DedicatedOsProfile`]'s wire shape; the platform serves rescue images from a
/// separate endpoint but describes them the same way.
pub type DedicatedRescueOs = DedicatedOsProfile;

/// An SSL certificate installed on the account, as returned by
/// [`crate::V3Client::list_ssl_certificates`] and [`crate::V3Client::get_ssl_certificate`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct SslCertificate {
    /// Certificate id.
    #[serde(default, rename = "sslCertificateId")]
    pub ssl_certificate_id: i64,
    /// Certificate name.
    #[serde(default)]
    pub name: String,
    /// Certificate description.
    #[serde(default)]
    pub description: String,
    /// Certificate fingerprint.
    #[serde(default)]
    pub fingerprint: String,
    /// Domains the certificate covers.
    #[serde(default)]
    pub domains: Vec<String>,
    /// Whether the certificate is active.
    #[serde(default, rename = "isActive")]
    pub is_active: bool,
    /// Certificate status.
    #[serde(default)]
    pub status: String,
    /// Certificate validity dates, when the platform has parsed them.
    #[serde(default)]
    pub dates: Option<SslCertificateDates>,
}

/// Validity dates for an [`SslCertificate`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct SslCertificateDates {
    /// Creation timestamp.
    #[serde(default)]
    pub created: String,
    /// Last update timestamp.
    #[serde(default)]
    pub updated: String,
    /// Timestamp the certificate becomes valid.
    #[serde(default, rename = "notBefore")]
    pub not_before: String,
    /// Expiration timestamp.
    #[serde(default)]
    pub expiration: String,
}

/// The address an [`NlbGroup`] matches incoming traffic against.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NlbGroupMatch {
    /// Address to match against.
    #[serde(default)]
    pub address: String,
}

/// Health check configuration for an [`NlbGroup`]'s backends.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NlbGroupHealthCheck {
    /// Whether health checking is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Health check method.
    #[serde(default)]
    pub method: String,
    /// Interval between checks, in seconds.
    #[serde(default)]
    pub interval: i64,
    /// Number of failed checks tolerated before a backend is marked unhealthy.
    #[serde(default)]
    pub retries: i64,
    /// Delay before the first check runs, in seconds.
    #[serde(default)]
    pub delay: i64,
    /// Timeout for a single check, in seconds.
    #[serde(default)]
    pub timeout: i64,
}

/// The port mapping for one rule in an [`NlbGroup`].
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NlbGroupRulePorts {
    /// Port the load balancer listens on.
    #[serde(default, rename = "match")]
    pub match_port: i64,
    /// Port traffic is forwarded to on the backend.
    #[serde(default)]
    pub internal: i64,
}

/// One traffic rule in an [`NlbGroup`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NlbGroupRule {
    /// Protocol the rule applies to.
    #[serde(default)]
    pub protocol: String,
    /// Id of the underlying network rule, absent until the group is created.
    #[serde(default, rename = "networkRuleId", skip_serializing_if = "is_zero_i64")]
    pub network_rule_id: i64,
    /// Port mapping for the rule.
    pub ports: NlbGroupRulePorts,
}

/// One backend in an [`NlbGroup`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NlbGroupBackend {
    /// Backend name.
    #[serde(default)]
    pub name: String,
    /// Address the backend is reached at.
    #[serde(default, rename = "internalAddress")]
    pub internal_address: String,
    /// Whether the backend is currently online, absent until the group is created.
    #[serde(default, rename = "isOnline", skip_serializing_if = "is_false")]
    pub is_online: bool,
    /// Id of the underlying network backend, absent until the group is created.
    #[serde(
        default,
        rename = "networkBackendId",
        skip_serializing_if = "is_zero_i64"
    )]
    pub network_backend_id: i64,
}

/// A group on a network load balancer, as returned by [`crate::V3Client::list_nlb_groups`] and
/// [`crate::V3Client::get_nlb_group`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NlbGroup {
    /// Group id.
    #[serde(default, rename = "networkGroupId")]
    pub network_group_id: i64,
    /// Group name.
    #[serde(default)]
    pub name: String,
    /// Group description.
    #[serde(default)]
    pub description: String,
    /// IP version the group balances, 4 or 6.
    #[serde(default, rename = "ipVersion")]
    pub ip_version: i64,
    /// Load balancing algorithm.
    #[serde(default)]
    pub algorithm: String,
    /// Whether the group is currently online.
    #[serde(default, rename = "isOnline")]
    pub is_online: bool,
    /// Address the group matches incoming traffic against.
    #[serde(default, rename = "match")]
    pub matcher: NlbGroupMatch,
    /// Health check configuration for the group's backends.
    #[serde(default, rename = "healthCheck")]
    pub health_check: NlbGroupHealthCheck,
    /// Traffic rules for the group.
    #[serde(default)]
    pub rules: Vec<NlbGroupRule>,
    /// Backends the group balances across.
    #[serde(default)]
    pub backends: Vec<NlbGroupBackend>,
}

/// The address and ports an [`HttpLbGroup`] matches incoming traffic against.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct HttpLbGroupMatch {
    /// Address to match against.
    #[serde(default)]
    pub address: String,
    /// Ports to match against.
    #[serde(default)]
    pub ports: String,
}

/// Active health check configuration for an [`HttpLbGroup`]'s backends.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct HttpLbGroupHealthCheckActive {
    /// Whether active health checking is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Interval between checks, in seconds.
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub interval: i64,
    /// Number of failed checks tolerated before a backend is marked unhealthy.
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub retries: i64,
    /// Delay before the first check runs, in seconds.
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub delay: i64,
    /// Timeout for a single check, in seconds. Always present on the wire, even when null.
    #[serde(default)]
    pub timeout: Option<i64>,
    /// Path requested by an HTTP health check.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
}

/// Passive health check configuration for an [`HttpLbGroup`]'s backends.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct HttpLbGroupHealthCheckPassive {
    /// Whether passive health checking is enabled.
    #[serde(default)]
    pub enabled: bool,
}

/// Health check configuration for an [`HttpLbGroup`]'s backends.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct HttpLbGroupHealthCheck {
    /// Active health check configuration.
    #[serde(default)]
    pub active: HttpLbGroupHealthCheckActive,
    /// Passive health check configuration.
    #[serde(default)]
    pub passive: HttpLbGroupHealthCheckPassive,
}

/// The domain and path an [`HttpLbGroupRule`] matches requests against.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct HttpLbGroupRuleMatch {
    /// Domain to match against.
    #[serde(default)]
    pub domain: String,
    /// Path to match against.
    #[serde(default)]
    pub path: String,
}

/// TLS termination settings for an [`HttpLbGroupRule`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct HttpLbGroupRuleSsl {
    /// Whether TLS termination is enabled for the rule.
    #[serde(default)]
    pub enabled: bool,
    /// Id of the SSL certificate to terminate with, when set.
    #[serde(default, rename = "sslCertificateId")]
    pub ssl_certificate_id: Option<i64>,
}

/// One traffic rule in an [`HttpLbGroup`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct HttpLbGroupRule {
    /// Id of the underlying HTTP rule, absent until the group is created.
    #[serde(default, rename = "httpRuleId", skip_serializing_if = "is_zero_i64")]
    pub http_rule_id: i64,
    /// Whether HTTP requests are redirected to HTTPS.
    #[serde(default, rename = "httpsRedirectEnabled")]
    pub https_redirect_enabled: bool,
    /// Domain and path the rule matches.
    #[serde(default, rename = "match")]
    pub matcher: HttpLbGroupRuleMatch,
    /// TLS termination settings for the rule.
    #[serde(default)]
    pub ssl: HttpLbGroupRuleSsl,
}

/// One backend in an [`HttpLbGroup`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct HttpLbGroupBackend {
    /// Backend name.
    #[serde(default)]
    pub name: String,
    /// Address the backend is reached at.
    #[serde(default, rename = "internalAddress")]
    pub internal_address: String,
    /// Whether the backend is currently online, absent until the group is created.
    #[serde(default, rename = "isOnline", skip_serializing_if = "is_false")]
    pub is_online: bool,
    /// Id of the underlying HTTP backend, absent until the group is created.
    #[serde(default, rename = "httpBackendId", skip_serializing_if = "is_zero_i64")]
    pub http_backend_id: i64,
}

/// A group on an HTTP load balancer, as returned by [`crate::V3Client::list_http_lb_groups`] and
/// [`crate::V3Client::get_http_lb_group`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct HttpLbGroup {
    /// Group id.
    #[serde(default, rename = "httpGroupId")]
    pub http_group_id: i64,
    /// Group name.
    #[serde(default)]
    pub name: String,
    /// Group description.
    #[serde(default)]
    pub description: String,
    /// Load balancing algorithm.
    #[serde(default)]
    pub algorithm: String,
    /// Whether sticky sessions are enabled.
    #[serde(default, rename = "stickySessionsEnabled")]
    pub sticky_sessions_enabled: bool,
    /// Whether TLS is used from the load balancer to the backends.
    #[serde(default, rename = "sslToBackendEnabled")]
    pub ssl_to_backend_enabled: bool,
    /// Port the backends listen on.
    #[serde(default, rename = "internalPort")]
    pub internal_port: i64,
    /// Whether the group is currently online.
    #[serde(default, rename = "isOnline")]
    pub is_online: bool,
    /// Address and ports the group matches incoming traffic against.
    #[serde(default, rename = "match")]
    pub matcher: HttpLbGroupMatch,
    /// Health check configuration for the group's backends.
    #[serde(default, rename = "healthCheck")]
    pub health_check: HttpLbGroupHealthCheck,
    /// Traffic rules for the group.
    #[serde(default)]
    pub rules: Vec<HttpLbGroupRule>,
    /// Backends the group balances across.
    #[serde(default)]
    pub backends: Vec<HttpLbGroupBackend>,
}

/// Returns true when an i64 is zero, for omitting default ids from a request body.
fn is_zero_i64(value: &i64) -> bool {
    *value == 0
}

/// Returns true when a bool is false, for omitting default flags from a request body.
fn is_false(value: &bool) -> bool {
    !*value
}

/// One data point in a [`StatisticResult`].
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Default)]
pub struct StatisticSample {
    /// Number of samples the point summarizes.
    #[serde(default)]
    pub count: f64,
    /// Number of resources the point covers.
    #[serde(default)]
    pub resources: f64,
    /// Average value across the point's samples.
    #[serde(default)]
    pub avg: f64,
    /// Summed value across the point's samples.
    #[serde(default)]
    pub sum: f64,
}

/// The result of one metric queried by [`crate::V3Client::query_statistics`],
/// [`crate::V3Client::query_networking_statistics`] or
/// [`crate::V3Client::query_anycast_statistics`].
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct StatisticResult {
    /// Name of the queried metric.
    pub metric: String,
    /// Service the metric was collected from.
    pub service: String,
    /// Time series data for the metric.
    pub data: Vec<StatisticSample>,
}

impl<'de> Deserialize<'de> for StatisticResult {
    /// Reconstructs the metric name from the wire response's `metric` map.
    ///
    /// The platform echoes the query as `{"metric": {"<name>": ...}}` rather than sending the
    /// name as a plain field, so the first (and only meaningful) key of that map is the metric
    /// name. Multiple keys are not expected; the lowest, sorted first, is taken when they occur,
    /// matching gona's behaviour.
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            #[serde(default)]
            metric: BTreeMap<String, Value>,
            #[serde(default)]
            service: String,
            #[serde(default)]
            data: Vec<StatisticSample>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let metric = wire.metric.into_keys().next().unwrap_or_default();
        Ok(StatisticResult {
            metric,
            service: wire.service,
            data: wire.data,
        })
    }
}

/// The time window covered by a [`MetricNames`] query.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct MetricTimeWindow {
    /// Window start, as formatted by the platform.
    #[serde(default)]
    pub start: String,
    /// Window end, as formatted by the platform.
    #[serde(default)]
    pub end: String,
    /// Window length in seconds.
    #[serde(default)]
    pub seconds: i64,
}

/// A summary statistic within a [`MetricName`].
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Default)]
pub struct MetricSummary {
    /// Summed value.
    #[serde(default)]
    pub sum: f64,
    /// Average value.
    #[serde(default)]
    pub avg: f64,
    /// Minimum value.
    #[serde(default)]
    pub min: f64,
    /// Maximum value.
    #[serde(default)]
    pub max: f64,
}

/// One named metric within [`MetricNames`], as returned by
/// [`crate::V3Client::get_metric_names`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct MetricName {
    /// Metric name, taken from the wire response's map key.
    #[serde(skip)]
    pub metric: String,
    /// Service the metric was collected from.
    #[serde(default)]
    pub service: String,
    /// Number of resources contributing to the metric.
    #[serde(default)]
    pub resources: i64,
    /// Average summary across the queried window.
    #[serde(default)]
    pub avg: MetricSummary,
    /// Summary as of the most recent sample.
    #[serde(default)]
    pub last: MetricSummary,
    /// Summed summary across the queried window.
    #[serde(default)]
    pub sum: MetricSummary,
}

/// The full response from [`crate::V3Client::get_metric_names`]: every metric the account can
/// query, alongside the time window the platform evaluated them over.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct MetricNames {
    /// Time window the metrics were evaluated over.
    pub time_window: MetricTimeWindow,
    /// Metrics available to query, ordered by name.
    pub metrics: Vec<MetricName>,
}

/// Parses the response body of the all-metrics view into [`MetricNames`].
///
/// The platform sends a flat object keyed by metric name, with the time window carried under
/// the `__timeWindow` key and every other key holding one metric. Keys are visited in sorted
/// order so the result does not depend on the platform's own key ordering.
pub(crate) fn parse_metric_names(value: Value) -> Result<MetricNames> {
    let Value::Object(object) = value else {
        return Err(Error::Decode(
            "get metric names: expected a JSON object".to_string(),
        ));
    };

    let time_window = match object.get("__timeWindow") {
        Some(window) => serde_json::from_value(window.clone())?,
        None => MetricTimeWindow::default(),
    };

    let mut names: Vec<&String> = object.keys().filter(|key| !key.starts_with("__")).collect();
    names.sort();

    let mut metrics = Vec::with_capacity(names.len());
    for name in names {
        let mut metric: MetricName = serde_json::from_value(object[name].clone())?;
        metric.metric = name.clone();
        metrics.push(metric);
    }

    Ok(MetricNames {
        time_window,
        metrics,
    })
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

fn is_zero(value: &i64) -> bool {
    *value == 0
}

// --- DDoS ---

/// A recorded or active DDoS attack, as returned by [`crate::Client::get_ddos_attacks`] and
/// [`crate::Client::get_ddos_active_attacks`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DdosAttack {
    /// Attack id.
    #[serde(default, rename = "id")]
    pub attack_id: i64,
    /// Timestamp the attack started.
    #[serde(default)]
    pub date_start: String,
    /// Timestamp the attack ended, when it has.
    #[serde(default)]
    pub date_end: String,
    /// Attack status code.
    #[serde(default)]
    pub status: i64,
    /// Targeted IP address.
    #[serde(default)]
    pub ip: String,
    /// Targeted prefix.
    #[serde(default)]
    pub prefix: String,
    /// Traffic direction.
    #[serde(default)]
    pub direction: String,
    /// Peak packets per second observed.
    #[serde(default)]
    pub pps: i64,
    /// Id of the mitigation rule that matched.
    #[serde(default)]
    pub rule_id: i64,
    /// Type of the mitigation rule that matched.
    #[serde(default)]
    pub rule_type: String,
    /// Duration the mitigation ban lasts, in seconds.
    #[serde(default)]
    pub ban_duration: i64,
    /// Name of the mitigation rule that matched.
    #[serde(default)]
    pub rule_name: String,
}

/// One top attack entry summarized on the DDoS dashboard.
///
/// The platform has not been observed sending any fields on this entry, so it carries none
/// yet; add them here once a response is seen to populate it.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DdosDashboardAttack {}

/// DDoS dashboard summary, as returned by [`crate::Client::get_ddos_dashboard`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DdosDashboard {
    /// Total attacks recorded over the requested period.
    #[serde(default)]
    pub total_attacks: i64,
    /// Number of mitigation rules currently active.
    #[serde(default)]
    pub active_rules: i64,
    /// Duration of the longest attack in the period, in seconds.
    #[serde(default)]
    pub longest_attack_seconds: i64,
    /// The most significant attacks in the period.
    #[serde(default)]
    pub top_attacks: Vec<DdosDashboardAttack>,
    /// The period, in days, the dashboard summarizes.
    #[serde(default)]
    pub period: i64,
}

/// Options for [`crate::Client::get_ddos_dashboard`]. Every field is optional; the platform
/// applies its own default when a field is omitted.
#[derive(Debug, Clone, Copy, Default)]
pub struct DdosDashboardOptions {
    /// Number of days to summarize.
    pub period: Option<i64>,
    /// Whether to include attacks that have already ended.
    pub include_ended: Option<bool>,
    /// Maximum number of top attacks to return.
    pub limit: Option<i64>,
}

/// A prefix protected by a [`DdosRule`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DdosRulePrefix {
    /// Prefix id.
    #[serde(default, rename = "id")]
    pub prefix_id: i64,
    /// The protected prefix.
    #[serde(default)]
    pub prefix: String,
    /// Prefix type.
    #[serde(default)]
    pub prefix_type: String,
    /// Prefix description.
    #[serde(default)]
    pub description: String,
    /// Packets per second allowed before mitigation engages.
    #[serde(default)]
    pub allowed_pps: i64,
}

/// One mitigation action within a [`DdosRule`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DdosRuleAction {
    /// Action identifier.
    #[serde(default)]
    pub name: String,
    /// Action type.
    #[serde(default)]
    pub action_type: String,
    /// Order the action runs in relative to other actions on the rule.
    #[serde(default)]
    pub run_order: i64,
    /// Action display name.
    #[serde(default)]
    pub action_name: String,
    /// Action description.
    #[serde(default)]
    pub action_description: String,
}

/// A DDoS mitigation rule, as returned by [`crate::Client::get_ddos_rules`] and
/// [`crate::Client::get_ddos_rule`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DdosRule {
    /// Rule id.
    #[serde(default, rename = "id")]
    pub rule_id: i64,
    /// Rule name.
    #[serde(default)]
    pub rule_name: String,
    /// Rule description.
    #[serde(default)]
    pub description: String,
    /// Prefixes the rule protects.
    #[serde(default)]
    pub prefixes: Vec<DdosRulePrefix>,
    /// Mitigation actions the rule applies.
    #[serde(default)]
    pub rules: Vec<DdosRuleAction>,
}

// --- Access control subnets ---

/// A subnet authorized to reach the account's management API, as returned by
/// [`crate::Client::get_access_control_subnets`] and [`crate::Client::get_access_control_subnet`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct AccessControlSubnet {
    /// Subnet id.
    #[serde(default, deserialize_with = "flexible_int")]
    pub id: i64,
    /// Subnet label.
    #[serde(default)]
    pub label: String,
    /// The authorized subnet, in CIDR notation.
    #[serde(default)]
    pub subnet: String,
}

/// Request body for creating an access control subnet.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateAccessControlSubnetRequest {
    /// Subnet label.
    pub label: String,
    /// The authorized subnet, in CIDR notation.
    pub subnet: String,
}

/// Request body for updating an access control subnet. Fields left unset are left unchanged.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateAccessControlSubnetRequest {
    /// Subnet label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The authorized subnet, in CIDR notation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet: Option<String>,
}

// --- Dedicated server build status (metal) ---

/// The status of a dedicated server build, as returned by
/// [`crate::Client::get_dedicated_server_build_status`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DedicatedServerBuildStatus {
    /// Billing package id.
    #[serde(default)]
    pub mbpkgid: i64,
    /// Free-form build log or response text.
    #[serde(default)]
    pub response: String,
    /// Build status text.
    #[serde(default)]
    pub status: String,
    /// Build completion percentage.
    #[serde(default)]
    pub percent: i64,
    /// Name of the image being built.
    #[serde(default)]
    pub image_name: String,
}

// --- BGP sessions ---

/// A prefix announced or accepted on a [`BgpSession`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct BgpSessionPrefix {
    /// Prefix id.
    #[serde(default)]
    pub id: i64,
    /// Id of the dedicated device the prefix belongs to.
    #[serde(default)]
    pub mb_id: i64,
    /// The announced prefix.
    #[serde(default)]
    pub prefix: String,
    /// AS path append value, when the platform reports one.
    #[serde(default)]
    pub append: Value,
    /// Rule type.
    #[serde(default)]
    pub rule_type: String,
    /// Prefix type.
    #[serde(default)]
    pub prefix_type: String,
    /// Prefix description.
    #[serde(default)]
    pub description: String,
    /// Date the prefix was added.
    #[serde(default)]
    pub date: String,
    /// Packets per second allowed on the prefix.
    #[serde(default)]
    pub allowed_pps: i64,
    /// Id of the owning BGP group.
    #[serde(default)]
    pub bgp_group_id: i64,
    /// Prefix id as recorded on the BGP session.
    #[serde(default)]
    pub prefix_id: i64,
}

/// A BGP session, as returned by [`crate::Client::get_bgp_session`] and
/// [`crate::Client::list_bgp_sessions`].
#[derive(Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct BgpSession {
    /// Session id.
    #[serde(default)]
    pub id: i64,
    /// The customer-side peer IP address.
    #[serde(default)]
    pub customer_peer_ip: String,
    /// Id of the owning BGP group.
    #[serde(default)]
    pub group_id: i64,
    /// Nonzero while the session is locked against changes.
    #[serde(default)]
    pub locked: i64,
    /// Session description.
    #[serde(default)]
    pub description: String,
    /// Session state, whose shape varies by platform version.
    #[serde(default)]
    pub state: Value,
    /// Routes received on the session, whose shape varies by platform version.
    #[serde(default)]
    pub routes_received: Value,
    /// Timestamp of the last session update, whose shape varies by platform version.
    #[serde(default)]
    pub last_update: Value,
    /// Configuration status code.
    #[serde(default)]
    pub config_status: i64,
    /// Session password, whose shape varies by platform version.
    #[serde(default)]
    pub password: Value,
    /// Prefixes announced or accepted on the session.
    #[serde(default)]
    pub prefixes: Vec<BgpSessionPrefix>,
    /// Export list applied to the session.
    #[serde(default)]
    pub export_list: String,
    /// BGP community, whose shape varies by platform version.
    #[serde(default)]
    pub community: Value,
    /// The provider-side peer IP address.
    #[serde(default)]
    pub provider_peer_ip: String,
    /// Session location display name.
    #[serde(default)]
    pub location: String,
    /// Session location latitude.
    #[serde(default)]
    pub latitude: String,
    /// Session location longitude.
    #[serde(default)]
    pub longitude: String,
    /// Name of the owning BGP group.
    #[serde(default)]
    pub group_name: String,
    /// Provider peer address family, such as `ipv4` or `ipv6`.
    #[serde(default)]
    pub provider_ip_type: String,
    /// Provider-side ASN.
    ///
    /// The platform sends this as a JSON string in some responses, so it is decoded
    /// tolerantly rather than requiring one wire shape.
    #[serde(default, deserialize_with = "flexible_int")]
    pub provider_asn: i64,
    /// Customer-side ASN, decoded with the same tolerance as `provider_asn`.
    #[serde(default, deserialize_with = "flexible_int")]
    pub customer_asn: i64,
}

// credential fields omitted from Debug
crate::redacted_debug!(
    BgpSession;
    id, customer_peer_ip, group_id, locked, description, state, routes_received, last_update,
    config_status, prefixes, export_list, community, provider_peer_ip, location, latitude,
    longitude, group_name, provider_ip_type, provider_asn, customer_asn;
    password
);

impl BgpSession {
    /// Reports whether the session is currently locked against changes.
    pub fn is_locked(&self) -> bool {
        self.locked == 1
    }

    /// Reports whether the provider peer address family is IPv4.
    pub fn is_provider_ip_type_v4(&self) -> bool {
        self.provider_ip_type == "ipv4"
    }
}

// --- Capacity ---

/// A billing package, as returned by [`crate::Client::get_billing_packages`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct BillingPackage {
    /// Billing package id.
    #[serde(default)]
    pub id: i64,
    /// Package name.
    #[serde(default)]
    pub name: String,
    /// Domain-scoped label, when set.
    #[serde(default)]
    pub domu_label: Option<String>,
    /// Underlying package id.
    #[serde(default, rename = "packageid")]
    pub package_id: i64,
    /// Domain associated with the package.
    #[serde(default)]
    pub domain: String,
    /// Recurring amount.
    #[serde(default)]
    pub amount: String,
    /// Billing cycle.
    #[serde(default, rename = "billingcycle")]
    pub billing_cycle: String,
    /// Domain lifecycle status.
    #[serde(default, rename = "domainstatus")]
    pub domain_status: String,
    /// Next due date.
    #[serde(default, rename = "nextduedate")]
    pub next_due_date: String,
    /// Whether the package includes a dedicated IP.
    #[serde(default, rename = "dedicatedip")]
    pub dedicated_ip: String,
}

/// Available cloud capacity for a package shape at a location, as returned by
/// [`crate::Client::get_cloud_capacity`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct CloudCapacity {
    /// Package id.
    #[serde(default)]
    pub pkg_id: i64,
    /// Package name.
    #[serde(default)]
    pub pkg_name: String,
    /// vCPU count.
    #[serde(default)]
    pub pkg_cpu: i64,
    /// RAM in megabytes.
    #[serde(default)]
    pub pkg_ram: i64,
    /// Disk size in gigabytes.
    #[serde(default)]
    pub pkg_disk: i64,
    /// Network allocation.
    #[serde(default)]
    pub pkg_net: i64,
    /// Port allocation.
    #[serde(default)]
    pub pkg_port: i64,
    /// Monthly price.
    #[serde(default)]
    pub monthly_price: f64,
    /// Number of units currently available.
    #[serde(default)]
    pub available: i64,
}

/// Available dedicated server capacity, as returned by
/// [`crate::Client::get_dedicated_capacity`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct DedicatedCapacity {
    /// Device id.
    #[serde(default)]
    pub device_id: i64,
    /// Location id.
    #[serde(default)]
    pub location_id: i64,
    /// Looking glass hostname for the device's location.
    #[serde(default)]
    pub looking_glass: String,
    /// Billing package id, when the device is already sold.
    #[serde(default)]
    pub mbpkgid: i64,
    /// Device name.
    #[serde(default)]
    pub name: String,
    /// Whether the netactuate provisioning system is enabled for the device.
    #[serde(default)]
    pub nps_enabled: i64,
    /// Public-facing description of the device.
    #[serde(default)]
    pub pub_description: String,
}

// --- Non-cloud packages ---

/// Location and billing detail shared by colocation and transit packages.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NonCloudPackageDetails {
    /// Datacenter name.
    #[serde(default)]
    pub dc_name: String,
    /// Datacenter IATA airport code.
    #[serde(default)]
    pub iata_code: String,
    /// Committed bandwidth.
    #[serde(default)]
    pub bw_commit: String,
    /// Overage billing type.
    #[serde(default)]
    pub overage_type: String,
    /// Overage rate.
    #[serde(default)]
    pub overage_rate: String,
    /// Billing package id of the aggregate bandwidth pool, when the package belongs to one.
    #[serde(default)]
    pub agg_bw_mbpkgid: String,
}

/// A colocation package, as returned by [`crate::Client::get_colocation_packages`] and
/// [`crate::Client::get_colocation_package`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ColocationPackage {
    /// Billing package id.
    #[serde(default)]
    pub mbpkgid: i64,
    /// Package lifecycle status.
    #[serde(default)]
    pub package_status: String,
    /// Fully qualified domain name associated with the package.
    #[serde(default)]
    pub fqdn: String,
    /// Billing cycle.
    #[serde(default, rename = "billingcycle")]
    pub billing_cycle: String,
    /// Next due date.
    #[serde(default, rename = "nextduedate")]
    pub next_due_date: String,
    /// Recurring amount.
    #[serde(default)]
    pub amount: String,
    /// Location and billing detail.
    #[serde(default)]
    pub details: NonCloudPackageDetails,
    /// Package status text.
    #[serde(default)]
    pub status: String,
}

/// A transit package, as returned by [`crate::Client::get_transit_packages`] and
/// [`crate::Client::get_transit_package`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct TransitPackage {
    /// Billing package id.
    #[serde(default)]
    pub mbpkgid: i64,
    /// Package lifecycle status.
    #[serde(default)]
    pub package_status: String,
    /// Fully qualified domain name associated with the package.
    #[serde(default)]
    pub fqdn: String,
    /// Billing cycle.
    #[serde(default, rename = "billingcycle")]
    pub billing_cycle: String,
    /// Next due date.
    #[serde(default, rename = "nextduedate")]
    pub next_due_date: String,
    /// Recurring amount.
    #[serde(default)]
    pub amount: String,
    /// Location and billing detail.
    #[serde(default)]
    pub details: NonCloudPackageDetails,
    /// Package status text.
    #[serde(default)]
    pub status: String,
}

// --- Cloud packages ---

/// A purchased cloud package, as returned by [`crate::Client::get_packages`] and
/// [`crate::Client::get_package`].
///
/// `id`, `locked` and `installed` arrive as either a quoted or an unquoted number depending on
/// the endpoint, so they are decoded tolerantly rather than requiring one wire shape.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Package {
    /// Billing package id.
    #[serde(default, rename = "mbpkgid", deserialize_with = "flexible_int")]
    pub id: i64,
    /// Package lifecycle status.
    #[serde(default, rename = "package_status")]
    pub status: String,
    /// Nonzero while the package is locked.
    #[serde(default, deserialize_with = "flexible_int")]
    pub locked: i64,
    /// Plan name.
    #[serde(default, rename = "name")]
    pub plan_name: String,
    /// Nonzero once the package is installed.
    #[serde(default, deserialize_with = "flexible_int")]
    pub installed: i64,
}

/// Request body for canceling a cloud package.
#[derive(Clone, Default, Serialize)]
pub struct CancelPackageRequest {
    /// Billing package id to cancel.
    pub mbpkgid: i64,
    /// Package to move usage to instead of canceling outright, when supported.
    #[serde(rename = "domU_package", skip_serializing_if = "Option::is_none")]
    pub domu_package: Option<String>,
    /// Free-form cancellation comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Cancellation type.
    pub cancel_type: String,
    /// Set to a nonzero value to confirm the cancellation.
    pub agree: i64,
    /// Account password, when the platform requires it to confirm cancellation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

// credential fields omitted from Debug
crate::redacted_debug!(
    CancelPackageRequest;
    mbpkgid, domu_package, comments, cancel_type, agree;
    password
);

// --- Longtail ---

/// The platform location detected for the caller's current IP address, as returned by
/// [`crate::Client::get_location_by_current_ip`].
///
/// The full response is kept in `raw` alongside the commonly used fields, since the platform
/// has been observed sending additional fields this type does not model.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct LocationByCurrentIp {
    /// Caller IP address, when the platform reports one.
    pub ip: String,
    /// Detected platform location, when the platform reports one.
    pub location: String,
    /// The complete response body.
    pub raw: Value,
}

impl<'de> Deserialize<'de> for LocationByCurrentIp {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let ip = value
            .get("ip")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let location = value
            .get("location")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        Ok(Self {
            ip,
            location,
            raw: value,
        })
    }
}

/// Graph data for a switch port, as returned by [`crate::Client::get_graph`].
///
/// The shape of the payload varies with the requested time range, so the complete response is
/// kept as-is rather than modelled further.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct Graph {
    /// The complete response body.
    pub raw: Value,
}

impl<'de> Deserialize<'de> for Graph {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            raw: Value::deserialize(deserializer)?,
        })
    }
}

// --- Account limits (vAPI3) ---

/// Usage and ceiling for one account-level limit, as returned by
/// [`crate::V3Client::get_account_limits`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct AccountLimit {
    /// Units currently used.
    #[serde(default)]
    pub used: i64,
    /// Maximum units allowed.
    #[serde(default)]
    pub max: i64,
    /// Plans allowed to consume this limit.
    #[serde(default, rename = "allowedPlans")]
    pub allowed_plans: Vec<String>,
}

// --- Locations ---

/// An available deployment location, as returned by [`crate::Client::get_locations`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Location {
    /// Location id.
    #[serde(default)]
    pub id: i64,
    /// Location code.
    #[serde(default)]
    pub name: String,
    /// IATA airport code for the location.
    #[serde(default)]
    pub iata_code: String,
    /// Continent the location is on.
    #[serde(default)]
    pub continent: String,
    /// URL of the country flag icon.
    #[serde(default)]
    pub flag: String,
    /// Nonzero when the location is disabled for new deployments.
    #[serde(default)]
    pub disabled: i64,
}

// --- Network IPs ---

/// The address family of a [`NetworkIpAddress`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpAddressFamily {
    /// An IPv4 address.
    Ipv4,
    /// An IPv6 address.
    Ipv6,
}

impl IpAddressFamily {
    /// Returns the wire representation of the address family, `ipv4` or `ipv6`.
    pub fn as_str(&self) -> &'static str {
        match self {
            IpAddressFamily::Ipv4 => "ipv4",
            IpAddressFamily::Ipv6 => "ipv6",
        }
    }
}

/// One IP address on a billing package's network, as returned within [`NetworkIps`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NetworkIpAddress {
    /// Address id.
    #[serde(default)]
    pub id: i64,
    /// Nonzero when this is the package's primary address.
    #[serde(default)]
    pub primary: i64,
    /// Reverse DNS entry.
    #[serde(default)]
    pub reverse: String,
    /// The IP address.
    #[serde(default)]
    pub ip: String,
    /// Gateway address.
    #[serde(default)]
    pub gateway: String,
    /// Netmask.
    #[serde(default)]
    pub netmask: String,
    /// Broadcast address.
    #[serde(default)]
    pub broadcast: String,
}

/// The IPv4 and IPv6 addresses on a billing package's network, as returned by
/// [`crate::Client::get_ips`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct NetworkIps {
    /// IPv4 addresses.
    #[serde(default, rename = "IPv4")]
    pub ipv4: Vec<NetworkIpAddress>,
    /// IPv6 addresses.
    #[serde(default, rename = "IPv6")]
    pub ipv6: Vec<NetworkIpAddress>,
}

impl NetworkIps {
    /// Maps every address on the package, in both forms an IPv6 address might be compared
    /// against, to the family it belongs to.
    ///
    /// An IPv6 address is indexed both as written and in fully expanded form, since callers may
    /// hold either shape.
    pub fn address_families(&self) -> BTreeMap<String, IpAddressFamily> {
        let mut families = BTreeMap::new();
        for address in &self.ipv4 {
            families.insert(address.ip.clone(), IpAddressFamily::Ipv4);
        }
        for address in &self.ipv6 {
            families.insert(address.ip.clone(), IpAddressFamily::Ipv6);
            if let Some(expanded) = expand_ipv6(&address.ip) {
                families.insert(expanded, IpAddressFamily::Ipv6);
            }
        }
        families
    }
}

/// Renders an IPv6 address in fully expanded form: eight colon-separated four-digit hex
/// groups, with no `::` compression.
fn expand_ipv6(address: &str) -> Option<String> {
    let parsed: std::net::Ipv6Addr = address.parse().ok()?;
    let groups: Vec<String> = parsed
        .segments()
        .iter()
        .map(|segment| format!("{segment:04x}"))
        .collect();
    Some(groups.join(":"))
}

#[cfg(test)]
mod domain_parity_tests {
    use super::*;

    #[test]
    fn network_ips_address_families_indexes_expanded_ipv6() {
        let mut ips = NetworkIps::default();
        ips.ipv4.push(NetworkIpAddress {
            id: 1,
            primary: 1,
            ip: "203.0.113.5".to_string(),
            ..Default::default()
        });
        ips.ipv6.push(NetworkIpAddress {
            id: 2,
            ip: "2001:db8::1".to_string(),
            ..Default::default()
        });
        let families = ips.address_families();
        assert_eq!(families.get("203.0.113.5"), Some(&IpAddressFamily::Ipv4));
        assert_eq!(families.get("2001:db8::1"), Some(&IpAddressFamily::Ipv6));
        assert_eq!(
            families.get("2001:0db8:0000:0000:0000:0000:0000:0001"),
            Some(&IpAddressFamily::Ipv6)
        );
    }

    #[test]
    fn ddos_dashboard_options_default_omits_every_query_param() {
        let options = DdosDashboardOptions::default();
        assert!(options.period.is_none());
        assert!(options.include_ended.is_none());
        assert!(options.limit.is_none());
    }

    #[test]
    fn bgp_session_helpers_read_locked_and_provider_ip_type() {
        let session = BgpSession {
            locked: 1,
            provider_ip_type: "ipv4".to_string(),
            ..Default::default()
        };
        assert!(session.is_locked());
        assert!(session.is_provider_ip_type_v4());

        let unlocked = BgpSession {
            locked: 0,
            provider_ip_type: "ipv6".to_string(),
            ..Default::default()
        };
        assert!(!unlocked.is_locked());
        assert!(!unlocked.is_provider_ip_type_v4());
    }

    #[test]
    fn location_by_current_ip_keeps_raw_alongside_named_fields() {
        let decoded: LocationByCurrentIp =
            serde_json::from_str(r#"{"ip":"203.0.113.5","location":"chi","extra":"kept"}"#)
                .expect("decode");
        assert_eq!(decoded.ip, "203.0.113.5");
        assert_eq!(decoded.location, "chi");
        assert_eq!(decoded.raw["extra"], "kept");
    }

    #[test]
    fn wireguard_peer_debug_redacts_keys_but_keeps_other_fields() {
        let peer = RouterVrfInterfaceWireguardPeer {
            wireguard_peer_id: 42,
            private_key: "fake-private-key-value".to_string(),
            pre_shared_key: Some("fake-pre-shared-key-value".to_string()),
            public_key: "fake-public-key-value".to_string(),
            name: Some("peer-name".to_string()),
            ..Default::default()
        };
        let debugged = format!("{:?}", peer);
        assert!(!debugged.contains("fake-private-key-value"));
        assert!(!debugged.contains("fake-pre-shared-key-value"));
        assert!(debugged.contains("REDACTED"));
        assert!(debugged.contains("fake-public-key-value"));
        assert!(debugged.contains("peer-name"));
        assert!(debugged.contains("42"));
    }

    #[test]
    fn secret_list_value_debug_is_not_over_redacted() {
        let value = SecretListValue {
            id: 1,
            secret_list_id: 2,
            secret_key: "fake-secret-key".to_string(),
            secret_value: "fake-secret-value".to_string(),
        };
        let debugged = format!("{:?}", value);
        assert!(debugged.contains("fake-secret-key"));
        assert!(debugged.contains("fake-secret-value"));
        assert!(!debugged.contains("REDACTED"));
    }
}
