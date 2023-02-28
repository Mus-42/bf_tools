use crate::ins::BfCode;

/// [`OptState`] - Optimization state
///
/// it just a collection of optimization passes
#[derive(Debug)]
pub struct OptState {
    passes: Vec<Box<dyn OptPass>>,
}

/// Builder for [`OptState`]
#[derive(Debug)]
pub struct OptStateBuilder(Vec<Box<dyn OptPass>>);

impl OptState {
    /// Create builder object for [`OptState`]
    #[inline]
    pub fn builder() -> OptStateBuilder {
        OptStateBuilder::new()
    }
    /// Run all state passes
    #[inline]
    pub fn run_passes(&mut self, mut code: BfCode) -> BfCode {
        loop {
            let mut is_changed = false;
            for pass in &self.passes {
                let mut cur_changed = false;
                code = pass.optimize(code, &mut cur_changed);
                is_changed |= cur_changed;
            }
            //TODO also check code len? (debug_assert)
            if is_changed {
                break;
            }
        }
        code
    }
}

impl OptStateBuilder {
    /// Create new builder for [`OptState`]
    #[inline]
    pub fn new() -> Self {
        OptStateBuilder(Vec::new())
    }
    /// Add default passes to state
    ///
    /// Now default passes is:
    /// [`passes::PassUseless`]
    #[inline]
    pub fn add_default_passes(self) -> Self {
        self.add_pass(Box::from(passes::PassUseless))
    }
    /// Add optimization pass to state
    #[inline]
    pub fn add_pass(mut self, pass: Box<dyn OptPass>) -> Self {
        self.0.push(pass);
        self
    }
    /// Finish building [`OptState`] and return them
    #[inline]
    pub fn build(self) -> OptState {
        OptState { passes: self.0 }
    }
}

impl Default for OptState {
    #[inline]
    fn default() -> Self {
        OptStateBuilder::default().build()
    }
}

impl Default for OptStateBuilder {
    #[inline]
    fn default() -> Self {
        Self::new().add_default_passes()
    }
}

/// Optimization pass trait
pub trait OptPass: std::fmt::Debug {
    /// Function for pass invocation
    ///
    /// is_changed - mark for [`OptState::run_passes`] when them needs to stop
    fn optimize(&self, code: BfCode, is_changed: &mut bool) -> BfCode;
}

/// Useless instruction pass
pub mod pass_useless;

/// All built-in passes grouped in one module
pub mod passes {
    pub use super::pass_useless::PassUseless;
}
