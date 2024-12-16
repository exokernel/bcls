//! This module provides an interface for interacting with the Google Compute Engine API.
//! It defines the `Compute` struct for making API calls and related helper functions.

mod records;

use std::vec;

use crate::http;
use serde_json::{Map, Value};

pub use records::Instance;

/// A trait for fetching authentication tokens.
pub trait TokenSource {
    /// Retrieves an authentication token.
    ///
    /// # Arguments
    ///
    /// * `project` - The ID of the Google Cloud project.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The authentication token on success.
    /// * `Err(Box<dyn std::error::Error>)` - An error if token retrieval fails.
    fn get_token(&self, project: &str) -> Result<String, Box<dyn std::error::Error>>;
}

/// Retrieves authentication tokens using the `gcloud` command-line tool.
pub struct GcloudTokenSource;

impl TokenSource for GcloudTokenSource {
    /// Executes the `gcloud` command to obtain an access token.
    ///
    /// # Arguments
    ///
    /// * `project` - The Google Cloud project ID.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The access token on success.
    /// * `Err(Box<dyn std::error::Error>)` - An error if the `gcloud` command fails
    ///   or if there's an issue processing the output.
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

/// A mock token source for testing purposes.
pub struct MockTokenSource {
    /// The mock token to return.
    mock_token: String,
}

impl TokenSource for MockTokenSource {
    /// Returns the configured mock token.
    ///
    /// # Arguments
    ///
    /// * `_project` - The project ID (ignored in this mock implementation).
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The mock token.
    fn get_token(&self, _project: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.mock_token.clone())
    }
}

/// An iterator that handles paginating through all the instances in a project.
/// Each call to `next` fetches a page of instances from the API as vectors of `Instance` structs.
struct InstancesPageIterator<'a, H: http::HttpTrait, T: TokenSource> {
    config: &'a ComputeConfig<H, T>,
    page_token: Option<String>,
    auth_token: String,
    finished: bool,
}

/// Implementation of the `InstanceIterator` struct.
impl<'a, H: http::HttpTrait, T: TokenSource> InstancesPageIterator<'a, H, T> {
    fn new(config: &'a ComputeConfig<H, T>, auth_token: String) -> Self {
        Self {
            config,
            page_token: None,
            auth_token,
            finished: false,
        }
    }
}

/// Implementation of the `Iterator` trait for `InstanceIterator`.
impl<H: http::HttpTrait, T: TokenSource> Iterator for InstancesPageIterator<'_, H, T> {
    type Item = Result<Vec<records::Instance>, Box<dyn std::error::Error>>;

    /// Fetches the next page of instances from the API.
    /// If there are no more pages, returns `None`.
    /// If an error occurs, returns an error result wrapped in `Some` and terminates the iteration by setting `finished`
    /// to `true`.
    /// If the next page is successfully fetched, returns a vector of instances wrapped in `Some`.
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // Construct the URL
        // <https://cloud.google.com/compute/docs/reference/rest/v1/instances/aggregatedList#http-request>
        let url = match &self.page_token {
            Some(token) => format!(
                "https://compute.googleapis.com/compute/v1/projects/{}/aggregated/instances?pageToken={}",
                self.config.project, token
            ),
            None => format!(
                "https://compute.googleapis.com/compute/v1/projects/{}/aggregated/instances",
                self.config.project
            ),
        };

        // Make the HTTP request
        let resp = match self.config.client.get(&self.auth_token, &url) {
            Ok(resp) => resp,
            Err(e) => return Some(Err(e)),
        };

        // Parse the response
        let json_response = match resp["items"].as_object() {
            Some(instances_json) => instances_json,
            None => return Some(Err("No items in response".into())),
        };

        // Convert the json response to a list of instance structs
        let mut error = false;
        let instance_list = json_response
            .iter() // Iterate over the zones
            // Convert the instances in each zone to a list of instances
            .flat_map(|(_, value)| {
                let object = value
                    .as_object()
                    .expect("Expected JSON object but got something else");
                object_to_instance_list(object)
            })
            // Filter out any errors that occurred during parsing
            // Print any errors and set the error flag to true replace the error with None which will filter it out
            .filter_map(|result| match result {
                Ok(instance) => Some(instance),
                Err(e) => {
                    println!("error: {:?}", e);
                    error = true;
                    None
                }
            })
            .collect::<Vec<_>>();

        // Check for errors
        if error {
            self.finished = true;
            return Some(Err("Error parsing instances".into()));
        }

        // Check for a next page token
        self.page_token = match resp["nextPageToken"].as_str() {
            Some(token) => Some(token.to_string()),
            None => {
                self.finished = true;
                None
            }
        };

        Some(Ok(instance_list))
    }
}

/// Configuration for the `Compute` service.
pub struct ComputeConfig<H: http::HttpTrait, T: TokenSource> {
    /// The Google Cloud project ID.
    pub project: String,
    /// The HTTP client implementation.
    pub client: H,
    /// The token source.
    pub token_source: T,
}

/// Provides an interface for interacting with the Google Compute Engine API.
pub struct Compute<H: http::HttpTrait, T: TokenSource> {
    /// The configuration for the `Compute` service.
    config: ComputeConfig<H, T>,
}

impl<H: http::HttpTrait, T: TokenSource> Compute<H, T> {
    /// Creates a new `Compute` instance.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the `Compute` service.
    ///
    /// # Returns
    ///
    /// * `Self` - A new `Compute` instance.
    pub fn new(config: ComputeConfig<H, T>) -> Self {
        Self { config }
    }

    /// Lists available zones in the project (currently unused).
    #[allow(dead_code)]
    pub fn list_zones(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones",
            self.config.project
        );

        println!("url: {:?}", url);
        let token = self.config.token_source.get_token(&self.config.project)?;
        let resp = self.config.client.get(&token, &url)?;
        let zones = resp["items"]
            .as_array()
            .ok_or("No items in response")?
            .iter()
            .map(|item| item["name"].as_str().unwrap().to_string())
            .collect::<Vec<String>>();

        Ok(zones)
    }

    /// Lists instances in the specified project
    /// # Returns
    ///
    /// * `Ok(Vec<Instance>)` - A vector of `Instance` structs representing the matching instances.
    /// * `Err(Box<dyn std::error::Error>)` - An error if the API call fails or if there's an
    ///   issue parsing the response.
    pub fn list_all_instances(&self) -> Result<Vec<records::Instance>, Box<dyn std::error::Error>> {
        // Fetch the auth token
        let auth_token = self.config.token_source.get_token(&self.config.project)?;

        // Create an iterator over the instances. This will handle pagination.
        let iter = InstancesPageIterator::new(&self.config, auth_token);

        // Collect the instances from the iterator returning either a vector of vectors of instances
        // or an error if one occurred during the iteration.
        let instances = iter.collect::<Result<Vec<_>, _>>()?;

        // Flatten the vector of vectors into a single vector iterator and collect it into a vector of instances.
        Ok(instances.into_iter().flatten().collect())
    }
}

/// Converts a JSON object representing a group of instances within a zone
/// into a vector of `Result<Instance, Box<dyn std::error::Error>>`.
///
/// This function takes a JSON object, extracts the "instances" array if present,
/// and attempts to convert each element of the array into an `Instance` struct.
/// Any errors encountered during the conversion are returned as part of the vector.
///
/// # Arguments
///
/// * `object` - The JSON object representing a group of instances within a zone.
///
/// # Returns
///
/// A vector of `Result<Instance, Box<dyn std::error::Error>>`. Each element
/// represents either a successfully parsed `Instance` or an error encountered
/// during parsing.
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
        let config = ComputeConfig {
            project: "test-project".to_string(),
            client: mock_http,
            token_source: MockTokenSource {
                mock_token: expected_token,
            },
        };
        let c = Compute::new(config);
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
        let config = ComputeConfig {
            project: "test-project".to_string(),
            client: mock_http,
            token_source: MockTokenSource {
                mock_token: "mock_token".to_string(),
            },
        };
        let c = Compute::new(config);
        let result = c.list_all_instances();
        let result = result.unwrap();

        // Assert that the function returned the expected result
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "instance1");
        assert_eq!(result[1].name, "instance2");
        assert_eq!(result[2].name, "instance3");
    }
}
