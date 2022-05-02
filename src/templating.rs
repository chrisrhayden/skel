use std::env;

use handlebars::{
    Context, Handlebars, Helper, JsonRender, Output, RenderContext, RenderError,
};

fn env_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let param = h.param(0).expect("did not get a parameter for env_helper");

    if param.value().is_null() {
        Err(RenderError::new("parameter is empty or not a string"))
    } else {
        let var = param.value().render();

        let env_value = env::var(&var);

        match env_value {
            Err(_) => Err(RenderError::new(format!(
                "did not find env var called {}",
                var
            ))),
            Ok(value) => {
                out.write(&value)?;
                Ok(())
            }
        }
    }
}

pub fn instantiate_handlebars<'reg>() -> Handlebars<'reg> {
    let mut handle = Handlebars::new();

    handle.register_helper("env", Box::from(env_helper));

    handle
}
