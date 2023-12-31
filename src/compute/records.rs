use serde_json::Value as JsonValue;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Instance {
    pub name: String,
    pub ip: String,
    pub zone: String,
    pub machine_type: String,
    pub cpu_platform: String,
    pub status: String,
    pub labels: String,
}

impl Instance {
    pub fn from_json(json: JsonValue) -> Result<Instance, Box<dyn Error>> {
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
            .to_string();
        let machine_type = json
            .get("machineType")
            .and_then(JsonValue::as_str)
            .ok_or("No machineType")?
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
            .and_then(JsonValue::as_str)
            .ok_or("No labels")?
            .to_owned();
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

    #[allow(dead_code)]
    pub fn as_string(&self) -> String {
        format!(
            "Name: {} IP: {} Zone: {} Machine Type: {} CPU Platform: {} Status: {} Labels: {}",
            self.name,
            self.ip,
            self.zone,
            self.machine_type,
            self.cpu_platform,
            self.status,
            self.labels
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
            "labels": "test-labels"
        });

        let instance = Instance::from_json(json).unwrap();
        assert_eq!(instance.name, "test-instance");
        assert_eq!(instance.ip, "127.0.0.1");
        assert_eq!(instance.zone, "test-zone");
        assert_eq!(instance.machine_type, "test-machine-type");
        assert_eq!(instance.cpu_platform, "test-cpu-platform");
        assert_eq!(instance.status, "test-status");
        assert_eq!(instance.labels, "test-labels");
    }
}
