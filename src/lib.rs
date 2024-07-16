use netsblox_stateflow::*;
use wasm_bindgen::prelude::*;
use netsblox_extension_macro::*;
use netsblox_extension_util::*;
use js_helpers::*;

#[wasm_bindgen]
extern "C" {
    fn alert(msg: &str);
}

#[netsblox_extension_info]
pub const INFO: ExtensionInfo = ExtensionInfo {
    name: "StateMachine",
};

#[wasm_bindgen]
#[netsblox_extension_menu_item("Visualize")]
pub fn visualize() {
    let xml = js!(window.world.children[0].getSerializedRole()).unwrap().as_string().unwrap();
    match Project::compile(&xml, None) {
        Ok(proj) => {
            let graphviz_code = graphviz::print(proj.to_graphviz(), &mut Default::default());
            let encoded = js_sys::encode_uri_component(&graphviz_code).as_string().unwrap();
            let url = format!("https://dreampuf.github.io/GraphvizOnline/#{encoded}");
            js!(window.open(url, "_blank")).unwrap();
        }
        Err(e) => alert(&format!("visualize error: {e:?}")),
    }
}

#[netsblox_extension_category]
pub const CATEGORY: CustomCategory = CustomCategory {
    name: "StateMachine",
    color: (150.0, 150.0, 150.0),
};

#[wasm_bindgen]
#[netsblox_extension_block(name = "smTransition", category = "StateMachine", spec = "transition %var to state %s", pass_proc = true, type_override = BlockType::Terminator)]
pub fn transition(proc: JsValue, machine: JsValue, state: JsValue) {
    js!(proc.doSetVar(machine, state)).unwrap();
    js!(proc.doStop()).unwrap();
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "smInState", category = "StateMachine", spec = "%var in state %s ?", pass_proc = true)]
pub fn check_state(proc: JsValue, machine: JsValue, state: JsValue) -> bool {
    js!(proc.context.variables.getVar(machine)).unwrap() == state
}
