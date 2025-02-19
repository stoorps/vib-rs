[![Build](https://github.com/stoorps/vib-silverblue/actions/workflows/build.yml/badge.svg)](https://github.com/stoorps/vib-silverblue/actions/workflows/build.yml)

# vib-rs
Currently, this is a repo for all of my vib crates. 

## vib-api
Rust library for creating vib plugins easily.
This includes macros, and models for the Recipe passed in by vib.

There is an example repo [available here](/examples/example-plugin/).

Example code:
```Rust
use std::ffi:: CString;
use std::os::raw::c_char;
use serde::{Serialize, Deserialize};
use vib_api::{Recipe, build_module, plugin_info};

 #[derive(Serialize, Deserialize)]
 #[plugin_info(name = "example-plugin", module_type = "0", use_container_cmds = "0")]
 pub struct ModuleInfo {
     pub name: String,
     pub r#type: String,

     //Add your custom fields here!
 }


 #[build_module]
 fn build(module: ModuleInfo, _recipe: Recipe) -> String {
    //add your plugin code here!
    "".into()
 }
```

## ostree-pkg
Vib plugin for installing packages & flatpaks, and adding remotes for each respective manager, either at build time or first login.

This is still WIP.

