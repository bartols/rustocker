use bollard::image::ListImagesOptions;
use bollard::network::ListNetworksOptions;
use bollard::query_parameters::ListContainersOptions;
use bollard::volume::ListVolumesOptions;
use bollard::{API_DEFAULT_VERSION, Docker};
use std::collections::HashMap;

pub struct DockerClient {
    docker: Docker,
}

impl DockerClient {
    pub async fn new() -> Result<Self, bollard::errors::Error> {
        // Try to connect to Docker daemon
        let docker = Docker::connect_with_local_defaults()?;

        // Test the connection
        let _version = docker.version().await?;

        Ok(Self { docker })
    }

    pub async fn list_containers(&self) -> Result<Vec<String>, bollard::errors::Error> {
        let options = Some(ListContainersOptions {
            all: true,
            ..Default::default()
        });

        let containers = self.docker.list_containers(options).await?;

        Ok(containers
            .into_iter()
            .filter_map(|container| {
                // Get the first name (without the leading slash)
                container.names.and_then(|names| {
                    names
                        .into_iter()
                        .next()
                        .map(|name| name.trim_start_matches('/').to_string())
                })
            })
            .collect())
    }

    pub async fn list_images(&self) -> Result<Vec<String>, bollard::errors::Error> {
        let options = Some(ListImagesOptions::<String> {
            all: false,
            ..Default::default()
        });

        let images = self.docker.list_images(options).await?;

        Ok(images
            .into_iter()
            .filter_map(|image| {
                // Get repository tags or use image ID
                let tags = image.repo_tags;
                if !tags.is_empty() && tags[0] != "<none>:<none>" {
                    Some(tags[0].clone())
                } else {
                    // Use first 12 chars of image ID as fallback
                    let id = image.id.clone();
                    if id.len() > 12 {
                        Some(format!("{}...", &id[7..19])) // Skip "sha256:" prefix
                    } else {
                        Some(id)
                    }
                }
            })
            .collect())
    }

    pub async fn list_networks(&self) -> Result<Vec<String>, bollard::errors::Error> {
        let options = Some(ListNetworksOptions::<String> {
            ..Default::default()
        });

        let networks = self.docker.list_networks(options).await?;

        Ok(networks
            .into_iter()
            .filter_map(|network| network.name)
            .collect())
    }

    pub async fn list_volumes(&self) -> Result<Vec<String>, bollard::errors::Error> {
        let options = Some(ListVolumesOptions::<String> {
            ..Default::default()
        });

        let response = self.docker.list_volumes(options).await?;

        Ok(response
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(|volume| volume.name)
            .collect())
    }

    // Additional methods for container management
    pub async fn get_container_status(&self, name: &str) -> Result<String, bollard::errors::Error> {
        let options = Some(ListContainersOptions {
            all: true,
            filters: {
                let mut filters = HashMap::new();
                filters.insert("name".to_string(), vec![name.to_string()]);
                Some(filters)
            },
            ..Default::default()
        });

        let containers = self.docker.list_containers(options).await?;

        Ok(containers
            .first()
            .and_then(|container| container.status.clone())
            .unwrap_or_else(|| "Unknown".to_string()))
    }
}
