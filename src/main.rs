use dioxus::prelude::*;
use regex::Regex;

fn format_release_note(note: &str) -> String {
    let url_re = Regex::new(r"https://github.com/.+/.+/pull/(\d+)").unwrap();
    let list_symbol_re = Regex::new(r"^\* ").unwrap();

    let mut ret = String::from("");
    for line in note.lines() {
        let maybe_caps = url_re.captures(line.trim());
        if let Some(caps) = maybe_caps {
            let url = caps.get(0).unwrap().as_str();
            let num = caps.get(1).unwrap().as_str();
            let fmt = format!("[#{} {}]", num, url);
            let res = url_re.replace(line, fmt);
            let res = list_symbol_re.replace(&res, " ");
            ret = ret + &res + "\n";
        }
    }
    ret.trim_end().to_string()
}

fn app() -> Element {
    let mut value = use_signal(|| String::from(""));

    let formatted = format_release_note(&value.read());

    rsx!(
        h1 {
            style: "\
                width: 100%;\
                text-align: center;\
                ",
            "Release notes 👉 Scrapbox"
        }

        div {
            style: "\
                display: flex;\
                justify-content: center;\
                align-items: center;\
                position: relative;
                ",

            textarea {
                style: "\
                    width: 40%;\
                    height: 500px;\
                    resize: none\
                    ",
                oninput: move |ev| {
                  *value.write() = ev.clone().value();
                },
                placeholder: "リリースノートのMarkdownを貼り付けてね",
            }

            textarea {
                style: "\
                    width: 40%;\
                    height: 500px;\
                    resize: none;\
                    ",
                value: "{formatted}"
            }
        }
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
        let ret = format_release_note(
            "* hogehoge by @konbu310 in https://github.com/konbu310/hyper-launcher/pull/1",
        );
        assert_eq!(
            ret,
            " hogehoge by @konbu310 in [#1 https://github.com/konbu310/hyper-launcher/pull/1]"
        );
    }
}
