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
        Ok(Value::List {
            vals: Vec::new(),
            span: call.head,
        })
    }
}

#[cfg(test)]
mod tests {
    use nu_plugin::Plugin;
    use nu_protocol::Span;

    use super::*;

    const SIMPLE_CALL: EvaluatedCall = EvaluatedCall {
        head: Span::unknown(),
        positional: Vec::new(),
        named: Vec::new(),
    };

    #[fixture]
    fn plugin() -> impl Plugin {
        NuPlugin
    }

    #[rstest]
    fn should_have_from_beancount_command(plugin: impl Plugin) {
        assert!(plugin
            .signature()
            .into_iter()
            .any(|s| &s.name == "from beancount"));
    }

    #[rstest]
    fn should_return_empty_list_for_empty_input(mut plugin: impl Plugin) {
        let result = plugin.run(
            "from beancount",
            &SIMPLE_CALL,
            &Value::String {
                val: "".into(),
                span: Span::unknown(),
            },
        );
        let Ok(Value::List { vals, .. }) = result else {
            panic!("Expected a list value but was: {result:?}");
        };
        assert!(vals.is_empty());
    }
}
