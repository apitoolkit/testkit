#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_desktop::tao::keyboard::Key;
use std::{collections::HashMap, mem::swap};

pub fn app_init() {
    dioxus_desktop::launch_cfg(
        app,
        dioxus_desktop::Config::default()
            .with_window(dioxus_desktop::WindowBuilder::new().with_maximized(true))
            .with_background_color((0, 0, 0, 1)).with_custom_head(r#"
            <style>
               html{background-color:black}
            </style>
            <script src="https://cdn.tailwindcss.com"></script>
            <script src="https://kit.fontawesome.com/67b52e9a3b.js" crossorigin="anonymous"></script>
            "#.to_string()),
    );
}

pub fn app(cx: Scope) -> Element {
    let _contenteditableRef = use_ref(cx, || "Please press - or ? for help");
    let ctxmenu = use_state(cx, || false);
    let _selectAction = |_action: &str| {};

    cx.render(rsx! {
        datalist { id: "methods-list",
            option { value: "GET" }
            option { value: "POST" }
            option { value: "PUT" }
            option { value: "DELETE" }
        }
        datalist { id: "req-subsection",
            option { value: "queryparams" }
            option { value: "headers" }
            option { value: "json_body" }
        }

        section { class: "bg-stone-900 text-gray-200 h-screen grid grid-cols-5 divide-x divide-gray-700",
            aside { class: "col-span-1" }
            div { class: "col-span-4 p-8",
                div { class: "text-right", p { "Press ? for help" } }
                div { class: "", RequestElement {} }
            }
        }
    })
}

#[derive(Default, Clone)]
struct RequestStep {
    queryparams: Option<Vec<(String, String)>>,
    headers: Option<HashMap<String, String>>,
    body: Option<ReqBody>,
}

#[derive(Default, Clone)]
enum ReqBody {
    #[default]
    None,
    Raw(String),
    FormData(HashMap<String, String>),
}

fn RequestElement(cx: Scope) -> Element {
    let toggle_sxn_option = use_state(cx, || false);
    let req_obj = use_ref(cx, RequestStep::default);

    let add_section = |p: &str| match p {
        "q" => req_obj
            .with_mut(|ro| ro.queryparams = ro.queryparams.clone().or(Some(Default::default()))),
        "h" => req_obj.with_mut(|ro| ro.headers = ro.headers.clone().or(Some(HashMap::new()))),
        "b" => req_obj.with_mut(|ro| ro.body = ro.body.clone().or(Some(ReqBody::default()))),
        _ => (),
    };

    cx.render(rsx! {
        div { class: "pl-4 border-l-[.5px] border-gray-600 pt-2 pb-4 space-y-2",
            div { class: " flex items-center space-x-3 ",
                a { class: " cursor-pointer", i { class: "fa-solid fa-chevron-down" } }
                div { class: "flex gap-3 flex-1",
                    input {
                        list: "methods-list",
                        id: "methods",
                        name: "methods",
                        placeholder: "GET",
                        class: "bg-transparent border-[.5px] border-gray-100 p-3 w-20"
                    }
                    input {
                        r#type: "text",
                        class: "bg-transparent border-[.5px] border-gray-100 p-3 flex-1"
                    }
                }
            },
            req_obj.with(|req_objj|{
                if let Some(queryparams) = req_objj.queryparams.clone() {
                         rsx!( HMSxn {datalist:queryparams, req_o: req_obj, title:"QueryParams", field:"q"})
                } else {rsx!( div {})}
            }),
            req_obj.with(|req_objj|{
                if let Some(_headers) = &req_objj.headers {
                    rsx!(HeadersSxn {})
                } else {rsx!( div {})}
            }),
            req_obj.with(|req_objj|{
                if let Some(_body) = &req_objj.body{
                    rsx!(BodySxn {})
                }else {rsx!( div {})}

            }),
            div { class: " relative",
                a {
                    class: " cursor-pointer p-2 inline-block",
                    onclick: move |_| toggle_sxn_option.with_mut(|x| *x = !*x),
                    i { class: "fa-solid fa-plus" }
                }
                if *toggle_sxn_option.get() {
                    rsx!{div { class: "flex flex-col gap-3 flex-1 absolute left-0 bg-stone-900 p-3 border-[.5px] border-gray-100 shadow-md rounded",
                        a {class: "cursor-pointer", onclick: move |_|add_section("q"),i {class: "fa-solid fa-plus"}, " Query Parameter"},
                        a {class: "cursor-pointer", onclick: move |_|add_section("h"),i {class: "fa-solid fa-plus"}, " Headers"},
                        a {class: "cursor-pointer", onclick: move |_|add_section("b"),i {class: "fa-solid fa-plus"}, " Body"},
                        a {class: "cursor-pointer", onclick: move |_|add_section("t"),i {class: "fa-solid fa-plus"}, " Tests"},
                    }}
                }
            }
        }
    })
}

#[inline_props]
fn QueryParamSxn<'a>(
    cx: Scope<'a>,
    req_o: &'a UseRef<RequestStep>,
    queryparams: Vec<(String, String)>,
) -> Element {
    let new_key = use_state(cx, || "".to_string());
    let new_value = use_state(cx, || "".to_string());
    cx.render(rsx! {
        div { class: "pl-4  space-x-3",
            div { class: "flex items-center",
                a { class: " cursor-pointer p-2", i { class: "fa-solid fa-chevron-down" } }
                span { class: "flex gap-3 flex-1", "QueryParams"}
            }
            for (i, (k, v)) in queryparams.iter().enumerate() {
                div { class: "pl-8 flex",
                    input {
                        r#type: "text",
                        name: "key",
                        placeholder: "key",
                        oninput:move |e| {
                            let mut p = queryparams.clone();
                            p[i] = (e.value.clone(), v.to_string());
                            req_o.write().queryparams = Some((p).to_vec());
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
                            let mut p = queryparams.clone();
                            p[i] = (k.to_string(), e.value.clone());
                            req_o.write().queryparams = Some((p).to_vec());
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
                        if (e.key == "Enter") {
                            let mut p = queryparams.clone();
                            p.push((new_key.get().to_string(), new_value.get().to_string()));
                            new_value.set("".to_string());
                            new_key.set("".to_string());
                            req_o.write().queryparams = Some((p).to_vec());

                        }
                    },
                    class: "flex-1 bg-transparent border-b-[.5px] border-gray-100 p-1 "
                }
                a { class: " cursor-pointer p-2", onclick: |_| {
                            let mut p = queryparams.clone();
                            p.push((new_key.get().to_string(), new_value.get().to_string()));
                            new_value.set("".to_string());
                            new_key.set("".to_string());
                            req_o.write().queryparams = Some((p).to_vec());
                }, i { class: "fa-solid fa-person-walking-arrow-loop-left" } }
            }
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
    let overwrite_reqo = |p: Vec<(String, String)>| {
        match *field {
            "q" => req_o.write().queryparams = Some(p),
            _ => (),
        };
    };
    cx.render(rsx! {
        div { class: "pl-4  space-x-3",
            div { class: "flex items-center",
                a { class: " cursor-pointer p-2", i { class: "fa-solid fa-chevron-down" } }
                span { class: "flex gap-3 flex-1", *title}
            }
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
    })
}

fn HeadersSxn(cx: Scope) -> Element {
    cx.render(rsx! {
        div { class: "pl-4  space-x-3",
            div { class: "flex items-center",
                a { class: " cursor-pointer p-2", i { class: "fa-solid fa-chevron-down" } }
                span { class: "flex gap-3 flex-1", "Headers"}
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

fn BodySxn(cx: Scope) -> Element {
    cx.render(rsx! {
        div { class: "pl-4  space-x-3",
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
