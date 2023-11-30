#![allow(non_snake_case)]

use dioxus::{html::label, prelude::*};
use std::collections::HashMap;

pub fn app_init() {
    dioxus_desktop::launch_cfg(
        app,
        dioxus_desktop::Config::default()
            .with_window(dioxus_desktop::WindowBuilder::new().with_maximized(true))
            .with_background_color((0, 0, 0, 1)).with_custom_head(r#"
            <style>
               html{background-color:black}
                   input::-webkit-datalist-option {
      background-color: #f1f1f1;
      padding: 8px;
    }
            </style>
            <script src="https://cdn.tailwindcss.com"></script>
            <script src="https://kit.fontawesome.com/67b52e9a3b.js" crossorigin="anonymous"></script>
            "#.to_string()),
    );
}

#[derive(Clone)]
struct TestGroup {
    title: String,
    file_path: Option<String>,
}

pub fn app(cx: Scope) -> Element {
    let _contenteditableRef = use_ref(cx, || "Please press - or ? for help");
    let ctxmenu = use_state(cx, || false);
    let showModal = use_state(cx, || false);
    let title = use_state(cx, || "".to_string());
    let _selectAction = |_action: &str| {};
    let test_groups = use_state(cx, || {
        vec![
            TestGroup {
                title: "Test Auth Flow".to_string(),
                file_path: None,
            },
            TestGroup {
                title: "Test New Post".to_string(),
                file_path: None,
            },
        ]
    });

    cx.render(rsx! {
        datalist { id: "methods-list",
            option { value: "GET" }
            option { value: "POST" }
            option { value: "PUT" }
            option { value: "DELETE" }
        }
        section { class: "bg-[#030303] xbg-stone-950 flex  text-gray-300 h-screen  divide-x divide-gray-700",
             if *showModal.get() {
               rsx!{div {
                class:"fixed inset-0 bg-gray-900 bg-opacity-75 z-10 flex items-center justify-center",
                onclick: move |_| showModal.set(false),
                div {
                    onclick: |e| e.stop_propagation(),
                    class: "w-96 bg-gray-800 rounded-lg shadow-lg p-10 flex flex-col gap-2",
                    label {class:"text-gray-300 font-bold text-lg","Title"},
                    input {
                        onchange: move |e| title.set(e.value.clone()),
                        class: "border border-gray-500 rounded px-2 py-1 bg-transparent outline-none w-full", 
                        value:"{*title.get()}"}
                    button {
                        onclick: move |_| {
                            let mut new_groups = test_groups.get().clone();
                            new_groups.push(TestGroup { title: title.get().clone(), file_path: None });
                            test_groups.set(new_groups);
                            title.set("".to_string());
                            showModal.set(false)
                            },
                        class: "bg-blue-500 text-white px-6 py-1 text-sm font-bold mt-4 w-max rounded-full hover:bg-blue-600","Done"}
                }
            }}
             }
            aside { class:"shrink p-4 text-lg pt-8 space-y-3 text-center", 
               a { class:"block p-3 rounded-md drop-shadow  border border-gray-400 cursor-pointer", "DE"}, 
               a { class:"block p-3 rounded-md drop-shadow  border border-gray-400 cursor-pointer", i {class: "fa fa-plus"}},
            }
            section { class: "flex-1 grid grid-cols-12 h-full bg-stone-950 divide-x divide-gray-800",
                aside { class: "col-span-2 py-16 px-3 h-full space-y-4", 
                    button{
                        onclick: move |_| {showModal.set(true)},
                        class: "block space-x-2 px-4 py-2 bg-gray-800 rounded-3xl text-center cursor-pointer hover:bg-gray-700 mb-5",
                        i {class: "fa fa-plus"},
                        span {"New test group"}    
                    }
                    test_groups.get().iter().map(|group|{
                        let file = group.file_path.clone().unwrap_or("".to_string());
                        rsx!(
                         button{
                        class: "cursor-pointer w-full block space-x-2 px-4 py-2 hover:bg-gray-800 rounded-xl flex items-center",
                        strong {class:"block",  "{group.title}"}
                        small {class:"block", "{file}"}    
                    })
                    }),
                }
                div { class: "col-span-10 p-8 h-full overflow-y-scroll",
                    div { class: "text-right", p { "Press ? for help" } }
                    div { class: "", TestBuilder(cx)}
                }
            }
        }
    })
}

fn TestBuilder(cx: Scope) -> Element {
    let stages = use_state(cx, || Vec::<RequestStep>::new());
    use_shared_state_provider(cx, || stages.get().clone());
    let stagesv = use_shared_state::<Vec<RequestStep>>(cx);
    match stagesv {
        None => cx.render(rsx! {div {"P"}}),
        Some(stages) => {
            let binding = stages.read().clone();
            let v = binding.iter().enumerate().map(|(i, stage)| {
                rsx! {RequestElement {
                    req: stage.clone(),
                    index: i,
                    key: "{i}",
                }}
            });
            cx.render(rsx! {
                    div {
                         v,
                         button {class: "bg-blue-900 py-1.5 px-3 rounded-full",
                         onclick: move |_| stages.with_mut(|s| s.push(RequestStep::default())),
                         i {class: "fa fa-plus"}}
                     }
            })
        }
    }
}

#[derive(Default, Clone, PartialEq)]
struct RequestStep {
    method: String,
    url: String,
    queryparams: Option<Vec<(String, String)>>,
    headers: Option<Vec<(String, String)>>,
    body: Option<ReqBody>,
    tests: Option<Vec<(String, String)>>,
}

#[derive(Default, Clone, PartialEq)]
enum ReqBody {
    #[default]
    None,
    Raw(String),
    FormData(HashMap<String, String>),
}

#[derive(Default, Clone, Copy, PartialEq)]
enum Tabs {
    #[default]
    Params,
    Headers,
    Body,
    Tests,
}

#[derive(Props, PartialEq)]
pub struct RequestElementProps {
    req: RequestStep,
    index: usize,
}

pub fn RequestElement<'a>(cx: Scope<'a, RequestElementProps>) -> Element<'a> {
    let hide_sxn = use_state(cx, || false);
    let tab_sxn = use_state(cx, || Tabs::Params);
    let index = cx.props.index;
    let req = cx.props.req.clone();

    let update_tab = |tab: Tabs| tab_sxn.set(tab);
    let stages = use_shared_state::<Vec<RequestStep>>(cx).unwrap();
    let remove_item = move || stages.write().remove(index);

    let update_method = move |method: String| {
        stages.write()[index].method = method;
    };
    let update_url = move |url: String| {
        stages.write()[index].url = url;
    };
    cx.render(rsx! {
        div { class: "pl-4 border-l-[.5px] border-gray-600 pt-2 pb-4",
            div { class: "flex space-x-3 w-full",
                div {
                class: "flex h-full flex-col gap-4",
                a { class: "inline-block cursor-pointer text-gray-400", onclick: |_| hide_sxn.with_mut(|hs| *hs = !*hs),
                    if *hide_sxn.get(){
                        rsx!{i { class: "w-3  fa-solid fa-chevron-right" } }
                    }else{
                        rsx!{i { class: "w-3 fa-solid fa-chevron-down" } }
                    },
                },
                button {
                    class: "cursor-pointer text-gray-600 rounded-full hover:text-red-500 flex items-center justify-center", 
                    onclick: move |_| {remove_item();},
                    i {class: "w-4 fa fa-solid fa-close"}
                }
                }
                div { class: "w-full border border rounded border-gray-900",
                div { class: "flex bg-gray-800 flex-1 py-2",
                    input {
                        list: "methods-list",
                        id: "methods",
                        value: "{req.method}",
                        name: "methods",
                        placeholder: "METHOD",
                        class: "bg-transparent border-r border-r-gray-900 px-3 outline-none focus:outline:none py-1 w-24 text-xs font-bold",
                        onchange: move |e| {
                             update_method(e.value.clone());
                           },
                    },
                    input {
                        r#type: "text",
                        placeholder: "Enter request URL",
                        value: "{req.url}",
                        class: "bg-transparent px-3 w-full outline-none focus:outline-none",
                        onchange: move |e| {
                              update_url(e.value.clone());
                           },
                    }
                },

                if !*hide_sxn.get(){
                rsx!{
                div { class: "relative pt-3 w-full",
                     nav {
                       ul {
                         class: "flex items-center text-gray-500 mx-2 px-2 gap-6 border-b-2 border-b-gray-900",
                         li {class: if *tab_sxn.get() == Tabs::Params { "border-b-2 border-b-blue-500" } else { "border-b-2 border-b-transparent" }, button {onclick: move |_| update_tab(Tabs::Params),"Params"}},
                         li {class: if *tab_sxn.get() == Tabs::Headers { "border-b-2 border-b-blue-500" } else { "border-b-2 border-b-transparent" }, button {onclick: move |_| update_tab(Tabs::Headers),"Headers"}},
                         li {class: if *tab_sxn.get() == Tabs::Body { "border-b-2 border-b-blue-500" } else { "border-b-2 border-b-transparent" }, button {onclick: move |_| update_tab(Tabs::Body),"Body"}},
                         li {class: if *tab_sxn.get() == Tabs::Tests { "border-b-2 border-b-blue-500" } else { "border-b-2 border-b-transparent" }, button {onclick: move |_| update_tab(Tabs::Tests),"Tests"}},
                       }
                     },

                     match tab_sxn.get() {
                         Tabs::Params => rsx!{
                            match req.queryparams  {
                                None =>  rsx!{HeadersParamsSxn{ tab: Tabs::Params, index:index}},
                                Some(vals) => rsx!{HeadersParamsSxn{ tab: Tabs::Params, vals: vals, index:index}},
                            }
                         },
                         Tabs::Headers => rsx!{
                            match req.headers {
                                 None => rsx!{HeadersParamsSxn{ tab: Tabs::Headers, index:index}},
                                 Some (vals) => rsx!{HeadersParamsSxn{ tab: Tabs::Headers, vals: vals, index:index}},
                            }
                         },
                         Tabs::Body => rsx!{
                             p {"Body"}
                         },
                         Tabs::Tests => rsx!{
                            match req.tests {
                                None => rsx!{AssertsElement{index:index}},
                                Some (vals) => rsx!{AssertsElement{vals: vals, index:index}}
                            }
                         },
                     }
                }
            }
            }

                }
            }
        }
    })
}

#[derive(Props, PartialEq)]
pub struct AssertElementProps {
    vals: Option<Vec<(String, String)>>,
    index: usize,
}

fn AssertsElement<'a>(cx: Scope<'a, AssertElementProps>) -> Element<'a> {
    let binding = cx
        .props
        .vals
        .clone()
        .unwrap_or(vec![("".to_string(), "".to_string())]);
    let index = cx.props.index;
    let stages = use_shared_state::<Vec<RequestStep>>(cx).unwrap();
    let update_val = move |i: usize, val: String, kind: String| {
        let sts = stages.read().clone();
        match sts[index].clone().tests {
            None => {
                if kind == "key" {
                    stages.write()[index].tests = Some(vec![(val, "".to_string())])
                } else {
                    stages.write()[index].tests = Some(vec![("".to_string(), val)])
                }
            }
            Some(vals) => {
                let mut tests = vals.clone();
                if kind == "key" {
                    tests[i].0 = val;
                } else {
                    tests[i].1 = val;
                }
                tests = tests
                    .iter()
                    .filter(|h| h.0 != "".to_string() || h.1 != "".to_string())
                    .cloned()
                    .collect();
                tests.push(("".to_string(), "".to_string()));
                stages.write()[index].tests = Some(tests)
            }
        }
    };
    let testItems = binding.iter().enumerate().map(|(i, (key, value))| {
        rsx!(
                div{ class: "flex w-full border border-gray-800 border-t-none rounded-b text-sm text-gray-300", 
                  input{
                     list: "assert-list",
                     id: "methods",
                     placeholder: "key",
                     onchange: move |e| {update_val(i, e.value.clone(), "key".to_string())},
                     value: "{key}", 
                     class: "bg-transparent outline-none px-3 py-1 border-r border-r-gray-800 w-60"
                },
                  input{
                    placeholder: "value",
                    value: "{value}", 
                    onchange: move |e| {update_val(i, e.value.clone(), "value".to_string())},
                    class: "bg-transparent outline-none w-full py-1 px-3"
                }
                },
    )});

    cx.render(rsx! {
        div { class: "flex flex-col p-2 w-full m-2",
             datalist {
                 class: "w-full rounded-lg bg-gray-800 text-gray-300 text-md",
                 id: "assert-list",
                 option { value: "ok" }
                 option { value: "array" }
                 option { value: "empty" }
                 option { value: "number" }
                 option { value: "boolean" }
                 option { value: "string" }
                 option { value: "notEmpty" }
                 option { value: "date" }
                 option { value: "null" }
             },
            div{class: "flex w-full border border-gray-800 rounded-t text-sm font-bold text-gray-500", div{class: "px-3 py-1 border-r border-r-gray-800 w-60", "Key"}, div{class: "w-full py-1 px-3","Value"}},
            testItems
      }
    })
}

#[derive(Props, PartialEq)]
pub struct HPElementProps {
    tab: Tabs,
    vals: Option<Vec<(String, String)>>,
    index: usize,
}

fn HeadersParamsSxn<'a>(cx: Scope<'a, HPElementProps>) -> Element {
    let binding = cx
        .props
        .vals
        .clone()
        .unwrap_or(vec![("".to_string(), "".to_string())]);
    let index = cx.props.index;
    let stages = use_shared_state::<Vec<RequestStep>>(cx).unwrap();
    let update_val = move |i: usize, val: String, kind: String| match cx.props.tab {
        Tabs::Params => {
            let qp = stages.read().clone();
            match qp[index].clone().queryparams {
                None => {
                    if kind == "key" {
                        stages.write()[index].queryparams = Some(vec![(val, "".to_string())])
                    } else {
                        stages.write()[index].queryparams = Some(vec![("".to_string(), val)])
                    }
                }
                Some(_v) => {
                    let mut params = stages.read()[index].clone().queryparams.unwrap();
                    if kind == "key" {
                        params[i].0 = val;
                    } else {
                        params[i].1 = val;
                    }
                    params = params
                        .iter()
                        .filter(|h| h.0 != "".to_string() || h.1 != "".to_string())
                        .cloned()
                        .collect();
                    params.push(("".to_string(), "".to_string()));
                    stages.write()[index].queryparams = Some(params)
                }
            }
        }
        _ => {
            let qp = stages.read().clone();
            match qp[index].clone().headers {
                None => {
                    if kind == "key" {
                        stages.write()[index].headers = Some(vec![(val, "".to_string())])
                    } else {
                        stages.write()[index].headers = Some(vec![("".to_string(), val)])
                    }
                }
                Some(_v) => {
                    let mut headers = stages.read()[index].clone().headers.unwrap();

                    if kind == "key" {
                        headers[i].0 = val;
                    } else {
                        headers[i].1 = val;
                    }

                    headers = headers
                        .iter()
                        .filter(|h| h.0 != "".to_string() || h.1 != "".to_string())
                        .cloned()
                        .collect();
                    headers.push(("".to_string(), "".to_string()));
                    stages.write()[index].headers = Some(headers)
                }
            }
        }
    };

    let items = binding.iter().enumerate().map(|( i, (k,v))| {
        rsx!(
             div{ class: "flex w-full border border-gray-800 border-t-none rounded-b text-sm text-gray-300", 
                  input{
                     placeholder: "key",
                     onchange: move |e| {update_val(i, e.value.clone(), "key".to_string())},
                     value: "{k}", 
                     class: "bg-transparent outline-none px-3 py-1 border-r border-r-gray-800 w-60"
                },
                  input{
                    placeholder: "value",
                    value: "{v}", 
                    onchange: move |e| {update_val(i, e.value.clone(), "value".to_string())},
                    class: "bg-transparent outline-none w-full py-1 px-3"
                }
                },
        )
    });

    cx.render(rsx! {
        div { class: "flex flex-col p-2 w-full m-2",
            div{class: "flex w-full border border-gray-800 rounded-t text-sm font-bold text-gray-500", div{class: "px-3 py-1 border-r border-r-gray-800 w-60", "Key"}, div{class: "w-full py-1 px-3","Value"}},
            items
      }
    })
}

#[inline_props]
fn HMSxn<'a>(
    cx: Scope<'a>,
    req_o: &'a UseRef<RequestStep>,
    title: &'a str,
    datalist: Vec<(String, String)>,
    field: &'a str,
) -> Element {
    let new_key = use_state(cx, || "".to_string());
    let new_value = use_state(cx, || "".to_string());
    let hide_sxn = use_state(cx, || false);
    let overwrite_reqo = |p: Vec<(String, String)>| {
        match *field {
            "q" => req_o.write().queryparams = Some(p),
            "h" => req_o.write().headers = Some(p),
            _ => (),
        };
    };
    cx.render(rsx! {
        div { class: "pl-4  space-x-3",
            div { class: "flex items-center",
                a { class: " cursor-pointer p-2", onclick: |_e| hide_sxn.with_mut(|hs| *hs = !*hs),
                if *hide_sxn.get(){
                     rsx!{ i { class: "w-4 fa-solid fa-chevron-right" }}
                }else{
                    rsx!{i { class: "w-4 fa-solid fa-chevron-down" } }
                },
                },
                span { class: "flex gap-3 flex-1", *title}
            }
            if !*hide_sxn.get() {
                rsx!{
                for (i, (k, v)) in datalist.iter().enumerate() {
                    div { class: "pl-8 flex",
                        input {
                            r#type: "text",
                            name: "key",
                            placeholder: "key",
                            oninput:move |e| {
                                let mut p = datalist.clone();
                                p[i] = (e.value.clone(), v.to_string());
                                overwrite_reqo((p).to_vec());
                            },
                            value: "{k}",
                            class: "bg-transparent border-b-[.5px] border-gray-100 p-1 w-64"
                        }
                        span { class: "shrink inline-block px-2", "=" }
                        input {
                            r#type: "text",
                            name: "value",
                            placeholder: "value",
                            oninput:move |e| {
                                let mut p = datalist.clone();
                                p[i] = (k.to_string(), e.value.clone());
                                overwrite_reqo((p).to_vec());
                            },
                            value: "{v}",
                            class: "flex-1 bg-transparent border-b-[.5px] border-gray-100 p-1 "
                        }
                    }
                },
                div { class: "pl-8 flex",
                    input {
                        r#type: "text",
                        name: "key",
                        placeholder: "key",
                        value: "{new_key.get()}",
                        oninput:move |e| {
                            new_key.set(e.value.clone())
                        },
                        class: "bg-transparent border-b-[.5px] border-gray-100 p-1 w-64"
                    }
                    span { class: "shrink inline-block px-2", "=" }
                    input {
                        r#type: "text",
                        name: "value",
                        placeholder: "value",
                        value: "{new_value.get()}",
                        oninput:move |e| {
                                new_value.set(e.value.clone())
                        },
                        onkeyup: move |e| {
                            if e.key().to_string() == "Enter" {
                                let mut p = datalist.clone();
                                p.push((new_key.get().to_string(), new_value.get().to_string()));
                                new_value.set("".to_string());
                                new_key.set("".to_string());
                                overwrite_reqo((p).to_vec());
                            }
                        },
                        class: "flex-1 bg-transparent border-b-[.5px] border-gray-100 p-1 "
                    }
                    a { class: " cursor-pointer p-2", onclick: move |_| {
                                let mut p = datalist.clone();
                                p.push((new_key.get().to_string(), new_value.get().to_string()));
                                new_value.set("".to_string());
                                new_key.set("".to_string());
                        overwrite_reqo((p).to_vec());
                    }, i { class: "fa-solid fa-person-walking-arrow-loop-left" } }
                }
            }
            }
        }
    })
}

fn BodySxn(cx: Scope) -> Element {
    cx.render(rsx! {
        div { class: "pl-4 space-x-3",
            div { class: "flex items-center",
                a { class: " cursor-pointer p-2", i { class: "fa-solid fa-chevron-down" } }
                div {
                    class:"flex-1 gap-3 space-x-4",
                    span { class: "inline-block", "Body "}
                    i { class:"inline-block fa fa-chevron-right "}
                    select {
                        class: "inline-block bg-stone-900",
                        option {"raw"},
                        option {"xml"},
                        option {"json"},
                        option {"form-data"},

                    }
                }
            }
            div { class: "pl-8 flex",
                input {
                    r#type: "text",
                    name: "key",
                    placeholder: "key",
                    class: "bg-transparent border-b-[.5px] border-gray-100 p-1 w-64"
                }
                span { class: "shrink inline-block px-2", "=" }
                input {
                    r#type: "text",
                    name: "value",
                    placeholder: "value",
                    class: "flex-1 bg-transparent border-b-[.5px] border-gray-100 p-1 "
                }
            }
        }
    })
}
