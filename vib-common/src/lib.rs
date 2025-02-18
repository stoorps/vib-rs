use std::collections::HashMap;

use serde::{Deserialize, Serialize};


#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Recipe {

    #[serde(rename = "Name")]
    #[serde(default)]
	pub name          : String,
	
    #[serde(rename = "Id")]
    #[serde(default)]
    pub id          : String,
	
    #[serde(rename = "Vibversion")]
    #[serde(default)]
    pub vib_version    : String,

    #[serde(rename = "Stages")]
    #[serde(default)]
	pub stages        :Vec<Stage>,
	
    #[serde(rename = "Path")]
    #[serde(default)]
    pub path          : String,

    #[serde(rename = "ParentPath")]
    #[serde(default)]
	pub parent_path    : String,
	
    #[serde(rename = "DownloadsPath")]
    #[serde(default)]
    pub downloads_path : String,

    #[serde(rename = "SourcesPath")]
    #[serde(default)]
	pub sources_path  : String,
	

    #[serde(rename = "IncludesPath")]
    #[serde(default)]
    pub includes_path  : String,
    
    #[serde(rename = "PluginPath")]
    #[serde(default)]
    pub plugin_path    : String,

    #[serde(rename = "Containerfile")]
    #[serde(default)]
	pub container_file : String,

}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Stage {
    #[serde(rename = "Id")]
    #[serde(default)]
	pub id          :String,

    #[serde(rename = "Base")]
    #[serde(default)]            
	pub base        :String,

    #[serde(rename = "Copy")]
    #[serde(default)]            
	pub copy        :Vec<Copy>,

    #[serde(rename = "Addincludes")]
    #[serde(default)]        
	pub add_includes :bool,

    #[serde(rename = "Labels")]
    #[serde(default)]              
	pub labels      :HashMap<String, String>,

    #[serde(rename = "Env")]
    #[serde(default)]
	pub env         :HashMap<String, String>,

    #[serde(rename = "Adds")]
    #[serde(default)]
	pub adds        :Vec<Add>,

    #[serde(rename = "Args")]
    #[serde(default)]             
	pub args        :HashMap<String, String>,

    #[serde(rename = "Runs")]
    #[serde(default)]
	pub runs        :Run,

    #[serde(rename = "Expose")]
    #[serde(default)]
	pub expose      :HashMap<String, String>,

    #[serde(rename = "Cmd")]
    #[serde(default)]
	pub cmd         :Cmd,

    #[serde(rename = "Entrypoint")]
    #[serde(default)]               
	pub entrypoint  :Entrypoint,
	
    
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
	name            :String,

    #[serde(rename = "Type")]
    #[serde(default)]               
	r#type             :PluginType,

    #[serde(rename = "UseContainerCmds")]
    #[serde(default)]               
	use_container_cmds :bool,
}

// Configuration for copying files or directories in a stage
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Copy {
    #[serde(rename = "From")]
    #[serde(default)]           
	from    :String,

    #[serde(rename = "SrcDst")]
    #[serde(default)]           
	src_dst  : HashMap<String, String>,

    #[serde(rename = "Workdir")]
    #[serde(default)]           
	work_dir :String,
}

// Configuration for adding files or directories in a stage
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Add {
    #[serde(rename = "SrcDst")]
    #[serde(default)]  
	src_dst : HashMap<String, String>,

    #[serde(rename = "Workdir")]
    #[serde(default)]  
	work_dir :String,
}

// Configuration for the entrypoint of a container
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Entrypoint {
    #[serde(rename = "Exec")]
    #[serde(default)]  
	exec    :Vec<String>,

    #[serde(rename = "Workdir")]
    #[serde(default)]  
	work_dir :String,
}

// Configuration for a command to run in the container
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Cmd {
    #[serde(rename = "Exec")]
    #[serde(default)]  
	exec    :Vec<String>,

    #[serde(rename = "Workdir")]
    #[serde(default)]  
	work_dir :String,
}

// Configuration for commands to run in the container
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Run {
    #[serde(rename = "Commands")]
    #[serde(default)]  
	commands :Vec<String>,

    #[serde(rename = "Workdir")]
    #[serde(default)]  
	work_dir  :String,
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Source {
    #[serde(default)]  
	url      :String,  
    
    #[serde(default)]  
	checksum :String,  
    
    #[serde(default)]  
	r#type    :String, 
    
    #[serde(default)]   
	commit   :String, 
    
    #[serde(default)]  
	tag      :String, 
    
    #[serde(default)]  
	branch   :String,
    
    #[serde(default)]  
	package :Vec<String>,
    
    #[serde(default)]  
	path     :String,  
}
