//! This module defines the `Instance` struct, which represents a Google Compute Engine instance,
//! and provides a `TryFrom` implementation for creating an `Instance` from JSON data.

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::error::Error;

/// Represents a Google Compute Engine instance.
#[derive(Debug, Clone)]
pub struct Instance {
    /// The name of the instance.
    pub name: String,
    /// The IP address of the instance.
    pub ip: String,
    /// The zone the instance is running in.
    pub zone: String,
    /// The machine type of the instance.
    pub machine_type: String,
    /// The CPU platform of the instance.
    pub cpu_platform: String,
    /// The status of the instance.
    pub status: String,
    /// The labels associated with the instance.
    pub labels: Option<HashMap<String, String>>,
    /// The region the instance is running in.
    pub region: String,
    /// The cell the instance is running in.
    pub cell: Option<String>,
}

impl TryFrom<JsonValue> for Instance {
    type Error = Box<dyn Error>;

    /// Attempts to create an `Instance` from a `JsonValue`.
    ///
    /// This function parses the JSON data and extracts the relevant fields
    /// to populate the `Instance` struct.  It returns an error if any required
    /// fields are missing or have invalid formats.
    ///
    /// # Arguments
    ///
    /// * `json` - The `JsonValue` containing the instance data.
    ///
    /// # Returns
    ///
    /// * `Ok(Instance)` - The created `Instance` on success.
    /// * `Err(Box<dyn Error>)` - An error if the JSON data is invalid or missing required fields.
    fn try_from(json: JsonValue) -> Result<Self, Self::Error> {
        let name = json
            .get("name")
            .and_then(JsonValue::as_str)
            .ok_or("Missing or invalid 'name' field")?
            .to_string();
        let ip = json
            .get("networkInterfaces")
            .and_then(JsonValue::as_array)
            .and_then(|arr| arr.first()) // Get the first network interface
            .and_then(|iface| iface.get("networkIP"))
            .and_then(JsonValue::as_str)
            .ok_or("Missing or invalid 'networkInterfaces[0].networkIP' field")?
            .to_string();
        let zone = json
            .get("zone")
            .and_then(JsonValue::as_str)
            .ok_or("Missing or invalid 'zone' field")?
            .split('/')
            .last()
            .ok_or("Invalid 'zone' format")?
            .to_string();
        let machine_type = json
            .get("machineType")
            .and_then(JsonValue::as_str)
            .ok_or("Missing or invalid 'machineType' field")?
            .split('/')
            .last()
            .ok_or("Invalid 'machineType' format")?
            .to_string();
        let cpu_platform = json
            .get("cpuPlatform")
            .and_then(JsonValue::as_str)
            .ok_or("Missing or invalid 'cpuPlatform' field")?
            .to_string();
        let status = json
            .get("status")
            .and_then(JsonValue::as_str)
            .ok_or("Missing or invalid 'status' field")?
            .to_string();
        let labels = json
            .get("labels")
            .and_then(JsonValue::as_object) // Convert to object or None
            // Convert the labels (which is Map<String, &Value> to a HashMap of String key-value pairs if as_object is
            // Some otherwise, if map was called on None it returns None
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect::<HashMap<String, String>>()
            });
        // Extract the cell from the labels if it exists
        let cell = labels
            .as_ref()
            .and_then(|labels| labels.get("cell"))
            .map(|cell| cell.to_string());

        // Extract the region from the zone
        let region = zone
            .split('-')
            .take(2) // TODO: can we assume that the region is always the first two parts of the zone?
            .collect::<Vec<&str>>()
            .join("-")
            .to_string();

        Ok(Instance {
            name,
            ip,
            zone,
            machine_type,
            cpu_platform,
            status,
            labels,
            region,
            cell,
        })
    }
}

impl Instance {
    /// Formats the `Instance` data into a human-readable string.
    ///
    /// This function creates a string representation of the `Instance`
    /// including all its fields and labels.  This is primarily intended
    /// for debugging and logging purposes.
    ///
    /// # Returns
    ///
    /// * `String` - The formatted string representation of the `Instance`.
    #[allow(dead_code)]
    pub fn as_string(&self) -> String {
        let labels_str = match &self.labels {
            Some(labels) => labels
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<String>>()
                .join(", "),
            None => "None".to_string(),
        };
        format!(
            "Name: {} IP: {} Zone: {} Machine Type: {} CPU Platform: {} Status: {} Labels: {}",
            self.name,
            self.ip,
            self.zone,
            self.machine_type,
            self.cpu_platform,
            self.status,
            labels_str
        )
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_instance_from_json() {
        // Test data representing a valid instance JSON
        let json = json!({
            "name": "test-instance",
            "networkInterfaces": [
                {
                    "networkIP": "127.0.0.1"
                }
            ],
            "zone": "projects/12345/zones/test-region-foo", // Full zone path
            "machineType": "projects/12345/machineTypes/test-machine-type", // Full machine type path
            "cpuPlatform": "test-cpu-platform",
            "status": "test-status",
            "labels": {
                "key1": "value1",
                "cell": "int-test-cell",
            }
        });

        // Attempt to create an Instance from the JSON data.
        let instance = Instance::try_from(json).unwrap();

        // Assertions to check if the Instance fields are correctly populated.
        assert_eq!(instance.name, "test-instance");
        assert_eq!(instance.ip, "127.0.0.1");
        assert_eq!(instance.zone, "test-region-foo"); // Extracted zone
        assert_eq!(instance.machine_type, "test-machine-type"); // Extracted machine type
        assert_eq!(instance.cpu_platform, "test-cpu-platform");
        assert_eq!(instance.status, "test-status");
        assert_eq!(instance.labels, {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("cell".to_string(), "int-test-cell".to_string());
            Some(map)
        });
        assert_eq!(instance.region, "test-region");
        assert_eq!(instance.cell, Some("int-test-cell".to_string()));
    }
}
