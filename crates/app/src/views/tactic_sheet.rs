//! Tactic › FM27 Sheet: step-by-step recreation guide in FM27 screen order (top bar, team shape
//! roles IP/OOP, PIs, TIs by FM section, set pieces), tickable steps, in-game vs Portuguese names,
//! plain-text copy and a printable HTML version (Print/PDF via the default browser).
//!
//! One `Sheet` model feeds both the GPUI view and the printable HTML.

use crate::state::{AppState, TacticTab, ToastKind};
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::collections::{HashMap, HashSet};
use ta_backend::open_path;
use ta_core::data::{
    l, mentality_name, pi_name, pt_terms, role_by_id, role_name, t_name, ti_by_key, ti_name,
    ti_opt_name, ti_val, FM_SECTIONS, TEAM_INSTRUCTIONS,
};
use ta_core::engine::{audit_tactic, set_piece_plan, synergy_phase_name, synergy_report};
use ta_core::types::{Lang, Player, Severity, Tactic};

/// Sheet-tab UI state owned by the root app (`app.sheet_ui`). Ticks last for the session,
/// like the checkboxes of the browser build.
#[derive(Debug, Clone, Default)]
pub struct SheetViewState {
    pub ticked: HashSet<String>,
}

/// Plain-text FM27 sheet (game terms), matching the JS `sheetText`.
pub fn sheet_text(t: &Tactic, lang: Lang) -> String {
    let role = |id: &str| role_by_id(id).map(|r| r.name).unwrap_or("?");
    let mut out = vec![
        t.name.clone(),
        format!("Style: Clean Slate | Mentality: {}", t.ment.as_str()),
        format!("In Possession: {} | Out of Possession: {}", t.ip_form, t.oop_form),
        String::new(),
        l(lang, "PAPÉIS (IP / OOP)", "ROLES (IP / OOP)").to_string(),
    ];
    for s in &t.slots {
        let code = if s.oop_code != s.ip_code {
            format!("{}→{}", s.ip_code, s.oop_code)
        } else {
            s.ip_code.clone()
        };
        let pis = if s.pis.is_empty() {
            String::new()
        } else {
            format!(" [{}]", s.pis.join("; "))
        };
        out.push(format!("{code}: {} / {}{pis}", role(&s.ip), role(&s.oop)));
    }
    out.push(String::new());
    out.push(
        l(lang, "INSTRUÇÕES DE EQUIPA (só as que mudam)", "TEAM INSTRUCTIONS (changes only)").to_string(),
    );
    for sec in &FM_SECTIONS {
        let ch: Vec<String> = sec
            .keys
            .iter()
            .filter_map(|&k| {
                let d = ti_by_key(k)?;
                let v = ti_val(t, k);
                (v != d.def).then(|| format!("{} = {}", d.name, d.opts.get(v).copied().unwrap_or("")))
            })
            .collect();
        if !ch.is_empty() {
            out.push(format!("{}: {}", sec.title, ch.join(", ")));
        }
    }
    out.join("\n")
}

// ============================================================================
// Sheet model
// ============================================================================

/// One step: tick box (when `mark`), label (+ optional note under it), value columns.
pub struct SheetRow {
    pub mark: bool,
    pub label: String,
    pub note: Option<String>,
    pub values: Vec<String>,
    pub strong: bool,
}

pub struct SheetBlock {
    pub title: String,
    /// Section heading (`true`) or sub-heading inside a section (`false`).
    pub major: bool,
    pub intro: Option<String>,
    pub columns: Option<Vec<String>>,
    pub rows: Vec<SheetRow>,
}

pub struct Sheet {
    pub title: String,
    pub intro: String,
    pub left: Vec<SheetBlock>,
    pub right: Vec<SheetBlock>,
}

fn row(mark: bool, label: impl Into<String>, value: impl Into<String>, strong: bool) -> SheetRow {
    SheetRow { mark, label: label.into(), note: None, values: vec![value.into()], strong }
}

fn block(title: impl Into<String>, major: bool) -> SheetBlock {
    SheetBlock { title: title.into(), major, intro: None, columns: None, rows: Vec::new() }
}

/// Builds the sheet. In PT with `sheet_names_pt` off, FM terms stay as they appear in the
/// English game (so the tactic can be recreated without mistakes); on, they are translated.
pub fn build_sheet(state: &AppState) -> Sheet {
    let lang = state.lang;
    let t = &state.tactic;
    let pt_names = lang == Lang::Pt && state.sheet_names_pt;
    let term = |s: &str| if pt_names { pt_terms(s) } else { s.to_string() };
    let pm: HashMap<&str, &Player> = state.squad.iter().map(|p| (p.id.as_str(), p)).collect();
    let has_pis = t.slots.iter().any(|s| !s.pis.is_empty());

    let mut top = block(l(lang, "1. Topo do ecrã de táticas", "1. Top of the tactics screen"), true);
    top.rows = vec![
        row(
            true,
            term("Style"),
            format!(
                "Clean Slate {}",
                l(lang, "(para começar sem instruções pré-definidas)", "(start without preset instructions)")
            ),
            true,
        ),
        row(
            true,
            term("Mentality"),
            if pt_names { mentality_name(t.ment, lang) } else { t.ment.as_str().to_string() },
            true,
        ),
        row(true, term("In Possession"), t.ip_form.clone(), true),
        row(true, term("Out of Possession"), t.oop_form.clone(), true),
    ];

    let mut shape = block(l(lang, "2. Team Shape — papéis", "2. Team Shape — roles"), true);
    shape.columns = Some(vec![
        l(lang, "Posição", "Position").to_string(),
        term("In Possession"),
        term("Out of Possession"),
    ]);
    let rn = |id: &str| {
        role_by_id(id)
            .map(|r| if pt_names { role_name(r, lang) } else { r.name.to_string() })
            .unwrap_or_else(|| "?".into())
    };
    for s in &t.slots {
        let mut pos = s.ip_code.clone();
        if s.oop_code != s.ip_code {
            pos = format!("{pos} → {}", s.oop_code);
        }
        shape.rows.push(SheetRow {
            mark: true,
            label: pos,
            note: s.pid.as_deref().and_then(|id| pm.get(id)).map(|p| p.name.clone()),
            values: vec![rn(&s.ip), rn(&s.oop)],
            strong: true,
        });
    }

    let mut left = vec![top, shape];
    if has_pis {
        let mut pis = block(l(lang, "3. Instruções individuais", "3. Player instructions"), true);
        for s in t.slots.iter().filter(|s| !s.pis.is_empty()) {
            let val = s
                .pis
                .iter()
                .map(|p| if pt_names { pi_name(p, lang) } else { p.clone() })
                .collect::<Vec<_>>()
                .join(" · ");
            pis.rows.push(row(true, s.ip_code.clone(), val, true));
        }
        left.push(pis);
    }

    let changed = TEAM_INSTRUCTIONS.iter().filter(|d| ti_val(t, d.key) != d.def).count();
    let mut tis = block(format!("{}. {}", if has_pis { 4 } else { 3 }, term("Team Instructions")), true);
    tis.intro = Some(format!(
        "{changed} {}",
        l(
            lang,
            "instrução(ões) fora do default, a negrito. As restantes ficam como estão.",
            "instruction(s) changed from default, in bold. Leave the rest as they are.",
        )
    ));
    let mut right = vec![tis];
    for sec in &FM_SECTIONS {
        let mut b = block(term(sec.title), false);
        for &k in sec.keys {
            let Some(d) = ti_by_key(k) else { continue };
            let v = ti_val(t, k);
            let ch = v != d.def;
            let opt = d.opts.get(v).copied().unwrap_or("");
            let (name, val) = if pt_names {
                (ti_name(d, lang), ti_opt_name(opt, lang))
            } else {
                (d.name.to_string(), opt.to_string())
            };
            b.rows.push(row(ch, name, val, ch));
        }
        right.push(b);
    }
    if state.squad.len() >= 11 {
        if let Some(r) = set_piece_plan(t, &state.squad, &state.opponent, lang) {
            let mut b = block(term("Set Pieces & Captains"), false);
            let nm = |x: Option<&str>| x.map(str::to_string).unwrap_or_else(|| "—".into());
            b.rows.push(row(true, term("Captain"), nm(r.caps.first().map(|x| x.player_name.as_str())), true));
            b.rows.push(row(true, term("Vice Captain"), nm(r.caps.get(1).map(|x| x.player_name.as_str())), true));
            b.rows.push(row(true, term("Corners (right)"), r.corner_right.player_name.clone(), true));
            b.rows.push(row(true, term("Corners (left)"), r.corner_left.player_name.clone(), true));
            b.rows.push(row(true, term("Free Kicks (direct)"), nm(r.fk_direct.as_ref().map(|x| x.player_name.as_str())), true));
            b.rows.push(row(true, term("Free Kicks (crossed)"), nm(r.fk_cross.as_ref().map(|x| x.player_name.as_str())), true));
            for (i, x) in r.pens.iter().enumerate() {
                b.rows.push(row(true, format!("{} {}", term("Penalty"), i + 1), x.player_name.clone(), true));
            }
            if let Some(x) = &r.long_throw {
                b.rows.push(row(true, term("Long Throws"), x.player_name.clone(), true));
            }
            for x in r.attack.iter().filter(|x| x.key != "taker") {
                b.rows.push(row(
                    true,
                    format!("{}{}", l(lang, "Canto ofensivo — ", "Attacking corner — "), x.label),
                    x.player_name.clone(),
                    true,
                ));
            }
            for x in &r.defend {
                b.rows.push(row(
                    true,
                    format!("{}{}", l(lang, "Canto defensivo — ", "Defending corner — "), x.label),
                    x.player_name.clone(),
                    true,
                ));
            }
            right.push(b);
        }
    }
    if t.ois > 0 {
        let mut b = block(term("Opposition Instructions"), false);
        b.rows.push(row(
            true,
            term("Opposition Instructions"),
            format!(
                "{} {}",
                t.ois,
                l(lang, "activa(s) — defina-as antes de cada jogo.", "active — set them before each match.")
            ),
            true,
        ));
        right.push(b);
    }

    Sheet {
        title: format!("{} — {}", l(lang, "Ficha para o FM27", "FM27 sheet"), t_name(t, lang)),
        intro: l(
            lang,
            "Recrie a tática no jogo por esta ordem e guarde-a lá: o FM27 gera o .fmf. Marque cada passo à medida que avança.",
            "Recreate the tactic in the game in this order and save it there: FM27 generates the .fmf. Tick each step as you go.",
        )
        .to_string(),
        left,
        right,
    }
}

// ============================================================================
// Print / PDF (printable HTML opened in the default browser)
// ============================================================================

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn blocks_html(blocks: &[SheetBlock], out: &mut String) {
    for b in blocks {
        out.push_str(&format!("<{0}>{1}</{0}>", if b.major { "h2" } else { "h3" }, esc(&b.title)));
        if let Some(i) = &b.intro {
            out.push_str(&format!("<p class=\"muted\">{}</p>", esc(i)));
        }
        if b.rows.is_empty() {
            continue;
        }
        out.push_str("<table>");
        if let Some(cols) = &b.columns {
            out.push_str("<tr><th></th>");
            for c in cols {
                out.push_str(&format!("<th>{}</th>", esc(c)));
            }
            out.push_str("</tr>");
        }
        for r in &b.rows {
            out.push_str(&format!(
                "<tr class=\"{}\"><td class=\"ck\">{}</td><td>{}{}</td>",
                if r.strong { "" } else { "muted" },
                if r.mark { "☐" } else { "" },
                esc(&r.label),
                r.note.as_deref().map(|n| format!("<div class=\"muted\">{}</div>", esc(n))).unwrap_or_default()
            ));
            for v in &r.values {
                if r.strong {
                    out.push_str(&format!("<td><b>{}</b></td>", esc(v)));
                } else {
                    out.push_str(&format!("<td>{}</td>", esc(v)));
                }
            }
            out.push_str("</tr>");
        }
        out.push_str("</table>");
    }
}

/// Printable HTML of the sheet; with `analysis`, the audit findings and synergy summary follow.
pub fn sheet_html(state: &AppState, analysis: bool) -> String {
    let lang = state.lang;
    let sheet = build_sheet(state);
    let mut out = format!(
        "<!doctype html><html lang=\"{}\"><head><meta charset=\"utf-8\"><title>{}</title><style>\
body{{font:13px/1.4 system-ui,-apple-system,Segoe UI,sans-serif;color:#111;margin:24px}}\
h1{{font-size:20px;margin:0 0 4px}}h2{{font-size:15px;margin:18px 0 6px}}h3{{font-size:13px;margin:12px 0 4px;color:#444}}\
table{{border-collapse:collapse;width:100%}}td,th{{border-bottom:1px solid #ddd;padding:3px 6px;text-align:left;vertical-align:top}}\
.ck{{width:16px}}.muted{{color:#666}}.cols{{display:flex;gap:24px}}.cols>div{{flex:1}}\
.crit{{color:#b91c1c}}.warn{{color:#b45309}}.good{{color:#15803d}}\
@media print{{body{{margin:0}}.cols{{display:block}}}}</style></head><body onload=\"window.print()\">",
        if lang == Lang::Pt { "pt" } else { "en" },
        esc(&sheet.title)
    );
    out.push_str(&format!("<h1>{}</h1><p class=\"muted\">{}</p><div class=\"cols\"><div>", esc(&sheet.title), esc(&sheet.intro)));
    blocks_html(&sheet.left, &mut out);
    out.push_str("</div><div>");
    blocks_html(&sheet.right, &mut out);
    out.push_str("</div></div>");

    if analysis {
        let t = &state.tactic;
        let audit = audit_tactic(t, &state.squad, lang);
        out.push_str(&format!(
            "<h2>{} — {}/100</h2><ul>",
            esc(l(lang, "Diagnóstico", "Diagnosis")),
            audit.overall
        ));
        for f in &audit.findings {
            let (cls, icon) = match f.sev {
                Severity::Crit => ("crit", "✖"),
                Severity::Warn => ("warn", "⚠"),
                Severity::Info => ("", "ℹ"),
                Severity::Good => ("good", "✔"),
            };
            out.push_str(&format!(
                "<li class=\"{cls}\">{icon} <b>{}</b> — {}{}</li>",
                esc(&f.title),
                esc(&f.text),
                f.fix
                    .as_deref()
                    .map(|x| format!(" <i>{}</i>", esc(x)))
                    .unwrap_or_default()
            ));
        }
        out.push_str("</ul>");
        let syn = synergy_report(t, &state.squad, lang);
        out.push_str(&format!(
            "<h2>{} — {}/100</h2><p>{}</p><ul>",
            esc(l(lang, "Sinergia entre papéis", "Role synergy")),
            syn.score,
            syn.phases
                .iter()
                .map(|(ph, s)| format!("{}: {}", esc(synergy_phase_name(*ph, lang)), s.score))
                .collect::<Vec<_>>()
                .join(" · ")
        ));
        for i in &syn.issues {
            out.push_str(&format!(
                "<li><b>{}</b>: {} <i>{}</i></li>",
                esc(synergy_phase_name(i.phase, lang)),
                esc(&i.text),
                esc(&i.fix)
            ));
        }
        out.push_str("</ul>");
    }
    out.push_str("</body></html>");
    out
}

/// Writes the printable sheet (plus the analysis when `analysis`) to a temp `.html` and opens it
/// in the default browser, which shows its print dialog (print or save as PDF).
pub fn spawn_print_sheet(app: &mut ArchitectApp, analysis: bool, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    let html = sheet_html(&app.state, analysis);
    let stem: String = t_name(&app.state.tactic, lang)
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect();
    let dir = std::env::temp_dir().join("fm27-tactical-architect");
    let path = dir.join(format!("{}.html", if stem.is_empty() { "tactic".into() } else { stem }));
    let res = std::fs::create_dir_all(&dir)
        .and_then(|_| std::fs::write(&path, html))
        .map_err(|e| e.to_string())
        .and_then(|_| open_path(&path));
    match res {
        Ok(()) => app.state.set_toast(
            l(
                lang,
                "Ficha aberta no navegador: imprima ou guarde em PDF.",
                "Sheet opened in your browser: print it or save it as PDF.",
            ),
            ToastKind::Success,
        ),
        Err(e) => app.state.set_toast(e, ToastKind::Error),
    }
    cx.notify();
}

// ============================================================================
// View
// ============================================================================

fn render_blocks(
    side: char,
    blocks: Vec<SheetBlock>,
    ticked: &HashSet<String>,
    cx: &mut Context<ArchitectApp>,
) -> Div {
    let theme = cx.theme().clone();
    let mut col = div()
        .v_flex()
        .flex_1()
        .min_w(px(380.0))
        .gap_1()
        .p_4()
        .rounded_lg()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary);
    for (bi, b) in blocks.into_iter().enumerate() {
        col = col.child(if b.major {
            div().pt_3().text_sm().font_semibold().text_color(theme.foreground).child(b.title)
        } else {
            div().pt_2().text_xs().font_semibold().text_color(theme.muted_foreground).child(b.title)
        });
        if let Some(i) = b.intro {
            col = col.child(div().text_xs().text_color(theme.muted_foreground).child(i));
        }
        let wide = b.columns.as_ref().map_or(1, Vec::len) > 2;
        if let Some(cols) = b.columns {
            let mut hdr = div()
                .h_flex()
                .gap_3()
                .text_xs()
                .font_semibold()
                .text_color(theme.muted_foreground)
                .child(div().w(px(16.0)));
            for (ci, c) in cols.into_iter().enumerate() {
                hdr = hdr.child(if ci == 0 { div().w(px(150.0)) } else { div().flex_1() }.child(c));
            }
            col = col.child(hdr);
        }
        for (ri, r) in b.rows.into_iter().enumerate() {
            let key = format!("{side}{bi}:{ri}");
            let on = ticked.contains(&key);
            let fg = if r.strong && !on { theme.foreground } else { theme.muted_foreground };
            let mut line = div()
                .h_flex()
                .gap_3()
                .py_1()
                .border_b_1()
                .border_color(theme.border)
                .text_xs()
                .child(div().w(px(16.0)).when(r.mark, |d| {
                    let k = key.clone();
                    d.child(Checkbox::new(SharedString::from(format!("sheet-ck-{key}"))).checked(on).on_click(
                        cx.listener(move |this, _: &bool, _, cx| {
                            if !this.sheet_ui.ticked.remove(&k) {
                                this.sheet_ui.ticked.insert(k.clone());
                            }
                            cx.notify();
                        }),
                    ))
                }))
                .child(
                    div()
                        .w(px(if wide { 150.0 } else { 190.0 }))
                        .v_flex()
                        .child(div().text_color(fg).when(wide, |d| d.font_semibold()).child(r.label))
                        .children(r.note.map(|n| div().text_color(theme.muted_foreground).child(n))),
                );
            for v in r.values {
                line = line.child(
                    div()
                        .flex_1()
                        .text_color(fg)
                        .when(r.strong, |d| d.font_semibold())
                        .when(on, |d| d.line_through())
                        .child(v),
                );
            }
            col = col.child(line);
        }
    }
    col
}

/// Renders Tactic › FM27 Sheet.
pub fn render_sheet(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> impl IntoElement {
    let lang = app.state.lang;
    let theme = cx.theme().clone();
    let sheet = build_sheet(&app.state);
    let copy_txt = sheet_text(&app.state.tactic, lang);
    let pt_names = app.state.sheet_names_pt;

    let names_toggle = (lang == Lang::Pt).then(|| {
        let pick = |id: &'static str, label: &'static str, tip: Option<&'static str>, value: bool| {
            Button::new(id)
                .small()
                .label(label)
                .when(pt_names == value, |b| b.primary())
                .when(pt_names != value, |b| b.outline())
                .when_some(tip, |b, t| b.tooltip(t))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.state.sheet_names_pt = value;
                    this.state.persist();
                    cx.notify();
                }))
        };
        div()
            .h_flex()
            .gap_1()
            .child(pick("sheet-names-en", "Nomes do jogo (EN)", Some("Como aparecem no FM27 em inglês"), false))
            .child(pick("sheet-names-pt", "Português", None, true))
    });

    let header = div()
        .h_flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .v_flex()
                .gap(px(2.0))
                .child(div().text_base().font_semibold().text_color(theme.foreground).child(sheet.title))
                .child(div().text_xs().text_color(theme.muted_foreground).child(sheet.intro)),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_center()
                .gap_2()
                .children(names_toggle)
                .child(
                    Button::new("sheet-print")
                        .outline()
                        .small()
                        .label(l(lang, "Imprimir/PDF", "Print/PDF"))
                        .tooltip(l(
                            lang,
                            "Abre a ficha no navegador, pronta a imprimir ou guardar em PDF.",
                            "Opens the sheet in your browser, ready to print or save as PDF.",
                        ))
                        .on_click(cx.listener(|this, _, _, cx| spawn_print_sheet(this, false, cx))),
                )
                .child(
                    Button::new("sheet-copy")
                        .outline()
                        .small()
                        .label(l(lang, "Copiar texto", "Copy text"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(copy_txt.clone()));
                            let lang = this.state.lang;
                            this.state.set_toast(l(lang, "Ficha copiada", "Sheet copied"), ToastKind::Success);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("sheet-back")
                        .ghost()
                        .small()
                        .label(l(lang, "Voltar", "Back"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.tactic_tab = TacticTab::Analyse;
                            cx.notify();
                        })),
                ),
        );

    let ticked = app.sheet_ui.ticked.clone();
    let left = render_blocks('L', sheet.left, &ticked, cx);
    let right = render_blocks('R', sheet.right, &ticked, cx);
    div()
        .v_flex()
        .gap_4()
        .child(header)
        .child(div().h_flex().flex_wrap().items_start().gap_4().child(left).child(right))
}

#[cfg(test)]
mod tests {
    // Not `use super::*`: the gpui glob would shadow the built-in `#[test]`.
    use super::{build_sheet, sheet_html, AppState, Lang};
    use crate::views::team::make_sample_squad;
    use ta_backend::StateStore;

    fn state(lang: Lang, names_pt: bool) -> AppState {
        let dir = std::env::temp_dir().join(format!("ta-sheet-{}-{lang:?}-{names_pt}", std::process::id()));
        let mut st = AppState::load_from(StateStore::new(dir.join("state.json")));
        st.lang = lang;
        st.sheet_names_pt = names_pt;
        st.squad = make_sample_squad();
        st
    }

    #[test]
    fn pt_sheet_keeps_game_names_unless_portuguese_is_chosen() {
        let game = build_sheet(&state(Lang::Pt, false));
        let pt = build_sheet(&state(Lang::Pt, true));
        assert_eq!(game.left[0].rows[2].label, "In Possession");
        assert_ne!(pt.left[0].rows[2].label, "In Possession");
        // English UI ignores the PT-names choice.
        let en = build_sheet(&state(Lang::En, true));
        assert_eq!(en.left[0].rows[2].label, "In Possession");
    }

    #[test]
    fn printable_html_escapes_and_includes_analysis_on_request() {
        let mut st = state(Lang::En, false);
        st.tactic.name = "<b>A&B</b>".into();
        let sheet_only = sheet_html(&st, false);
        assert!(sheet_only.contains("&lt;b&gt;A&amp;B&lt;/b&gt;"));
        assert!(!sheet_only.contains("<b>A&B</b>"));
        assert!(!sheet_only.contains("Diagnosis"));
        assert!(sheet_html(&st, true).contains("Diagnosis"));
    }
}
