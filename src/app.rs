use leptos::task::spawn_local;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: JsValue) -> JsValue;
}

#[derive(Deserialize, Serialize)]
struct DebugContent<'a> {
    message: &'a str,
}

#[component]
pub fn App() -> impl IntoView {
    let (paths, set_paths) = signal(Vec::<String>::new());

    // 直接设置事件监听器
    spawn_local(async move {
        let closure = Closure::wrap(Box::new(move |event: JsValue| {
            // 直接提取 event.payload.paths
            if let Some(paths_array) = extract_paths_from_event(event) {

                let content = format!("{:?}", paths_array);
                set_paths.set(paths_array);
                
                spawn_local(async move {
                    let args = serde_wasm_bindgen::to_value(&DebugContent { message: content.as_str() }).unwrap();
                    let _ = invoke("dbg_in_terminal", args).await;
                });
            }
        }) as Box<dyn FnMut(JsValue)>);

        let _ = listen("tauri://drag-drop", closure.as_ref().into()).await;
        closure.forget();
    });

    view! {
        <div>"整个窗口都是拖放区"</div>
        <p>{ move || paths.get().into_iter().map(|p| view!{ <li>{p}</li> }).collect_view() }</p>
    }
}

// 定义事件数据结构
#[derive(Deserialize)]
struct DragDropEvent {
    payload: DragDropPayload,
}

#[derive(Deserialize)]
struct DragDropPayload {
    paths: Vec<String>,
}

// 辅助函数：从事件对象中提取 paths
fn extract_paths_from_event(event: JsValue) -> Option<Vec<String>> {
    // 使用 serde 直接反序列化
    let drag_event: DragDropEvent = serde_wasm_bindgen::from_value(event).ok()?;
    Some(drag_event.payload.paths)
}