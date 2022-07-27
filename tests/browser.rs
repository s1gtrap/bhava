use wasm_bindgen_test::*;

use yew::prelude::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

struct App<C>(C::Properties)
where
    C: Component;

impl<C> Component for App<C>
where
    C: Component,
    C::Properties: Clone + std::fmt::Debug,
{
    type Message = C::Properties;
    type Properties = C::Properties;

    fn create(ctx: &Context<Self>) -> Self {
        App(ctx.props().clone())
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.0 = msg;
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <C ..self.0.clone() />
        }
    }
}

fn render<C>(props: C::Properties) -> String
where
    C: Component,
{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let elem = document.create_element("div").unwrap();
    yew::start_app_with_props_in_element::<C>(elem.clone(), props);
    elem.inner_html()
}

fn render_with_changes<C, I>(props: C::Properties, i: I) -> String
where
    C: Component,
    I: IntoIterator<Item = C::Properties>,
    C::Properties: Clone + std::fmt::Debug,
{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let elem = document.create_element("div").unwrap();
    let app = yew::start_app_with_props_in_element::<App<C>>(elem.clone(), props);
    for p in i {
        app.send_message(p.clone());
    }
    elem.inner_html()
}

#[wasm_bindgen_test]
fn test_editor_init() {
    assert_eq!(
        render::<bhava::Editor>(bhava::Props {
            content: "".into(),
            spans: vec![],
        }),
        r#"<div contenteditable="true"></div>"#,
    );
    assert_eq!(
        render::<bhava::Editor>(bhava::Props {
            content: "Hello, world!".into(),
            spans: vec![],
        }),
        r#"<div contenteditable="true">Hello, world!</div>"#,
    );
    assert_eq!(
        render::<bhava::Editor>(bhava::Props {
            content: "Hello, world!".to_owned(),
            spans: vec![(
                bhava::Mask::Byte(1),
                bhava::Mask::Byte(4),
                "bg-red".to_owned(),
            )],
        }),
        r#"<div contenteditable="true"><span>H</span><span class="bg-red">ell</span><span class="">o, world!</span></div>"#,
    );
    assert_eq!(
        render::<bhava::Editor>(bhava::Props {
            content: "Hello, world!".to_owned(),
            spans: vec![(
                bhava::Mask::Byte(2),
                bhava::Mask::Byte(10),
                "bg-blue".to_owned(),
            )],
        }),
        r#"<div contenteditable="true"><span>He</span><span class="bg-blue">llo, wor</span><span class="">ld!</span></div>"#,
    );
    assert_eq!(
        render::<bhava::Editor>(bhava::Props {
            content: "Hello, world!".to_owned(),
            spans: vec![
                (
                    bhava::Mask::Byte(2),
                    bhava::Mask::Byte(3),
                    "bg-red".to_owned(),
                ),
                (
                    bhava::Mask::Byte(5),
                    bhava::Mask::Byte(11),
                    "bg-blue".to_owned(),
                ),
            ],
        }),
        r#"<div contenteditable="true"><span>He</span><span class="bg-red">l</span><span class="">lo</span><span class="bg-blue">, worl</span><span class="">d!</span></div>"#,
    );
    assert_eq!(
        render::<bhava::Editor>(bhava::Props {
            content: "Hello, world!".to_owned(),
            spans: vec![
                (
                    bhava::Mask::Byte(2),
                    bhava::Mask::Byte(9),
                    "bg-blue".to_owned(),
                ),
                (
                    bhava::Mask::Byte(5),
                    bhava::Mask::Byte(7),
                    "bg-red".to_owned(),
                ),
            ],
        }),
        r#"<div contenteditable="true"><span>He</span><span class="bg-blue">llo</span><span class="bg-blue bg-red">, </span><span class="bg-blue">wo</span><span class="">rld!</span></div>"#,
    );
    assert_eq!(
        render::<bhava::Editor>(bhava::Props {
            content: "Hello, world!".to_owned(),
            spans: vec![
                (
                    bhava::Mask::Byte(1),
                    bhava::Mask::Byte(7),
                    "bg-blue".to_owned(),
                ),
                (
                    bhava::Mask::Byte(5),
                    bhava::Mask::Byte(10),
                    "bg-red".to_owned(),
                ),
            ],
        }),
        r#"<div contenteditable="true"><span>H</span><span class="bg-blue">ello</span><span class="bg-blue bg-red">, </span><span class="bg-red">wor</span><span class="">ld!</span></div>"#,
    );
    assert_eq!(
        render::<bhava::Editor>(bhava::Props {
            content: "Hello, world!".to_owned(),
            spans: vec![
                (
                    bhava::Mask::Byte(1),
                    bhava::Mask::Byte(6),
                    "bg-red".to_owned(),
                ),
                (
                    bhava::Mask::Byte(3),
                    bhava::Mask::Byte(11),
                    "bg-blue".to_owned(),
                ),
            ],
        }),
        r#"<div contenteditable="true"><span>H</span><span class="bg-red">el</span><span class="bg-red bg-blue">lo,</span><span class="bg-blue"> worl</span><span class="">d!</span></div>"#,
    );
    assert_eq!(
        render::<bhava::Editor>(bhava::Props {
            content: "Hello, world!".to_owned(),
            spans: vec![
                (
                    bhava::Mask::Byte(1),
                    bhava::Mask::Byte(5),
                    "bg-red".to_owned(),
                ),
                (
                    bhava::Mask::Byte(3),
                    bhava::Mask::Byte(8),
                    "bg-green".to_owned(),
                ),
                (
                    bhava::Mask::Byte(7),
                    bhava::Mask::Byte(10),
                    "bg-blue".to_owned(),
                ),
            ],
        }),
        r#"<div contenteditable="true"><span>H</span><span class="bg-red">el</span><span class="bg-red bg-green">lo</span><span class="bg-green">, </span><span class="bg-green bg-blue">w</span><span class="bg-blue">or</span><span class="">ld!</span></div>"#,
    );
}

#[wasm_bindgen_test]
fn test_editor_changes() {
    assert_eq!(
        render_with_changes::<bhava::Editor, _>(
            bhava::Props {
                content: "Hello, world!".into(),
                spans: vec![],
            },
            [bhava::Props {
                content: "Hello, world!".into(),
                spans: vec![],
            }],
        ),
        r#"<div contenteditable="true">Hello, world!</div>"#,
    );
    assert_eq!(
        render_with_changes::<bhava::Editor, _>(
            bhava::Props {
                content: "Hello, world!".into(),
                spans: vec![],
            },
            [bhava::Props {
                content: "Hello, world!".into(),
                spans: vec![(
                    bhava::Mask::Byte(4),
                    bhava::Mask::Byte(9),
                    "bg-blue".to_owned(),
                )],
            }],
        ),
        r#"<div contenteditable="true"><span>Hell</span><span class="bg-blue">o, wo</span><span class="">rld!</span></div>"#,
    );
    assert_eq!(
        render_with_changes::<bhava::Editor, _>(
            bhava::Props {
                content: "Hello, world!".into(),
                spans: vec![],
            },
            [bhava::Props {
                content: "Hello, world!".into(),
                spans: vec![
                    (
                        bhava::Mask::Byte(2),
                        bhava::Mask::Byte(7),
                        "bg-blue".to_owned(),
                    ),
                    (
                        bhava::Mask::Byte(4),
                        bhava::Mask::Byte(9),
                        "bg-red".to_owned(),
                    ),
                ],
            }],
        ),
        r#"<div contenteditable="true"><span>He</span><span class="bg-blue">ll</span><span class="bg-blue bg-red">o, </span><span class="bg-red">wo</span><span class="">rld!</span></div>"#,
    );
    /*assert_eq!(
        render_with_changes::<bhava::Editor, _>(
            bhava::Props {
                content: "Hello, world!".into(),
                spans: vec![(
                    bhava::Mask::Byte(5),
                    bhava::Mask::Byte(8),
                    "green".to_owned(),
                )],
            },
            [bhava::Props {
                content: "Hello, world!".into(),
                spans: vec![],
            }],
        ),
        r#"<div contenteditable="true"><span>Hello</span><span class="">, w</span><span class="">orld!</span></div>"#,
    );*/
}
