use lemonfiber_ports::service::{Entry, EntryWidget, Group, PageWidget, Panels};
use serde_json::{json, Value};

use super::{rendered, Refused, LONGEST};

fn entry() -> Entry {
    Entry {
        name: "Jellyfin".to_owned(),
        icon: "jellyfin.png".to_owned(),
        link: "http://192.0.2.10:8096".to_owned(),
        description: "Watch your library".to_owned(),
        check: Some("http://jellyfin:8096/health".to_owned()),
        widget: None,
    }
}

fn panels(entries: Vec<Entry>, widgets: Vec<PageWidget>) -> Panels {
    Panels {
        groups: vec![Group {
            name: "Watch".to_owned(),
            entries,
        }],
        widgets,
    }
}

fn page(panels: &Panels) -> Value {
    rendered(panels)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null)
}

fn refusal(entry: Entry) -> Option<Refused> {
    rendered(&panels(vec![entry], Vec::new())).err()
}

#[test]
fn every_group_entry_and_widget_is_written_where_dashy_reads_it() {
    let mut monitored = entry();
    monitored.name = "Uptime Kuma".to_owned();
    monitored.icon = "https://cdn.example/kuma.svg".to_owned();
    monitored.check = None;
    monitored.widget = Some(EntryWidget::StatusPage {
        url: "http://uptime-kuma:3001/".to_owned(),
        page: "household".to_owned(),
    });
    let asked = panels(
        vec![entry(), monitored],
        vec![
            PageWidget::Resources,
            PageWidget::Search {
                url: "https://search.example/?q=".to_owned(),
            },
        ],
    );

    assert_eq!(
        page(&asked),
        json!({
            "pageInfo": {"title": "Home"},
            "appConfig": {
                "preventWriteToDisk": true,
                "disableConfiguration": true,
                "webSearch": {
                    "searchEngine": "custom",
                    "customSearchEngine": "https://search.example/?q=",
                    "openingMethod": "newtab"
                }
            },
            "sections": [
                {
                    "name": "Watch",
                    "items": [
                        {
                            "title": "Jellyfin",
                            "description": "Watch your library",
                            "icon": "hl-jellyfin",
                            "url": "http://192.0.2.10:8096",
                            "statusCheck": true,
                            "statusCheckUrl": "http://jellyfin:8096/health"
                        },
                        {
                            "title": "Uptime Kuma",
                            "description": "Watch your library",
                            "icon": "https://cdn.example/kuma.svg",
                            "url": "http://192.0.2.10:8096",
                            "statusCheck": false
                        }
                    ],
                    "widgets": [
                        {
                            "type": "uptime-kuma-status-page",
                            "options": {"host": "http://uptime-kuma:3001", "slug": "household"}
                        }
                    ]
                },
                {"name": "This machine", "items": [], "widgets": [{"type": "system-info"}]}
            ]
        })
    );
}

#[test]
fn a_page_with_no_widgets_has_no_search_and_no_machine_section() {
    assert_eq!(
        page(&panels(vec![entry()], Vec::new()))
            .pointer("/sections")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        page(&panels(vec![entry()], Vec::new()))
            .pointer("/appConfig")
            .cloned(),
        Some(json!({"preventWriteToDisk": true, "disableConfiguration": true}))
    );
}

#[test]
fn the_empty_panel_list_is_the_page_the_plugin_ships() {
    assert_eq!(
        rendered(&Panels {
            groups: Vec::new(),
            widgets: Vec::new(),
        })
        .ok()
        .as_deref(),
        Some(include_str!("../../../config/dashy/conf.yml"))
    );
}

#[test]
fn an_icon_file_name_is_drawn_from_the_dashboard_icons_and_any_other_name_as_it_is() {
    for (asked, drawn) in [
        ("sonarr.png", "hl-sonarr"),
        ("radarr.svg", "hl-radarr"),
        ("lidarr.webp", "hl-lidarr"),
        ("mdi-television", "mdi-television"),
        ("notes.txt", "notes.txt"),
        ("si-github", "si-github"),
    ] {
        let mut one = entry();
        one.icon = asked.to_owned();
        assert_eq!(
            page(&panels(vec![one], Vec::new()))
                .pointer("/sections/0/items/0/icon")
                .cloned(),
            Some(json!(drawn)),
            "{asked}"
        );
    }
}

#[test]
fn an_address_that_is_not_plain_http_is_refused_naming_it() {
    for link in [
        "javascript:alert(1)",
        "ftp://192.0.2.10",
        "http://",
        "http:///path",
        "http://192.0.2.10/<script>",
        "http://DASHY_HOST:8096",
    ] {
        let mut one = entry();
        one.link = link.to_owned();
        assert_eq!(
            refusal(one),
            Some(Refused::Address(link.to_owned())),
            "{link}"
        );
    }
    let mut long = entry();
    long.link = format!("http://{}", "a".repeat(LONGEST));
    assert!(matches!(refusal(long), Some(Refused::Address(_))));
}

#[test]
fn a_check_a_status_page_and_a_search_are_held_to_the_address_grammar() {
    let mut checked = entry();
    checked.check = Some("file:///etc/passwd".to_owned());
    assert_eq!(
        refusal(checked),
        Some(Refused::Address("file:///etc/passwd".to_owned()))
    );

    let mut monitored = entry();
    monitored.widget = Some(EntryWidget::StatusPage {
        url: "gopher://kuma".to_owned(),
        page: "household".to_owned(),
    });
    assert_eq!(
        refusal(monitored),
        Some(Refused::Address("gopher://kuma".to_owned()))
    );

    let mut paged = entry();
    paged.widget = Some(EntryWidget::StatusPage {
        url: "http://uptime-kuma:3001".to_owned(),
        page: "VUE_APP_SLUG".to_owned(),
    });
    assert_eq!(
        refusal(paged),
        Some(Refused::Text("VUE_APP_SLUG".to_owned()))
    );

    assert_eq!(
        rendered(&panels(
            vec![entry()],
            vec![PageWidget::Search {
                url: "data:text/html,x".to_owned()
            }]
        ))
        .err(),
        Some(Refused::Address("data:text/html,x".to_owned()))
    );
}

#[test]
fn an_icon_that_is_neither_a_name_nor_an_address_is_refused_naming_it() {
    for icon in [
        "",
        ".hidden",
        "fa fa-film",
        "javascript:alert(1)",
        "VITE_APP_ICON.png",
    ] {
        let mut one = entry();
        one.icon = icon.to_owned();
        assert_eq!(refusal(one), Some(Refused::Icon(icon.to_owned())), "{icon}");
    }
}

#[test]
fn text_holding_a_control_character_or_an_expansion_is_refused_naming_it() {
    for name in ["Line\nbreak", "DASHY_SECRET", "VITE_APP_X", "VUE_APP_Y"] {
        let mut one = entry();
        one.name = name.to_owned();
        assert_eq!(refusal(one), Some(Refused::Text(name.to_owned())), "{name}");
    }
    let mut described = entry();
    described.description = "x".repeat(LONGEST + 1);
    assert!(matches!(refusal(described), Some(Refused::Text(_))));
    let mut grouped = panels(vec![entry()], Vec::new());
    if let Some(group) = grouped.groups.first_mut() {
        group.name = "\u{7}".to_owned();
    }
    assert_eq!(
        rendered(&grouped).err(),
        Some(Refused::Text("\u{7}".to_owned()))
    );
}

#[test]
fn markup_in_any_text_is_refused_naming_it() {
    let payload = "<img src=x onerror=alert(1)>";
    let mut named = entry();
    named.name = payload.to_owned();
    let mut described = entry();
    described.description = payload.to_owned();
    let mut paged = entry();
    paged.widget = Some(EntryWidget::StatusPage {
        url: "http://uptime-kuma:3001".to_owned(),
        page: payload.to_owned(),
    });
    for asked in [named, described, paged] {
        assert_eq!(refusal(asked), Some(Refused::Text(payload.to_owned())));
    }
    let mut grouped = panels(vec![entry()], Vec::new());
    if let Some(group) = grouped.groups.first_mut() {
        group.name = payload.to_owned();
    }
    assert_eq!(
        rendered(&grouped).err(),
        Some(Refused::Text(payload.to_owned()))
    );
    for markup in ["a\"b", "a'b", "a`b", "a&b", "a>b"] {
        let mut one = entry();
        one.description = markup.to_owned();
        assert_eq!(
            refusal(one),
            Some(Refused::Text(markup.to_owned())),
            "{markup}"
        );
    }
}

#[test]
fn a_check_or_status_page_off_the_stacks_network_is_refused_naming_it() {
    for asked in [
        "http://127.0.0.1:8080/health",
        "http://169.254.169.254/latest/meta-data",
        "http://[fd00:ec2::254]/",
        "http://metadata.google.internal/computeMetadata/v1",
        "http://metadata/computeMetadata/v1",
        "http://METADATA/computeMetadata/v1",
        "http://instance-data/latest/meta-data",
        "https://kubernetes:443/api",
        "http://ip6-localhost:8080/",
        "http://ip6-loopback:8080/",
        "http://ip6-localnet/",
        "http://ip6-mcastprefix/",
        "http://ip6-allnodes/",
        "http://ip6-allrouters/",
        "http://3f4e5d6c7b8a:8080/healthz",
        "http://0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef/",
        "http://localhost:8080/healthz",
        "https://example.com/health",
        "http://user:pass@media:8096/health",
        "http://-media:8096/health",
        "http://media-:8096/health",
        "http://8096:8096/health",
        "http://[media/health",
    ] {
        let mut checked = entry();
        checked.check = Some(asked.to_owned());
        assert_eq!(
            refusal(checked),
            Some(Refused::Unreached(asked.to_owned())),
            "{asked}"
        );
        let mut monitored = entry();
        monitored.widget = Some(EntryWidget::StatusPage {
            url: asked.to_owned(),
            page: "household".to_owned(),
        });
        assert_eq!(
            refusal(monitored),
            Some(Refused::Unreached(asked.to_owned())),
            "{asked}"
        );
    }
}

#[test]
fn a_check_on_a_service_of_the_stack_is_written_as_it_parses() {
    for (asked, written) in [
        ("http://media:8096/health", "http://media:8096/health"),
        (
            "https://uptime-kuma:3001/api",
            "https://uptime-kuma:3001/api",
        ),
        (
            "http://gluetun:8081/api/v2/app/version?a=b",
            "http://gluetun:8081/api/v2/app/version?a=b",
        ),
        ("http://MEDIA:8096", "http://media:8096/"),
        ("http://3f4e5d6c7b8:8080/x", "http://3f4e5d6c7b8:8080/x"),
        ("http://mediaserver12:80/x", "http://mediaserver12/x"),
        ("http://media:80/./health/../ping", "http://media/ping"),
    ] {
        let mut checked = entry();
        checked.check = Some(asked.to_owned());
        assert_eq!(
            page(&panels(vec![checked], Vec::new()))
                .pointer("/sections/0/items/0/statusCheckUrl")
                .cloned(),
            Some(json!(written)),
            "{asked}"
        );
    }
}

#[test]
fn a_check_carrying_a_fragment_is_refused_naming_it() {
    let mut checked = entry();
    checked.check = Some("http://media:8096/health#f".to_owned());
    assert_eq!(
        refusal(checked),
        Some(Refused::Address("http://media:8096/health#f".to_owned()))
    );
}

#[test]
fn a_status_page_is_written_as_the_origin_it_parses_to() {
    for (asked, written) in [
        ("http://uptime-kuma:3001", "http://uptime-kuma:3001"),
        ("http://Uptime-Kuma:3001/", "http://uptime-kuma:3001"),
        ("http://uptime-kuma:80", "http://uptime-kuma"),
        ("https://uptime-kuma:443/", "https://uptime-kuma"),
    ] {
        let mut monitored = entry();
        monitored.widget = Some(EntryWidget::StatusPage {
            url: asked.to_owned(),
            page: "household".to_owned(),
        });
        assert_eq!(
            page(&panels(vec![monitored], Vec::new()))
                .pointer("/sections/0/widgets/0/options/host")
                .cloned(),
            Some(json!(written)),
            "{asked}"
        );
    }
}

#[test]
fn a_status_page_carrying_a_path_query_or_fragment_is_refused_naming_it() {
    for asked in [
        "http://media:8096/x",
        "http://media:8096/?a=b",
        "http://media:8096?a=b",
        "http://media:8096/#f",
        "http://media:8096/api/status-page/other?",
    ] {
        let mut monitored = entry();
        monitored.widget = Some(EntryWidget::StatusPage {
            url: asked.to_owned(),
            page: "household".to_owned(),
        });
        assert_eq!(
            refusal(monitored),
            Some(Refused::Address(asked.to_owned())),
            "{asked}"
        );
    }
}

#[test]
fn a_status_page_name_that_could_name_another_path_is_refused_naming_it() {
    for page in ["../../api/secret", "house/hold", "House", "", "a?b"] {
        let mut monitored = entry();
        monitored.widget = Some(EntryWidget::StatusPage {
            url: "http://uptime-kuma:3001".to_owned(),
            page: page.to_owned(),
        });
        assert_eq!(
            refusal(monitored),
            Some(Refused::Text(page.to_owned())),
            "{page}"
        );
    }
}
