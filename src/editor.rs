use std::collections::HashSet;

use wasm_bindgen::JsCast;

use web_sys::Element;

use yew::prelude::*;

pub struct Editor {
    node_ref: NodeRef,
    cursor: Option<web_sys::Range>,
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub class: Option<String>,
    pub content: String,
    pub highlights: Vec<super::interval::Interval<String>>,
    pub on_change: Callback<String>,
}

impl Editor {
    fn content(node_ref: NodeRef) -> String {
        fn stringify(node: web_sys::Node) -> String {
            let child_nodes = node.child_nodes();
            let mut string = String::new();
            for i in 0..child_nodes.length() {
                let child_node = child_nodes.item(i).unwrap();
                string.push_str(&match child_node.node_type() {
                    web_sys::Node::ELEMENT_NODE if &child_node.node_name() == "DIV" => "\n".into(),
                    web_sys::Node::ELEMENT_NODE => stringify(child_node),
                    web_sys::Node::TEXT_NODE => child_node.node_value().unwrap(),
                    n => unimplemented!("node type {}", n),
                })
            }
            string
        }

        stringify(node_ref.cast().unwrap())
    }

    fn get_cursor(&self) -> Option<usize> {
        let window = web_sys::window().unwrap();
        let sel = window.get_selection().unwrap().unwrap();
        sel.get_range_at(0).ok().and_then(|r| {
            if r.start_container()
                .unwrap()
                .parent_node()
                .unwrap()
                .parent_node()
                .unwrap()
                != self.node_ref.get().unwrap()
            {
                return None;
            }
            let n = r
                .start_container()
                .unwrap()
                .parent_node()
                .unwrap()
                .dyn_ref::<web_sys::HtmlElement>()
                .unwrap()
                .dataset()
                .get("start")
                .unwrap()
                .parse::<usize>()
                .unwrap();
            Some(n + r.start_offset().unwrap() as usize)
        })
    }

    fn set_cursor(&mut self, cursor: Option<usize>) {
        // TODO
    }

    fn clear(&mut self) {
        while let Some(node) = self.node_ref.cast::<web_sys::Node>().unwrap().first_child() {
            self.node_ref
                .cast::<web_sys::Node>()
                .unwrap()
                .remove_child(&node);
        }
    }

    fn render(&mut self, ctx: &yew::Context<Self>) {
        log::info!(
            "re-render ({:?}, {:?})",
            ctx.props().content,
            ctx.props().highlights,
        );
        let text = self
            .node_ref
            .cast::<Element>()
            .unwrap()
            .text_content()
            .unwrap();
        let cursor = self.get_cursor();
        let content = Self::content(self.node_ref.clone());

        self.clear();

        // render
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let range = document.create_range().unwrap();
        let sel = window.get_selection().unwrap().unwrap();

        for (s, e, node) in super::interval::Merge::from_iter(
            std::iter::once(super::interval::Interval(
                0,
                ctx.props().content.chars().count(),
                "".to_owned(),
            ))
            .chain(ctx.props().highlights.clone()),
        )
        .map(|super::interval::Interval(s, e, c)| {
            let mut node = document.create_element("span").unwrap();
            for class in c {
                node.dyn_ref::<web_sys::Element>()
                    .unwrap()
                    .class_list()
                    .add_1(&class);
            }
            node.dyn_ref::<web_sys::HtmlElement>()
                .unwrap()
                .dataset()
                .set("start", &format!("{}", s))
                .unwrap();
            node.dyn_ref::<web_sys::HtmlElement>()
                .unwrap()
                .dataset()
                .set("end", &format!("{}", e))
                .unwrap();
            node.set_text_content(Some(
                &ctx.props().content[ctx
                    .props()
                    .content
                    .char_indices()
                    .nth(s)
                    .map(|(c, _)| c)
                    .unwrap_or(ctx.props().content.len())
                    ..ctx
                        .props()
                        .content
                        .char_indices()
                        .nth(e)
                        .map(|(i, _)| i)
                        .unwrap_or(ctx.props().content.len())],
            ));
            (s, e, node)
        }) {
            /*).map(|super::interval::Interval(s, e, c)| html! {
                <span class={classes!(c.iter().collect::<Vec<_>>())}>
                    {
                        &ctx.props().content[
                            ctx.props().content.char_indices().nth(s).unwrap().0..
                                ctx.props().content.char_indices().nth(e).map(|(i, _)| i).unwrap_or(ctx.props().content.len())]
                    }
                </span>
            }) {*/
            self.node_ref
                .cast::<web_sys::Node>()
                .unwrap()
                .append_child(&node);
            if let Some(c) = cursor {
                if s <= c && c <= e {
                    let node = node.child_nodes().get(0).unwrap();
                    sel.remove_all_ranges();
                    sel.collapse_with_offset(Some(&node), (c - s) as _);
                    //sel.add_range(&range);
                }
            }
        }

        /*let text = document.create_text_node(&content);
        self.node_ref
            .cast::<web_sys::Node>()
            .unwrap()
            .append_child(&text);*/

        self.set_cursor(cursor);
    }
}

impl Component for Editor {
    type Message = ();
    type Properties = Props;

    fn create(_: &yew::Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
            cursor: None,
        }
    }

    fn changed(&mut self, ctx: &yew::Context<Self>) -> bool {
        self.render(ctx);
        false
    }

    fn update(&mut self, _: &yew::Context<Self>, _: Self::Message) -> bool {
        false
    }

    fn rendered(&mut self, _: &yew::Context<Self>, _: bool) {
        //self.render(); // yuck!
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        log::trace!("view()");
        let spans = super::interval::Merge::from_iter(
            std::iter::once(super::interval::Interval(
                0,
                ctx.props().content.chars().count(),
                "".to_owned(),
            ))
            .chain(ctx.props().highlights.clone()),
        );
        let oninput = {
            let node_ref = self.node_ref.clone();
            ctx.props().on_change.reform(move |_| {
                let content = Self::content(node_ref.clone());
                content
            })
        };
        html! {
            <div class={&ctx.props().class} style="white-space:pre" {oninput} ref={self.node_ref.clone()} contenteditable="true">
                {
                    for spans.map(|super::interval::Interval(s, e, c)| html! {
                        <span data-start={format!("{}", s)} data-end={format!("{}", e)} class={classes!(c.iter().collect::<Vec<_>>())}>
                            {
                                &ctx.props().content[
                                    ctx.props().content.char_indices().nth(s).unwrap().0..
                                        ctx.props().content.char_indices().nth(e).map(|(i, _)| i).unwrap_or(ctx.props().content.len())]
                            }
                        </span>
                    })
                }
            </div>
        }
    }
}
