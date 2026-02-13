use std::collections::HashMap;
use std::sync::LazyLock;
use tauri::ipc::Invoke;

#[derive(Debug)]
pub struct AutoCommand {
    pub name: &'static str,
    pub handler: fn() -> fn(Invoke) -> bool,
}

inventory::collect!(AutoCommand);

static COMMAND_REGISTRY: LazyLock<HashMap<&'static str, fn(Invoke) -> bool>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        for cmd in inventory::iter::<AutoCommand> {
            map.insert(cmd.name, (cmd.handler)());
        }
        map
    });

pub fn dynamic_invoke_handler(invoke: Invoke) -> bool {
    let cmd_name = invoke.message.command();
    if let Some(cmd) = COMMAND_REGISTRY.get(cmd_name) {
        cmd(invoke)
    } else {
        false
    }
}
