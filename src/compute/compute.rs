#![allow(clippy::module_inception)]

// This'll be a module for interacting with gcloud compute REST API
// https://cloud.google.com/compute/docs/reference/rest/v1/instances/list
use super::records;
use reqwest::blocking::Client as ReqwestClient;
use serde_json::Value as JsonValue;

// A struct for our compute app service
pub struct Compute {
    pub project: String,
    client: ReqwestClient,
}

// A builder function for our compute app service
pub fn new_compute(project: String) -> Compute {
    Compute {
        project,
        client: ReqwestClient::new(),
    }
}

impl Compute {
    #[allow(dead_code)]
    pub async fn list_zones(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones",
            self.project
        );

        let token = get_token(&self.project)?;

        println!("url: {:?}", url);
        let resp = self
            .client
            .get(url)
            .bearer_auth(token)
            .send()?
            .json::<JsonValue>()?;
        let zones = resp["items"]
            .as_array()
            .ok_or("No items in response")?
            .iter()
            .map(|x| x["name"].as_str().unwrap().to_string())
            .collect::<Vec<String>>();

        Ok(zones)
    }

    pub fn list_instances(
        &self,
        instance_name: &str,
    ) -> Result<Vec<records::Instance>, Box<dyn std::error::Error>> {
        let filter = format!("(name eq .*{}.*)", instance_name);
        let encoded_filter = urlencoding::encode(&filter);
        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/aggregated/instances?filter={}",
            self.project, encoded_filter
        );
        let token = get_token(&self.project)?;

        let resp = self
            .client
            .get(url)
            .bearer_auth(token)
            .send()?
            .json::<JsonValue>()?;

        // Get the instances from the JSON response and convert them to a Vec<Instance>
        let instances = resp["items"]
            .as_object()
            .ok_or("No items in response")?
            .iter()
            .flat_map(|(_, zone_data)| {
                zone_data["instances"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|inst| records::Instance::from_json(inst.clone()))
                    .collect::<Vec<Result<records::Instance, Box<dyn std::error::Error>>>>()
            })
            .flatten()
            .collect::<Vec<records::Instance>>();

        Ok(instances)
    }
}

// Helper functions

// Run `gcloud auth application-default print-access-token --project=PROJECT` for the given project
// and return the token
fn get_token(project: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("fetching token for project: {:?}", project);
    let output = std::process::Command::new("gcloud")
        .args([
            "auth",
            "application-default",
            "print-access-token",
            "--project",
            project,
        ])
        .output()?;
    let token = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(token)
}
