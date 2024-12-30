use gpui::{
    canvas, div, point, px, size, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Path, Pixels, Point, Render, Styled, TextStyle, ViewContext,
};

pub struct GraphViewer {
    rects: Vec<Bounds<Pixels>>,
    start: Point<Pixels>,
    last_pos: Option<Point<Pixels>>,
    panning: bool,
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
        let rects = vec![
            Bounds {
                origin: point(px(450.), px(100.)),
                size: size(px(200.), px(200.)),
            },
            Bounds {
                origin: point(px(250.), px(400.)),
                size: size(px(200.), px(300.)),
            },
            Bounds {
                origin: point(px(550.), px(400.)),
                size: size(px(200.), px(100.)),
            },
        ];

        Self {
            start: point(px(0.), px(0.)),
            rects,
            last_pos: None,
            panning: false,
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
        let rects = self.rects.clone();
        let start = self.start;
        div()
            .size_full()
            .child(
                canvas(
                    move |_bounds, _cx| {},
                    move |_bounds, _prepaint_data, cx| {
                        for mut rect in rects {
                            rect.origin += start;
                            cx.paint_path(draw_rect(rect, px(5.0)), gpui::black());
                        }
                        let tx = cx.text_system();
                        let msg = "Hello, World!";
                        let text_height = px(24.);
                        if let Ok(line) = tx.shape_line(
                            msg.into(),
                            text_height,
                            &[TextStyle::default().to_run(msg.len())],
                        ) {
                            let _ = line.paint(point(px(50.), px(50.)), text_height, cx);
                        }
                    },
                )
                .size_full(),
            )
            .on_mouse_down(
                gpui::MouseButton::Middle,
                cx.listener(|this, ev: &gpui::MouseDownEvent, _| {
                    this.panning = true;
                    this.last_pos.replace(ev.position);
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Middle,
                cx.listener(|this, ev: &gpui::MouseUpEvent, _| {
                    this.panning = false;
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
