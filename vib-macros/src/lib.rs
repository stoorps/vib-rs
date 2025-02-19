
use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;
use syn::{
    parse_macro_input, DeriveInput, Lit, Meta,
    NestedMeta,  Type,
};

/// Generates the `extern "C" fn BuildModule` required by vib.
///
/// This needs to go on a build function, and must have two parameters:
/// * `module` - Your ModuleInfo struct, which uses the macro '[plugin_info]`.
/// * `recipe` - Of type 'vib_api::Recipe'
/// ```
/// # use std::ffi:: CString;
/// # use std::os::raw::c_char;
/// use vib_macros::{build_module, plugin_info}; 
/// use serde::{Serialize, Deserialize};
/// #
/// # #[derive(Debug, Deserialize)]
/// # pub struct Recipe; 
///
/// #[derive(Serialize, Deserialize)]
/// #[plugin_info(name = "example-plugin", module_type = "0", use_container_cmds = "0")]
/// pub struct ModuleInfo {
///     pub name: String,
///     pub r#type: String,
/// }
///

/// #[build_module]
/// fn build(module: ModuleInfo, _recipe: Recipe) -> String {
///    //add your plugin code here!
///    "".into()
/// }
/// ```
#[proc_macro_attribute]
pub fn build_module(_attr: TokenStream, item: TokenStream) -> TokenStream {

    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = input_fn.sig.ident.clone();
    let mut module_type: Option<Type> = None;

    // Find the module type (the one that's NOT Recipe)
    for input in &input_fn.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if !is_recipe_type(&pat_type.ty) { // Check if it's NOT Recipe
                module_type = Some(*pat_type.ty.clone());
                break;
            }
        }
    }

    let module_type = module_type.expect("A parameter (other than Recipe) must be present, and must implement the `module_info` macro");


    let expanded = quote! {
        #input_fn
    
        #[no_mangle]
        pub unsafe extern "C" fn BuildModule(
            module_interface: *const std::ffi::c_char,
            recipe_interface: *const std::ffi::c_char,
        ) -> *mut std::ffi::c_char {
            use std::ffi::{CStr, CString}; // Import necessary modules
    
            let recipe = CStr::from_ptr(recipe_interface);
            let module = CStr::from_ptr(module_interface);
    
            let module = String::from_utf8_lossy(module.to_bytes()).to_string();
            let recipe = String::from_utf8_lossy(recipe.to_bytes()).to_string();
    
            let module: #module_type = match serde_json::from_str(&module) {
                Ok(v) => v,
                Err(error) => {
                    let error_message = format!("ERROR: {}", error); // Format error message *outside*
                    return CString::new(error_message).unwrap().into_raw();
                }
            };
    
            let recipe: Recipe = match serde_json::from_str(&recipe) {
                Ok(v) => v,
                Err(error) => {
                    let error_message = format!("ERROR: {}", error); // Format error message *outside*
                    return CString::new(error_message).unwrap().into_raw();
                }
            };
    
            let cmd = #fn_name(module, recipe);
            let rtrn = CString::new(cmd).expect("ERROR: CString::new failed");
            rtrn.into_raw()
        }
    };

    expanded.into()
}


fn is_recipe_type(ty: &Type) -> bool {
    if let Type::Path(path) = ty {
        if let Some(ident) = path.path.get_ident() {
            return ident == "Recipe"; // Or whatever your Recipe type is named
        }
    }
    false
}

/// Generates the `extern "C" fn PlugInfo()` required by vib.
///
/// This needs to go on your ModuleInfo struct, and requries a `name: String` and `type: String` field at a minimum for vib to be happy.
/// 
/// Bear in mind that `type` is an invalid name for a field. See below example.
/// 
/// This macro requires three parameters:
/// 
/// * `name`   - The name of your plugin. 
/// * `module_type` - BuildPlugins should use '0' as a value, FinalizePlugins are '1'
/// * `use_container_cmds` - No idea what this does yet... I keep this at '0'
/// ```
/// # use std::ffi:: CString;
/// # use std::os::raw::c_char;
/// use vib_macros::{plugin_info}; 
/// use serde::{Serialize, Deserialize};
/// #
/// # #[derive(Debug, Deserialize)]
/// # pub struct Recipe; 
///
/// #[derive(Serialize, Deserialize)]
/// #[plugin_info(name = "example-plugin", module_type = "0", use_container_cmds = "0")]
/// pub struct ModuleInfo {
///     pub name: String,
///     pub r#type: String,
///     // OR
///     #[serde(rename = "type")]
///     pub module_type: String
/// }
///
/// ```
#[proc_macro_attribute]
pub fn plugin_info(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let attrs = parse_macro_input!(attr as syn::AttributeArgs);

    let mut name = String::new();
    let mut module_type = String::new();
    let mut use_container_cmds = String::new();

    for attr in attrs {
        match attr {
            NestedMeta::Meta(Meta::NameValue(nv)) => {
                let ident = nv.path.get_ident().unwrap().to_string();
                let lit = nv.lit;
                match ident.as_str() {
                    "name" => name = get_lit_string(&lit),
                    "module_type" => module_type = get_lit_string(&lit),
                    "use_container_cmds" => use_container_cmds = get_lit_string(&lit),
                    _ => panic!("Unknown attribute: {}", ident),
                }
            }
            _ => panic!("Invalid attribute format"),
        }
    }

    let generated_code = quote! {
        #input

        
        #[no_mangle]
        pub unsafe extern "C" fn PlugInfo() -> *mut c_char {

            let json_str = format!("\"Name\":\"{}\",\"Type\":\"{}\",\"Usecontainercmds\":{}", #name, #module_type, #use_container_cmds);
            let rtrn = CString::new(json_str).unwrap();
        
            
            rtrn.into_raw()
        }
    };

    generated_code.into()
}

fn get_lit_string(lit: &Lit) -> String {
    match lit {
        Lit::Str(s) => s.value(),
        _ => panic!("Attribute value must be a string literal"),
    }
}