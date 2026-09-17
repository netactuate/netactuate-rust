use crate::error::{Error, Result};
use crate::models::{decode_required, DnsRecord, DnsZone, Server, ServerBuild};
#[cfg(feature = "blocking")]
use crate::transport::ReqwestTransport;
use crate::transport::{api_key_from_env, build_url, redact_url, DynTransport, Method, Request};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use url::Url;

const BASE_ENDPOINT: &str = "https://vapi2.netactuate.com/api/";

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
