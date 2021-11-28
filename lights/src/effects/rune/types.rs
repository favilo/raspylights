use palette::LinSrgb;
use runestick::{Any, ContextError, Module, Protocol};

#[derive(Debug, Clone, Any)]
#[repr(transparent)]
pub(crate) struct Scrixels(pub(crate) Vec<LinSrgb<u8>>);

impl Scrixels {
    fn set(&mut self, idx: usize, (r, g, b): (u8, u8, u8)) {
        self.0[idx] = LinSrgb::new(r, g, b);
    }

    fn get(&self, idx: usize) -> Option<(u8, u8, u8)> {
        self.0.get(idx).map(|r| r.into_components())
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<&[LinSrgb<u8>]> for Scrixels {
    fn from(v: &[LinSrgb<u8>]) -> Self {
        Self(v.to_vec())
    }
}

impl From<&mut [LinSrgb<u8>]> for Scrixels {
    fn from(v: &mut [LinSrgb<u8>]) -> Self {
        Self(v.to_vec())
    }
}

pub(crate) fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_item(&["rgb"]);
    module.ty::<Scrixels>()?;
    module.inst_fn("set", Scrixels::set)?;
    module.inst_fn("get", Scrixels::get)?;
    module.inst_fn("len", Scrixels::len)?;
    Ok(module)
}
