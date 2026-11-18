mod data;
mod row;

use gpui_kit::component::button::Button;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::{ActiveTheme, h_flex, v_flex};
use gpui_kit::*;
use gpui_kit::base::StyledExt;
use std::time::Duration;

/// BENCH_MODE selects an automated run: startup quits after the first tick,
/// idle quits after 10 seconds, animate refilters the list every 16 ms for 600 ticks.
struct Compare {
    items: Vec<String>,
    query: String,
    frame: u64,
    input: Entity<InputState>,
    _subscription: Subscription,
    _ticker: Option<Task<()>>,
}

impl Compare {
    fn new(mode: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
        let subscription = cx.subscribe_in(&input, window, |this: &mut Self, state, event: &InputEvent, _, cx| {
            if let InputEvent::Change = event {
                this.query = state.read(cx).value().to_string();
                cx.notify();
            }
        });
        let ticker = (!mode.is_empty()).then(|| {
            let interval = Duration::from_millis(if mode == "idle" { 10_000 } else { 16 });
            cx.spawn_in(window, async move |this, cx| {
                let mut tick: u64 = 0;
                loop {
                    cx.background_executor().timer(interval).await;
                    tick += 1;
                    let running = this.update_in(cx, |this, window, cx| this.tick(&mode, tick, window, cx));
                    if !matches!(running, Ok(true)) {
                        break;
                    }
                }
            })
        });
        Self { items: data::make_items(2000), query: String::new(), frame: 0, input, _subscription: subscription, _ticker: ticker }
    }

    fn tick(&mut self, mode: &str, tick: u64, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if mode != "animate" || tick >= 600 {
            cx.quit();
            return false;
        }
        self.frame = tick;
        self.query = (tick % 10).to_string();
        let query = self.query.clone();
        self.input.update(cx, |state, cx| state.set_value(query, window, cx));
        cx.notify();
        true
    }
}

impl Render for Compare {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let visible = data::filter_items(&self.items, &self.query);
        let header = format!("Showing {} items, frame {}", visible.len(), self.frame);
        let rows = visible.iter().map(|item| div().flex_shrink_0().child(row::row(item, cx))).collect::<Vec<_>>();
        v_flex()
            .size_full()
            .p_4()
            .gap_3()
            .bg(cx.theme().background)
            .child(div().text_lg().font_semibold().child(header))
            .child(
                h_flex().gap_2().child(div().flex_1().child(Input::new(&self.input))).child(
                    Button::new("add").label("Add item").on_click(cx.listener(|this, _, _, cx| {
                        let next = format!("Item {}", this.items.len());
                        this.items.push(next);
                        cx.notify();
                    })),
                ),
            )
            .child(div().flex_1().min_h_0().child(div().id("list").size_full().children(rows).overflow_y_scrollbar()))
    }
}

fn main() {
    let mode = std::env::var("BENCH_MODE").unwrap_or_default();
    gpui_kit::application().run(move |cx| {
        gpui_kit::init(cx);
        let bounds = Bounds::centered(None, size(px(900.), px(700.)), cx);
        let options = WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() };
        gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| Compare::new(mode, window, cx))).expect("open window");
    });
}
