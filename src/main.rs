mod nu;

use nu_plugin::{serve_plugin, MsgPackSerializer};

fn main() {
    serve_plugin(&mut nu::Plugin, MsgPackSerializer {})
}
