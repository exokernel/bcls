use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Instance {
    pub name: String,
    pub ip: String,
    pub zone: String,
    pub machine_type: String,
    pub cpu_platform: String,
    pub status: String,
    pub labels: HashMap<String, String>,
}

impl TryFrom<JsonValue> for Instance {
    type Error = Box<dyn Error>;

    fn try_from(json: JsonValue) -> Result<Self, Self::Error> {
        let name = json
            .get("name")
            .and_then(JsonValue::as_str)
            .ok_or("No name")?
            .to_string();
        let ip = json
            .get("networkInterfaces")
            .and_then(JsonValue::as_array)
            .ok_or("No networkInterfaces")?[0]
            .get("networkIP")
            .and_then(JsonValue::as_str)
            .ok_or("No networkIP")?
            .to_string();
        let zone = json
            .get("zone")
            .and_then(JsonValue::as_str)
            .ok_or("No zone")?
            .split('/')
            .last()
            .ok_or("Invalid zone format")?
            .to_string();
        let machine_type = json
            .get("machineType")
            .and_then(JsonValue::as_str)
            .ok_or("No machineType")?
            .split('/')
            .last()
            .ok_or("Invalid machineType format")?
            .to_string();
        let cpu_platform = json
            .get("cpuPlatform")
            .and_then(JsonValue::as_str)
            .ok_or("No cpuPlatform")?
            .to_string();
        let status = json
            .get("status")
            .and_then(JsonValue::as_str)
            .ok_or("No status")?
            .to_string();
        let labels = json
            .get("labels")
            .and_then(JsonValue::as_object)
            .ok_or("No labels")?
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect::<HashMap<String, String>>();
        Ok(Instance {
            name,
            ip,
            zone,
            machine_type,
            cpu_platform,
            status,
            labels,
        })
    }
}

impl Instance {
    #[allow(dead_code)]
    pub fn as_string(&self) -> String {
        let labels_str = self
            .labels
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>()
            .join(", ");
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

// Test section

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_instance_from_json() {
        let json = json!({
            "name": "test-instance",
            "networkInterfaces": [
                {
                    "networkIP": "127.0.0.1"
                }
            ],
            "zone": "test-zone",
            "machineType": "test-machine-type",
            "cpuPlatform": "test-cpu-platform",
            "status": "test-status",
            "labels": {
                "key1": "value1",
                "key2": "value2"
            }
        });

        let instance = Instance::try_from(json).unwrap();

        assert_eq!(instance.name, "test-instance");
        assert_eq!(instance.ip, "127.0.0.1");
        assert_eq!(instance.zone, "test-zone");
        assert_eq!(instance.machine_type, "test-machine-type");
        assert_eq!(instance.cpu_platform, "test-cpu-platform");
        assert_eq!(instance.status, "test-status");
        assert_eq!(instance.labels, {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map
        });
    }
}
