use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut show_drawing_pad = use_signal(|| false);

    rsx! {
        div {
            style: "text-align: center; margin-top: 20px;",
            button {
                onclick: move |_| {
                    show_drawing_pad.set(!show_drawing_pad());
                },
                style: "padding: 10px 20px; font-size: 16px; background: #4CAF50; color: white; border: none; border-radius: 5px; cursor: pointer; margin-bottom: 20px;",
                if show_drawing_pad() {
                    "Hide Drawing Pad"
                } else {
                    "🎨 Show Drawing Pad"
                }
            }
            
            if show_drawing_pad() {
                div {
                    style: "margin: 20px auto; max-width: 800px;",
                    h2 {
                        style: "margin-bottom: 10px; color: #333;",
                        "Drawing Pad"
                    }
                    p {
                        style: "margin-bottom: 10px; color: #666;",
                        "Click and drag to draw. Press the Clear button to clear the canvas."
                    }
                    DrawingPad {}
                }
            }
        }
    }
}

#[component]
fn DrawingPad() -> Element {
    let mut is_drawing = use_signal(|| false);
    let mut current_path = use_signal(|| Vec::<(f64, f64)>::new());
    let mut all_paths = use_signal(|| Vec::<Vec<(f64, f64)>>::new());

    let clear_canvas = move |_| {
        current_path.set(Vec::new());
        all_paths.set(Vec::new());
    };

    let start_drawing = move |event: Event<MouseData>| {
        is_drawing.set(true);
        let coords = event.element_coordinates();
        let mut new_path = Vec::new();
        new_path.push((coords.x, coords.y));
        current_path.set(new_path);
    };

    let draw = move |event: Event<MouseData>| {
        if is_drawing() {
            let coords = event.element_coordinates();
            let mut path = current_path();
            path.push((coords.x, coords.y));
            current_path.set(path);
        }
    };

    let stop_drawing = move |_| {
        if is_drawing() {
            is_drawing.set(false);
            let mut paths = all_paths();
            paths.push(current_path());
            all_paths.set(paths);
            current_path.set(Vec::new());
        }
    };

    // Generate SVG path data
    let generate_path_data = move |path: &Vec<(f64, f64)>| -> String {
        if path.is_empty() {
            return String::new();
        }
        
        let mut path_data = format!("M {} {}", path[0].0, path[0].1);
        for point in path.iter().skip(1) {
            path_data.push_str(&format!(" L {} {}", point.0, point.1));
        }
        path_data
    };

    rsx! {
        div {
            style: "text-align: center;",
            button {
                onclick: clear_canvas,
                style: "padding: 8px 16px; margin-bottom: 10px; background: #ff4444; color: white; border: none; border-radius: 4px; cursor: pointer;",
                "Clear Canvas"
            }
            div {
                style: "border: 2px solid #ccc; border-radius: 8px; display: inline-block; background: white; position: relative;",
                svg {
                    onmousedown: start_drawing,
                    onmousemove: draw,
                    onmouseup: stop_drawing,
                    onmouseleave: stop_drawing,
                    style: "display: block; cursor: crosshair;",
                    width: "800",
                    height: "600",
                    view_box: "0 0 800 600",
                    
                    // Draw all completed paths
                    for path in all_paths().iter() {
                        path {
                            d: generate_path_data(path),
                            stroke: "black",
                            stroke_width: "3",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            fill: "none"
                        }
                    }
                    
                    // Draw current path being drawn
                    if !current_path().is_empty() {
                        path {
                            d: generate_path_data(&current_path()),
                            stroke: "black",
                            stroke_width: "3",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            fill: "none"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn launch_drawing_pad() {
    use drawing_pad::DrawingApp;
    use winit::event_loop::{EventLoopBuilder};
    use winit::platform::x11::EventLoopBuilderExtX11;
    
    let event_loop = EventLoopBuilder::new()
        .with_any_thread(true)
        .build()
        .unwrap();
    let mut app = DrawingApp::new();
    event_loop.run_app(&mut app).unwrap();
}
