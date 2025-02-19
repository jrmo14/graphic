pub mod layout;

use gpui::{
    canvas, div, point, px, size, Bounds, Corners, Div, Edges, Hsla, InteractiveElement,
    IntoElement, PaintQuad, ParentElement, Path, Pixels, Point, Render, SharedString, Styled,
    TextStyle, ViewContext, WindowContext, WrappedLine,
};
use layout::{GraphLayout, Radius};
use petgraph::graph::{DiGraph, NodeIndex};

#[derive(Clone, Debug)]
struct Node {
    bounds: Bounds<Pixels>,
    text: SharedString,
}

impl Radius for Node {
    fn radius(&self) -> f32 {
        (self.bounds.size.width.max(self.bounds.size.height) / 2.).0
    }
}

pub enum Message {
    LayoutRequest {
        graph: DiGraph<Vec<String>, EdgeKind>,
        entry: NodeIndex,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum EdgeKind {
    Take,
    NoTake,
    Fallthrough,
    Switch,
}

#[derive(Copy, Clone, Debug)]
struct Edge {
    kind: EdgeKind,
    start: Point<Pixels>,
    mid: f32,
    around: Option<Bounds<Pixels>>,
    end: Point<Pixels>,
}

#[derive(Clone, Debug)]
struct LayoutData {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

pub struct GraphViewer {
    start: Point<Pixels>,
    last_pos: Option<Point<Pixels>>,
    layout_data: Option<LayoutData>,
    pending_message: Option<Message>,
}

/// Generate path to wrap around `points `with `stroke_width`
/// Assumes that `path` is all right angles
fn draw_arrow(points: &[Point<Pixels>], stroke_width: Pixels) -> Path<Pixels> {
    let half_width = stroke_width / 2.;
    let triangle_size = stroke_width * 1.5;

    let compute_offset = |a: Point<Pixels>, b: Point<Pixels>, c: Point<Pixels>| {
        if a.x == b.x {
            // Dealing with a vertical line
            if a.y < c.y {
                if a.x < c.x {
                    // into right turn
                    point(half_width, -half_width)
                } else {
                    // into left turn
                    point(half_width, half_width)
                }
            } else if a.x < c.x {
                // into right turn
                point(-half_width, -half_width)
            } else {
                // into left turn
                point(-half_width, half_width)
            }
        } else if a.y < c.y {
            if a.x < c.x {
                // Turn down and to the right
                point(half_width, -half_width)
            } else {
                // Down and to the left
                point(half_width, half_width)
            }
        } else if a.x < c.x {
            // up and to the right
            point(-half_width, -half_width)
        } else {
            // up and to the left
            point(-half_width, half_width)
        }
    };

    // Start the path
    let mut path = Path::new(points[0]);
    path.line_to(points[0] + point(half_width, px(0.)));

    // Trace forward
    points.windows(3).for_each(|pts| {
        let offset = compute_offset(pts[0], pts[1], pts[2]);
        path.line_to(pts[1] + offset);
    });
    // Triangle
    let last = *points.last().unwrap();
    path.line_to(last + point(half_width, -triangle_size));
    path.line_to(last + point(half_width + triangle_size, -triangle_size));
    path.line_to(last);
    path.line_to(last + point(-half_width + -triangle_size, -triangle_size));
    path.line_to(last + point(-half_width, -triangle_size));
    // Trace back
    points.windows(3).rev().for_each(|pts| {
        let offset = compute_offset(pts[2] * -1., pts[1] * -1., pts[0] * -1.);
        path.line_to(pts[1] - offset);
    });
    // Close
    path.line_to(points[0] + point(-half_width, px(0.)));

    path
}

fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
    (1. - t) * v0 + t * v1
}

fn draw_edge(mut edge: Edge, offset: Point<Pixels>, stroke_width: Pixels) -> (Path<Pixels>, Hsla) {
    let color = match edge.kind {
        EdgeKind::Take => gpui::green(),
        EdgeKind::NoTake => gpui::red(),
        EdgeKind::Fallthrough => gpui::opaque_grey(0.2, 1.),
        EdgeKind::Switch => gpui::blue(),
    };
    edge.start += offset;
    edge.end += offset;
    let edge_offset = stroke_width * 10.;
    let path = if edge.start.y > edge.end.y {
        let outer_x = if edge.start.x < edge.end.x {
            if let Some(outer) = edge.around {
                outer.origin.x - edge_offset
            } else {
                edge.start.x - edge_offset
            }
        } else if let Some(outer) = edge.around {
            outer.origin.x + outer.size.width + edge_offset
        } else {
            edge.start.x + edge_offset
        } + offset.x;
        draw_arrow(
            &[
                edge.start,
                edge.start + point(px(0.), edge_offset),
                point(outer_x, edge.start.y + edge_offset),
                point(outer_x, edge.end.y - edge_offset),
                edge.end - point(px(0.), edge_offset),
                edge.end,
            ],
            stroke_width,
        )
    } else {
        draw_arrow(
            &[
                edge.start,
                point(
                    edge.start.x,
                    px(lerp(edge.start.y.0, edge.end.y.0, edge.mid)),
                ),
                point(edge.end.x, px(lerp(edge.start.y.0, edge.end.y.0, edge.mid))),
                edge.end,
            ],
            stroke_width,
        )
    };

    (path, color)
}

fn create_edge(
    src: Bounds<Pixels>,
    sink: Bounds<Pixels>,
    kind: EdgeKind,
    mid: f32,
    around: Option<Bounds<Pixels>>,
) -> Edge {
    Edge {
        kind,
        start: src.bottom_left() + point(px(src.size.width / px(2.)), px(0.)),
        end: sink.top_right() - point(px(sink.size.width / px(2.)), px(0.)),
        mid,
        around,
    }
}

type PrepaintData = Vec<(WrappedLine, Point<Pixels>)>;
const TEXT_START_OFF: Point<Pixels> = point(px(5.), px(5.));
const TEXT_HEIGHT: Pixels = px(18.);
const TEXT_SPACING: Pixels = px(2.);

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
        let edges = vec![
            create_edge(
                nodes[0].bounds - point(px(15.), px(0.)),
                nodes[1].bounds,
                EdgeKind::Take,
                0.5,
                None,
            ),
            create_edge(
                nodes[0].bounds,
                nodes[2].bounds,
                EdgeKind::NoTake,
                0.3,
                None,
            ),
            create_edge(
                nodes[2].bounds,
                nodes[0].bounds,
                EdgeKind::Fallthrough,
                0.5,
                Some(nodes[2].bounds),
            ),
            create_edge(
                nodes[1].bounds,
                nodes[0].bounds - point(px(15.0), px(0.)),
                EdgeKind::Fallthrough,
                0.5,
                Some(nodes[1].bounds),
            ),
        ];

        Self {
            start: point(px(0.), px(0.)),
            layout_data: Some(LayoutData { nodes, edges }),
            // layout_data: None,
            last_pos: None,
            pending_message: None,
        }
    }

    pub fn clear(&mut self, cx: &mut ViewContext<Self>) {
        cx.notify();
    }

    pub fn handle_message(&mut self, message: Message) {
        match message {
            Message::LayoutRequest { .. } => self.pending_message.replace(message),
        };
    }

    pub fn paint_no_data(&mut self, _cx: &mut ViewContext<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(cream())
            .size_full()
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .text_xl()
            .text_color(gpui::black())
            .child(format!("No graph to display"))
    }

    // TODO might be fun to animate the canvas with simulation data
    fn prepaint(
        &mut self,
    ) -> impl 'static + FnOnce(Bounds<Pixels>, &mut WindowContext) -> PrepaintData {
        let start = self.start;
        // Take here because we want to move and clear the message
        // let pending_message = self.pending_message.take();

        // Unwrap safety... we won't be painting to the canvas if we don't have layout data
        let layout_data = self.layout_data.clone();
        // TODO need to layout text on top of itself to determine the size of the nodes
        // Use that to create boxes, emit node and edge locations like that
        // If there is a pending message, then we need to use that to update the layout
        // We should also change the prepaint data to return the layout info
        // Sould also make sure to only draw items that are contained within the boundary of the view
        move |_bounds, cx| -> PrepaintData {
            let nodes = layout_data.unwrap().nodes;
            let tx = cx.text_system();
            let sty = TextStyle {
                color: cream(),
                ..Default::default()
            };
            let mut text_to_draw = vec![];
            for node in nodes {
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
                    text_to_draw.push((line, cur_line_orig));
                    cur_line_orig += point(px(0.), TEXT_HEIGHT + TEXT_SPACING);
                }
            }
            text_to_draw
        }
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
        let Some(layout_data) = &self.layout_data else {
            return self.paint_no_data(cx);
        };
        let nodes = layout_data.nodes.clone();
        let edges = layout_data.edges.clone();
        let start = self.start;
        div()
            .bg(cream())
            .size_full()
            .child(
                canvas(self.prepaint(), move |_bounds, text_to_draw, cx| {
                    for node in nodes {
                        let quad = PaintQuad {
                            bounds: node.bounds + start,
                            corner_radii: Corners::all(px(5.)),
                            background: gpui::black().into(),
                            border_widths: Edges::all(px(2.)),
                            border_color: gpui::green(),
                        };
                        cx.paint_quad(quad);
                    }
                    for edge in edges {
                        let (path, color) = draw_edge(edge, start, px(3.));
                        cx.paint_path(path, color);
                    }

                    for (text, offset) in text_to_draw {
                        let _ = text.paint(offset, TEXT_HEIGHT, cx);
                    }
                })
                .size_full(),
            )
            .on_mouse_down(
                gpui::MouseButton::Middle,
                cx.listener(|this, ev: &gpui::MouseDownEvent, _| {
                    if this.last_pos.is_some() {
                        return;
                    }
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
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, ev: &gpui::MouseDownEvent, _| {
                    if this.last_pos.is_some() || !ev.modifiers.control {
                        return;
                    }
                    this.last_pos.replace(ev.position);
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, ev: &gpui::MouseUpEvent, _| {
                    let Some(last_pos) = this.last_pos else {
                        return;
                    };
                    let delta = ev.position - last_pos;
                    this.start += delta;
                    this.last_pos = None;
                }),
            )
            .on_scroll_wheel(cx.listener(|this, ev: &gpui::ScrollWheelEvent, cx| {
                if this.last_pos.is_some() {
                    return;
                }
                let delta = match ev.delta {
                    gpui::ScrollDelta::Pixels(p) => p,
                    gpui::ScrollDelta::Lines(p) => point(px(p.x * -10.), px(p.y * -10.)),
                };
                this.start += delta;
                cx.notify();
            }))
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
