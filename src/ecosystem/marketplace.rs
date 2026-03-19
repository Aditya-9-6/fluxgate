use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, debug, error};
use crate::wasm_filter::WasmManager;

pub struct Plugin {
    pub name: String,
    pub author: String,
    pub capability: String, // "Guardrail", "Classifier", "Enricher"
    pub wasm_module_path: Option<String>,
}

pub struct MarketplaceManager {
    pub installed_plugins: HashMap<String, Plugin>,
    pub wasm_manager: Arc<WasmManager>,
}

impl MarketplaceManager {
    pub fn new(wasm_manager: Arc<WasmManager>) -> Self {
        let mut plugins = HashMap::new();
        plugins.insert("pii-detector-pro".to_string(), Plugin {
            name: "PII Detector Pro".to_string(),
            author: "SecureAI Corp".to_string(),
            capability: "Guardrail".to_string(),
            wasm_module_path: Some("/etc/fluxgate/plugins/pii_pro.wasm".to_string()),
        });
        
        Self { 
            installed_plugins: plugins,
            wasm_manager,
        }
    }

    /// Dynamically installs a new plugin into the FluxGate environment.
    pub fn install_plugin(&mut self, plugin: Plugin) {
        info!("🛒 [V12-MARKETPLACE] Installing WASM plugin: {} by {}", plugin.name, plugin.author);
        self.installed_plugins.insert(plugin.name.clone(), plugin);
    }

    /// Executes the capability of an installed plugin using the WASM runtime.
    pub fn execute_plugin(&self, plugin_name: &str, input: &str) -> String {
        debug!("🔌 [V12-MARKETPLACE] Invoking WASM capability for: {}", plugin_name);
        
        if let Some(plugin) = self.installed_plugins.get(plugin_name) {
            match self.wasm_manager.execute_filter(plugin_name, input) {
                Ok(result) => result,
                Err(e) => {
                    error!("❌ [V12-MARKETPLACE] WASM Execution Failed for {}: {}", plugin_name, e);
                    input.to_string() // Fallback to raw input
                }
            }
        } else {
            input.to_string()
        }
    }
}
