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
    PROFILE_INTRO_DRAFT, PUBLICATIONS, PUBLICATIONS_DRAFT, SKILLS, SKILLS_DRAFT, TALKS, TEACHING,
};

// Assets are registered with the `asset!()` macro (the 0.7 convention) and
// linked from the document below. They get hashed/cache-busted at build time.
const MAIN_CSS: Asset = asset!("/assets/main.css");

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
    rsx! {
        LanguageToggle {}
        SiteNav {}
        main { Outlet::<Route> {} }
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
    let (name, _): (&str, ()) = (profile_str("name"), ());
    let title = profile_loc("title").pick(lang);
    let location = profile_loc("location").pick(lang);
    let email = profile_str("email");
    let linkedin = profile_str("linkedin");
    let phone = profile_str("phone");

    rsx! {
        header { class: "home-header",
            h1 { "{name}" }
            p { class: "title", "{title}" }
            address { class: "contact",
                a { href: "mailto:{email}", "{email}" }
                span { " · " }
                a { href: "https://linkedin.com/in/{linkedin}", "LinkedIn" }
                span { " · " }
                a { href: "tel:{phone}", "{phone}" }
                span { " · " }
                "{location}"
            }
        }
        FrDraftNotice { lang, draft: PROFILE_INTRO_DRAFT }
        div { class: "prose", dangerous_inner_html: PROFILE_INTRO.pick(lang) }
    }
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
                span { class: "period", { format_period(entry.start, entry.end, lang) } }
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
        FrDraftNotice { lang, draft: SKILLS_DRAFT }
        div { class: "prose", dangerous_inner_html: SKILLS.pick(lang) }
    }
}

#[component]
fn Publications() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { { match lang { Lang::En => "Publications", Lang::Fr => "Publications" } } }
        FrDraftNotice { lang, draft: PUBLICATIONS_DRAFT }
        p { class: "pub-note",
            { match lang {
                Lang::En => "Primary author in bold.",
                Lang::Fr => "Premier auteur en gras.",
            } }
        }
        div { class: "prose", dangerous_inner_html: PUBLICATIONS.pick(lang) }
    }
}

#[component]
fn Cv() -> Element {
    let lang = *LANG.read();
    rsx! {
        h1 { "CV" }
        p {
            { match lang {
                Lang::En => "Downloadable CVs — a full English CV and a two-page French CV — will be generated from the same Markdown source via Typst.",
                Lang::Fr => "Les CV téléchargeables — un CV complet en anglais et un CV de deux pages en français — seront générés depuis la même source Markdown via Typst.",
            } }
        }
    }
}

/// Discreet notice shown on FR pages while that page's French translation is a
/// draft. `draft` is per-page: the experience/education pages pass whether any
/// of their entries carry `fr_draft`; the skills/publications pages pass their
/// own `*_DRAFT` flag.
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
