use std::ffi:: CString;
use std::os::raw::c_char;
use serde::{Deserialize, Serialize};
use serde_json;
use vib_api::Recipe;
use vib_api::{build_module, plugin_info};

//This macro creates the PlugInfo() required by vib
#[derive(Serialize, Deserialize)]
#[plugin_info(name = "example-name", module_type ="0", use_container_cmds ="0" )]
struct ModuleInfo {
    name: String,
    r#type: String,

    //Add your plugin module fields here!
    packages: Option<Vec<String>>,
    #[serde(default)]
    flags: Vec<String>
}

//This macro creates the extern `BuildModule` required by vib.
//It also automatically deserializes your ModuleInfo & the Recipe struct from the JSON string parameters provided by vib.
#[build_module]
fn build(module: ModuleInfo, _recipe: Recipe) -> String {
    //add your plugin code here!
    format!("pkg install -y {} {}", module.flags.join(" "), module.packages.unwrap_or(vec!["".to_string()]).join(" "))
}