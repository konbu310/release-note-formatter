use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use regex::Regex;
use wasm_bindgen_futures::JsFuture;

static CSS: Asset = asset!("/assets/main.css");

const DEFAULT_EXCLUDED_PREFIXES: &str = "build(deps):\nbuild(deps-dev):\nhotfix:";

fn parse_excluded_prefixes(input: &str) -> Vec<String> {
    input
        .lines()
        .map(str::trim)
        .filter(|prefix| !prefix.is_empty())
        .map(String::from)
        .collect()
}

#[cfg(test)]
#[allow(dead_code)]
fn default_excluded_prefixes() -> Vec<String> {
    parse_excluded_prefixes(DEFAULT_EXCLUDED_PREFIXES)
}

fn is_excluded_change(line: &str, excluded_prefixes: &[String]) -> bool {
    let trimmed = line.trim_start();
    let content = trimmed.strip_prefix("* ").unwrap_or(trimmed);

    excluded_prefixes
        .iter()
        .any(|prefix| content.starts_with(prefix))
}

fn format_release_note(note: &str, excluded_prefixes: &[String]) -> String {
    let url_re = Regex::new(r"https://github.com/.+/.+/pull/(\d+)").unwrap();
    let list_symbol_re = Regex::new(r"^\* ").unwrap();
    let bot_user_re = Regex::new(r"(@[^\s\[]+)\[([^\]]+)\]").unwrap();

    let mut ret = String::from("");
    for line in note.lines() {
        if is_excluded_change(line, excluded_prefixes) {
            continue;
        }

        let maybe_caps = url_re.captures(line.trim());
        if let Some(caps) = maybe_caps {
            let url = caps.get(0).unwrap().as_str();
            let num = caps.get(1).unwrap().as_str();
            let fmt = format!("[#{} {}]", num, url);
            let res = url_re.replace(line, fmt);
            let res = list_symbol_re.replace(&res, " ");
            let res = bot_user_re.replace_all(&res, "$1($2)");
            ret = ret + &res + "\n";
        }
    }
    ret.trim_end().to_string()
}

async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let Some(window) = web_sys::window() else {
        return Err(String::from("window is not available"));
    };

    let clipboard = window.navigator().clipboard();
    let promise = clipboard.write_text(&text);

    JsFuture::from(promise)
        .await
        .map_err(|_| String::from("clipboard write failed"))?;

    Ok(())
}

fn app() -> Element {
    let mut value = use_signal(|| String::from(""));
    let mut excluded_prefixes_input = use_signal(|| DEFAULT_EXCLUDED_PREFIXES.to_string());
    let mut copy_status = use_signal(|| String::from("コピー"));
    let mut copy_status_version = use_signal(|| 0_u64);

    let raw_note = value.read().clone();
    let excluded_prefixes_value = excluded_prefixes_input.read().clone();
    let excluded_prefixes = parse_excluded_prefixes(&excluded_prefixes_value);
    let formatted = format_release_note(&raw_note, &excluded_prefixes);
    let formatted_for_copy = formatted.clone();
    let is_copy_disabled = formatted.is_empty();
    let copy_status_text = copy_status.read().clone();

    rsx!(
        div {
            class: "app-shell",

            h1 {
                class: "app-title",
                "GitHub Realeases 👉 Cosense"
            }

            div {
                class: "stack-section",

                label {
                    class: "field-label",
                    "除外するプレフィックス"
                }

                textarea {
                    class: "editor-textarea prefixes-input",
                    value: "{excluded_prefixes_value}",
                    oninput: move |ev| {
                        *excluded_prefixes_input.write() = ev.value();
                    },
                    placeholder: "1行に1つずつ入力",
                },
            }

            div {
                class: "editor-grid",

                div {
                    class: "editor-panel",

                    label {
                        class: "field-label",
                        "リリースノート"
                    }

                    textarea {
                        class: "editor-textarea note-input",
                        oninput: move |ev| {
                            *value.write() = ev.value();
                        },
                        placeholder: "リリースノートのMarkdownを貼り付けてね",
                    }
                }

                div {
                    class: "editor-panel",

                    label {
                        class: "field-label",
                        "整形結果"
                    }

                    textarea {
                        class: "editor-textarea note-output",
                        value: "{formatted}",
                        readonly: true,
                    }

                    button {
                        class: "copy-button",
                        disabled: is_copy_disabled,
                        onclick: move |_| {
                            let formatted_for_copy = formatted_for_copy.clone();
                            let status_version = {
                                let mut version = copy_status_version.write();
                                *version += 1;
                                *version
                            };
                            async move {
                                match copy_to_clipboard(formatted_for_copy).await {
                                    Ok(()) => {
                                        *copy_status.write() = "コピーしました✌️".to_string();
                                    }
                                    Err(_) => {
                                        *copy_status.write() = "コピーに失敗しました😭".to_string();
                                    }
                                }

                                TimeoutFuture::new(2_000).await;

                                if *copy_status_version.read() == status_version {
                                    *copy_status.write() = "コピー".to_string();
                                }
                            }
                        },
                        "{copy_status_text}"
                    }
                }
            }
        }

        document::Stylesheet { href: CSS }
    )
}

fn main() {
    dioxus::launch(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_one_line() {
        let excluded_prefixes = default_excluded_prefixes();
        let ret = format_release_note(
            "* hogehoge by @konbu310 in https://github.com/konbu310/hyper-launcher/pull/1",
            &excluded_prefixes,
        );
        assert_eq!(
            ret,
            " hogehoge by @konbu310 in [#1 https://github.com/konbu310/hyper-launcher/pull/1]"
        );
    }

    #[test]
    fn format_bot_user_name() {
        let excluded_prefixes = default_excluded_prefixes();
        let ret = format_release_note(
            "* bump foo by @dependabot[bot] in https://github.com/konbu310/hyper-launcher/pull/1",
            &excluded_prefixes,
        );
        assert_eq!(
            ret,
            " bump foo by @dependabot(bot) in [#1 https://github.com/konbu310/hyper-launcher/pull/1]"
        );
    }

    #[test]
    fn exclude_default_prefixes() {
        let excluded_prefixes = default_excluded_prefixes();
        let ret = format_release_note(
            "\
* build(deps): bump foo by @dependabot[bot] in https://github.com/konbu310/hyper-launcher/pull/1
* feat: add bar by @konbu310 in https://github.com/konbu310/hyper-launcher/pull/2",
            &excluded_prefixes,
        );
        assert_eq!(
            ret,
            " feat: add bar by @konbu310 in [#2 https://github.com/konbu310/hyper-launcher/pull/2]"
        );
    }
}
