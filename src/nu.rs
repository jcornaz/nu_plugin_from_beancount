use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::{Category, Signature, Value};

pub struct Plugin;

impl nu_plugin::Plugin for Plugin {
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
