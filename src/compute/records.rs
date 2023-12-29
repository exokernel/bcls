use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn from_json(json: JsonValue) -> Option<Instance> {
        let name = json["name"].as_str()?.to_owned();
        let ip = json["networkInterfaces"][0]["networkIP"]
            .as_str()?
            .to_owned();
        let zone = json["zone"].as_str()?.to_owned();
        let machine_type = json["machineType"].as_str()?.to_owned();
        let cpu_platform = json["cpuPlatform"].as_str()?.to_owned();
        let status = json["status"].as_str()?.to_owned();
        let labels = json["labels"].to_string();

        Some(Instance {
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
