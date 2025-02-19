use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::os::raw::c_char;
use std::path::Path;
use vib_api::{build_module, plugin_info, Recipe};

#[derive(Default, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Manager {
    #[default]
    dnf,
    dnf5,
    flatpak,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Action {
    #[default]
    install,
    uninstall,
    add_remote,
    remove_remote,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum On {
    #[default]
    build,
    boot,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum As {
    #[default]
    system,
    user,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[plugin_info(name = "ostree-pkg", module_type = "0", use_container_cmds = "0")]
struct PkgModule {
    name: String,
    r#type: String,

    #[serde(default)]
    packages: Vec<String>,

    #[serde(default)]
    remotes: Vec<String>,

    #[serde(default)]
    manager: Manager,

    #[serde(default)]
    action: Action,

    #[serde(default)]
    on: On,

    #[serde(default)]
    r#as: As,

    #[serde(default)]
    args: Vec<String>,
}

const SYSTEM_SERVICE: &str = "
[Unit]
Description=Install Packages after boot
Wants=network-online.target
After=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/bin/ostree-pkg-system
Restart=on-failure
RestartSec=30

[Install]
WantedBy=default.target";

const USER_SERVICE: &str = "
[Unit]
Description=Install Packages after boot
Wants=network-online.target
After=ostree-pkg-system.service

[Service]
Type=oneshot
ExecStart=/usr/bin/ostree-pkg-user
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
            script_dir.join("ostree-pkg-system"),
            service_parent_dir.join("system"),
            service_parent_dir.join("system/ostree-pkg-system"),
            "--system ostree-pkg-system",
        ),
        As::user => (
            script_dir.join("ostree-pkg-user"),
            service_parent_dir.join("user"),
            service_parent_dir.join("user/ostree-pkg-user"),
            "--user ostree-pkg-system",
        ),
    };

    let pkg_mgr: &str;
    let action: &str;
    let mut is_error = false;

    match module.manager {
        Manager::dnf => {
            pkg_mgr = "dnf";
            action = match module.action {
                Action::install => "install -y",
                Action::uninstall => "uninstall -y",
                Action::add_remote => {
                    is_error = true;
                    "Error: add_remote is not supported on dnf"
                }
                Action::remove_remote => {
                    is_error = true;
                    "Error: remove_remote is not supported on dnf"
                }
            }
        }
        Manager::dnf5 => {
            pkg_mgr = "dnf5";
            action = match module.action {
                Action::install => "install -y",
                Action::uninstall => "uninstall -y",
                Action::add_remote => "-y copr enable",
                Action::remove_remote => "-y copr remove",
            };
        }

        Manager::flatpak => {
            pkg_mgr = "flatpak";
            action = match module.action {
                Action::install => "install --noninteractive",
                Action::uninstall => "uninstall --noninteractive",
                Action::add_remote => "remote-add --if-not-exists",
                Action::remove_remote => "remote-delete",
            };
        }
    }

    if is_error {
        return action.into();
    }

    let params = match module.action {
        Action::install | Action::uninstall => module.packages.join(" "),
        Action::add_remote | Action::remove_remote => module.remotes.join(" "),
    };

    let command = format!("{pkg_mgr} {action} {} {params}", module.args.join(" "));

    match module.on {
        On::build => return command,

        On::boot => {
            let command = format!("{pkg_mgr} {action} {} {params}", module.args.join(" "));

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
                    if let Err(e) = writeln!(script_file, "{command}") {
                        return format!("Couldn't write script file: {e}");
                    }

                    if !service_dir.exists() {
                        match create_dir_all(service_dir.clone()) {
                            Ok(_) => {}
                            Err(e) => {
                                return format!(
                                    "Error creating {}: {e}",
                                    service_dir.display()
                                );
                            }
                        }
                    }

                    if service_path.exists() {
                        return "echo \"service already created\"".into()
                    }
                        

                    let mut service_file = match OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(service_path.clone())
                    {
                        Ok(service_file) => service_file,
                        Err(e) => {
                            return format!(
                                "Error creating {}: {e}",
                                service_path.display()
                            )
                        }
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_build_module_install_dnf_build() {
        let module = PkgModule {
            name: "test".to_string(),
            r#type: "ostree-pkg".to_string(),
            packages: vec!["package1".to_string(), "package2".to_string()],
            manager: Manager::dnf,
            action: Action::install,
            on: On::build,
            ..Default::default()
        };
        let recipe = Recipe {
            includes_path: "/tmp".to_string(), // Doesn't matter for this test
            ..Default::default()
        };
        let result = build(module, recipe);
        assert_eq!(result, "dnf install -y  package1 package2");
    }

    #[test]
    fn test_build_module_uninstall_dnf_boot_system() {
        let temp_dir = tempdir().unwrap();
        let includes_path = temp_dir.path().to_str().unwrap().to_string();
        let module = PkgModule {
            name: "test".to_string(),
            r#type: "ostree-pkg".to_string(),
            packages: vec!["package1".to_string()],
            manager: Manager::dnf,
            action: Action::uninstall,
            on: On::boot,
            r#as: As::system,
            ..Default::default()
        };
        let recipe = Recipe {
            includes_path: includes_path.clone(),
            ..Default::default()
        };
        let result = build(module, recipe);
        assert_eq!(result, "systemctl enable --system ostree-pkg-system");

        let script_path = Path::new(&includes_path).join("usr/bin/ostree-pkg-system");
        let script_content = fs::read_to_string(script_path).unwrap();
        assert_eq!(script_content, "dnf uninstall -y  package1\n");

        let service_path = Path::new(&includes_path).join("etc/systemd/system/ostree-pkg-system");
        let service_content = fs::read_to_string(service_path).unwrap();
        assert_eq!(service_content, format!("{SYSTEM_SERVICE}\n"));

        temp_dir.close().unwrap(); // Clean up
    }

    #[test]
    fn test_build_module_add_remote_dnf5_boot_user() {
        let temp_dir = tempdir().unwrap();
        let includes_path = temp_dir.path().to_str().unwrap().to_string();

        let module = PkgModule {
            name: "test".to_string(),
            r#type: "ostree-pkg".to_string(),
            remotes: vec!["myrepo".to_string()],
            manager: Manager::dnf5,
            action: Action::add_remote,
            on: On::boot,
            r#as: As::user,
            ..Default::default()
        };
        let recipe = Recipe {
            includes_path: includes_path.clone(),
            ..Default::default()
        };
        let result = build(module, recipe);
        assert_eq!(result, "systemctl enable --user ostree-pkg-system");

        let script_path = Path::new(&includes_path).join("usr/bin/ostree-pkg-user");
        let script_content = fs::read_to_string(script_path).unwrap();
        assert_eq!(script_content, "dnf5 -y copr enable  myrepo\n");

        let service_path = Path::new(&includes_path).join("etc/systemd/user/ostree-pkg-user");
        let service_content = fs::read_to_string(service_path).unwrap();
        assert_eq!(service_content, format!("{USER_SERVICE}\n"));

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_build_module_install_flatpak_build_with_args() {
         let module = PkgModule {
            name: "test".to_string(),
            r#type: "ostree-pkg".to_string(),
            packages: vec!["app1".to_string()],
            manager: Manager::flatpak,
            action: Action::install,
            on: On::build,
            args: vec!["--user".to_string()],
            ..Default::default()
        };
        let recipe = Recipe {
            includes_path: "/tmp".to_string(), // Doesn't matter for this test
            ..Default::default()
        };
        let result = build(module, recipe);
        assert_eq!(result, "flatpak install --noninteractive --user app1");
    }

    #[test]
    fn test_build_module_add_remote_flatpak_boot() {
        let temp_dir = tempdir().unwrap();
        let includes_path = temp_dir.path().to_str().unwrap().to_string();

        let module = PkgModule {
            name: "test".to_string(),
            r#type: "ostree-pkg".to_string(),
            remotes: vec!["flathub".to_string()],
            manager: Manager::flatpak,
            action: Action::add_remote,
            on: On::boot,
            r#as: As::user,
            ..Default::default()
        };
        let recipe = Recipe {
            includes_path: includes_path.clone(),
            ..Default::default()
        };
        let result = build(module, recipe);
        assert_eq!(result, "systemctl enable --user ostree-pkg-system");

        let script_path = Path::new(&includes_path).join("usr/bin/ostree-pkg-user");
        let script_content = fs::read_to_string(script_path).unwrap();
        assert_eq!(script_content, "flatpak remote-add --if-not-exists  flathub\n");

        let service_path = Path::new(&includes_path).join("etc/systemd/user/ostree-pkg-user");
        let service_content = fs::read_to_string(service_path).unwrap();
        assert_eq!(service_content, format!("{USER_SERVICE}\n"));

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_build_module_dnf_add_remote_error() {
        let module = PkgModule {
            name: "test".to_string(),
            r#type: "ostree-pkg".to_string(),
            remotes: vec!["myrepo".to_string()],
            manager: Manager::dnf,
            action: Action::add_remote,
            on: On::build,
            ..Default::default()
        };
        let recipe = Recipe {
            includes_path: "/tmp".to_string(), // Doesn't matter for this test
            ..Default::default()
        };
        let result = build(module, recipe);
        assert_eq!(result, "Error: add_remote is not supported on dnf");
    }

    #[test]
    fn test_build_module_empty_packages_remotes() {
        let module = PkgModule {
            name: "test".to_string(),
            r#type: "ostree-pkg".to_string(),
            manager: Manager::dnf,
            action: Action::install,
            on: On::build,
            ..Default::default()
        };
        let recipe = Recipe {
            includes_path: "/tmp".to_string(), // Doesn't matter for this test
            ..Default::default()
        };
        let result = build(module, recipe);
        assert_eq!(result, "dnf install -y  "); // Two spaces at the end are intentional
    }


}