struct Instance {
    module: Arc<ModuleInner>,
    exports: Exports,
}

impl Instance {
    fn load<T: Loader>(&self, loader: T) -> Result<T::Instance, T::Error>;
    fn func<Args, Rets>(&self, name: &str) -> ResolveResult<Func<Args, Rets, Wasm>>;
    fn resolve_func(&self, name: &str) -> ResolveResult<usize>;
    fn dyn_func(&self, name: &str) -> ResolveResult<DynFunc>;
    fn call(&self, name: &str, params: &[Value]) -> CallResult<Vec<Value>>;
    fn context(&self) -> &Ctx;
    fn context_mut(&mut self) -> &mut Ctx;
    fn exports(&self) -> ExportsIter;
    fn module(&self) -> Module;
    fn get_internal(&self, fields: &InternalField) -> u64;
    fn set_internal(&self, fields: &InternalField, value: u64);
}
