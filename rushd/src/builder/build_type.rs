use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BuildType {
    TrunkWasm {
        location: String,
        dockerfile_path: String,
    },
    RustBinary {
        location: String,
        dockerfile_path: String,
    },
    Script {
        location: String,
        dockerfile_path: String,
    },
    Ingress {
        components: Vec<String>,
        dockerfile_path: String,
    },
    PureDockerImage {
        image_name_with_tag: String,
    },
    PureKubernetes,
    KubernetesInstallation {
        namespace: String,
    },
    ApiDocumentation {
        component: String,
        open_api: String,
    },
}
