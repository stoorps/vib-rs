use std::ffi:: CString;
use std::os::raw::c_char;
use serde::{Deserialize, Serialize};
use serde_json;
use vib_api::Recipe;
use vib_api::{build_plugin, module_info};

//#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
#[module_info(name = "example-name", module_type ="0", use_container_cmds ="0" )]
struct PkgModule {
    name: String,
    r#type: String,

    packages: Option<Vec<String>>,
    #[serde(default)]
    flags: Vec<String>
}

#[build_plugin]
fn build_module(module: PkgModule, _recipe: Recipe) -> String {

   
    format!("pkg install -y {} {}", module.flags.join(" "), module.packages.unwrap_or(vec!["".to_string()]).join(" "))

}