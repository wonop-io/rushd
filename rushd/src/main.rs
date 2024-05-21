mod container;
mod utils;
mod toolchain;
mod cluster;
mod builder;
mod gitignore;

use crate::toolchain::Platform;
use clap::{arg, Command,Arg};
use tokio::io;
use std::{path::Path, sync::Arc};
use crate::container::ContainerReactor;
use crate::utils::Directory;
use crate::toolchain::ToolchainContext;
use cluster::Minikube;
use colored::Colorize;
use crate::builder::Config;
use std::env;
use std::fs::File;
use std::io::Read;
use serde::Deserialize;
use std::collections::HashMap;


fn setup_environment() {
    
    // Set the RUSHD_ROOT environment variable
    let binding = env::current_dir().unwrap();
    let rushd_root = binding
        .ancestors()
        .find(|dir| dir.join(".git").exists())
        .expect("Unable to find git repository amounts ancestors");
    env::set_var("RUSHD_ROOT", rushd_root);

    // Set the HOME environment variable if not already set
    if env::var("HOME").is_err() {
        if let Some(home) = env::var_os("USERPROFILE") {
            env::set_var("HOME", home);
        } else {
            panic!("The HOME environment variable is not set.");
        }
    }
    // Set the PATH environment variable
    let home_dir = env::var_os("HOME").unwrap();
    let cargo_bin = Path::new(&home_dir).join(".cargo/bin");
    let current_path = env::var_os("PATH").unwrap();
    // let new_path = env::join_paths([current_path, cargo_bin.into()].iter()).unwrap();
    // env::set_var("PATH", new_path);

    // Set toolchain environment variables for macOS ARM architecture
    if cfg!(target_os = "macos") && cfg!(target_arch = "arm") {
        // TODO: Toolpath should be searched for
        let toolchain_path = "/opt/homebrew/Cellar/x86_64-unknown-linux-gnu/7.2.0/bin/";
        env::set_var("CC", format!("{}x86_64-unknown-linux-gnu-gcc", toolchain_path));
        env::set_var("CXX", format!("{}x86_64-unknown-linux-gnu-g++", toolchain_path));
        env::set_var("AR", format!("{}x86_64-unknown-linux-gnu-ar", toolchain_path));
        env::set_var("RANLIB", format!("{}x86_64-unknown-linux-gnu-ranlib", toolchain_path));
        env::set_var("NM", format!("{}x86_64-unknown-linux-gnu-nm", toolchain_path));
        env::set_var("STRIP", format!("{}x86_64-unknown-linux-gnu-strip", toolchain_path));
        env::set_var("OBJDUMP", format!("{}x86_64-unknown-linux-gnu-objdump", toolchain_path));
        env::set_var("OBJCOPY", format!("{}x86_64-unknown-linux-gnu-objcopy", toolchain_path));
        env::set_var("LD", format!("{}x86_64-unknown-linux-gnu-ld", toolchain_path));
    }

}



#[derive(Debug, Deserialize)]
struct RushdConfig {
    env: HashMap<String, String>,
}

fn load_config() {
    let config_path = "rushd.yaml";
    let mut file = File::open(config_path).expect("Unable to open the config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read the config file");

    let config: RushdConfig = serde_yaml::from_str(&contents).expect("Error parsing the config file");

    for (key, value) in config.env {
        std::env::set_var(key, value);
    }
}





#[tokio::main]
async fn main() -> io::Result<()> {
    setup_environment();

    // TODO: Get the rushd root by go levels up until you find ".git" directory
    let root_dir = std::env::var("RUSHD_ROOT").unwrap();
    let _guard = Directory::chdir(&root_dir);
    load_config();

    dotenv::dotenv().ok();    
    
    let matches = Command::new("rushd")
        .version("0.1.0")
        .author("Your Name <your_email@example.com>")
        .about("Rush is designed as an all-around support unit for developers, transforming the development workflow with its versatile capabilities. It offers a suite of tools for building, deploying, and managing applications, adapting to the diverse needs of projects with ease.")
        .arg(arg!(target_arch : --arch <TARGET_ARCH> "Target architecture"))
        .arg(arg!(target_os : --os <TARGET_OS> "Target OS"))
        .arg(arg!(environment : --env <ENVIRONMENT> "Environment"))
        .arg(arg!(docker_registry : --registry <DOCKER_REGISTRY> "Docker Registry"))
        .arg(Arg::new("product_name").required(true))
        .subcommand(Command::new("describe")
            .about("Describes the current configuration")
            .subcommand(Command::new("toolchain")
                .about("Describes the current toolchain")
            )
            .subcommand(Command::new("images")
                .about("Describes the current images")
            )            
            .subcommand(Command::new("services")
                .about("Describes the current services")
            )
            .subcommand(Command::new("build-script")
                .about("Describes the current build script")
                .arg(Arg::new("component_name").required(true))
            )            
            .subcommand(Command::new("build-context")
                .about("Describes the current build context")
                .arg(Arg::new("component_name").required(true))
            )
            .subcommand(Command::new("artefacts")
                .about("Describes the current artefacts")
                .arg(Arg::new("component_name").required(true))
            )            
            .subcommand(Command::new("k8s")
                .about("Describes the current k8s")
            )                        
        )
        .subcommand(Command::new("dev"))
        .subcommand(Command::new("build"))
        .subcommand(Command::new("push"))
        .subcommand(Command::new("minikube")
            .about("Runs tasks on minikube")
            .subcommand(Command::new("dev"))
            .subcommand(Command::new("start"))
            .subcommand(Command::new("stop"))
            .subcommand(Command::new("delete"))
        )
        .subcommand(Command::new("rollout")
            .about("Rolls out the product into staging or production")
        )
        .subcommand(Command::new("deploy"))
        .subcommand(Command::new("install"))
        .subcommand(Command::new("uninstall"))
        .subcommand(Command::new("apply"))
        .subcommand(Command::new("unapply"))

        .get_matches();


    let target_arch = if let Some(target_arch) = matches.get_one::<String>("target_arch") {
        target_arch.clone()
    } else {
        "x86_64".to_string()
    };

    let target_os = if let Some(target_os) = matches.get_one::<String>("target_os") {
        target_os.clone()
    } else {
        "linux".to_string()
    };

    let environment = if let Some(environment) = matches.get_one::<String>("environment") {
        environment.clone()
    } else {
        "dev".to_string()
    };

    


    let docker_registry = if let Some(docker_registry) = matches.get_one::<String>("docker_registry") {
        docker_registry.clone()
    } else {
        std::env::var("DOCKER_REGISTRY").expect("DOCKER_REGISTRY environment variable not found")
    };


    let product_name = matches.get_one::<String>("product_name").unwrap();

    let config = match Config::new(&root_dir, product_name, &environment, &docker_registry) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let toolchain = Arc::new(ToolchainContext::new(Platform::default(), Platform::new(&target_os, &target_arch)));
    toolchain.setup_env();

    
    let mut reactor = match ContainerReactor::from_product_dir(config.clone(), toolchain.clone()) {
        Ok(reactor) => reactor,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };   

    let minikube = Minikube::new(toolchain.clone());     

    if let Some(matches) = matches.subcommand_matches("describe") {
        if let Some(_) = matches.subcommand_matches("toolchain") {
            println!("{:#?}",toolchain);
            std::process::exit(0);
        }

        if let Some(_) = matches.subcommand_matches("images") {
            println!("{:#?}", reactor.images());
            std::process::exit(0);
        }

        if let Some(_) = matches.subcommand_matches("services") {
            println!("{:#?}", reactor.services());
            std::process::exit(0);
        }



        if let Some(_) = matches.subcommand_matches("build-script") {
            let component_name = matches.get_one::<String>("component_name").unwrap();
            let image = reactor.get_image(component_name).expect("Component not found");
            let ctx = image.generate_build_context();

            println!("{}", image.build_script(&ctx).unwrap());
            std::process::exit(0);
        }

        if let Some(_) = matches.subcommand_matches("build-context") {
            let component_name = matches.get_one::<String>("component_name").unwrap();
            let image = reactor.get_image(component_name).expect("Component not found");
            let ctx = image.generate_build_context();
            println!("{:#?}", ctx);
            std::process::exit(0);
        }

        if let Some(_) = matches.subcommand_matches("artefacts") {
            let _pop_dir = Directory::chdir(reactor.product_directory());
            let component_name = matches.get_one::<String>("component_name").unwrap();
            let image = reactor.get_image(component_name).expect("Component not found");
            let ctx = image.generate_build_context();
            for (k,v) in image.spec().build_artefacts() {
                let message = format!("{} {}", "Artefact".green(), k.white());
                println!("{}\n",&message.bold());
                
                println!("{}\n", v.render(&ctx));
            }
            std::process::exit(0);
        }

        if let Some(_) = matches.subcommand_matches("k8s") {
            let manifests = reactor.cluster_manifests();
            for component in manifests.components() {
                println!("{} -> {}", component.input_directory().display(), component.output_directory().display());                
                let spec = component.spec();
                let ctx = spec.generate_build_context(Some(toolchain.clone()));
                for manifest in component.manifests() {
                    println!("{}", manifest.render(&ctx));
                }
                println!("");
            }
            std::process::exit(0);
        }

    }

    // Setting the context
    match reactor.select_kubernetes_context(config.kube_context()).await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    if let Some(matches) = matches.subcommand_matches("minikube") {
        if let Some(_) = matches.subcommand_matches("start") {
            match minikube.start().await {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        if let Some(_) = matches.subcommand_matches("stop") {
            match minikube.stop().await {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }        
        if let Some(_) = matches.subcommand_matches("delete") {
            match minikube.delete().await {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }               
    }

    if let Some(_) = matches.subcommand_matches("dev") {        
        match reactor.launch().await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    
    if let Some(_) = matches.subcommand_matches("build") {
        match reactor.build().await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(_) = matches.subcommand_matches("push") {
        match reactor.build_and_push().await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }


    if let Some(_) = matches.subcommand_matches("rollout") {
        match reactor.rollout().await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }


    // TODO: Check if the context is set correctly

    if let Some(_) = matches.subcommand_matches("install") {
        match reactor.install_manifests().await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
    
    if let Some(_) = matches.subcommand_matches("uninstall") {
        match reactor.uninstall_manifests().await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
    

    if let Some(_) = matches.subcommand_matches("deploy") {
        match reactor.deploy().await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(_) = matches.subcommand_matches("apply") {
        match reactor.apply().await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(_) = matches.subcommand_matches("unapply") {
        match reactor.unapply().await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }


    Ok(())
}


