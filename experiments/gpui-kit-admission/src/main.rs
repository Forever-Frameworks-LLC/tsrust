//! Direct GPUI Kit reference for the first Tsrust UI slice: a counter, a
//! controlled text input and a conditional child. The generated `.tsrx` bridge
//! must reproduce this behavior and be compared against it.
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;

struct Slice {
    count: i64,
    name: Entity<InputState>,
    greeting: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl Slice {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("Name"));
        let subscription = cx.subscribe_in(&name, window, |this, state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.greeting = state.read(cx).value().to_string().into();
                cx.notify();
            }
        });
        Self { count: 0, name, greeting: SharedString::default(), _subscriptions: vec![subscription] }
    }
}

impl Render for Slice {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = div().flex().flex_col().size_full().items_center().justify_center().gap_2()
            .child(format!("Count: {}", self.count))
            .child(Button::new("increment").primary().label("Increment")
                .on_click(cx.listener(|this, _, _, cx| { this.count += 1; cx.notify(); })))
            .child(Input::new(&self.name));
        // Conditional child: present only while the controlled input is non-empty.
        if !self.greeting.is_empty() { column = column.child(format!("Hello, {}!", self.greeting)); }
        column
    }
}

fn main() {
    application().with_assets(assets::Assets).run(|cx| {
        init(cx);
        open_window(WindowOptions::default(), cx, |window, cx| cx.new(|cx| Slice::new(window, cx)))
            .expect("Failed to open window");
    });
}
