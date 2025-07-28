use bollard::models::ImageSummary;
use bollard::models::SystemVersion;
use bollard::query_parameters::{
    ListContainersOptions, ListImagesOptionsBuilder, ListNetworksOptionsBuilder,
    ListVolumesOptionsBuilder,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use std::collections::HashMap;

pub struct DockerClient {
    docker: Docker,
    pub version: SystemVersion,
}

impl DockerClient {
    pub async fn new() -> Result<Self, bollard::errors::Error> {
        // Try to connect to Docker daemon
        let docker = Docker::connect_with_local_defaults()?;

        // get version
        let version = docker.version().await?;

        println!("Connected to Docker version: {:?}", version);

        Ok(Self { docker, version })
    }

    pub async fn connect(host: &str, timeout: u64) -> Result<Self, bollard::errors::Error> {
        // Format the host as a proper URL
        let host_url = format!("tcp://{}", host);

        // Connect using HTTP (no SSL)
        let docker = Docker::connect_with_http(&host_url, timeout, bollard::API_DEFAULT_VERSION)?;

        // get version
        let version = docker.version().await?;

        Ok(Self { docker, version })
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

    pub async fn list_images(&self) -> Result<Vec<ImageSummary>, bollard::errors::Error> {
        let options = ListImagesOptionsBuilder::new().all(true).build();
        self.docker.list_images(Some(options)).await
    }

    pub async fn list_networks(&self) -> Result<Vec<String>, bollard::errors::Error> {
        let options = ListNetworksOptionsBuilder::new().build();

        let networks = self.docker.list_networks(Some(options)).await?;

        Ok(networks
            .into_iter()
            .filter_map(|network| network.name)
            .collect())
    }

    pub async fn list_volumes(&self) -> Result<Vec<String>, bollard::errors::Error> {
        let options = ListVolumesOptionsBuilder::new().build();

        let response = self.docker.list_volumes(Some(options)).await?;

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
