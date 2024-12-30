use gpui::{
    canvas, div, point, px, size, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Path, Pixels, Point, Render, SharedString, Styled, TextStyle, ViewContext,
};

#[derive(Clone, Debug)]
struct Node {
    bounds: Bounds<Pixels>,
    text: SharedString,
}

pub struct GraphViewer {
    nodes: Vec<Node>,
    start: Point<Pixels>,
    last_pos: Option<Point<Pixels>>,
}

pub fn draw_rect(bounds: Bounds<Pixels>, rounding: Pixels) -> Path<Pixels> {
    let start = bounds.bottom_left() - point(px(0.), rounding);
    let zero = px(0.);
    let mut path = Path::new(start);
    let top_left = bounds.bottom_left() - point(zero, bounds.size.height);
    path.line_to(top_left + point(zero, rounding));
    path.curve_to(top_left + point(rounding, zero), top_left);
    path.line_to(bounds.top_right() - point(rounding, zero));
    path.curve_to(
        bounds.top_right() + point(zero, rounding),
        bounds.top_right(),
    );
    path.line_to(bounds.bottom_right() - point(zero, rounding));
    path.curve_to(
        bounds.bottom_right() - point(rounding, zero),
        bounds.bottom_right(),
    );
    path.line_to(bounds.bottom_left() + point(rounding, zero));
    path.curve_to(start, bounds.bottom_left());
    path
}

impl GraphViewer {
    pub fn new() -> Self {
        let nodes = vec![
            Node {
                bounds: Bounds {
                    origin: point(px(450.), px(100.)),
                    size: size(px(200.), px(200.)),
                },
                text: "Hello, World!\ntest123".into(),
            },
            Node {
                bounds: Bounds {
                    origin: point(px(250.), px(400.)),
                    size: size(px(200.), px(300.)),
                },
                text: "xor eax, eax\npop edi\nret".into(),
            },
            Node {
                bounds: Bounds {
                    origin: point(px(550.), px(400.)),
                    size: size(px(200.), px(100.)),
                },
                text: "inc rax\njmp .LOOP_START".into(),
            },
        ];

        Self {
            start: point(px(0.), px(0.)),
            nodes,
            last_pos: None,
        }
    }

    pub fn clear(&mut self, cx: &mut ViewContext<Self>) {
        cx.notify();
    }
}

pub const fn cream() -> Hsla {
    Hsla {
        h: 35. / 360.,
        s: 0.95,
        l: 0.95,
        a: 1.,
    }
}

impl Render for GraphViewer {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let nodes = self.nodes.clone();
        let start = self.start;
        const TEXT_START_OFF: Point<Pixels> = point(px(5.), px(5.));
        const TEXT_HEIGHT: Pixels = px(24.);
        div()
            .bg(cream())
            .size_full()
            .child(
                canvas(
                    move |_bounds, _cx| {},
                    move |_bounds, _prepaint_data, cx| {
                        for node in &nodes {
                            let moved_bounds = Bounds {
                                origin: node.bounds.origin + start,
                                ..node.bounds
                            };
                            cx.paint_path(draw_rect(moved_bounds, px(5.0)), gpui::black());
                        }
                        let tx = cx.text_system();
                        let mut sty = TextStyle::default();
                        sty.color = cream();
                        let mut lines_to_draw = vec![];
                        for node in &nodes {
                            let moved_bounds = Bounds {
                                origin: node.bounds.origin + start + TEXT_START_OFF,
                                ..node.bounds
                            };
                            let Ok(text) = tx.shape_text(
                                node.text.clone(),
                                TEXT_HEIGHT,
                                &[sty.to_run(node.text.len())],
                                Some(moved_bounds.size.width),
                            ) else {
                                return;
                            };
                            let mut cur_line_orig = moved_bounds.origin;
                            for line in text {
                                lines_to_draw.push((line, cur_line_orig));
                                cur_line_orig += point(px(0.), TEXT_HEIGHT + px(2.));
                            }
                        }

                        for (line, offset) in lines_to_draw {
                            let _ = line.paint(offset, TEXT_HEIGHT, cx);
                        }
                    },
                )
                .size_full(),
            )
            .on_mouse_down(
                gpui::MouseButton::Middle,
                cx.listener(|this, ev: &gpui::MouseDownEvent, _| {
                    this.last_pos.replace(ev.position);
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Middle,
                cx.listener(|this, ev: &gpui::MouseUpEvent, _| {
                    let Some(last_pos) = this.last_pos else {
                        return;
                    };
                    let delta = ev.position - last_pos;
                    this.start += delta;
                    this.last_pos = None;
                }),
            )
            .on_mouse_move(cx.listener(|this, ev: &gpui::MouseMoveEvent, cx| {
                let Some(drag_start) = this.last_pos else {
                    return;
                };
                let delta = ev.position - drag_start;
                this.last_pos.replace(ev.position);
                this.start += delta;

                cx.notify();
            }))
    }
}
