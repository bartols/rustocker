use bollard::models::ImageSummary;
use bollard::models::SystemVersion;
use bollard::query_parameters::{
    ListContainersOptions, ListImagesOptionsBuilder, ListNetworksOptionsBuilder,
    ListVolumesOptionsBuilder,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub id: String,               // Full ID per operazioni
    pub display_id: String,       // Troncato per display
    pub repo_tag: String,         // "nginx:latest" o "<none>:<none>"
    pub size_formatted: String,   // "142.3 MB"
    pub created_ago: String,      // "2d"
    pub containers_count: String, // "3" o "-"
}

#[derive(Debug, Clone)]
pub struct ImageInspectDetails {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub size_formatted: String,
    pub created_formatted: String,
    pub architecture: String,
    pub os: String,
    pub env: Vec<String>,
    pub exposed_ports: Vec<String>,
    pub working_dir: String,
    pub entrypoint: Vec<String>,
    pub cmd: Vec<String>,
    pub labels: HashMap<String, String>,
}

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

    pub async fn list_images(&self) -> Result<Vec<ImageInfo>, bollard::errors::Error> {
        let options = ListImagesOptionsBuilder::new().all(true).build();
        let images = self.docker.list_images(Some(options)).await?;

        Ok(images
            .into_iter()
            .map(|img| ImageInfo {
                id: img.id.clone(),
                display_id: Self::format_image_id(&img),
                repo_tag: Self::format_image_name(&img),
                size_formatted: Self::format_size(img.size),
                created_ago: Self::format_time_ago(img.created),
                containers_count: Self::format_containers_count(img.containers),
            })
            .collect())
    }

    pub async fn inspect_image(
        &self,
        image_id: &str,
    ) -> Result<ImageInspectDetails, bollard::errors::Error> {
        let inspect_result = self.docker.inspect_image(image_id).await?;

        // Format creation time
        let created_formatted = if let Some(created) = &inspect_result.created {
            use chrono::{DateTime, Utc};
            if let Ok(dt) = DateTime::parse_from_rfc3339(created) {
                dt.with_timezone(&Utc)
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string()
            } else {
                created.clone()
            }
        } else {
            "Unknown".to_string()
        };

        // Extract environment variables
        let env = if let Some(config) = &inspect_result.config {
            config.env.clone().unwrap_or_default()
        } else {
            Vec::new()
        };

        // Extract exposed ports
        let exposed_ports = if let Some(config) = &inspect_result.config {
            if let Some(exposed_ports) = &config.exposed_ports {
                exposed_ports.keys().cloned().collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Extract working directory
        let working_dir = if let Some(config) = &inspect_result.config {
            config.working_dir.clone().unwrap_or_default()
        } else {
            String::new()
        };

        // Extract entrypoint and cmd
        let entrypoint = if let Some(config) = &inspect_result.config {
            config.entrypoint.clone().unwrap_or_default()
        } else {
            Vec::new()
        };

        let cmd = if let Some(config) = &inspect_result.config {
            config.cmd.clone().unwrap_or_default()
        } else {
            Vec::new()
        };

        // Extract labels
        let labels = if let Some(config) = &inspect_result.config {
            config.labels.clone().unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(ImageInspectDetails {
            id: inspect_result.id.unwrap_or_default(),
            repo_tags: inspect_result.repo_tags.unwrap_or_default(),
            size_formatted: Self::format_size(inspect_result.size.unwrap_or(0)),
            created_formatted,
            architecture: inspect_result
                .architecture
                .unwrap_or_else(|| "Unknown".to_string()),
            os: inspect_result.os.unwrap_or_else(|| "Unknown".to_string()),
            env,
            exposed_ports,
            working_dir,
            entrypoint,
            cmd,
            labels,
        })
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

    // Helper methods for image operations
    pub fn format_image_name(image: &ImageSummary) -> String {
        if !image.repo_tags.is_empty() {
            image.repo_tags[0].clone()
        } else {
            "<none>:<none>".to_string()
        }
    }

    pub fn format_image_id(image: &ImageSummary) -> String {
        if image.id.len() > 12 {
            format!("{}...", &image.id[7..19]) // Skip "sha256:" prefix
        } else {
            image.id.clone()
        }
    }

    pub fn format_size(bytes: i64) -> String {
        if bytes < 0 {
            return "-".to_string();
        }

        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as i64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    pub fn format_time_ago(timestamp: i64) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let diff = now - timestamp;

        if diff < 60 {
            "< 1m".to_string()
        } else if diff < 3600 {
            format!("{}m", diff / 60)
        } else if diff < 86400 {
            format!("{}h", diff / 3600)
        } else if diff < 2592000 {
            format!("{}d", diff / 86400)
        } else if diff < 31536000 {
            format!("{}mo", diff / 2592000)
        } else {
            format!("{}y", diff / 31536000)
        }
    }

    pub fn format_containers_count(count: i64) -> String {
        if count >= 0 {
            count.to_string()
        } else {
            "-".to_string()
        }
    }
}
