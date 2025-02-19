use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub use vib_macros::*;

#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Recipe {
    #[serde(rename = "Name")]
    #[serde(default)]
    pub name: String,

    #[serde(rename = "Id")]
    #[serde(default)]
    pub id: String,

    #[serde(rename = "Vibversion")]
    #[serde(default)]
    pub vib_version: String,

    #[serde(rename = "Stages")]
    #[serde(default)]
    pub stages: Vec<Stage>,

    #[serde(rename = "Path")]
    #[serde(default)]
    pub path: String,

    #[serde(rename = "ParentPath")]
    #[serde(default)]
    pub parent_path: String,

    #[serde(rename = "DownloadsPath")]
    #[serde(default)]
    pub downloads_path: String,

    #[serde(rename = "SourcesPath")]
    #[serde(default)]
    pub sources_path: String,

    #[serde(rename = "IncludesPath")]
    #[serde(default)]
    pub includes_path: String,

    #[serde(rename = "PluginPath")]
    #[serde(default)]
    pub plugin_path: String,

    #[serde(rename = "Containerfile")]
    #[serde(default)]
    pub container_file: String,
    //TODO: Finalize
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Stage {
    #[serde(default)]
    pub id: String,

    #[serde(default)]
    pub base: String,

    pub copy: Option<Vec<Copy>>,

    #[serde(rename = "addincludes")]
    pub add_includes: bool,

    pub labels: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
    pub adds: Option<Vec<Add>>,
    pub args: Option<HashMap<String, String>>,
    pub runs: Option<Run>,
    pub expose: Option<HashMap<String, String>>,

    #[serde(default)]
    pub cmd: Option<Cmd>,

    #[serde(rename = "Entrypoint")]
    #[serde(default)]
    pub entrypoint: Entrypoint,
    //TODO: Module support.
    //Modules     : []interface{}
}


#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PluginType {
    #[default]
    BuildPlugin,
    FinalizePlugin,
}

// Information about a plugin
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct PluginInfo {
    #[serde(rename = "Name")]
    #[serde(default)]
    name: String,

    #[serde(rename = "Type")]
    #[serde(default)]
    r#type: PluginType,

    #[serde(rename = "UseContainerCmds")]
    #[serde(default)]
    use_container_cmds: bool,
}

// Configuration for copying files or directories in a stage
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Copy {
    #[serde(rename = "From")]
    #[serde(default)]
    from: String,

    #[serde(rename = "SrcDst")]
    #[serde(default)]
    src_dst: HashMap<String, String>,

    #[serde(rename = "Workdir")]
    #[serde(default)]
    work_dir: String,
}

// Configuration for adding files or directories in a stage
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Add {
    #[serde(rename = "SrcDst")]
    #[serde(default)]
    src_dst: HashMap<String, String>,

    #[serde(rename = "Workdir")]
    #[serde(default)]
    work_dir: String,
}

// Configuration for the entrypoint of a container
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Entrypoint {
    #[serde(rename = "Exec")]
    exec: Option<Vec<String>>,

    #[serde(rename = "Workdir")]
    #[serde(default)]
    work_dir: String,
}

// Configuration for a command to run in the container
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Cmd {
    #[serde(rename = "Exec")]
    exec: Option<Vec<String>>,

    #[serde(rename = "Workdir")]
    #[serde(default)]
    work_dir: String,
}

// Configuration for commands to run in the container
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Run {
    #[serde(rename = "Commands")]
    commands: Option<Vec<String>>,

    #[serde(rename = "Workdir")]
    #[serde(default)]
    work_dir: String,
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Source {
    #[serde(default)]
    url: String,

    #[serde(default)]
    checksum: String,

    #[serde(default)]
    r#type: String,

    #[serde(default)]
    commit: String,

    #[serde(default)]
    tag: String,

    #[serde(default)]
    branch: String,

    #[serde(default)]
    package: Vec<String>,

    #[serde(default)]
    path: String,
}
