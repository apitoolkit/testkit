#![allow(non_snake_case)]

use dioxus::prelude::*;

pub fn app_init() {
    dioxus_desktop::launch_cfg(
        app,
        dioxus_desktop::Config::default()
            .with_window(dioxus_desktop::WindowBuilder::new().with_maximized(true)),
    );
}

pub fn app(cx: Scope) -> Element {
    let _contenteditableRef = use_ref(cx, || "Please press - or ? for help");
    let ctxmenu = use_state(cx, || false);
    let _ctxmenu_class = if *ctxmenu.get() { "" } else { "hidden" };

    let _selectAction = |_action: &str| {};

    cx.render(rsx! {
        // script { src: "https://cdn.tailwindcss.com" }
        script { src: "https://cdn.tailwindcss.com" }
        script { src: "https://kit.fontawesome.com/67b52e9a3b.js", crossorigin: "anonymous" }

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

fn RequestElement(cx: Scope) -> Element {
    let toggle_sxn_option = use_state(cx, || false);

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
            QueryParamSxn {},
            HeadersSxn {},
            BodySxn {},
            div { class: " relative",
                a {
                    class: " cursor-pointer p-2 inline-block",
                    onclick: move |_| toggle_sxn_option.with_mut(|x| *x = !*x),
                    i { class: "fa-solid fa-plus" }
                }
                if *toggle_sxn_option.get() {
                    rsx!{div { class: "flex flex-col gap-3 flex-1 absolute left-0 bg-stone-900 p-3 border-[.5px] border-gray-100 shadow-md rounded",
                        a {class: "cursor-pointer", i {class: "fa-solid fa-plus"}, " Query Parameter"},
                        a {class: "cursor-pointer",i {class: "fa-solid fa-plus"}, " Headers"},
                        a {class: "cursor-pointer",i {class: "fa-solid fa-plus"}, " Body"},
                        a {class: "cursor-pointer",i {class: "fa-solid fa-plus"}, " Tests"},
                    }}
                }
            }
        }
    })
}

fn QueryParamSxn(cx: Scope) -> Element {
    cx.render(rsx! {
        div { class: "pl-4  space-x-3",
            div { class: "flex items-center",
                a { class: " cursor-pointer p-2", i { class: "fa-solid fa-chevron-down" } }
                span { class: "flex gap-3 flex-1", "QueryParams"}
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
