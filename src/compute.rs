mod records;

// This'll be a module for interacting with gcloud compute REST API
// https://cloud.google.com/compute/docs/reference/rest/v1/instances/list

use crate::http;
pub use records::Instance; // rexport the Instance struct

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
#[cfg(any(not(test), rust_analyzer))]
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

#[cfg(test)]
fn get_token(project: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("fetching token for project: {:?}", project);
    // Return a mock token for tests
    Ok("mock_token".to_string())
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::MockHttpTrait;
    use mockall::predicate;
    use serde_json::json;

    #[test]
    fn test_list_zones() {
        let mut mock_http = MockHttpTrait::new();

        // Set up expectations
        let expected_token = "mock_token".to_string();
        let expected_result = vec!["zone1".to_string(), "zone2".to_string()];
        mock_http
            .expect_get()
            .with(
                predicate::eq(expected_token),
                predicate::eq(
                    "https://compute.googleapis.com/compute/v1/projects/test-project/zones",
                ),
            )
            .return_once(move |_, _| Ok(json!({"items": [{"name": "zone1"}, {"name": "zone2"}]})));

        // Create a Compute instance with the mock HttpTrait
        let c = Compute::new("test-project".to_string(), mock_http);
        let result = c.list_zones();
        let result = result.unwrap();

        // Assert that the function returned the expected result
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_list_instances() {
        let mut mock_http = MockHttpTrait::new();

        // Set up expectations
        mock_http.expect_get().return_once(move |_, _| {
            Ok(json!({
                "items": {
                    "zone1": {
                        "instances": [
                            {
                                "name": "instance1",
                                "networkInterfaces": [
                                    {
                                        "networkIP": "127.0.0.1",
                                    },
                                ],
                                "zone": "zone1",
                                "machineType": "machine-type1",
                                "cpuPlatform": "cpu-platform1",
                                "status": "status1",
                                "labels": "labels1",
                            },
                            {
                                "name": "instance2",
                                "networkInterfaces": [
                                    {
                                        "networkIP": "127.0.0.2",
                                    },
                                ],
                                "zone": "zone1",
                                "machineType": "machine-type2",
                                "cpuPlatform": "cpu-platform2",
                                "status": "status2",
                                "labels": "labels2",
                            },
                        ],
                    },
                    "zone2": {
                        "instances": [
                            {
                                "name": "instance3",
                                "networkInterfaces": [
                                    {
                                        "networkIP": "127.0.0.3",
                                    },
                                ],
                                "zone": "zone2",
                                "machineType": "machine-type3",
                                "cpuPlatform": "cpu-platform3",
                                "status": "status3",
                                "labels": "labels3",
                            },
                        ],
                    },
                },
            }))
        });

        // Create a Compute instance with the mock HttpTrait
        let c = Compute::new("test-project".to_string(), mock_http);
        let result = c.list_instances("instance");
        let result = result.unwrap();

        // Assert that the function returned the expected result
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "instance1");
        assert_eq!(result[1].name, "instance2");
        assert_eq!(result[2].name, "instance3");
    }
}
