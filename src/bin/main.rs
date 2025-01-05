use gpui::{
    div, prelude::*, App, AppContext, Context, ModelContext, Render, View, ViewContext,
    WindowOptions,
};
use graphic::{layout::GraphLayout, GraphViewer};
struct Grphic {
    graph_viewer: View<GraphViewer>,
}

impl Grphic {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            graph_viewer: cx.new_view(|_| GraphViewer::new()),
        }
    }
}
impl Render for Grphic {
    fn render(&mut self, _: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .font_family(".SystemUIFont")
            .bg(gpui::white())
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .justify_between()
                    .items_center()
                    .child("Hold middle mouse move viewport"),
            )
            .child(self.graph_viewer.clone())
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(
            WindowOptions {
                focus: true,
                ..Default::default()
            },
            |cx| cx.new_view(Grphic::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
