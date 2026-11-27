//! Shared FM Bridge controls (JS `fmButton` / `A.bridgeTactic`): quiet status refresh, the
//! "… in FM" buttons (connected: primary + action; not connected: outline, opens Settings and the
//! tooltip says what is missing) and the off-thread tactic write.

use crate::state::{BackendBridgeTransport, NavSection, ToastKind};
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::Sizable;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use ta_core::data::l;
use ta_core::live::{ApplyTacticOptions, FmbClient};
use ta_core::types::{BridgeStatus, Lang};

pub fn bridge_ready(st: &BridgeStatus) -> bool {
    st.ok && st.ready
}

/// Why the bridge cannot be used right now (tooltip of a disconnected "… in FM" button).
pub fn bridge_off_reason(st: &BridgeStatus, lang: Lang) -> &'static str {
    if st.ok {
        l(
            lang,
            "O FM está aberto mas ainda não carregou um jogo.",
            "FM is open but has not loaded a game yet.",
        )
    } else {
        l(
            lang,
            "Precisa do plugin Tactical Bible Bridge dentro do FM (Definições › Integrações).",
            "Needs the Tactical Bible Bridge plugin inside FM (Settings › Integrations).",
        )
    }
}

/// "… in FM" button. Ready: primary, `desc` tooltip, runs `on_ready`. Otherwise: outline, the
/// missing-piece tooltip, and a click opens Settings (where the plugin is installed / checked).
pub fn fm_button(
    app: &ArchitectApp,
    id: &'static str,
    label: &str,
    desc: &str,
    running: bool,
    on_ready: impl Fn(&mut ArchitectApp, &mut Context<ArchitectApp>) + 'static,
    cx: &mut Context<ArchitectApp>,
) -> Button {
    let st = &app.state.bridge_status;
    let ready = bridge_ready(st);
    let tip = if ready {
        desc.to_string()
    } else {
        bridge_off_reason(st, app.state.lang).to_string()
    };
    Button::new(id)
        .small()
        .label(format!("● {label}"))
        .when(ready, |b| b.primary())
        .when(!ready, |b| b.outline())
        .loading(running)
        .tooltip(tip)
        .on_click(cx.listener(move |this, _, _, cx| {
            if bridge_ready(&this.state.bridge_status) {
                on_ready(this, cx);
            } else {
                this.section = NavSection::Settings;
                cx.notify();
            }
        }))
}

/// Re-reads the bridge status off-thread without toasts; re-renders only when it changed.
pub fn spawn_refresh_bridge_status(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    cx.spawn(async move |this, cx| {
        let st = cx
            .background_executor()
            .spawn(async move { FmbClient::new(BackendBridgeTransport, lang).status(true) })
            .await;
        let _ = this.update(cx, |this, cx| {
            if this.state.bridge_status != st {
                this.state.bridge_status = st;
                cx.notify();
            }
        });
    })
    .detach();
}

/// Writes `state.tactic` into FM (active tactic, or a new one when `create`), off-thread.
pub fn spawn_bridge_tactic(app: &mut ArchitectApp, create: bool, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    if app.analyse_ui.bridge_running {
        return;
    }
    if app.state.squad.len() < 11 {
        app.state.set_toast(
            l(
                lang,
                "Precisa de pelo menos 11 jogadores no plantel",
                "You need at least 11 players in the squad",
            ),
            ToastKind::Warn,
        );
        cx.notify();
        return;
    }
    let tactic = app.state.tactic.clone();
    let squad = app.state.squad.clone();
    app.analyse_ui.bridge_running = true;
    app.state.set_busy(l(lang, "A aplicar a tática no FM…", "Applying the tactic in FM…"), None);
    cx.notify();

    cx.spawn(async move |this, cx| {
        let res = cx
            .background_executor()
            .spawn(async move {
                let mut client = FmbClient::new(BackendBridgeTransport, lang);
                client.apply_tactic(&tactic, &squad, ApplyTacticOptions { create }, |_| {})
            })
            .await;
        let _ = this.update(cx, |this, cx| {
            this.analyse_ui.bridge_running = false;
            this.state.clear_busy();
            let lang = this.state.lang;
            match res {
                Ok(done) => {
                    this.analyse_ui.tac_done = Some(done);
                    this.state.set_toast(
                        l(lang, "Tática aplicada no FM", "Tactic applied in FM"),
                        ToastKind::Success,
                    );
                }
                Err(err) => this.state.set_toast(err.to_string(), ToastKind::Error),
            }
            spawn_refresh_bridge_status(this, cx);
            cx.notify();
        });
    })
    .detach();
}
