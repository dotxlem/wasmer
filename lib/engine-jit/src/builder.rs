use crate::JITEngine;
use wasmer_compiler::{CompilerConfig, Features, Target};

/// The JIT builder
pub struct JIT {
    #[allow(dead_code)]
    compiler_config: Option<Box<dyn CompilerConfig>>,
    target: Option<Target>,
    features: Option<Features>,
}

impl JIT {
    /// Create a new JIT
    pub fn new<T>(compiler_config: T) -> Self
    where
        T: Into<Box<dyn CompilerConfig>>,
    {
        Self {
            compiler_config: Some(compiler_config.into()),
            target: None,
            features: None,
        }
    }

    /// Create a new headless JIT
    pub fn headless() -> Self {
        Self {
            compiler_config: None,
            target: None,
            features: None,
        }
    }

    /// Set the target
    pub fn target(mut self, target: Target) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the features
    pub fn features(mut self, features: Features) -> Self {
        self.features = Some(features);
        self
    }

    /// Build the `JITEngine` for this configuration
    #[cfg(feature = "compiler")]
    pub fn engine(self) -> JITEngine {
        let target = self.target.unwrap_or_default();
        if let Some(compiler_config) = self.compiler_config {
            let features = self
                .features
                .unwrap_or_else(|| compiler_config.default_features_for_target(&target));
            let compiler = compiler_config.compiler();
            JITEngine::new(compiler, target, features)
        } else {
            JITEngine::headless()
        }
    }

    /// Build the `JITEngine` for this configuration
    #[cfg(not(feature = "compiler"))]
    pub fn engine(self) -> JITEngine {
        JITEngine::headless()
    }
}
