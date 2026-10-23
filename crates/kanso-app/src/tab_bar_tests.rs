//! The tab bar's scrolling row: the active tab is brought into view, and
//! the row scrolls back when a tab at its start is chosen.

use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, ParentElement as _, Render, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, point, px, size,
};

use crate::tab_bar::{TAB_BAR_HEIGHT, TabBar};
use crate::tabs::TabMeta;

struct Harness {
    bar: Entity<TabBar>,
}

impl Render for Harness {
    fn render(
        &mut self,
        _: &mut Window,
        _: &mut gpui_kit::Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        div()
            .w(px(360.))
            .h(px(TAB_BAR_HEIGHT))
            .child(self.bar.clone())
    }
}

fn setup(
    cx: &mut TestAppContext,
    count: usize,
    active: usize,
) -> (AnyWindowHandle, Entity<TabBar>) {
    let (window, harness) = cx.update(|cx| {
        gpui_kit::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                point(px(0.), px(0.)),
                size(px(360.), px(80.)),
            ))),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| {
                let bar = cx.new(|cx| {
                    let mut bar = TabBar::default();
                    let tabs: Vec<TabMeta> = (0..count)
                        .map(|n| TabMeta {
                            id: format!("note:{n}"),
                            title: format!("Note number {n}"),
                            locked: false,
                        })
                        .collect();
                    bar.set(tabs, active, window, cx);
                    bar
                });
                window.refresh();
                Harness { bar }
            })
        })
        .unwrap()
    });
    cx.run_until_parked();
    let bar = cx.read(|cx| harness.read(cx).bar.clone());
    (window, bar)
}

fn draw_window(cx: &mut TestAppContext, window: AnyWindowHandle) {
    for _ in 0..3 {
        window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                window.simulate_next_frame(cx);
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }
}

#[gpui_kit::test]
fn the_active_tab_is_scrolled_into_view(cx: &mut TestAppContext) {
    let (window, bar) = setup(cx, 20, 19);
    draw_window(cx, window);
    let offset = cx.read(|cx| bar.read(cx).scroll_offset());
    let bounds = cx.read(|cx| bar.read(cx).scroll_bounds());
    let last = cx
        .read(|cx| bar.read(cx).scroll_bounds_for_item(19))
        .expect("the last tab is laid out");
    assert!(offset.x < px(0.), "the row scrolls right: {offset:?}");
    assert!(
        last.left() + offset.x >= bounds.left() && last.right() + offset.x <= bounds.right(),
        "the active tab is in view: {last:?} + {offset:?} in {bounds:?}"
    );
}

#[gpui_kit::test]
fn choosing_the_first_tab_scrolls_back_to_it(cx: &mut TestAppContext) {
    let (window, bar) = setup(cx, 20, 19);
    draw_window(cx, window);
    window
        .update(cx, |_, window, cx| {
            bar.update(cx, |bar, cx| bar.activate_for_test(0, window, cx))
        })
        .unwrap();
    draw_window(cx, window);
    let offset = cx.read(|cx| bar.read(cx).scroll_offset());
    assert!(
        offset.x >= px(0.),
        "the row is back at the start: {offset:?}"
    );
}
