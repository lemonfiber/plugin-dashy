//! A panel list as the page Dashy reads, every value held to its grammar first.
//!
//! Every value comes from a `dashboard.show@1` request, so from every plugin's entries.
//! An address is `http` or `https` and nothing a browser runs; an address Dashy's own
//! server asks, a check or a status page, is also a service on the stack's network: its
//! host is one of the panel list's `hosts`, the names the core gives a check at, each a
//! single label and never a name a container's hosts file holds, a container's id or a
//! metadata or cluster name, and it is written as it parses; a status page is such a service's
//! origin; an icon is a name of letters, digits, `.`, `_` and `-`, or such an address;
//! every other value is text with no control character, no markup and no variable Dashy
//! reads from its environment. The page is JSON that serde quotes, so no value can carry
//! structure of its own.

use lemonfiber_ports::service::{Entry, EntryWidget, PageWidget, Panels};
use serde::Serialize;

/// The page Dashy reads, in its configuration directory.
pub const PAGE: &str = "conf.yml";

/// What the page is titled.
const TITLE: &str = "Home";

/// The section the page's own widgets are shown in.
const PAGE_SECTION: &str = "This machine";

/// The search engine Dashy sends to an address it is given.
const SEARCH_ENGINE: &str = "custom";

/// Where a search from its search box opens.
const SEARCH_OPENS: &str = "newtab";

/// Dashy's widget for a monitor's status page.
const STATUS_PAGE: &str = "uptime-kuma-status-page";

/// Dashy's widget for the machine's processor and memory use.
const SYSTEM: &str = "system-info";

/// How Dashy names an icon from the dashboard icons.
const DASHBOARD_ICON: &str = "hl-";

/// The picture files an icon's name may end in.
const PICTURES: [&str; 3] = ["png", "svg", "webp"];

/// What Dashy replaces from its environment wherever a value holds it.
const EXPANDED: [&str; 3] = ["DASHY_", "VITE_APP_", "VUE_APP_"];

/// The characters markup is written with, which no text carries.
const MARKUP: [char; 6] = ['<', '>', '"', '\'', '`', '&'];

/// The single-label names that are not a service on the stack's network: those a
/// container's own hosts file gives the loopback and IPv6's fixed addresses, and the
/// metadata and cluster names a host's search domains complete.
const NOT_SERVICES: [&str; 10] = [
    "localhost",
    "ip6-localhost",
    "ip6-loopback",
    "ip6-localnet",
    "ip6-mcastprefix",
    "ip6-allnodes",
    "ip6-allrouters",
    "metadata",
    "instance-data",
    "kubernetes",
];

/// The lengths of a container's id, short and full, each a name its hosts file or the
/// engine's resolver answers for that container.
const CONTAINER_IDS: [usize; 2] = [12, 64];

/// The schemes Dashy's server asks a service over.
const ASKED_OVER: [&str; 2] = ["http", "https"];

/// The longest value written.
const LONGEST: usize = 2048;

/// Why a panel list was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// An address that is not `http` or `https`, or carries what an address does not.
    Address(String),
    /// An icon that is neither a plain name nor an address.
    Icon(String),
    /// Text holding a control character, markup or something Dashy expands.
    Text(String),
    /// An address Dashy's server would ask whose host is not one of the panel list's
    /// `hosts`, a service on the stack's network.
    Unreached(String),
}

/// The page Dashy reads for `panels`.
///
/// # Errors
///
/// [`Refused`] naming the first value that is not of its grammar.
pub fn rendered(panels: &Panels) -> Result<String, Refused> {
    let mut sections = panels
        .groups
        .iter()
        .map(|group| {
            Ok(Section {
                name: text(&group.name)?,
                items: group
                    .entries
                    .iter()
                    .map(|entry| item(entry, &panels.hosts))
                    .collect::<Result<_, _>>()?,
                widgets: group
                    .entries
                    .iter()
                    .filter_map(|entry| entry.widget.as_ref())
                    .map(|widget| entry_widget(widget, &panels.hosts))
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect::<Result<Vec<_>, Refused>>()?;
    if panels.widgets.contains(&PageWidget::Resources) {
        sections.push(Section {
            name: PAGE_SECTION.to_owned(),
            items: Vec::new(),
            widgets: vec![Widget {
                kind: SYSTEM.to_owned(),
                use_proxy: None,
                options: None,
            }],
        });
    }
    let web_search = panels
        .widgets
        .iter()
        .find_map(|widget| match widget {
            PageWidget::Search { url } => Some(url),
            PageWidget::Resources => None,
        })
        .map(|url| {
            Ok(WebSearch {
                search_engine: SEARCH_ENGINE.to_owned(),
                custom_search_engine: address(url)?,
                opening_method: SEARCH_OPENS.to_owned(),
            })
        })
        .transpose()?;
    Ok(written(&Configuration {
        page_info: PageInfo {
            title: TITLE.to_owned(),
        },
        app_config: AppConfig {
            prevent_write_to_disk: true,
            disable_configuration: true,
            web_search,
        },
        sections,
    }))
}

/// The page as Dashy reads it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Configuration {
    page_info: PageInfo,
    app_config: AppConfig,
    sections: Vec<Section>,
}

/// What the page says of itself.
#[derive(Debug, Serialize)]
struct PageInfo {
    title: String,
}

/// How Dashy runs the page: never saving it, never editing it, and searching where it is
/// told to.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppConfig {
    prevent_write_to_disk: bool,
    disable_configuration: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    web_search: Option<WebSearch>,
}

/// Where a search from the page's search box goes.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WebSearch {
    search_engine: String,
    custom_search_engine: String,
    opening_method: String,
}

/// One group of entries, and the widgets beside them.
#[derive(Debug, Serialize)]
struct Section {
    name: String,
    items: Vec<Item>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    widgets: Vec<Widget>,
}

/// One entry as Dashy reads it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    title: String,
    description: String,
    icon: String,
    url: String,
    status_check: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_check_url: Option<String>,
}

/// A widget as Dashy names it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Widget {
    #[serde(rename = "type")]
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_proxy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<StatusPage>,
}

/// Where a monitor's status page is read from.
#[derive(Debug, Serialize)]
struct StatusPage {
    host: String,
    slug: String,
}

/// One entry as Dashy reads it, every value held to its grammar and its check to `hosts`.
fn item(entry: &Entry, hosts: &[String]) -> Result<Item, Refused> {
    Ok(Item {
        title: text(&entry.name)?,
        description: text(&entry.description)?,
        icon: icon(&entry.icon)?,
        url: address(&entry.link)?,
        status_check: entry.check.is_some(),
        status_check_url: entry
            .check
            .as_deref()
            .map(|check| reached(check, hosts).map(String::from))
            .transpose()?,
    })
}

/// One entry's widget as Dashy reads it, read through Dashy's own server, which reaches
/// only the addresses the page names, each on one of `hosts`.
fn entry_widget(widget: &EntryWidget, hosts: &[String]) -> Result<Widget, Refused> {
    match widget {
        EntryWidget::StatusPage { url, page } => Ok(Widget {
            kind: STATUS_PAGE.to_owned(),
            use_proxy: Some(true),
            options: Some(StatusPage {
                host: origin(url, hosts)?,
                slug: slug(page)?,
            }),
        }),
    }
}

/// `value`, where it is an `http` or `https` address of the characters an address is
/// written in, with nothing Dashy expands.
fn address(value: &str) -> Result<String, Refused> {
    let lower = value.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"));
    let plain = value.len() <= LONGEST
        && rest.is_some_and(|rest| !rest.is_empty() && !rest.starts_with('/'))
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'-' | b'.'
                        | b'_'
                        | b'~'
                        | b':'
                        | b'/'
                        | b'?'
                        | b'#'
                        | b'['
                        | b']'
                        | b'@'
                        | b'!'
                        | b'&'
                        | b'\''
                        | b'('
                        | b')'
                        | b'*'
                        | b'+'
                        | b','
                        | b';'
                        | b'='
                        | b'%'
                )
        })
        && unexpanded(value);
    if plain {
        Ok(value.to_owned())
    } else {
        Err(Refused::Address(value.to_owned()))
    }
}

/// `value` parsed, where it is an address of a service on the stack's network: `http` or
/// `https`, with no account and no fragment, and a host that is one of `hosts` and one
/// label of a service's name.
fn reached(value: &str, hosts: &[String]) -> Result<url::Url, Refused> {
    let asked = address(value)?;
    let parsed = url::Url::parse(&asked).map_err(|_| Refused::Unreached(asked.clone()))?;
    let served = ASKED_OVER.contains(&parsed.scheme())
        && parsed.username().is_empty()
        && parsed.password().is_none()
        && matches!(parsed.host(), Some(url::Host::Domain(name))
            if stack_name(name) && hosts.iter().any(|host| host == name));
    if !served {
        Err(Refused::Unreached(asked))
    } else if parsed.fragment().is_some() {
        Err(Refused::Address(asked))
    } else {
        Ok(parsed)
    }
}

/// The origin `value` names, `scheme://host[:port]`, where it is an address of a service on
/// the stack's network with no path and no query.
fn origin(value: &str, hosts: &[String]) -> Result<String, Refused> {
    let parsed = reached(value, hosts)?;
    if parsed.path() == "/" && parsed.query().is_none() {
        Ok(parsed.origin().ascii_serialization())
    } else {
        Err(Refused::Address(value.to_owned()))
    }
}

/// Whether `name` is one label of a service's name: lowercase letters, digits and `-`,
/// starting and ending with a letter or digit, none of the names that are not a service,
/// and not shaped as a container's id.
fn stack_name(name: &str) -> bool {
    !NOT_SERVICES.contains(&name)
        && !container_id(name)
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && name.bytes().any(|byte| byte.is_ascii_lowercase())
}

/// Whether `name` has the shape of a container's id: lowercase hexadecimal of either
/// length the engine writes one in.
fn container_id(name: &str) -> bool {
    CONTAINER_IDS.contains(&name.len())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// `value`, where it is a status page's name as Uptime Kuma writes one: lowercase
/// letters, digits and `-`, so the two addresses read from it name no other path.
fn slug(value: &str) -> Result<String, Refused> {
    let named = !value.is_empty()
        && value.len() <= LONGEST
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if named {
        Ok(value.to_owned())
    } else {
        Err(Refused::Text(value.to_owned()))
    }
}

/// `value`, where it is an icon's plain name or address.
fn icon(value: &str) -> Result<String, Refused> {
    let named = !value.is_empty()
        && value.len() <= LONGEST
        && !value.starts_with('.')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if named && unexpanded(value) {
        Ok(drawn(value))
    } else {
        address(value).map_err(|_| Refused::Icon(value.to_owned()))
    }
}

/// `value`, where it holds no control character and nothing Dashy expands.
fn text(value: &str) -> Result<String, Refused> {
    let plain = value.len() <= LONGEST
        && !value
            .chars()
            .any(|letter| letter.is_control() || MARKUP.contains(&letter))
        && unexpanded(value);
    if plain {
        Ok(value.to_owned())
    } else {
        Err(Refused::Text(value.to_owned()))
    }
}

/// Whether `value` names none of the variables Dashy reads from its environment.
fn unexpanded(value: &str) -> bool {
    !EXPANDED.iter().any(|prefix| value.contains(prefix))
}

/// An icon's plain name as Dashy draws it: a file name from the dashboard icons by its
/// stem, any other name as it is.
fn drawn(value: &str) -> String {
    value
        .rsplit_once('.')
        .filter(|(stem, kind)| !stem.is_empty() && PICTURES.contains(kind))
        .map_or_else(
            || value.to_owned(),
            |(stem, _)| format!("{DASHBOARD_ICON}{stem}"),
        )
}

/// `value` as the file holds it.
fn written(value: &impl Serialize) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default() + "\n"
}

#[cfg(test)]
mod tests;
