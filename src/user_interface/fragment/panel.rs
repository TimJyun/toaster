use crate::user_interface::fragment::chat::ChatFragment;
use crate::user_interface::fragment::session_list::SessionListFragment;
use crate::user_interface::router::AppRoute;

use crate::imgs::MENU;
use crate::user_interface::window::{use_window_size, use_window_size_provider};
use dioxus::dioxus_core::Element;
use dioxus::prelude::*;
use dioxus_signals::{Signal, Writable};
use tracing::debug;

#[component]
pub fn PanelFragment(mut session_name: Signal<String>) -> Element {
    let window_signal = use_window_size();
    let open = use_signal(|| window_signal.read().is_widescreen());

    if window_signal.read().is_widescreen() {
        rsx! {
            WidePanelFragment { session_name, open }
        }
    } else {
        rsx! {
            NarrowPanelFragment { session_name, open }
        }
    }
}

#[component]
fn WidePanelFragment(mut session_name: Signal<String>, open: Signal<bool>) -> Element {
    rsx! {
        div { class: "size-full flex flex-row",
            if open() {
                WideOpenPanel { session_name, open }
            } else {
                WideClosePanel { open }
            }
            div { class: "h-full flex-1",
                ChatFragment { session_name }
            }
        }
    }
}

#[component]
fn WideOpenPanel(mut session_name: Signal<String>, open: Signal<bool>) -> Element {
    let nav = use_navigator();
    rsx! {
        div { class: "h-full w-1/5",
            div { class: "flex flex-row",
                span { class: "text-3xl", "Toaster" }
                span { class: "flex-1" }
                input {
                    class: "border rounded-xl p-1 m-1 size-8",
                    r#type: "image",
                    alt: "fold",
                    src: MENU,
                    onclick: move |_| {
                        open.set(false);
                    },
                }
            }
            SessionListFragment {
                onselect: move |selected| {
                    debug!("change to: {selected}");
                    nav.push(AppRoute::ChatPage {
                        session_name: selected,
                    });
                },
            }
        }
    }
}

#[component]
fn WideClosePanel(open: Signal<bool>) -> Element {
    rsx! {
        div { class: "h-full",
            input {
                class: "border rounded-xl p-1 m-1 size-8",
                r#type: "image",
                alt: "unfold",
                src: MENU,
                onclick: move |_| {
                    open.set(true);
                },
            }
        }
    }
}

#[component]
fn NarrowPanelFragment(mut session_name: Signal<String>, open: Signal<bool>) -> Element {
    rsx! {
        if open() {
            NarrowSideBar { session_name, open }
        } else {
            input {
                class: "border rounded-xl p-1 m-1 size-8",
                class: "fixed",
                r#type: "image",
                alt: "unfold",
                src: MENU,
                onclick: move |_| {
                    open.set(true);
                },
            }
        }
        ChatFragment { session_name }
    }
}

#[component]
fn NarrowSideBar(mut session_name: Signal<String>, open: Signal<bool>) -> Element {
    let nav = use_navigator();

    rsx! {
        div {
            class: "size-full fixed bg-[rgba(0,0,0,0.4)]",
            onclick: move |_| {
                debug!("confirm: cancel by background");
                open.set(false);
            },
            div {
                class: "w-85/100 h-full",
                style: "background-color: #fefefe;",
                onclick: |evt| {
                    debug!("confirm: stop propagation");
                    evt.stop_propagation();
                },
                SessionListFragment {
                    onselect: move |selected| {
                        debug!("change to: {selected}");
                        nav.push(AppRoute::ChatPage {
                            session_name: selected,
                        });
                        open.set(false);
                    },
                }
            }
        }
    }
}
