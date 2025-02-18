// Copyright 2024 axtlos <axtlos@disroot.org>
// SPDX-License-Identifier: GPL-3.0-ONLY

use serde::{Deserialize, Serialize};
use vib_common::Recipe;
use std::ffi::{CStr, CString};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::os::raw::c_char;
use std::path::Path;

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

#[derive(Serialize, Deserialize)]
struct PkgModule {
    name: String,
    r#type: String,

    //#[serde(rename = "Packages")]
    #[serde(default)]
    packages: Vec<String>,

    //#[serde(rename = "Remotes")]
    #[serde(default)]
    remotes: Vec<String>,

    //#[serde(rename = "Manager")]
    #[serde(default)]
    manager: Manager,

   // #[serde(rename = "Action")]
    #[serde(default)]
    action: Action,

    //#[serde(rename = "On")]
    #[serde(default)]
    on: On,

    //#[serde(rename = "ExtraFlags")]
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
ExecStart=/etc/silverblue-pakages
Restart=on-failure
RestartSec=30

[Install]
WantedBy=default.target";

// const USER_SERVICE: &str = "
// [Unit]
// Description=Configure Flatpaks for current user
// Wants=network-online.target
// After=system-flatpak-setup.service

// [Service]
// Type=simple
// ExecStart=/usr/bin/user-flatpak-setup
// Restart=on-failure
// RestartSec=30

// [Install]
// WantedBy=default.target";

#[no_mangle]
pub unsafe extern "C" fn PlugInfo() -> *mut c_char {
    let rtrn =
        CString::new("{\"Name\":\"silverblue-package\",\"Type\":\"0\",\"Usecontainercmds\":0}")
            .expect("ERROR: CString::new failed");
    rtrn.into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn BuildModule(
    module_interface: *const c_char,
    recipe_interface: *const c_char,
) -> *mut c_char {
    let recipe = CStr::from_ptr(recipe_interface);
    let module = CStr::from_ptr(module_interface);
    let cmd = build_module(
        String::from_utf8_lossy(module.to_bytes()).to_string(),
        String::from_utf8_lossy(recipe.to_bytes()).to_string(),
    );
    let rtrn = CString::new(cmd).expect("ERROR: CString::new failed");
    rtrn.into_raw()
}

fn build_module(module_interface: String, recipe_interface: String) -> String {
    let module: PkgModule = match serde_json::from_str(&module_interface) {
        Ok(v) => v,
        Err(error) => return format!("ERROR: {}", error),
    };


    let recipe: Recipe = match serde_json::from_str(&recipe_interface) {
        Ok(v) => v,
        Err(error) => return format!("ERROR: {}", error),
    };


    let parent_path = Path::new(&recipe.includes_path);
    let target_dir = parent_path.join("includes.container/etc");
    let target_dir =  Path::new(&target_dir);
    let script_path = target_dir.with_file_name("siverblue-packages");
    let script_path = Path::new(&script_path);

    let service_dir = Path::new("includes.container/etc/systemd/system/");
    let service_path = service_dir.with_file_name("silverblue-packages-setup.service");
    let service_path = Path::new(&service_path);

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
                },
                Action::remove_remote => {
                    is_error = true;
                    "Error: remove_remote is not supported on dnf"
                },
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

    if is_error
    {
        return action.into();
    }

    let params = match module.action {
        Action::install | Action::uninstall => module.packages.join(" "),
        Action::add_remote | Action::remove_remote => module.remotes.join(" "),
    };

    let command = format!(
        "{pkg_mgr} {action} {} {params}",
        module.args.join(" ")
    );

    match module.on {
        On::build => return command,

        On::boot => {

            let command = format!(
                "{pkg_mgr} {action} {} {params}",
                module.args.join(" ")
            );

            let file = match script_path.exists() {
                true => {
                    OpenOptions::new().append(true).open(script_path)
                }
                false => {
                    if !target_dir.exists()
                    {
                        match create_dir_all(target_dir)
                        {
                            Ok(_) => {},
                            Err(e) => {
                                return format!("Error creating {}: {e}", target_dir.display());
                            }
                        }
                    }

                    OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(script_path)
                }
            };

            match file 
            {
                Ok(mut file) => {

                    if let Err(e) = writeln!(file, "{command}") {
                        
                        return format!("Couldn't write to file: {e}");
                    }

                    if !service_path.exists()
                    {
                        if !service_dir.exists()
                        {
                            match create_dir_all(service_dir)
                            {
                                Ok(_) => {},
                                Err(e) => {
                                    return format!("Error creating {}: {e}", target_dir.display());
                                }
                            }

                            let mut service_file = match OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(service_path)
                            {
                                Ok(service_file) => service_file,
                                Err(e) => return format!("Error creating {}: {e}", service_path.display()),
                            };

                            if let Err(e) = writeln!(service_file, "{SYSTEM_SERVICE}") {
                        
                                return format!("Couldn't write to file: {e}");
                            }
    

                        }
                    }


                    return "systemctl enable --system silverblue-packages-setup.service".into();

                },
                Err(e) => {
                    return format!("Error setting up boot module: {e}");
                },
            }
        }
    }
}
