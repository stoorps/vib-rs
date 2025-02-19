use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::os::raw::c_char;
use std::path::Path;
use vib_api::{build_module, plugin_info, Recipe};

#[derive(Default, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum As {
    #[default]
    system,
    user,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[plugin_info(name = "boot-shell", module_type = "0", use_container_cmds = "0")]
struct PkgModule {
    name: String,
    r#type: String,

    #[serde(default)]
    packages: Vec<String>,

    #[serde(default)]
    remotes: Vec<String>,
    #[serde(default)]
    r#as: As,

    #[serde(default)]
    commands: Vec<String>,
}

const SYSTEM_SERVICE: &str = "
[Unit]
Description=Runs scripts after boot
Wants=network-online.target
After=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/bin/boot-shell-system
Restart=on-failure
RestartSec=30

[Install]
WantedBy=default.target";

const USER_SERVICE: &str = "
[Unit]
Description=Runs scripts after boot
Wants=network-online.target
After=boot-shell-system.service

[Service]
Type=oneshot
ExecStart=/usr/bin/boot-shell-user
Restart=on-failure
RestartSec=30

[Install]
WantedBy=default.target
";


#[build_module]
fn build(module: PkgModule, recipe: Recipe) -> String {
    let includes_dir = Path::new(&recipe.includes_path);
    let service_parent_dir = includes_dir.join("etc/systemd/");
    let script_dir = includes_dir.join("usr/bin/");

    let (script_path, service_dir, service_path, service_cmd) = match module.r#as {
        As::system => (
            script_dir.join("boot-shell-system"),
            service_parent_dir.join("system"),
            service_parent_dir.join("system/boot-shell-system"),
            "--system boot-shell-system",
        ),
        As::user => (
            script_dir.join("boot-shell-user"),
            service_parent_dir.join("user"),
            service_parent_dir.join("user/boot-shell-user"),
            "--user boot-shell-user",
        ),
    };

    println!("{}\n{}\n{}",includes_dir.display(),script_path.display(),service_path.display());


    let mut script = "
    
    #!/bin/bash
    -oeu pipefail

    ".to_owned();


    for cmd in module.commands
    {
        script.push_str(&format!("{cmd}\n"));
    }



    let script_file = match script_path.exists() {
        true => OpenOptions::new().append(true).open(script_path),
        false => {
            let script_dir = script_dir.as_path();
            if !script_dir.exists() {
                match create_dir_all(script_dir) {
                    Ok(_) => {}
                    Err(e) => {
                        return format!("Error creating {}: {e}", script_dir.display());
                    }
                }
            }

            OpenOptions::new()
                .write(true)
                .create(true)
                .open(script_path)
        }
    };

    match script_file {
        Ok(mut script_file) => {
            if let Err(e) = writeln!(script_file, "{script}") {
                return format!("Couldn't write to file: {e}");
            }

            if !service_dir.exists() {
                match create_dir_all(service_dir.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        return format!("Error creating {}: {e}", service_dir.display());
                    }
                }
            }

            if service_path.exists()
            {
                //Already created and enabled
                return "echo \"service already created\"".into()
            }

            let mut service_file = match OpenOptions::new()
                .write(true)
                .create(true)
                .open(service_path.clone())
            {
                Ok(service_file) => service_file,
                Err(e) => return format!("Error creating {}: {e}", service_path.display()),
            };

            let service_definition = match module.r#as {
                As::system => SYSTEM_SERVICE,
                As::user => USER_SERVICE,
            };

            if let Err(e) = writeln!(service_file, "{service_definition}") {
                return format!("Couldn't write to file: {e}");
            }

            return format!("systemctl enable {service_cmd}");
        }
        Err(e) => {
            return format!("Error setting up boot module: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_build_system_module() {
        let tmp_dir = tempdir().unwrap().into_path();
        let recipe = Recipe {
            includes_path: tmp_dir.to_str().unwrap().to_string(),
            ..Default::default()
        };
        let module = PkgModule {
            name: "test-boot-shell".to_string(),
            r#type: "boot-shell".to_string(),
            r#as: As::system,
            commands: vec!["echo 'Hello, system!'".to_string()],
            ..Default::default()
        };

        let result = build(module, recipe);
        assert_eq!(result, "systemctl enable --system boot-shell-system");

        let script_path = tmp_dir.join("usr/bin/boot-shell-system");
        assert!(script_path.exists());
        
        let script_content = fs::read_to_string(script_path).unwrap();
        assert!(script_content.contains("echo 'Hello, system!'"));

        let service_path = tmp_dir.join("etc/systemd/system/boot-shell-system");
        assert!(service_path.exists());
        let service_content = fs::read_to_string(service_path).unwrap();
        assert_eq!(service_content, format!("{SYSTEM_SERVICE}\n"));
    }

    #[test]
    fn test_build_user_module() {
        let tmp_dir = tempdir().unwrap().into_path();
        let recipe = Recipe {
            includes_path: tmp_dir.to_str().unwrap().to_string(),
            ..Default::default()
        };
        let module = PkgModule {
            name: "test-boot-shell".to_string(),
            r#type: "boot-shell".to_string(),
            r#as: As::user,
            commands: vec!["echo 'Hello, user!'".to_string()],
            ..Default::default()
        };

        let result = build(module, recipe);
        assert_eq!(result, "systemctl enable --user boot-shell-user");

        let script_path = tmp_dir.join("usr/bin/boot-shell-user");
        assert!(script_path.exists());
        let script_content = fs::read_to_string(script_path).unwrap();
        assert!(script_content.contains("echo 'Hello, user!'"));

        let service_path = tmp_dir.join("etc/systemd/user/boot-shell-user");
        assert!(service_path.exists());
        let service_content = fs::read_to_string(service_path).unwrap();
        assert_eq!(service_content, format!("{USER_SERVICE}\n"));
    }


    #[test]
    fn test_existing_files() {
        let tmp_dir = tempdir().unwrap();
        let includes_path = tmp_dir.path().to_str().unwrap().to_string();
        let recipe = Recipe {
            includes_path: includes_path.clone(),
            ..Default::default()
        };
        let module = PkgModule {
            name: "test-boot-shell".to_string(),
            r#type: "boot-shell".to_string(),
            r#as: As::system,
            commands: vec!["echo 'First command'".to_string()],
            ..Default::default()
        };

        build(module.clone(), recipe.clone()); // First build creates files

        let module2 = PkgModule {
            commands: vec!["echo 'Second command'".to_string()], // Modified commands
            ..module
        };
        build(module2, recipe); // Second build modifies existing files


        let script_path = tmp_dir.path().join("usr/bin/boot-shell-system");
        assert!(script_path.exists());
        let script_content = fs::read_to_string(script_path).unwrap();
        assert!(script_content.contains("echo 'First command'"));
        assert!(script_content.contains("echo 'Second command'")); //Check if appended

        let service_path = tmp_dir.path().join("etc/systemd/system/boot-shell-system");
        assert!(service_path.exists());
        let service_content = fs::read_to_string(service_path).unwrap();
        assert_eq!(service_content, format!("{SYSTEM_SERVICE}\n"));
    }
}