/// Identifies one layer by its root motion and ancestor layer labels.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LayerPath(String);

impl LayerPath {
    /// Starts a path at an externally selected root motion.
    pub fn root(motion_label: &str) -> Self {
        Self(motion_label.to_owned())
    }

    /// Returns the path of a direct child layer.
    pub fn child(&self, layer_label: &str) -> Self {
        let mut path = self.0.clone();
        path.push('/');
        path.push_str(layer_label);
        Self(path)
    }

    /// Returns the slash-separated path used by override lookups.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Obtain from a string.
    pub fn from_string(str: String) -> Self {
        Self(str)
    }

    /// Returns the root motion label of this path.
    pub fn root_label(&self) -> &str {
        self.0.split('/').next().unwrap_or_default()
    }

    /// Iterates over the root motion label followed by layer labels.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }
}
