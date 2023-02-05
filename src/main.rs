#[cfg(test)]
#[macro_use]
extern crate rstest;

use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::{Category, Signature, Value};

use nu_plugin::{serve_plugin, MsgPackSerializer};

fn main() {
    serve_plugin(&mut NuPlugin, MsgPackSerializer {})
}

pub struct NuPlugin;

impl nu_plugin::Plugin for NuPlugin {
    fn signature(&self) -> Vec<Signature> {
        vec![Signature::build("from beancount")
            .usage("Convert from beancount to structured data")
            .category(Category::Formats)]
    }

    fn run(
        &mut self,
        _name: &str,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        Ok(Value::Nothing { span: call.head })
    }
}

#[cfg(test)]
mod tests {
    use nu_plugin::Plugin;

    use super::*;

    #[fixture]
    fn plugin() -> impl Plugin {
        NuPlugin
    }

    #[rstest]
    fn should_have_from_beancount_signature(plugin: impl Plugin) {
        assert!(plugin
            .signature()
            .into_iter()
            .any(|s| &s.name == "from beancount"));
    }
}
