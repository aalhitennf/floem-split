use floem::{
    event::{Event, EventListener},
    kurbo::Size,
    peniko::Color,
    pointer::PointerMoveEvent,
    reactive::{with_scope, RwSignal, Scope, SignalGet, SignalUpdate},
    style::{CursorStyle, Style},
    style_class,
    unit::{Px, PxPct, PxPctAuto, UnitExt},
    views::{clip, empty, h_stack, v_stack, Decorators, Empty, Stack},
    AnyView, IntoView,
};

style_class!(pub SplitDraggerHorizontalClass);
style_class!(pub SplitDraggerVerticalClass);

#[derive(Clone, Copy)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

pub struct Split {
    cx: Scope,

    a: AnyView,
    b: AnyView,

    min_split: PxPct,
    default_split: PxPct,
    reset_on_dbl_click: bool,
    dynamic: bool,
    orientation: SplitOrientation,

    size: Size,
    split_value: PxPct,
    dragging: bool,

    dragger_size: Px,
    dragger_style: Style,
    dragging_style: Style,
}

impl Split {
    #[must_use]
    pub fn new(a: impl IntoView, b: impl IntoView) -> Self {
        let cx = Scope::current().create_child();

        Self {
            cx,

            a: a.into_any(),
            b: b.into_any(),

            min_split: 50.0.px().into(),
            default_split: PxPct::Pct(50.0),
            reset_on_dbl_click: true,
            dynamic: true,
            orientation: SplitOrientation::Horizontal,

            size: Size::ZERO,
            split_value: PxPct::Pct(50.0),
            dragging: false,

            dragger_size: 4.px(),
            dragger_style: Style::new(),
            dragging_style: Style::new(),
        }
    }

    #[must_use]
    /// Minimum size that split can be resized to
    pub fn min_split(mut self, value: impl Into<PxPct>) -> Self {
        self.min_split = value.into();
        self
    }

    #[must_use]
    /// Default size of the split
    pub fn default_split(mut self, value: impl Into<PxPct>) -> Self {
        let value = value.into();
        self.default_split = value;
        self.split_value = value;
        self
    }

    #[must_use]
    /// Should split keep its ratio on area resize i.e. window resize
    pub fn dynamic(mut self, value: bool) -> Self {
        self.dynamic = value;
        self
    }

    #[must_use]
    /// Horizontal
    pub fn horizontal(mut self) -> Self {
        self.orientation = SplitOrientation::Horizontal;
        self
    }

    #[must_use]
    /// Horizontal
    pub fn vertical(mut self) -> Self {
        self.orientation = SplitOrientation::Vertical;
        self
    }

    #[must_use]
    /// Disable reset on double click
    pub fn disable_reset(mut self) -> Self {
        self.reset_on_dbl_click = false;
        self
    }

    #[must_use]
    // Shortcut for dragger size
    pub fn dragger_size(mut self, size: impl Into<Px>) -> Self {
        self.dragger_size = size.into();
        self
    }

    #[must_use]
    /// Customize dragger style
    pub fn dragger_style(mut self, f: impl FnOnce(Style) -> Style) -> Self {
        self.dragger_style = f(self.dragger_style);
        self
    }

    #[must_use]
    /// Customize dragger style when dragging
    pub fn dragging_style(mut self, f: impl FnOnce(Style) -> Style) -> Self {
        self.dragging_style = f(self.dragging_style);
        self
    }
}

impl IntoView for Split {
    type V = Stack;

    fn into_view(self) -> Self::V {
        let cx = self.cx;

        let min_split = self.min_split;
        let default_split = self.default_split;
        let reset_on_dbl_click = self.reset_on_dbl_click;
        let dynamic = self.dynamic;

        let size = cx.create_rw_signal(self.size);
        let split_value = cx.create_rw_signal(self.split_value);
        let dragging = cx.create_rw_signal(self.dragging);

        let orientation = self.orientation;

        let views = (self.a, self.b);

        let dragger_size = self.dragger_size;

        let dragger_style =
            build_dragger_style(dragging, dragger_size, orientation).apply(self.dragger_style);

        let dragging_style = self.dragging_style;

        let mut dragger = dragger(false, dragging).style(move |s| {
            let ds = dragger_style.clone();
            let drs = dragging_style.clone();
            s.apply(ds).apply_if(dragging.get(), move |s| s.apply(drs))
        });

        if reset_on_dbl_click {
            dragger = dragger.on_double_click_stop(move |_| {
                split_value.set(default_split);
            });
        }

        with_scope(cx, || {
            if matches!(orientation, SplitOrientation::Horizontal) {
                build_split_h(
                    views,
                    size,
                    split_value,
                    dragging,
                    min_split,
                    dynamic,
                    dragger,
                )
            } else {
                build_split_v(
                    views,
                    size,
                    split_value,
                    dragging,
                    min_split,
                    dynamic,
                    dragger,
                )
            }
        })
    }
}

fn build_dragger_style(dragging: RwSignal<bool>, size: Px, orientation: SplitOrientation) -> Style {
    Style::new()
        .apply_if(matches!(orientation, SplitOrientation::Horizontal), |s| {
            s.min_width(size)
                .width(size)
                .height_full()
                .cursor(CursorStyle::ColResize)
        })
        .apply_if(matches!(orientation, SplitOrientation::Vertical), |s| {
            s.min_height(size)
                .height(size)
                .width_full()
                .cursor(CursorStyle::RowResize)
        })
        .background(Color::rgb8(205, 205, 205))
        .hover(|s| {
            s.background(Color::rgb8(41, 98, 218))
                .border_color(Color::rgb8(41, 98, 218))
        })
        .apply_if(dragging.get(), |s| {
            s.border_color(Color::rgb8(41, 98, 218))
                .background(Color::rgb8(41, 98, 218))
        })
}

fn build_split_h(
    (a, b): (impl IntoView + 'static, impl IntoView + 'static),
    area_size: RwSignal<Size>,
    width: RwSignal<PxPct>,
    dragging: RwSignal<bool>,
    min_split: PxPct,
    dynamic: bool,
    dragger: Empty,
) -> Stack {
    let a_con = clip(a).style(move |s| s.min_width(min_split).width(width.get()));
    let b_con = clip(b).style(move |s| {
        let rs = area_size.get();
        let w = px_w(rs.width, width.get());
        let b_pct = ((w / rs.width) * 100.0).abs();
        let b_con_w = PxPctAuto::Pct(100.0 - b_pct);
        s.min_width(min_split).width(b_con_w)
    });

    h_stack((a_con, dragger, b_con))
        .on_resize(move |rect| {
            area_size.set(rect.size());
        })
        .on_event_stop(EventListener::DragOver, move |e| {
            if let Event::PointerMove(PointerMoveEvent { pos, .. }) = e {
                if dragging.get() {
                    if dynamic {
                        let pct = (pos.x / area_size.get().width) * 100.0;
                        width.set(PxPct::Pct(pct));
                    } else {
                        width.set(PxPct::Px(pos.x));
                    }
                }
            }
        })
        .style(Style::size_full)
}

fn build_split_v(
    (a, b): (impl IntoView + 'static, impl IntoView + 'static),
    area_size: RwSignal<Size>,
    height: RwSignal<PxPct>,
    dragging: RwSignal<bool>,
    min_split: PxPct,
    dynamic: bool,
    dragger: Empty,
) -> Stack {
    let a_con = clip(a).style(move |s| s.min_height(min_split).height(height.get()));
    let b_con = clip(b).style(move |s| {
        let rs = area_size.get();
        let w = px_w(rs.height, height.get());
        let b_pct = ((w / rs.height) * 100.0).abs();
        let b_con_w = PxPctAuto::Pct(100.0 - b_pct);
        s.min_height(min_split).height(b_con_w)
    });

    v_stack((a_con, dragger, b_con))
        .on_resize(move |rect| {
            area_size.set(rect.size());
        })
        .on_event_stop(EventListener::DragOver, move |e| {
            if let Event::PointerMove(PointerMoveEvent { pos, .. }) = e {
                if dragging.get() {
                    if dynamic {
                        let pct = (pos.y / area_size.get().height) * 100.0;
                        height.set(PxPct::Pct(pct));
                    } else {
                        height.set(PxPct::Px(pos.y));
                    }
                }
            }
        })
        .style(Style::size_full)
}

fn dragger(horizontal: bool, dragging: RwSignal<bool>) -> Empty {
    empty()
        .on_event_stop(EventListener::DragStart, move |_| {
            dragging.set(true);
        })
        .on_event_stop(EventListener::DragEnd, move |_| {
            dragging.set(false);
        })
        .on_event_stop(EventListener::DoubleClick, move |_| {
            dragging.set(false);
        })
        .class_if(move || horizontal, SplitDraggerHorizontalClass)
        .class_if(move || !horizontal, SplitDraggerVerticalClass)
        .draggable()
        .dragging_style(|s| {
            s.background(Color::TRANSPARENT)
                .border(0)
                .box_shadow_color(Color::TRANSPARENT)
        })
}

#[inline]
fn px_w(width: f64, w: PxPct) -> f64 {
    match w {
        PxPct::Pct(p) => width * (p / 100.0),
        PxPct::Px(p) => p,
    }
}

/// Create vertical (top|bottom) split with default values
pub fn v_split(a: impl IntoView, b: impl IntoView) -> Split {
    Split::new(a, b).vertical()
}

/// Create horizontal (left|right) split with default values
pub fn h_split(a: impl IntoView, b: impl IntoView) -> Split {
    Split::new(a, b)
}
