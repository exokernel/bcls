#![allow(clippy::module_inception)]

// This'll be a module for interacting with gcloud compute REST API
// https://cloud.google.com/compute/docs/reference/rest/v1/instances/list
use super::records;
use crate::http;

// A struct for our compute app service
// It has a client field that conforms to the HttpTrait
pub struct Compute<T: http::HttpTrait> {
    project: String,
    client: T,
}

impl<T: http::HttpTrait> Compute<T> {
    // A builder function for our compute app service
    pub fn new(project: String, client: T) -> Compute<T> {
        Compute { project, client }
    }

    #[allow(dead_code)]
    pub fn list_zones(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones",
            self.project
        );

        println!("url: {:?}", url);
        let token = get_token(&self.project)?;
        let resp = self.client.get(&token, &url)?;
        let zones = resp["items"]
            .as_array()
            .ok_or("No items in response")?
            .iter()
            .map(|item| item["name"].as_str().unwrap().to_string())
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
        let resp = self.client.get(&token, &url)?;

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

// Tests

#[cfg(test)]
mod tests {

    use super::*;
    use mockall::*;
    use serde_json::json;
    use crate::http::http::MockHttpTrait;

    #[test]
    fn test_list_zones() {
        let mut mock_http = MockHttpTrait::new();


        // Set up expectations
        let expected_token = "test_token";
        let expected_url = "http://example.com";
        let expected_result = vec!["zone1".to_string(), "zone2".to_string()];
        mock_http.expect_get()
            .with(predicate::eq(expected_token), predicate::eq(expected_url))
            .return_once(move |_, _| Ok(json!({"items": [{"name": "zone1"}, {"name": "zone2"}]})));

        // Create a Compute instance with the mock HttpTrait
        let c = Compute::new("test_project".to_string(), mock_http);
        let result = c.list_zones();
        let result = result.unwrap();

        // Assert that the function returned the expected result
        assert_eq!(result, expected_result);
    }
}
