use dioxus::prelude::*;
use regex::Regex;

fn format_release_note(note: &str) -> String {
    let url_re = Regex::new(r"https://github.com/.+/.+/pull/(\d+)").unwrap();
    let list_symbol_re = Regex::new(r"^\* ").unwrap();

    let mut ret = String::from("");
    for line in note.lines() {
        let caps = url_re.captures(line.trim()).unwrap();
        let url = caps.get(0).unwrap().as_str();
        let num = caps.get(1).unwrap().as_str();
        let fmt = format!("[#{} {}]", num, url);
        let res = url_re.replace(line, fmt);
        let res = list_symbol_re.replace(&res, " ");
        ret = ret + &res + "\n";
    }
    ret.trim_end().to_string()
}

fn app(cx: Scope) -> Element {
    let value = use_state(&cx, || String::from(""));

    let formatted = format_release_note(&value);

    cx.render(rsx!(
        div {
            textarea {
                oninput: move |ev| {
                    value.set(ev.value.clone());
                },
            }
            textarea {
                value: "{formatted}"
            }
        }
    ))
}

fn main() {
    dioxus::web::launch(app);
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
