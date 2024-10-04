use floem::{
    event::{Event, EventListener},
    keyboard::{KeyCode, KeyEvent, PhysicalKey},
    peniko::Color,
    style::Style,
    unit::UnitExt,
    views::{container, text, Decorators},
    AnyView, IntoView, View,
};
use floem_split::{h_split, v_split, SplitDraggerVerticalClass};

fn main() {
    floem::launch(main_view);
}

fn main_view() -> impl IntoView {
    fn centered_textbox(txt: &str) -> AnyView {
        container(text(txt))
            .style(|s| s.size_full())
            .style(|s| s.justify_center().items_center().size_full())
            .into_any()
    }

    let top = centered_textbox("top\ndefault size 35%\nmin height 50px\ndragger height 10px");

    let nested_btm_left = centered_textbox("nested btm_left\ndefault size 35%\nmin width 25%");
    let nested_btm_right = centered_textbox("nested btm_right\nmin width 25%");

    let btm_left = centered_textbox("btm_left\nmin width 100px");

    let nested_btm = h_split(nested_btm_left, nested_btm_right)
        .min_split(25.pct())
        .default_split(35.pct())
        .dynamic(false)
        .dragger_style(|s| {
            s.background(Color::FIREBRICK)
                .hover(|s| s.background(Color::MEDIUM_SEA_GREEN))
        });

    let btm = h_split(btm_left, nested_btm)
        .min_split(100)
        .dragging_style(|s| s.background(Color::PALE_GOLDENROD));

    let main_split = v_split(top, btm)
        .default_split(15.pct())
        .min_split(50)
        .dynamic(false)
        // You can customize dragger of specific split
        .dragger_style(|s| {
            s.background(Color::LAWN_GREEN)
                .height(20)
                .hover(|s| s.background(Color::MINT_CREAM))
        })
        // Style when dragging is also customizable
        .dragging_style(|s| s.background(Color::HOT_PINK))
        .style(|s| s.size_full());

    let view = container(main_split)
        .keyboard_navigatable()
        .style(Style::size_full);

    let id = view.id();

    view.on_event_cont(EventListener::KeyDown, move |e| {
        if let Event::KeyDown(KeyEvent { key, .. }) = e {
            if !key.repeat && key.physical_key == PhysicalKey::Code(KeyCode::F11) {
                id.inspect();
            }
        }
    })
    .style(|s| {
        s.background(Color::parse("#212223").unwrap())
            .color(Color::parse("#929292").unwrap())
            .border(4.px())
            .border_color(Color::rgb8(205, 205, 205))
            .class(SplitDraggerVerticalClass, |s| {
                s.background(Color::REBECCA_PURPLE)
                    .hover(|s| s.background(Color::REBECCA_PURPLE))
            })
    })
}
