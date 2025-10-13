use clap::{Parser};
use std::process;
use std::process::Command;

use docker_api::{opts::{ContainerStopOpts}, Docker, Result as DockerResult};
use std::io::{self, Write};
use std::path::Path;
use compose_rs::{Compose, ComposeCommand};

use std::env;
use Result;

/// BranchSpawn - A tool for managing git branches
#[derive(Parser)]
#[command(name = "branchspawn")]
#[command(author = "Owen Hope")]
#[command(version = "0.1.0")]
#[command(about = "A CLI tool for creating DB containers dedicated for your git branches", long_about = None)]
struct Cli {
    /// Optional name to operate on
    #[arg(short, long)]
    branch_name: Option<String>,

    #[arg(short, long)]
    compose_file_path: Option<String>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

#[cfg(unix)]
pub fn new_docker() -> DockerResult<Docker> {
    Ok(Docker::unix("/var/run/docker.sock"))
}

#[cfg(not(unix))]
pub fn new_docker() -> DockerResult<Docker> {
    Docker::new("tcp://127.0.0.1:8080")
}

fn get_current_branch() -> Result<String, String> {
    let output = Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;

    if !output.status.success() {
        return Err("Git command did not execute successfully".to_string());
    }

    let branch_name = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse git output: {}", e))?
        .trim()
        .to_string()
        .replace("/", "-");

    if branch_name.is_empty() {
        return Err("No current branch found".to_string());
    }

    Ok(branch_name)
}

async fn check_port_conflict(docker: &Docker, port: u16, container_name: &str) -> Result<(), String> {

    // List all containers with the specified port published
    let containers = docker
        .containers()
        .list(&Default::default())
        .await
        .map_err(|e| format!("Failed to list containers: {}", e))?;

    // Filter out our own container and collect conflicting container names
    let conflicting_containers: Vec<String> = containers
        .into_iter()
        .filter_map(|container| {
            if let Some(ports) = container.ports {
                for p in ports {
                    if p.public_port == Some(port) && container.names.as_ref().map_or(true,
                         |names| !names.contains(&format!("/{}", container_name))) {
                        return container.names.map(|names| names.join(", ").trim_start_matches('/').to_string());
                    }
                }
            }
            None
        })
        .collect();

    if !conflicting_containers.is_empty() {
        println!("Found other containers using port {}: {}", port, conflicting_containers.join(", "));
        
        print!("Do you want to stop these containers to avoid port conflicts? (y/N): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();
        
        if input == "y" || input == "yes" {
            println!("Stopping containers using port {}...", port);
            
            for container_name in &conflicting_containers {
                let stop_opts = ContainerStopOpts::builder().build();
                match docker.containers().get(container_name).stop(&stop_opts).await {
                    Ok(_) => println!("Stopped container: {}", container_name),
                    Err(e) => eprintln!("Failed to stop container {}: {}", container_name, e),
                }
            }
        } else {
            println!("Warning: Port conflict may prevent your database from starting correctly.");
        }
    }

    Ok(())
}

fn start_from_compose(_compose_file: &str) -> Result<(), String> {
    println!("Starting container from compose file: {}", _compose_file);

    let compose = Compose::builder()
        .path(_compose_file)
        .build()
        .map_err(|e| format!("Failed to build compose: {}", e));

    if let Err(e) = compose?.up().exec() {
        return Err(format!("Failed to start container: {}", e));
    }

    Ok(())
}

async fn check_container_exists(docker: &Docker, container_name: &str) -> Result<bool, String> {
        // List all containers with the specified port published
    let containers = docker
        .containers()
        .list(&Default::default())
        .await
        .map_err(|e| format!("Failed to list containers: {}", e))?;

    for container in containers {
        if let Some(names) = container.names {
            if names.contains(&format!("/{}", container_name)) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

async fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let docker = new_docker().expect("We should have a docker connection");

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    let clean_branch_name = match &cli.branch_name {
        Some(name) => {
            println!("Branch name provided: {}", name);
            name.replace("/", "-")
        }
        None => {
            println!("No branch name provided will pull from current branch");
            let current_branch = get_current_branch()?;
            println!("Current branch is: {}", current_branch);
            current_branch
        }
    };

        let compose_file = match &cli.compose_file_path {
        Some(path) => {
            println!("Using custom compose file path: {}", path);
            if !Path::new(path).exists() {
                return Err(format!("Compose file does not exist at path: {}", path));
            }
            path
        }
        None => {
            let default_path = "compose.yml";
            println!("No compose file path provided, using default: {}", default_path);
            default_path
        }
    };

    let container_name = format!("{}-postgres", &clean_branch_name);
    println!("Container name will be: {}", container_name);
    env::set_var("CONTAINER_NAME", &container_name);

    check_port_conflict(&docker, 5432, &container_name).await?;

    if check_container_exists(&docker, &container_name).await? {
        println!("Container for branch '{}' already exists.", container_name);
        if (docker.containers().get(&container_name).inspect().await
            .map_err(|e| format!("Failed to inspect container: {}", e))?)
            .state
            .map_or(false, |state| state.status.unwrap() == "running") {
            println!("Container is already running.");
            return Ok(());
        }
        docker.containers().get(&container_name).start().await
            .map_err(|e| format!("Failed to start existing container: {}", e))?;
        return Ok(());
    } else {
        println!("No existing container for branch '{}'. Creating a new one...", container_name);
        start_from_compose(compose_file)?;
    }

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'branchspawn -v -v -v' or 'branchspawn -vvv')
    match cli.debug {
        0 => {}
        1 => println!("Debug mode: Debug information enabled"),
        2 => println!("Debug mode: Extra verbose debug information enabled"),
        _ => println!("Debug mode: Don't be ridiculous"),
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
