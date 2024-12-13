mod records;

// This'll be a module for interacting with gcloud compute REST API
// https://cloud.google.com/compute/docs/reference/rest/v1/instances/list

use std::vec;

use crate::http;
use serde_json::{Map, Value};

pub use records::Instance;

/// A trait for fetching the token
pub trait TokenSource {
    fn get_token(&self, project: &str) -> Result<String, Box<dyn std::error::Error>>;
}

/// A struct for fetching the token from gcloud
pub struct GcloudTokenSource;

impl TokenSource for GcloudTokenSource {
    fn get_token(&self, project: &str) -> Result<String, Box<dyn std::error::Error>> {
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

        if output.status.success() {
            let token = String::from_utf8(output.stdout)?.trim().to_string();
            Ok(token)
        } else {
            let err = String::from_utf8(output.stderr)?;
            Err(err.into())
        }
    }
}

// Mock token source for testing
pub struct MockTokenSource {
    mock_token: String,
}

impl TokenSource for MockTokenSource {
    fn get_token(&self, _project: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.mock_token.clone())
    }
}

/// A struct for our compute app service
/// It has a client field that conforms to the HttpTrait
pub struct Compute<H: http::HttpTrait, T: TokenSource> {
    project: String,
    client: H,
    token_source: T,
}

impl<H: http::HttpTrait, T: TokenSource> Compute<H, T> {
    // A builder function for our compute app service
    pub fn new(project: String, client: H, token_source: T) -> Self {
        Self {
            project,
            client,
            token_source,
        }
    }

    #[allow(dead_code)]
    pub fn list_zones(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones",
            self.project
        );

        println!("url: {:?}", url);
        let token = self.token_source.get_token(&self.project)?;
        let resp = self.client.get(&token, &url)?;
        let zones = resp["items"]
            .as_array()
            .ok_or("No items in response")?
            .iter()
            .map(|item| item["name"].as_str().unwrap().to_string())
            .collect::<Vec<String>>();

        Ok(zones)
    }

    /// Calls the gcloud compute API and returns a Result containing a vector of instances or an error
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

        let token = self.token_source.get_token(&self.project)?;
        let resp = self.client.get(&token, &url)?;
        let stuff = resp["items"].as_object().ok_or("No items in response")?;

        let mut error = false;
        let mut instance_list = vec![];
        for (_, value) in stuff.iter() {
            let object: &Map<String, Value> = value.as_object().unwrap();
            let instance_result_list = object_to_instance_list(object);
            for result in instance_result_list {
                match result {
                    Ok(instance) => {
                        instance_list.push(instance);
                    }
                    Err(e) => {
                        println!("error: {:?}", e);
                        error = true;
                    }
                };
            }
        }

        match error {
            true => {
                let err: Box<dyn std::error::Error> = From::from("error parsing instances");
                Err(err)
            }
            false => Ok(instance_list),
        }
    }
}

// Helper functions

fn object_to_instance_list(
    object: &Map<String, Value>,
) -> Vec<Result<Instance, Box<dyn std::error::Error>>> {
    object
        .get("instances")
        .and_then(|value| value.as_array())
        .unwrap_or(&vec![])
        .iter()
        .map(|instance| Instance::try_from(instance.clone()))
        .collect()
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
                predicate::eq(expected_token.clone()),
                predicate::eq(
                    "https://compute.googleapis.com/compute/v1/projects/test-project/zones",
                ),
            )
            .return_once(move |_, _| Ok(json!({"items": [{"name": "zone1"}, {"name": "zone2"}]})));

        // Create a Compute instance with the mock HttpTrait
        let c = Compute::new(
            "test-project".to_string(),
            mock_http,
            MockTokenSource {
                mock_token: expected_token,
            },
        );
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
                                "labels": {
                                    "key1": "value1",
                                    "key2": "value2",
                                },
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
                                "labels": {
                                    "key3": "value3",
                                    "key4": "value4",
                                },
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
                                "labels": {
                                    "key5": "value5",
                                    "key6": "value6",
                                },
                            },
                        ],
                    },
                },
            }))
        });

        // Create a Compute instance with the mock HttpTrait
        let c = Compute::new(
            "test-project".to_string(),
            mock_http,
            MockTokenSource {
                mock_token: "mock_token".to_string(),
            },
        );
        let result = c.list_instances("instance");
        let result = result.unwrap();

        // Assert that the function returned the expected result
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "instance1");
        assert_eq!(result[1].name, "instance2");
        assert_eq!(result[2].name, "instance3");
    }
}
