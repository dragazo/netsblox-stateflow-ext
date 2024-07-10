use stateflow_trans::*;
use wasm_bindgen::prelude::*;
use netsblox_extension_macro::*;
use netsblox_extension_util::*;
use web_sys::js_sys::*;
use web_sys::*;

#[wasm_bindgen]
extern "C" {
    fn alert(msg: &str);
}

macro_rules! js {
    ($root:ident) => { $root.clone() };
    ($root:ident . $f:ident ( $($args:expr),*$(,)? )) => { Reflect::apply(&js!($root.$f).dyn_ref().unwrap(), &$root, &(vec![$($args),*] as Vec<JsValue>).into_iter().collect()) };
    ($root:ident . $path:ident $($rest:tt)*) => {{
        let zzz = Reflect::get(&$root, &stringify!($path).into()).unwrap_or_else(|_| JsValue::undefined());
        js!(zzz $($rest)*)
    }};
    ($root:ident [ $idx:expr ] $($rest:tt)*) => {{
        let zzz = $root.dyn_ref::<Array>().map(|arr| arr.get($idx)).unwrap_or_else(|| JsValue::undefined());
        js!(zzz $($rest)*)
    }};
}

#[netsblox_extension_info]
pub const INFO: ExtensionInfo = ExtensionInfo {
    name: "StateMachine",
};

#[wasm_bindgen]
#[netsblox_extension_menu_item("Visualize")]
pub fn visualize() {
    let window = window().unwrap().dyn_into::<JsValue>().unwrap();
    let xml = js!(window.world.children[0].getSerializedRole()).unwrap().as_string().unwrap();
    match Project::compile(&xml, None) {
        Ok(proj) => {
            let graphviz_code = graphviz::print(proj.to_graphviz(), &mut Default::default());
            let encoded = js!(window.encodeURIComponent(graphviz_code.into())).unwrap().as_string().unwrap();
            let url = format!("https://dreampuf.github.io/GraphvizOnline/#{encoded}");
            js!(window.open(url.into(), "_blank".into())).unwrap();
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
