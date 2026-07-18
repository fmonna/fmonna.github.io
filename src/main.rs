// Florence Monna — personal site skeleton (Dioxus 0.7 fullstack + router + SSG).
//
// This is a scaffold: pages are empty placeholders and content is not yet wired
// to the Markdown source (that migration is a separate task). The goal of this
// file is to establish the structure — routing, layout shell, language toggle,
// and prerendering config — so content can be dropped in later.
//
// Dioxus 0.7 SSG is a fullstack-only feature. The pieces:
//   - the `static_routes` server function returns every URL to prerender;
//   - `Route::static_routes()` (derived from the `#[route]` attrs) lists the
//     non-dynamic routes automatically;
//   - the `IncrementalRendererConfig` below caches each served route's SSR
//     output to `public/<route>/index.html` (because `clear_cache(false)`).
//
// NOTE: with this crate's `default = ["web"]` (not `fullstack`), `dx bundle
// --web --ssg` builds clean but does NOT run the pre-crawl — the `"Pre-render
// ing..."` phase never triggers and only the CSR loader `index.html` is
// written. The prerendered pages are produced instead by running the server
// binary and requesting each `static_routes` URL (the incremental renderer
// caches them). The deploy workflow scripts that: build, run server, crawl,
// upload `public/`. See memory note `dioxus-ssg-bundle-crawl`.
// Docs: https://dioxuslabs.com/learn/0.7/essentials/fullstack/static_site_generation/

use dioxus::prelude::*;

mod content;
use content::{
    ARCHIVE, BLOG, EDUCATION, Entry, EXPERIENCE, Lang, PORTFOLIO, PROFILE, PROFILE_INTRO,
    PROFILE_INTRO_DRAFT, PUBLICATIONS, SKILLS, TALKS, TEACHING,
};

// Assets are registered with the `asset!()` macro (the 0.7 convention) and
// linked from the document below. They get hashed/cache-busted at build time.
const MAIN_CSS: Asset = asset!("/assets/main.css");
const PORTRAIT: Asset = asset!("/assets/images/portrait.jpg");

fn main() {
    // LaunchBuilder (not the bare `dioxus::launch`) lets us attach server-only
    // config: the `ServeConfig` with an incremental renderer, which is what SSG
    // builds on. `server_only!` strips this block from the WASM client build.
    dioxus::LaunchBuilder::new()
        .with_cfg(server_only! {
            ServeConfig::builder()
                .incremental(
                    dioxus::server::IncrementalRendererConfig::new()
                        // Prerendered HTML + assets land in `public/` next to the
                        // server binary (dx emits the bundle there). Don't clear it
                        // on each build — the WASM/JS client lives in there too.
                        .static_dir(
                            std::env::current_exe()
                                .unwrap()
                                .parent()
                                .unwrap()
                                .join("public"),
                        )
                        .clear_cache(false),
                )
                .enable_out_of_order_streaming()
        })
        .launch(app);
}

/// Top-level component: persistent layout shell (nav + language toggle) with the
/// router outlet below it.
fn app() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        // `Router` provides the routing context that `Link` (in SiteNav, inside
        // the Layout) and `Outlet` depend on. Without it, `Link` panics under SSR
        // — which silently kills every route during `dx bundle --ssg` prerendering
        // (see dioxus-router-0.7.9/src/components/link.rs:159, debug-only panic).
        // The persistent shell (nav + toggle + footer) lives in `Layout`, applied
        // to every route via `#[layout(Layout)]` on the `Route` enum below.
        Router::<Route> {}
    }
}

/// Persistent layout shell applied to every route via `#[layout(Layout)]`. It must
/// contain exactly one `Outlet::<Route> {}`, which renders the matched page.
#[component]
fn Layout() -> Element {
    // Keep the document's `<html lang>` in sync with the language toggle. The
    // toggle flips a client-side `GlobalSignal`, but the static `<html lang="en">`
    // baked into the shell never changes on its own — without this, FR pages are
    // read as English by screen readers and FR hyphenation never engages. SSG
    // prerenders each route with the default `Lang::En`, so the static HTML is
    // correct on first paint; this effect then corrects it live after a toggle.
    //
    // `document::eval` runs a JS string in the WASM client (the same `document`
    // module the stylesheet `<link>` below uses). `use_effect` is client-only —
    // it does not run during SSG prerender, so the prerendered HTML is untouched.
    use_effect(move || {
        let code = match *LANG.read() {
            Lang::En => "en",
            Lang::Fr => "fr",
        };
        let _ = document::eval(&format!(
            "document.documentElement.setAttribute('lang', {code:?});"
        ));
    });

    rsx! {
        a { href: "#main", class: "skip-link", "Skip to content" }
        LanguageToggle {}
        SiteNav {}
        main { id: "main", Outlet::<Route> {} }
        footer { p { "© 2026 Florence Monna" } }
    }
}

// --- Routing ---------------------------------------------------------------
// Each `#[route]` variant maps to a URL and a same-named component. Non-dynamic
// routes are collected by `Route::static_routes()` and prerendered by SSG.
// When content lands, these bodies become real sections driven by Markdown.

// `#[layout(Layout)]` wraps every route below it in the persistent shell
// (nav + toggle + footer). It needs no closing `#[end_layout]` here — that only
// applies when you want to end the layout group *before* the enum does. Trailing
// `#[end_layout]` directly before the closing `}` fails to parse ("expected
// identifier, found `}`"), which is why it's omitted (matches the 0.7 template).
#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/experience")]
    Experience {},
    #[route("/education")]
    Education {},
    #[route("/teaching")]
    Teaching {},
    #[route("/skills")]
    Skills {},
    #[route("/publications")]
    Publications {},
    #[route("/talks")]
    Talks {},
    #[route("/portfolio")]
    Portfolio {},
    #[route("/blog")]
    Blog {},
    #[route("/archive")]
    Archive {},
    #[route("/cv")]
    Cv {},
}

#[component]
fn Home() -> Element {
    let lang = *LANG.read();
    let name = profile_str("name");
    let title = profile_loc("title").pick(lang);
    let tagline = profile_loc("tagline").pick(lang);
    let now = profile_loc("now").pick(lang);
    let location = profile_loc("location").pick(lang);
    let email = profile_str("email");
    let linkedin = profile_str("linkedin");
    let github = profile_str("github");
    let scholar = profile_str("scholar");
    let orcid = profile_str("orcid");

    rsx! {
        header { class: "home-header",
            div { class: "home-text",
                h1 { "{name}" }
                p { class: "title", "{title}" }
                if !tagline.is_empty() {
                    p { class: "tagline", "{tagline}" }
                }
                if !now.is_empty() {
                    p { class: "now-callout", "{now}" }
                }
                nav { class: "id-strip",
                    if !email.is_empty() {
                        a { href: "mailto:{email}", title: email, class: "id-link",
                            IconEmail {}
                        }
                    }
                    if !linkedin.is_empty() {
                        a { href: "https://linkedin.com/in/{linkedin}", title: "LinkedIn", class: "id-link",
                            IconLinkedIn {}
                        }
                    }
                    if !github.is_empty() {
                        a { href: "https://github.com/{github}", title: "GitHub", class: "id-link",
                            IconGitHub {}
                        }
                    }
                    if !scholar.is_empty() {
                        a { href: "{scholar}", title: "Google Scholar", class: "id-link",
                            IconScholar {}
                        }
                    }
                    if !orcid.is_empty() {
                        a { href: "https://orcid.org/{orcid}", title: "ORCID", class: "id-link",
                            IconOrcid {}
                        }
                    }
                }
                address { class: "contact",
                    if !location.is_empty() { "{location}" }
                }
            }
            div { class: "home-portrait",
                img {
                    class: "portrait",
                    src: PORTRAIT,
                    alt: "Portrait of {name}",
                    width: "160",
                    height: "160",
                }
            }
        }
        FrDraftNotice { lang, draft: PROFILE_INTRO_DRAFT }
        div { class: "prose", dangerous_inner_html: PROFILE_INTRO.pick(lang) }
    }
}

// Inline academic-network icons (24x24, currentColor). No icon-font or image
// dependency — matches the dependency-free constraint.
#[component]
fn IconEmail() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "20", height: "20", fill: "none", stroke: "currentColor", stroke_width: "2",
        rect { x: "3", y: "5", width: "18", height: "14", rx: "2" }
        path { d: "M3 7l9 6 9-6" }
    } }
}
#[component]
fn IconLinkedIn() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "20", height: "20", fill: "currentColor",
        path { d: "M4.98 3.5A2.5 2.5 0 1 1 0 3.5a2.5 2.5 0 0 1 4.98 0zM0 8h5v16H0zM7 8h4.8v2.2h.07c.67-1.2 2.3-2.5 4.73-2.5 5 0 6 3.3 6 7.6V24h-5v-7.4c0-1.8 0-4-2.5-4s-2.9 1.9-2.9 3.9V24H7z" }
    } }
}
#[component]
fn IconGitHub() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "20", height: "20", fill: "currentColor",
        path { d: "M12 0a12 12 0 0 0-3.8 23.4c.6.1.8-.3.8-.6v-2c-3.3.7-4-1.6-4-1.6-.6-1.4-1.3-1.7-1.3-1.7-1.1-.7.1-.7.1-.7 1.2.1 1.8 1.2 1.8 1.2 1 1.8 2.7 1.3 3.4 1 .1-.8.4-1.3.7-1.6-2.6-.3-5.4-1.3-5.4-5.7 0-1.3.4-2.3 1.2-3.1-.1-.3-.5-1.5.1-3.1 0 0 1-.3 3.3 1.2a11.5 11.5 0 0 1 6 0C17.3 4.5 18.3 4.8 18.3 4.8c.6 1.6.2 2.8.1 3.1.8.8 1.2 1.8 1.2 3.1 0 4.4-2.8 5.4-5.4 5.7.4.4.8 1.1.8 2.2v3.3c0 .3.2.7.8.6A12 12 0 0 0 12 0z" }
    } }
}
#[component]
fn IconScholar() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "20", height: "20", fill: "currentColor",
        path { d: "M12 24a12 12 0 1 1 0-24 12 12 0 0 1 0 24zm.5-19c-3 0-5 2-5 4.7 0 2.5 1.8 4.3 4.4 4.3 1.4 0 2.6-.5 3.4-1.3-.3 1.7-1.7 2.9-3.6 3.2-1 .2-2 .1-2.8-.2l-.5 1.6c1 .4 2.1.5 3.2.4 3.8-.4 6.4-3.3 6.4-7.4C18 6.5 15.7 5 12.5 5zm0 2.2c1.4 0 2.3 1 2.3 2.4s-.9 2.4-2.3 2.4-2.3-1-2.3-2.4.9-2.4 2.3-2.4z" }
    } }
}
#[component]
fn IconOrcid() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "20", height: "20", fill: "currentColor",
        path { d: "M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.6 0 12 0zM7.5 5h1.4v10.5H7.5zM8.2 3.3a.9.9 0 1 1 0 1.8.9.9 0 0 1 0-1.8zm3.2 1.7v10.5h1.4v-2.6c0-.4 0-.7.1-1 .3-.7 1-1.4 1.9-1.4 1.1 0 1.6.8 1.6 2v3h1.4v-3.2c0-2.3-1.2-3.4-2.8-3.4-1.3 0-2 .7-2.3 1.2V5z" }
    } }
}

#[component]
fn Experience() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { { match lang { Lang::En => "Professional experience", Lang::Fr => "Expérience professionnelle" } } }
        FrDraftNotice { lang, draft: EXPERIENCE.iter().any(|e| e.fr_draft) }
        section { class: "timeline",
            for e in EXPERIENCE.iter() {
                ExperienceItem { entry: *e, lang }
            }
        }
    }
}

#[component]
fn ExperienceItem(entry: Entry, lang: Lang) -> Element {
    rsx! {
        article { class: "entry",
            div { class: "entry-head",
                h2 { { entry.heading.pick(lang) } }
                span { class: "org", { entry.org } }
                if !entry.org.is_empty() && !entry.location.pick(lang).is_empty() {
                    span { " · " }
                }
                if !entry.location.pick(lang).is_empty() {
                    span { class: "loc", { entry.location.pick(lang) } }
                }
                span { class: "period", { format_period(entry.start, entry.end, lang) } }
            }
            div { class: "entry-body", dangerous_inner_html: entry.body(lang) }
        }
    }
}

#[component]
fn Education() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { { match lang { Lang::En => "Education", Lang::Fr => "Formation" } } }
        FrDraftNotice { lang, draft: EDUCATION.iter().any(|e| e.fr_draft) }
        section { class: "timeline",
            for e in EDUCATION.iter() {
                ExperienceItem { entry: *e, lang }
            }
        }
    }
}

#[component]
fn Teaching() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { { match lang { Lang::En => "Teaching", Lang::Fr => "Enseignement" } } }
        FrDraftNotice { lang, draft: TEACHING.iter().any(|e| e.fr_draft) }
        section { class: "timeline",
            for e in TEACHING.iter() {
                ExperienceItem { entry: *e, lang }
            }
        }
    }
}

#[component]
fn Talks() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { { match lang { Lang::En => "Talks", Lang::Fr => "Talks" } } }
        FrDraftNotice { lang, draft: TALKS.iter().any(|e| e.fr_draft) }
        section { class: "timeline",
            for e in TALKS.iter() {
                ExperienceItem { entry: *e, lang }
            }
        }
    }
}

#[component]
fn Portfolio() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { { match lang { Lang::En => "Portfolio", Lang::Fr => "Portfolio" } } }
        FrDraftNotice { lang, draft: PORTFOLIO.iter().any(|e| e.fr_draft) }
        section { class: "cards",
            for e in PORTFOLIO.iter() {
                EntryCard { entry: *e, lang }
            }
        }
    }
}

#[component]
fn Blog() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { { match lang { Lang::En => "Blog", Lang::Fr => "Blog" } } }
        FrDraftNotice { lang, draft: BLOG.iter().any(|e| e.fr_draft) }
        section { class: "cards",
            for e in BLOG.iter() {
                EntryCard { entry: *e, lang }
            }
        }
    }
}

#[component]
fn Archive() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { { match lang { Lang::En => "Archive", Lang::Fr => "Archives" } } }
        FrDraftNotice { lang, draft: ARCHIVE.iter().any(|e| e.fr_draft) }
        section { class: "cards",
            for e in ARCHIVE.iter() {
                EntryCard { entry: *e, lang }
            }
        }
    }
}

/// Lighter card variant for portfolio/blog/archive entries: heading + period,
/// an optional org/location line (hidden when both are empty — e.g. a blog
/// post carries only a title), then the body. Same `Entry` shape as the
/// timeline items, just laid out without the left accent rail.
#[component]
fn EntryCard(entry: Entry, lang: Lang) -> Element {
    let org = entry.org;
    let loc = entry.location.pick(lang);
    rsx! {
        article { class: "card",
            div { class: "entry-head",
                h2 { { entry.heading.pick(lang) } }
                if !entry.start.is_empty() {
                    span { class: "period", { format_period(entry.start, entry.end, lang) } }
                }
            }
            if !org.is_empty() || !loc.is_empty() {
                p { class: "meta",
                    if !org.is_empty() { span { class: "org", { org } } }
                    if !org.is_empty() && !loc.is_empty() { span { " · " } }
                    if !loc.is_empty() { span { class: "loc", { loc } } }
                }
            }
            div { class: "entry-body", dangerous_inner_html: entry.body(lang) }
        }
    }
}

#[component]
fn Skills() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { { match lang { Lang::En => "Skills", Lang::Fr => "Compétences" } } }
        FrDraftNotice { lang, draft: SKILLS.iter().any(|e| e.fr_draft) }
        section { class: "cards",
            for e in SKILLS.iter() {
                EntryCard { entry: *e, lang }
            }
        }
    }
}

#[component]
fn Publications() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { { match lang { Lang::En => "Publications", Lang::Fr => "Publications" } } }
        FrDraftNotice { lang, draft: PUBLICATIONS.iter().any(|e| e.fr_draft) }
        p { class: "pub-note",
            { match lang {
                Lang::En => "Primary author in bold.",
                Lang::Fr => "Premier auteur en gras.",
            } }
        }
        section { class: "cards",
            for e in PUBLICATIONS.iter() {
                EntryCard { entry: *e, lang }
            }
        }
    }
}

#[component]
fn Cv() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { "CV" }
        p {
            { match lang {
                Lang::En => "Downloadable CVs — a full English CV and a two-page French CV — generated from the same Markdown source as the site, rendered with Typst.",
                Lang::Fr => "CV téléchargeables — un CV complet en anglais et un CV condensé en deux pages en français — générés depuis la même source Markdown que le site, mis en page avec Typst.",
            } }
        }
        p {
            a { href: "/cv-en.pdf", "CV (English, full) [PDF]" }
        }
        p {
            a { href: "/cv-fr.pdf", "CV (français, 2 pages) [PDF]" }
        }
    }
}

/// Discreet notice shown on FR pages while that page's French translation is a
/// draft. `draft` is per-page: every collection page passes whether any of its
/// entries carry `fr_draft`.
#[component]
fn FrDraftNotice(lang: Lang, draft: bool) -> Element {
    if lang == Lang::Fr && draft {
        rsx! {
            p { class: "fr-draft-notice",
                "🇫🇷 La version française est une ébauche de traduction, en cours de relecture."
            }
        }
    } else {
        rsx! { span { class: "sr-only", "" } }
    }
}

// --- Layout shell ----------------------------------------------------------

/// Bilingual toggle (EN/FR). Stores the choice in a global signal; content
/// components read it via `*LANG.read()`. `Lang` is defined in `crate::content`.
static LANG: GlobalSignal<Lang> = Signal::global(|| Lang::En);

#[component]
fn LanguageToggle() -> Element {
    rsx! {
        nav { class: "lang-toggle",
            button {
                disabled: *LANG.read() == Lang::En,
                onclick: move |_| *LANG.write() = Lang::En,
                "EN"
            }
            button {
                disabled: *LANG.read() == Lang::Fr,
                onclick: move |_| *LANG.write() = Lang::Fr,
                "FR"
            }
        }
    }
}

#[component]
fn SiteNav() -> Element {
    let lang = *LANG.read();
    let label = |en: &'static str, fr: &'static str| -> &'static str {
        match lang {
            Lang::En => en,
            Lang::Fr => fr,
        }
    };
    rsx! {
        nav { class: "site-nav",
            // Populated sections first…
            Link { to: Route::Home {},         { label("Home", "Accueil") } }
            Link { to: Route::Experience {},   { label("Experience", "Expérience") } }
            Link { to: Route::Education {},    { label("Education", "Formation") } }
            Link { to: Route::Teaching {},     { label("Teaching", "Enseignement") } }
            Link { to: Route::Skills {},       { label("Skills", "Compétences") } }
            Link { to: Route::Publications {}, { label("Publications", "Publications") } }
            // …then placeholder sections (still only seeded sample entries),
            // dimmed so visitors see they're not yet populated.
            Link { to: Route::Talks {},        class: "draft", { label("Talks", "Exposés") } }
            Link { to: Route::Portfolio {},    class: "draft", { label("Portfolio", "Portfolio") } }
            Link { to: Route::Blog {},         class: "draft", { label("Blog", "Blog") } }
            Link { to: Route::Archive {},      class: "draft", { label("Archive", "Archives") } }
            Link { to: Route::Cv {},           "CV" }
        }
    }
}

// --- SSG: enumerate routes to prerender ------------------------------------
// Required by `dx bundle --web --ssg`. The CLI calls this server function,
// crawls each returned URL, and writes the rendered HTML to `public/`.

// --- content helpers -------------------------------------------------------
// PROFILE is generated as &[(&str, &str)] where each value is either a plain
// scalar ("name" => "Florence Monna") or an EN\0FR pair ("title", "location").
// These helpers fish values out by key.

fn profile_str(key: &str) -> &'static str {
    PROFILE
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| {
            // take the part before any NUL separator (the EN form)
            v.split('\u{0}').next().unwrap_or(v)
        })
        .unwrap_or("")
}

fn profile_loc(key: &str) -> content::LStr {
    let raw = PROFILE
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .unwrap_or("");
    let mut parts = raw.splitn(2, '\u{0}');
    let en = parts.next().unwrap_or("");
    let fr = parts.next().unwrap_or("");
    content::LStr { en, fr: if fr.is_empty() { en } else { fr } }
}

/// Format a start/end pair as a human range, bilingual. Each endpoint may be
/// YYYY, YYYY-MM, or YYYY-MM-DD (the teaching entries use full ISO dates). An
/// empty `end` means "current".
fn format_period(start: &str, end: &str, lang: Lang) -> String {
    let s = format_date(start, lang);
    let e = if end.is_empty() {
        match lang {
            Lang::En => "present".to_string(),
            Lang::Fr => "présent".to_string(),
        }
    } else {
        format_date(end, lang)
    };
    format!("{} — {}", s, e)
}

/// Format a single date as "Sep 2024" / "sept. 2024". Accepts YYYY (-> "2024"),
/// YYYY-MM, or YYYY-MM-DD; anything unparseable is returned as-is.
fn format_date(ym: &str, lang: Lang) -> String {
    let bytes = ym.as_bytes();
    // YYYY-MM[-DD] — take the year and the two-digit month, ignore the day.
    if ym.len() >= 7 && bytes[4] == b'-' {
        let year = &ym[..4];
        let month = &ym[5..7];
        let name = month_name(month, lang);
        if name.is_empty() {
            year.to_string()
        } else {
            format!("{} {}", name, year)
        }
    } else {
        ym.to_string()
    }
}

fn month_name(m: &str, lang: Lang) -> &'static str {
    let en = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let fr = [
        "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.",
        "nov.", "déc.",
    ];
    let idx: usize = match m.parse::<i32>() {
        Ok(n) if (1..=12).contains(&n) => (n - 1) as usize,
        _ => return "",
    };
    match lang {
        Lang::En => en[idx],
        Lang::Fr => fr[idx],
    }
}

// --- SSG: enumerate routes to prerender (cont.) ----------------------------

#[server(endpoint = "static_routes")]
async fn static_routes() -> Result<Vec<String>, ServerFnError> {
    Ok(Route::static_routes()
        .iter()
        .map(ToString::to_string)
        .collect())
}
