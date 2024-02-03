use std::{borrow::BorrowMut, collections::{hash_map::DefaultHasher, HashMap}, hash::{Hash, Hasher}};

// TODO: fix spacing, fix constraint.

use wavefunc::{lay_mats, Mat};
use leptos::*;
use web_sys::wasm_bindgen::JsValue;
#[allow(dead_code)]
fn log<T: Into<JsValue>>(v: T) {
    web_sys::console::log_1(&v.into());
}
fn make_grid(w: usize, h: usize) -> Vec<(usize, usize, usize)> {
    let mut hash = DefaultHasher::new();
    w.hash(&mut hash);
    h.hash(&mut hash);
    let hash = hash.finish();
    (0..(w*h)).map(|x| (x, 0, x + hash as usize)).collect::<Vec<_>>()
}
fn color(v: usize) -> &'static str {
    match v { 0 => "green", 1 => "red", 2 => "blue", _ => "gray"}
}
const SIZE: usize = 40;
#[component]
fn App() -> impl IntoView {
    let first = create_signal(None);
    let second = create_signal(None);
    let value = create_signal(0);
    let solution = create_signal(None);
    let width = create_signal(5usize);
    let height = create_signal(5usize);
    let grid = create_signal(make_grid(5, 5));
    create_effect(move |_| {
        grid.1.set(make_grid(width.0.get(), height.0.get()));
    });
    view! {
        <div
            style:position = "relative"
            style:width = move || format!("{}px",width.0.get() * SIZE)
            style:height = move || format!("{}px",height.0.get() * SIZE)>
            <For
                each=move || grid.0.get()
                key=|x| *x
                children=move |(i, j, ..)| {
                    let x = i % width.0.get();
                    let y = i / width.0.get();
                    view! {
                        <div
                            style:width = format!("{}px", SIZE)
                            style:height = format!("{}px", SIZE)
                            style:position = "absolute"
                            style:left = move || format!("{}px", SIZE * x)
                            style:top = move || format!("{}px", SIZE * y)
                            style:background-color = move || first.0.get().and_then(|x| (x==i).then_some("black")).unwrap_or(second.0.get().and_then(|x| (x==i).then_some("white")).unwrap_or(color(j)))
                            on:click = move |_| {
                                match (first.0.get(), second.0.get()) {
                                    (Some(_), Some(_)) | (None, _) => {
                                        first.1.set(Some(i));
                                        second.1.set(None);
                                    },
                                    _ => {
                                        second.1.set(Some(i));
                                    }
                                }
                            }
                        >{j.to_string()}</div>
                    }
            } />
        </div>
        <Show when = move || solution.0.get().is_some() fallback=|| view! { }>
            <div
                style:margin="5px"
                style:position="relative"
                style:width=move || format!("{}px", SIZE * width.0.get())
                style:height=move || format!("{}px", SIZE * height.0.get())>
                <For
                    each = move || solution.0.get().unwrap()
                    key = move |x| *x
                    children = move |(x, y, v)| {
                        let mut sides = [true; 4];
                        match v {
                            Mat::Right => sides[0] = false,
                            Mat::Up => sides[1] = false,
                            Mat::Left => sides[2] = false,
                            Mat::Down => sides[3] = false,
                            _ => {}
                        }
                        view! {
                            <div
                                style:border-right-style=sides[0].then_some("solid").unwrap_or_default()
                                style:border-top-style=sides[1].then_some("solid").unwrap_or_default()
                                style:border-left-style=sides[2].then_some("solid").unwrap_or_default()
                                style:border-bottom-style=sides[3].then_some("solid").unwrap_or_default()
                                style:width=format!("{}px", SIZE)
                                style:height=format!("{}px", SIZE)
                                style:position="absolute"
                                style:left=format!("{}px", SIZE * x)
                                style:top=format!("{}px", SIZE * y)
                                style:background-color = move || {
                                    let p: (usize, usize, usize) = grid.0.get()[x + y * width.0.get()];
                                    color(p.1)
                                }
                            ></div>
                        }
                    }
                />
            </div>
        </Show>
        <input type="number"
            on:input=move |ev| {
                let value: usize = event_target_value(&ev).parse().unwrap_or_default();
                width.1.set(value);
            }
            value = "5"
        />
        <input type="number"
            on:input=move |ev| {
                let value: usize = event_target_value(&ev).parse().unwrap_or_default();
                height.1.set(value);
            }
            value = "5"
        />
        <button on:click = move |_| {
            let grid = grid.0.get();
            let mut itr = grid.iter().map(|(_, x, _)| *x);
            let coloring = (0..height.0.get()).map(move |_| itr.borrow_mut().take(width.0.get()).collect::<Vec<_>>()).collect::<Vec<_>>();
            let s = lay_mats(&coloring, width.0.get(), height.0.get()).map(|x| {
                x.into_iter().enumerate().flat_map(|(y, i)| {
                    i.into_iter().enumerate().map(move |(x, i)| (x, y, i))
                }).collect::<Vec<_>>()
            });
            solution.1.set(s);
        }>"Generate"</button>
        <input type="number"
            on:input=move |ev| {
                let _value: usize = event_target_value(&ev).parse().unwrap_or_default();
                value.1.set(_value);
            }
            value = "0"
        />
        <button on:click = move |_| {
            if let Some((first, second)) = first.0.get().zip(second.0.get()) {
                let fx = first % width.0.get();
                let fy = first / width.0.get();
                let sx = second % width.0.get();
                let sy = second / width.0.get();
                grid.1.update(|grid| {
                    for i in fx.min(sx)..(fx.max(sx)+1) {
                        for j in fy.min(sy)..(fy.max(sy)+1) {
                            grid[i + j * width.0.get()].1 = value.0.get();
                        }
                    }
                })
            }
        }>"Set"</button>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}