use super::*;
use egui::{Color32, Style, TextFormat, WidgetText, text::LayoutJob};
use log::error;

pub fn cstr_parse(s: &str, alpha: f32, style: &Style) -> WidgetText {
    let mut job = LayoutJob::default();
    cstr_parse_into_job(s, alpha, &mut job, style);
    WidgetText::LayoutJob(job)
}

pub fn cstr_parse_into_job(s: &str, alpha: f32, job: &mut LayoutJob, style: &Style) {
    let mut cur = String::new();
    let mut style_state: StyleState = default();
    let mut parse_state = ParseState::Text;
    for c in s.chars() {
        match c {
            '[' => {
                style_state.append(&mut cur, alpha, job, style);
                parse_state = ParseState::Token;
            }
            ']' => {
                if parse_state == ParseState::Token {
                    let s = cur.cstr_s(CstrStyle::Bold);
                    cstr_parse_into_job(&s, alpha, job, style);
                    parse_state = ParseState::Text;
                    cur.clear();
                } else {
                    style_state.append(&mut cur, alpha, job, style);
                    style_state.pop();
                }
            }
            '#' => {
                if parse_state == ParseState::Token {
                    parse_state = ParseState::HexColor;
                }
                cur.push(c);
            }
            ' ' => {
                match parse_state {
                    ParseState::Token => {
                        style_state.push_token(&cur);
                        cur.clear();
                    }
                    ParseState::HexColor => {
                        match Color32::from_hex(&cur) {
                            Ok(c) => style_state.push(CstrStyle::Color(c)),
                            Err(e) => error!("Failed to parse hex color \"{cur}\": {e:?}"),
                        };
                        cur.clear();
                    }
                    ParseState::Text => cur.push(c),
                };
                parse_state = ParseState::Text;
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        style_state.append(&mut cur, alpha, job, style);
    }
}
