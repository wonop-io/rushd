use crate::builder::Artefact;
use crate::builder::BuildContext;
use crate::builder::Config;
use crate::builder::{BuildScript, BuildType};
use crate::container::{ServiceSpec, ServicesSpec};
use crate::ToolchainContext;
use core::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ComponentBuildSpec {
    pub build_type: BuildType,
    pub product_name: String,
    pub component_name: String,
    pub color: String,

    pub build: Option<String>,
    pub watch_path: Option<String>,
    pub mount_point: Option<String>,
    pub subdomain: Option<String>,
    pub artefacts: Option<std::collections::HashMap<String, String>>,
    pub artefact_output_dir: String,
    pub docker_extra_run_args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
    pub volumes: Option<HashMap<String, String>>,
    pub port: Option<u16>,
    pub target_port: Option<u16>,
    pub k8s: Option<String>, // TODO: Refactor to k8s_dir
    pub priority: u64,


    // Set after loading
    pub config: Arc<Config>,
    pub services: Option<Arc<ServicesSpec>>,
    pub tagged_image_name: Option<String>,
}

impl ComponentBuildSpec {
    pub fn set_services(&mut self, services: Arc<ServicesSpec>) {
        self.services = Some(services);
    }

    pub fn set_tagged_image_name(&mut self, tagged_image_name: String) {
        self.tagged_image_name = Some(tagged_image_name);
    }

    pub fn config(&self) -> Arc<Config> {
        self.config.clone()
    }

    pub fn from_yaml(config: Arc<Config>, yaml_section: &serde_yaml::Value) -> Self {
        let product_name = config.product_name();
        let build_type = match yaml_section
            .get("build_type")
            .expect("build_type is required")
            .as_str()
            .unwrap()
        {
            "TrunkWasm" => BuildType::TrunkWasm {
                location: yaml_section
                    .get("location")
                    .expect("location is required for TrunkWasm")
                    .as_str()
                    .unwrap()
                    .to_string(),
                dockerfile_path: yaml_section
                    .get("dockerfile")
                    .expect("dockerfile_path is required")
                    .as_str()
                    .unwrap()
                    .to_string(),
            },
            "RustBinary" => BuildType::RustBinary {
                location: yaml_section
                    .get("location")
                    .expect("location is required for RustBinary")
                    .as_str()
                    .unwrap()
                    .to_string(),
                dockerfile_path: yaml_section
                    .get("dockerfile")
                    .expect("dockerfile_path is required")
                    .as_str()
                    .unwrap()
                    .to_string(),
            },
            "Script" => BuildType::Script {
                location: yaml_section
                    .get("location")
                    .expect("location is required for Script")
                    .as_str()
                    .unwrap()
                    .to_string(),
                dockerfile_path: yaml_section
                    .get("dockerfile")
                    .expect("dockerfile_path is required")
                    .as_str()
                    .unwrap()
                    .to_string(),
            },
            "Ingress" => BuildType::Ingress {
                components: yaml_section
                    .get("components")
                    .expect("components are required for Ingress")
                    .as_sequence()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect(),
                dockerfile_path: yaml_section
                    .get("dockerfile")
                    .expect("dockerfile_path is required")
                    .as_str()
                    .unwrap()
                    .to_string(),
            },
            "Image" => BuildType::PureDockerImage {
                image_name_with_tag: yaml_section
                    .get("image")
                    .expect("image is required for PureDockerImage")
                    .as_str()
                    .unwrap()
                    .to_string(),
                command: yaml_section.get("command").map(|v| v.as_str().unwrap().to_string()),
                entrypoint: yaml_section.get("entrypoint").map(|v| v.as_str().unwrap().to_string()),
            },
            "K8sOnly" => BuildType::PureKubernetes,
            "K8sInstall" => BuildType::KubernetesInstallation {
                namespace: yaml_section
                    .get("namespace")
                    .expect("namespace is required for KubernetesInstallation")
                    .as_str()
                    .unwrap()
                    .to_string(),
            },
            "ApiDocumentation" => BuildType::ApiDocumentation {
                component: yaml_section
                    .get("component")
                    .expect("component is required for ApiDocumentation")
                    .as_str()
                    .unwrap()
                    .to_string(),
                open_api: yaml_section
                    .get("open_api")
                    .expect("open_api is required for ApiDocumentation")
                    .as_str()
                    .unwrap()
                    .to_string(),
            },
            _ => panic!("Invalid build_type"),
        };

        let cwd = std::env::current_dir()
            .expect("Failed to get current working directory")
            .to_str()
            .unwrap()
            .to_string();

        ComponentBuildSpec {
            build_type,
            build: yaml_section
                .get("build")
                .map(|v| v.as_str().unwrap().to_string()),

            watch_path: yaml_section
                .get("watch")
                .map(|v| v.as_str().unwrap().to_string()),
            color: yaml_section
                .get("color")
                .map_or("blue".to_string(), |v| v.as_str().unwrap().to_string()),
            product_name: product_name.to_string(),
            component_name: yaml_section
                .get("component_name")
                .expect("component_name is required")
                .as_str()
                .unwrap()
                .to_string(),
            mount_point: yaml_section
                .get("mount_point")
                .map(|v| v.as_str().unwrap().to_string()),
            subdomain: yaml_section
                .get("subdomain")
                .map(|v| v.as_str().unwrap().to_string()),
            artefacts: yaml_section.get("artefacts").map(|v| {
                v.as_mapping()
                    .unwrap()
                    .iter()
                    .map(|(k, val)| {
                        (
                            k.as_str().unwrap().to_string(),
                            val.as_str().unwrap().to_string(),
                        )
                    })
                    .collect()
            }),
            artefact_output_dir: yaml_section
                .get("artefact_output_dir")
                .map_or("target/rushd".to_string(), |v| {
                    v.as_str().unwrap().to_string()
                }),
            docker_extra_run_args: yaml_section
                .get("docker_extra_run_args")
                .map_or_else(|| Vec::new(), |v| {
                    v.as_sequence()
                     .unwrap()
                     .iter()
                     .map(|item| item.as_str().unwrap().to_string())
                     .collect()
                }),                
            env: yaml_section.get("env").map(|v| {
                v.as_mapping()
                    .unwrap()
                    .iter()
                    .map(|(k, val)| {
                        (
                            k.as_str().unwrap().to_string(),
                            val.as_str().unwrap().to_string(),
                        )
                    })
                    .collect()
            }),
            volumes: yaml_section.get("volumes").map(|v| {
                v.as_mapping()
                    .unwrap()
                    .iter()
                    .map(|(k, val)| {
                        let absolute_path = std::path::Path::new(&cwd)
                            .join(k.as_str().unwrap())
                            .to_str()
                            .unwrap()
                            .to_string();
                        (absolute_path, val.as_str().unwrap().to_string())
                    })
                    .collect()
            }),
            port: yaml_section.get("port").map(|v| v.as_u64().unwrap() as u16),
            target_port: yaml_section
                .get("target_port")
                .map(|v| v.as_u64().unwrap() as u16),
            k8s: yaml_section
                .get("k8s")
                .map(|v| v.as_str().unwrap().to_string()),
            priority: yaml_section
                .get("priority")
                .map_or(100, |v| v.as_u64().unwrap()),
            config,
            services: None,
            tagged_image_name: None,
        }
    }

    pub fn build_script(&self, ctx: &BuildContext) -> String {
        match &self.build {
            Some(build) => build.clone(),
            None => BuildScript::new(self.build_type.clone()).render(ctx),
        }
    }

    pub fn build_artefacts(&self) -> HashMap<String, Artefact> {
        let mut ret = HashMap::new();
        match &self.artefacts {
            Some(artefacts) => {
                for (k, v) in artefacts.into_iter() {
                    let artefact = Artefact::new(k.to_string(), v.to_string());
                    // let script = artefact.render(ctx);
                    ret.insert(k.to_string(), artefact);
                }
            }
            None => {}
        }
        ret
    }

    pub fn generate_build_context(&self, toolchain: Option<Arc<ToolchainContext>>) -> BuildContext {
        let services = self
            .services
            .clone()
            .expect("No services found for docker image");
        let (location, services) = match &self.build_type {
            BuildType::TrunkWasm { location, .. } => (Some(location.clone()), None),
            BuildType::RustBinary { location, .. } => (Some(location.clone()), None),
            BuildType::Script { location, .. } => (Some(location.clone()), None),
            BuildType::Ingress { components, .. } => {
                let services = services
                    .iter()
                    .filter(|(k, _)| components.contains(k))
                    .map(|(k, v)| (k.clone(), v.clone()));
                let services: HashMap<String, ServiceSpec> = services.into_iter().collect();
                (None, Some(services))
            }
            BuildType::PureDockerImage { .. } => (None, None),
            BuildType::ApiDocumentation { .. } => (None, None),
            BuildType::PureKubernetes => (None, None),
            BuildType::KubernetesInstallation { .. } => (None, None),
        };
        let toolchain = toolchain.clone().expect("No toolchain available");

        let product_name = self.product_name.clone();
        let product_uri = slug::slugify(&product_name);

        BuildContext {
            toolchain: (*toolchain).clone(),
            build_type: self.build_type.clone(),
            location: location,
            target: toolchain.target().clone(),
            host: toolchain.host().clone(),
            rust_target: toolchain.target().to_rust_target(),
            services: services.unwrap_or_default(),
            environment: self.config.environment().to_string(),
            domain: self.config.domain().to_string(),
            product_name: product_name,
            product_uri: product_uri,
            component: self.component_name.clone(),
            docker_registry: self.config.docker_registry().to_string(),
            image_name: self.tagged_image_name.clone().unwrap_or_default(),
        }
    }
}
