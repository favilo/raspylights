use palette::LinSrgb;
use rune::{Any, ContextError, Module};

#[derive(Debug, Clone, Any)]
#[repr(transparent)]
pub(crate) struct Scrixels(pub(crate) Vec<LinSrgb<u8>>);

impl Scrixels {
    #[rune::function]
    fn set(&mut self, idx: usize, (r, g, b): (u8, u8, u8)) {
        self.0[idx] = LinSrgb::new(r, g, b);
    }

    #[rune::function]
    fn get(&self, idx: usize) -> Option<(u8, u8, u8)> {
        self.0.get(idx).map(|r| r.into_components())
    }

    #[rune::function]
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
    // let mut module = Module::with_item(["rgb"])?;
    let mut module = Module::new();
    module.ty::<Scrixels>()?;
    module.function_meta(Scrixels::set)?;
    module.function_meta(Scrixels::get)?;
    module.function_meta(Scrixels::len)?;
    Ok(module)
}
