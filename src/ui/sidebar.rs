use super::shell::Shell;
use super::{View, TogglePalette, ToggleSidebar};
use gpui_kit::{div, prelude::*, rems, Context, Div, SharedString};
use gpui_omarchy::{button, icon, keycap, ActiveTheme as _, ButtonVariant, IconName};

fn item(
    id: impl Into<SharedString>,
    icon_name: IconName,
    label: &'static str,
    shortcut: &'static str,
    active: bool,
    collapsed: bool,
    cx: &mut Context<Shell>,
) -> gpui_omarchy::Button {
    let id: SharedString = id.into();
    let t = cx.omarchy().clone();
    let variant = if active {
        ButtonVariant::Primary
    } else {
        ButtonVariant::Secondary
    };
    button(id, "", variant, cx)
        .accessibility_label(label)
        .w_full()
        .justify_start()
        .child(icon(icon_name).size(rems(1.)))
        .when(!collapsed, |b| {
            b.child(div().flex_1().child(label))
                .child(div().text_color(t.secondary).child(keycap(shortcut, cx)))
        })
}

pub fn sidebar(shell: &Shell, cx: &mut Context<Shell>) -> Div {
    let t = cx.omarchy().clone();
    let collapsed = shell.collapsed();
    let current = shell.view();

    let mut nav = Vec::new();
    for view in View::ALL {
        nav.push(
            item(
                format!("nav-{}", view.label()),
                view.icon(),
                view.label(),
                view.shortcut(),
                current == view,
                collapsed,
                cx,
            )
            .on_click(cx.listener(move |this, _, window, cx| this.navigate(view, window, cx))),
        );
    }

    let command = item(
        "nav-command",
        IconName::Command,
        "Command",
        "Ctrl+K",
        false,
        collapsed,
        cx,
    )
    .on_click(cx.listener(|_, _, window, cx| {
        window.dispatch_action(Box::new(TogglePalette), cx);
    }));

    let collapse = item(
        "nav-collapse",
        if collapsed {
            IconName::PanelLeftOpen
        } else {
            IconName::PanelLeftClose
        },
        "Collapse",
        "Ctrl+B",
        false,
        collapsed,
        cx,
    )
    .on_click(cx.listener(|_, _, window, cx| {
        window.dispatch_action(Box::new(ToggleSidebar), cx);
    }));

    div()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .w(if collapsed { rems(3.25) } else { rems(15.) })
        .h_full()
        .gap(rems(0.25))
        .p(rems(0.5))
        .border_r_1()
        .border_color(t.border)
        .bg(t.surface)
        .font_family(t.font.clone())
        .text_size(rems(0.75))
        .text_color(t.foreground)
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .py(rems(0.75))
                .text_color(t.accent)
                .font_weight(gpui_kit::FontWeight::BOLD)
                .child(if collapsed { "Ch" } else { "115  Ch" }),
        )
        .children(nav)
        .child(div().flex_1())
        .child(command)
        .child(div().h(rems(0.0625)).bg(t.border))
        .child(collapse)
}
