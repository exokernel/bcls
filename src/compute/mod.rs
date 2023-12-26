// This'll be a module for interacting with gcloud compute REST API
use reqwest::Client;
use serde_json::Value;

// A struct for our compute app service
pub struct Compute {
    pub project: String,
    pub token: String,
    client: reqwest::Client,
}

// A builder function for our compute app service
pub fn new_compute(project: String, token: String) -> Compute {
    Compute {
        project,
        token,
        client: Client::new(),
    }
}

impl Compute {
    pub async fn list_zones(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones",
            self.project
        );
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .json::<Value>()
            .await?;
        let zones = resp["items"]
            .as_array()
            .ok_or("No items in response")?
            .iter()
            .map(|x| x["name"].as_str().unwrap().to_string())
            .collect::<Vec<String>>();

        Ok(zones)
    }
}

// Use reqwest to list all the instances that match a filter
// GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/instances
//
