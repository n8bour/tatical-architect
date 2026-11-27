//! Tactic › Import .fmf: inspects an FM27 tactic file (container, streams, strings,
//! readable role/instruction/mentality hints), exports its strings, and creates a
//! fresh tactic from the file name + mentality hint.

use crate::state::{NavSection, TacticTab, ToastKind};
use crate::views::team::apply_tactic_to_app;
use crate::widgets::common::{badge_pill, BadgeTone};
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::*;
use std::path::PathBuf;
use ta_backend::{pick_open_file, pick_save_file};
use ta_core::data::l;
use ta_core::engine::hungarian::new_tactic;
use ta_core::engine::parsers::inspect_fmf;
use ta_core::types::{FmfInspectReport, Lang, Mentality};

/// View-local state for Tactic › Import.
#[derive(Debug, Clone, Default)]
pub struct ImportViewState {
    /// File name currently being analysed.
    pub loading: Option<String>,
    pub report: Option<FmfInspectReport>,
    /// `(file name, error)` of the last failed read.
    pub error: Option<(String, String)>,
    pub show_strings: bool,
}

/// Reads and inspects a `.fmf` off-thread, storing the report in `app.import_ui`.
pub fn spawn_inspect_fmf(app: &mut ArchitectApp, path: PathBuf, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tactic.fmf".into());
    app.import_ui = ImportViewState { loading: Some(name.clone()), ..Default::default() };
    app.state.set_busy(format!("{}{}", l(lang, "A analisar ", "Analysing "), name), None);
    cx.notify();
    cx.spawn(async move |this, cx| {
        let task_name = name.clone();
        let res = cx
            .background_executor()
            .spawn(async move { std::fs::read(&path).map(|buf| inspect_fmf(&task_name, &buf)) })
            .await;
        let _ = this.update(cx, |this, cx| {
            this.state.clear_busy();
            this.import_ui.loading = None;
            match res {
                Ok(r) => this.import_ui.report = Some(r),
                Err(e) => this.import_ui.error = Some((name, e.to_string())),
            }
            cx.notify();
        });
    })
    .detach();
}

fn stem(name: &str) -> String {
    let low = name.to_lowercase();
    if low.ends_with(".fmf") {
        name[..name.len() - 4].to_string()
    } else {
        name.to_string()
    }
}

fn export_strings(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    let Some(r) = &app.import_ui.report else { return };
    let default_name = format!("{}.strings.txt", r.name);
    let Some(path) = pick_save_file(
        l(lang, "Exportar textos", "Export strings"),
        &default_name,
        &[("Text", &["txt"])],
    ) else {
        return;
    };
    match std::fs::write(&path, r.strings.join("\n")) {
        Ok(()) => app.state.set_toast(
            format!("{} {}", l(lang, "Textos exportados para", "Strings exported to"), path.display()),
            ToastKind::Success,
        ),
        Err(e) => app.state.set_toast(e.to_string(), ToastKind::Error),
    }
    cx.notify();
}

fn apply_hints(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    let Some(r) = &app.import_ui.report else { return };
    let ment = r.matches.mentality.iter().find_map(|m| Mentality::from_name(m));
    let name = stem(&r.name);
    let mut t = new_tactic(&app.state.tactic.ip_form, &app.state.tactic.oop_form, lang);
    if let Some(m) = ment {
        t.ment = m;
    }
    t.name = name;
    t.source = "fmf".into();
    apply_tactic_to_app(app, t, true, cx);
    app.selected_slot = None;
    app.state.set_toast(
        l(
            lang,
            "Tática criada — escolha formações, papéis e instruções como no jogo.",
            "Tactic created — pick formations, roles and instructions as in the game.",
        ),
        ToastKind::Success,
    );
    cx.notify();
}

fn chips(label: &str, items: &[String], cx: &App) -> Div {
    div()
        .h_flex()
        .flex_wrap()
        .items_center()
        .gap_1()
        .text_xs()
        .child(div().font_semibold().text_color(cx.theme().foreground).child(format!("{label}:")))
        .children(items.iter().map(|x| badge_pill(x.clone(), BadgeTone::Muted, cx)))
}

fn kv(k: &str, v: String, cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .h_flex()
        .gap_3()
        .text_xs()
        .child(div().w(px(170.0)).text_color(theme.muted_foreground).child(k.to_string()))
        .child(div().flex_1().font_semibold().text_color(theme.foreground).child(v))
}

fn report_view(r: &FmfInspectReport, show_strings: bool, lang: Lang, cx: &mut Context<ArchitectApp>) -> Div {
    let theme = cx.theme().clone();
    let stem_name = stem(&r.name);
    let streams = if r.streams.is_empty() {
        l(lang, "nenhum", "none").to_string()
    } else {
        r.streams
            .iter()
            .map(|s| format!("{}@{} → {}B", s.fmt, s.off, s.len))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut out = div()
        .v_flex()
        .gap_2()
        .child(div().text_sm().font_semibold().text_color(theme.foreground).child(r.name.clone()))
        .child(kv(l(lang, "Tamanho", "Size"), format!("{} bytes", r.size), cx))
        .child(kv(
            l(lang, "Entropia", "Entropy"),
            format!(
                "{:.2} bits/byte {}",
                r.entropy,
                if r.entropy > 7.5 { l(lang, "(comprimido/cifrado)", "(compressed/encrypted)") } else { "" }
            ),
            cx,
        ))
        .child(kv(l(lang, "Cabeçalho", "Header"), r.head.clone(), cx))
        .child(kv("ASCII", r.head_ascii.clone(), cx))
        .child(kv(l(lang, "Streams descomprimidos", "Decompressed streams"), streams, cx))
        .child(kv(
            l(lang, "Outras assinaturas", "Other signatures"),
            if r.magic.is_empty() { "—".into() } else { r.magic.join(", ") },
            cx,
        ))
        .child(kv(l(lang, "Textos encontrados", "Strings found"), r.strings.len().to_string(), cx));

    let create_label = format!(
        "{} “{}” {}",
        l(lang, "Criar", "Create"),
        stem_name,
        l(lang, "no analisador", "in the analyser")
    );
    if r.container.recognized {
        let c = &r.container;
        let secs = c
            .sections
            .iter()
            .map(|x| {
                format!(
                    "{} {}B{}",
                    x.name,
                    x.len,
                    if x.kind == "zstd" { l(lang, " — índice zstd", " — zstd index") } else { "" }
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let blocks = c.blocks.iter().map(|b| format!("{}B", b.cipher)).collect::<Vec<_>>().join(", ");
        let largest = c.largest.map(|b| b.cipher).unwrap_or(0);
        out = out.child(
            div()
                .v_flex()
                .gap_2()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(theme.border)
                .bg(theme.muted)
                .text_xs()
                .text_color(theme.foreground)
                .child(div().font_semibold().child(l(lang, "✓ Formato .fmf do FM27 reconhecido.", "✓ FM27 .fmf format recognised.")))
                .child(format!(
                    "{} {} ({secs}) {} {} {} ({blocks}). {} ({largest}B {}) {}",
                    c.sections.len(),
                    l(lang, "secções", "sections"),
                    l(lang, "e", "and"),
                    c.blocks.len(),
                    l(lang, "ficheiros internos cifrados", "encrypted inner files"),
                    l(lang, "O maior", "The largest"),
                    l(lang, "cifrados", "encrypted"),
                    l(lang, "é a tática.", "is the tactic."),
                ))
                .child(div().child(l(
                    lang,
                    "Resultado: esta app não lê o conteúdo — recrie a tática no analisador a partir do nome do ficheiro.",
                    "Result: this app can't read the content — recreate the tactic in the analyser from the file name.",
                )))
                .child(
                    div().child(
                        Button::new("fmf-create")
                            .primary()
                            .small()
                            .label(create_label)
                            .on_click(cx.listener(|this, _, _, cx| apply_hints(this, cx))),
                    ),
                ),
        );
    }

    out = out.child(div().pt_2().text_sm().font_semibold().text_color(theme.foreground).child(l(
        lang,
        "Texto legível",
        "Readable text",
    )));
    let m = &r.matches;
    if r.decoded || !m.mentality.is_empty() || !m.formations.is_empty() {
        if !m.roles.is_empty() {
            out = out.child(chips(l(lang, "Papéis", "Roles"), &m.roles, cx));
        }
        if !m.instructions.is_empty() {
            out = out.child(chips(l(lang, "Instruções", "Instructions"), &m.instructions, cx));
        }
        if !m.mentality.is_empty() {
            out = out.child(chips(l(lang, "Mentalidade", "Mentality"), &m.mentality, cx));
        }
        if !m.formations.is_empty() {
            out = out.child(chips(l(lang, "Formações", "Formations"), &m.formations, cx));
        }
        out = out
            .child(div().text_xs().text_color(theme.muted_foreground).child(l(
                lang,
                "Os nomes encontrados indicam o que a tática contém, mas não a que posição pertencem. Use-os como guia para recriar a tática no analisador.",
                "The names found show what the tactic contains, but not which position they belong to. Use them as a guide to recreate the tactic in the analyser.",
            )))
            .child(
                div().child(
                    Button::new("fmf-hints")
                        .primary()
                        .small()
                        .label(l(lang, "Usar nome/mentalidade e abrir analisador", "Use name/mentality and open analyser"))
                        .on_click(cx.listener(|this, _, _, cx| apply_hints(this, cx))),
                ),
            );
    } else {
        out = out.child(div().text_xs().text_color(theme.muted_foreground).child(l(
            lang,
            "Nenhum nome de papel/instrução legível no ficheiro.",
            "No readable role/instruction names in the file.",
        )));
    }

    out = out.child(
        div()
            .h_flex()
            .gap_2()
            .pt_2()
            .child(
                Button::new("fmf-strings")
                    .outline()
                    .small()
                    .label(l(lang, "Exportar textos (.txt)", "Export strings (.txt)"))
                    .on_click(cx.listener(|this, _, _, cx| export_strings(this, cx))),
            )
            .child(
                Button::new("fmf-show-strings")
                    .ghost()
                    .small()
                    .label(if show_strings {
                        l(lang, "Esconder textos", "Hide strings")
                    } else {
                        l(lang, "Ver primeiros 200 textos", "Show first 200 strings")
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.import_ui.show_strings = !this.import_ui.show_strings;
                        cx.notify();
                    })),
            ),
    );
    if show_strings {
        let text = r.strings.iter().take(200).cloned().collect::<Vec<_>>().join("\n");
        out = out.child(
            div()
                .id("fmf-strings-list")
                .max_h(px(300.0))
                .overflow_y_scroll()
                .p_2()
                .rounded_md()
                .bg(theme.muted)
                .font_family("monospace")
                .text_xs()
                .text_color(theme.foreground)
                .child(text),
        );
    }
    out
}

/// Renders Tactic › Import.
pub fn render_import(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> impl IntoElement {
    let lang = app.state.lang;
    let theme = cx.theme().clone();
    let panel = || {
        div()
            .v_flex()
            .flex_1()
            .min_w(px(360.0))
            .gap_3()
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .bg(theme.secondary)
    };

    let left = panel()
        .child(
            div()
                .v_flex()
                .items_center()
                .gap_2()
                .p_6()
                .rounded_lg()
                .border_2()
                .border_dashed()
                .border_color(theme.border)
                .child(div().text_sm().font_semibold().text_color(theme.foreground).child(l(
                    lang,
                    "Arraste o .fmf para a janela ou",
                    "Drop the .fmf onto the window or",
                )))
                .child(
                    Button::new("fmf-pick")
                        .primary()
                        .small()
                        .label(l(lang, "Escolher ficheiro", "Choose file"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            let lang = this.state.lang;
                            if let Some(p) = pick_open_file(
                                l(lang, "Escolher tática .fmf", "Choose .fmf tactic"),
                                &[("FM tactic", &["fmf"])],
                            ) {
                                spawn_inspect_fmf(this, p, cx);
                            }
                        })),
                )
                .child(div().text_xs().text_color(theme.muted_foreground).child(l(
                    lang,
                    "Os ficheiros ficam em Documentos/Sports Interactive/Football Manager 27/tactics",
                    "Files live in Documents/Sports Interactive/Football Manager 27/tactics",
                ))),
        )
        .child(
            div()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(rgb(0xd97706))
                .text_xs()
                .text_color(theme.foreground)
                .child(l(
                    lang,
                    "Limite do formato: no FM27, o conteúdo da tática dentro do .fmf está comprimido e cifrado pelo jogo. Esta app não o decifra. O inspector confirma o formato e mostra a estrutura; também pode recriar a tática no analisador (≈1 minuto).",
                    "Format limit: in FM27 the tactic content inside the .fmf is compressed and encrypted by the game. This app doesn't decrypt it. The inspector confirms the format and shows the structure; you can also recreate the tactic in the analyser (≈1 minute).",
                )),
        )
        .child(
            div().h_flex().child(
                Button::new("fmf-manual")
                    .outline()
                    .small()
                    .label(l(lang, "Recriar manualmente →", "Recreate manually →"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.section = NavSection::Tactic;
                        this.tactic_tab = TacticTab::Analyse;
                        cx.notify();
                    })),
            ),
        );

    let st = &app.import_ui;
    let muted = |s: String| {
        div()
            .p_6()
            .text_sm()
            .text_color(theme.muted_foreground)
            .child(s)
    };
    let right = if let Some(name) = &st.loading {
        panel().child(muted(format!("{}{}…", l(lang, "A analisar ", "Analysing "), name)))
    } else if let Some((name, err)) = &st.error {
        panel().child(
            div()
                .text_sm()
                .text_color(rgb(0xef4444))
                .child(format!("{} {name}: {err}", l(lang, "Erro ao ler", "Error reading"))),
        )
    } else if let Some(r) = &st.report {
        panel().child(report_view(r, st.show_strings, lang, cx))
    } else {
        panel().child(muted(l(lang, "Nenhum ficheiro inspeccionado.", "No file inspected yet.").into()))
    };

    div()
        .v_flex()
        .gap_4()
        .child(div().text_base().font_semibold().text_color(theme.foreground).child(l(lang, "Importar", "Import")))
        .child(div().h_flex().flex_wrap().items_start().gap_4().child(left).child(right))
}
