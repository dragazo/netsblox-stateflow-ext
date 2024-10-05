use netsblox_stateflow::*;
use wasm_bindgen::prelude::*;
use netsblox_extension_macro::*;
use netsblox_extension_util::*;
use js_helpers::*;

#[netsblox_extension_info]
pub const INFO: ExtensionInfo = ExtensionInfo {
    name: "StateMachine",
};

#[wasm_bindgen(start)]
pub fn setup() {
    console_error_panic_hook::set_once();

    let s = js!(window.document.createElement("script")).unwrap();
    js!(s.onload = () => {
        window.Viz.instance().then((x) => {
            window.viz_js_instance = x;
        });
    }).unwrap();
    js!(s.src = "https://extensions.netsblox.org/extensions/StateMachine/viz-standalone.js").unwrap();
    js!(window.document.body.appendChild(s)).unwrap();

    let s = js!(window.document.createElement("link")).unwrap();
    js!(s.rel = "stylesheet").unwrap();
    js!(s.type = "text/css").unwrap();
    js!(s.href = "https://pseudomorphic.netsblox.org/style.css").unwrap();
    js!(window.document.head.appendChild(s)).unwrap();

    let s = js!(window.document.createElement("script")).unwrap();
    js!(s.onload = () => {
        window.sm_visualizer = window.createDialog("State Machine");
        window.setupDialog(window.sm_visualizer);

        window.sm_invalidate_last_timestamp = 0;
        window.sm_invalidate = (force) => {
            const t = +new Date();
            if (!force && t - window.sm_invalidate_last_timestamp < 500) {
                return;
            }
            window.sm_invalidate_last_timestamp = t;

            try {
                if (window.sm_visualizer.getClientRects().length > 0) {
                    window.StateMachine_fns.visualize();
                }
            } catch (e) {
                console.error(e);
            }
        };

        const passive_invalidate = () => {
            window.sm_invalidate(false);
            setTimeout(passive_invalidate, 1000);
        };
        passive_invalidate();
    }).unwrap();
    js!(s.src = "https://pseudomorphic.netsblox.org/script.js").unwrap();
    js!(window.document.body.appendChild(s)).unwrap();
}

#[wasm_bindgen]
#[netsblox_extension_menu_item("Visualize")]
pub fn visualize() {
    let dialog = js!(window.sm_visualizer).unwrap();
    let content = js!(dialog.querySelector("content")).unwrap();
    js!(content.textContent = "").unwrap();

    let xml = js!(window.world.children[0].getSerializedRole()).unwrap().as_string().unwrap();
    match Project::compile(&xml, None, Settings { omit_unknown_blocks: true }) {
        Ok(proj) => {
            let graphviz_code = graphviz::print(proj.to_graphviz(), &mut Default::default());
            let svg = js!(window.viz_js_instance.renderSVGElement(graphviz_code)).unwrap();
            js!(content.appendChild(svg)).unwrap();
        }
        Err(e) => {
            js!(content.textContent = format!("visualize error: {e:?}")).unwrap();
        }
    }

    js!(window.showDialog(dialog)).unwrap();
}

#[wasm_bindgen]
#[netsblox_extension_menu_item("Copy Stateflow Code")]
pub fn copy_stateflow_code() {
    let xml = js!(window.world.children[0].getSerializedRole()).unwrap().as_string().unwrap();
    match Project::compile(&xml, None, Settings { omit_unknown_blocks: true }).and_then(|x| x.to_stateflow()) {
        Ok(res) => match js!(window.navigator.clipboard.writeText({ res.as_str() })) {
            Ok(_) => (),
            Err(_) => { js!(window.alert("failed to copy to clipboard (check browser permissions)")).unwrap(); }
        }
        Err(e) => { js!(window.alert(format!("translation error: {e:?}"))).unwrap(); }
    }
}

#[netsblox_extension_category]
pub const CATEGORY: CustomCategory = CustomCategory {
    name: "StateMachine",
    color: (150.0, 150.0, 150.0),
};

#[netsblox_extension_label_part]
pub const VAR_TYPE: LabelPart = LabelPart {
    spec: "smVarType",
    text: None,
    numeric: false,
    readonly: true,
    menu: Some(&[
        Menu::Entry { label: "local", value: "local" },
        Menu::Entry { label: "input", value: "input" },
        Menu::Entry { label: "output", value: "output" },
    ]),
};

fn unknown_var(var: &JsValue) -> JsError {
    JsError::new(&format!("unknown variable: {}", var.as_string().unwrap_or_default()))
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "smInState", category = "StateMachine", spec = "%var in state %s ?", pass_proc = true)]
pub fn in_state(proc: JsValue, machine: JsValue, state: JsValue) -> Result<bool, JsError> {
    let val = js!(proc.context.variables.getVar(machine)).map_err(|_| unknown_var(&machine))?;
    Ok(js!(window.snapEquals(val, state)).unwrap().as_bool().unwrap())
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "smTransition", category = "StateMachine", spec = "transition %var to state %s", pass_proc = true, type_override = BlockType::Terminator)]
pub fn transition(proc: JsValue, machine: JsValue, state: JsValue) -> Result<(), JsError> {
    js!(proc.doSetVar(machine, state)).map_err(|_| unknown_var(&machine))?;
    js!(proc.doStop()).unwrap();
    js!(window.sm_invalidate(true)).unwrap();
    Ok(())
}

#[wasm_bindgen]
#[netsblox_extension_block(name = "smMarkVar", category = "StateMachine", spec = "mark var %var as %smVarType", defaults = "[null, 'local']", pad_top = true)]
pub fn mark_var(_var: JsValue, _type: JsValue) {}
