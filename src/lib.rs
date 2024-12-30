use std::{ops::Deref, rc::Rc};

use gpui::{
    canvas, div, point, px, size, Bounds, Corners, Edges, Hsla, InteractiveElement, IntoElement,
    PaintQuad, ParentElement, Path, Pixels, Point, Render, SharedString, Styled, TextStyle,
    ViewContext,
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
        let nodes = Rc::new(self.nodes.clone());
        let nodes_pre = nodes.clone();
        let start = self.start;
        const TEXT_START_OFF: Point<Pixels> = point(px(5.), px(5.));
        const TEXT_HEIGHT: Pixels = px(24.);
        div()
            .bg(cream())
            .size_full()
            .child(
                canvas(
                    move |_bounds, cx| {
                        let tx = cx.text_system();
                        let mut sty = TextStyle::default();
                        sty.color = cream();
                        let mut lines_to_draw = vec![];
                        for node in nodes_pre.deref() {
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
                                continue;
                            };
                            let mut cur_line_orig = moved_bounds.origin;
                            for line in text {
                                lines_to_draw.push((line, cur_line_orig));
                                cur_line_orig += point(px(0.), TEXT_HEIGHT + px(2.));
                            }
                        }
                        lines_to_draw
                    },
                    move |_bounds, lines_to_draw, cx| {
                        for node in nodes.deref() {
                            let quad = PaintQuad {
                                bounds: node.bounds + start,
                                corner_radii: Corners::all(px(5.)),
                                background: gpui::black().into(),
                                border_widths: Edges::all(px(2.)),
                                border_color: gpui::green(),
                            };
                            cx.paint_quad(quad);
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
