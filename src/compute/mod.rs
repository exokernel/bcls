// This'll be a module for interacting with gcloud compute REST API
use reqwest::Client;
use serde_json::Value;

// A struct for our compute app service
pub struct Compute {
    pub project: String,
    client: reqwest::Client,
}

// A builder function for our compute app service
pub fn new_compute(project: String) -> Compute {
    Compute {
        project,
        client: Client::new(),
    }
}

impl Compute {
    //pub async fn list_zones(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    //    let url = format!(
    //        "https://compute.googleapis.com/compute/v1/projects/{}/zones",
    //        self.project
    //    );
    //    println!("url: {:?}", url);
    //    let resp = self
    //        .client
    //        .get(&url)
    //        .bearer_auth(&self.token)
    //        .send()
    //        .await?
    //        .json::<Value>()
    //        .await?;
    //    let zones = resp["items"]
    //        .as_array()
    //        .ok_or("No items in response")?
    //        .iter()
    //        .map(|x| x["name"].as_str().unwrap().to_string())
    //        .collect::<Vec<String>>();
    //
    //    Ok(zones)
    //}

    pub async fn list_instances(
        &self,
        instance_name: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let filter = format!("(name eq .*{}.*)", instance_name);
        let encoded_filter = urlencoding::encode(&filter);
        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/aggregated/instances?filter={}",
            self.project, encoded_filter
        );
        let token = get_token(&self.project)?;

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(resp)
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
