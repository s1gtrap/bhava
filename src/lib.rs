use std::{cmp::Ordering, collections::VecDeque, ops::Sub};

use wasm_bindgen::JsCast;

use web_sys::{Element, Node};

use yew::prelude::*;

pub mod utils;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mask {
    Byte(usize),
    Bit(usize, usize, u8),
}

impl Mask {
    pub(crate) fn floor(&self) -> usize {
        match self {
            Mask::Byte(i) => *i,
            Mask::Bit(s, _, _) => *s,
        }
    }

    pub(crate) fn ceil(&self) -> usize {
        match self {
            Mask::Byte(i) => *i,
            Mask::Bit(s, l, _) => s + l,
        }
    }
}

impl Ord for Mask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd<Mask> for Mask {
    fn partial_cmp(&self, other: &Mask) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Mask::Byte(a), Mask::Byte(b)) => a.partial_cmp(b),
            (Mask::Bit(a, _, _), Mask::Byte(b)) => a.partial_cmp(b),
            (Mask::Byte(a), Mask::Bit(b, _, _)) => a.partial_cmp(b),
            (Mask::Bit(a, _, c), Mask::Bit(b, _, d)) if a == b => c.partial_cmp(d),
            (Mask::Bit(a, _, _), Mask::Bit(b, _, _)) => a.partial_cmp(b),
        }
    }
}

impl Sub<usize> for Mask {
    type Output = Mask;

    fn sub(self, rhs: usize) -> Self::Output {
        match self {
            Mask::Byte(i) => Mask::Byte(i - rhs),
            Mask::Bit(i, l, s) => Mask::Bit(i - rhs, l, s),
        }
    }
}

#[derive(Debug)]
pub struct NodeIter {
    current: usize,
    offset: usize,
    node: Node,
}

impl NodeIter {
    fn is_empty(&self) -> bool {
        (self.current as u32) >= self.node.child_nodes().length()
    }

    fn current(&self) -> (usize, usize, Node) {
        let item = self.node.child_nodes().item(self.current as _).unwrap();
        (
            self.offset,
            self.offset + item.text_content().unwrap().len(),
            item,
        )
    }

    fn split(&self, i: Mask) -> (Node, Node) {
        log::trace!("NodeIter::split({self:?}, {i:?})");

        let document = web_sys::window().unwrap().document().unwrap();

        fn insert_after(n: &Node, c: &Node) {
            match n.next_sibling() {
                Some(s) => n.parent_node().unwrap().insert_before(c, Some(&s)).unwrap(),
                None => n.parent_node().unwrap().append_child(c).unwrap(),
            };
        }

        let c = self.current();
        match i {
            Mask::Byte(i) if c.0 < i && i < c.1 => match c.2.node_type() {
                Node::TEXT_NODE => {
                    let text = c.2.text_content().unwrap();
                    c.2.set_text_content(Some(&text[..i]));
                    let tail: &Node = &document.create_text_node(&text[i..]);
                    insert_after(&c.2, &tail);
                    (c.2.clone(), tail.clone())
                }
                Node::ELEMENT_NODE => {
                    let text = c.2.text_content().unwrap();
                    let head: &Element = c.2.dyn_ref().unwrap();

                    log::debug!(
                        "{:?} head={:?}",
                        text,
                        (0..head.class_list().length())
                            .into_iter()
                            .map(|i| head.class_list().item(i).unwrap())
                            .collect::<Vec<_>>()
                    );

                    c.2.set_text_content(Some(&text[..i]));
                    let tail: Element = document.create_element("span").unwrap();

                    for c in (0..head.class_list().length())
                        .into_iter()
                        .map(|i| head.class_list().item(i).unwrap())
                    {
                        tail.class_list().add_1(&c).unwrap();
                    }

                    log::debug!(
                        "{:?} tail={:?}",
                        text,
                        (0..tail.class_list().length())
                            .into_iter()
                            .map(|i| head.class_list().item(i).unwrap())
                            .collect::<Vec<_>>()
                    );

                    insert_after(&c.2, &tail);
                    let tail: &Node = &tail;
                    tail.set_text_content(Some(&text[i..]));
                    (c.2.clone(), tail.clone())
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    fn next(&mut self) {
        let _el = self.node.dyn_ref::<Element>().unwrap();
        log::debug!(
            "childNodes={:?}",
            (0..self.node.child_nodes().length())
                .map(|i| {
                    self.node
                        .child_nodes()
                        .item(i)
                        .unwrap()
                        .dyn_into::<Element>()
                        .ok()
                        .map(|el| {
                            (
                                el.text_content(),
                                (0..el.class_list().length())
                                    .into_iter()
                                    .map(|i| el.class_list().item(i).unwrap())
                                    .collect::<Vec<_>>(),
                            )
                        })
                })
                .collect::<Vec<_>>(),
        );
        self.current += 1;
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub content: String,
    pub spans: Vec<(Mask, Mask, String)>,
}

#[derive(Debug)]
pub struct Editor {
    div: NodeRef,
    spans: Vec<(Mask, Mask, String)>,
}

impl Component for Editor {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Editor {
            div: NodeRef::default(),
            spans: ctx.props().spans.clone(),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        log::warn!("changed props {:?}", ctx.props());

        for span in self.spans.iter().filter(|s| !ctx.props().spans.contains(s)) {
            log::warn!("removing {:?}", span);
            utils::remove_span(
                self.div.cast::<Node>().unwrap(),
                (span.2.clone(), span.0, span.1),
            );
        }

        for span in ctx.props().spans.iter().filter(|s| !self.spans.contains(s)) {
            log::warn!("inserting {:?}", span);
            utils::insert_span(
                self.div.cast::<Node>().unwrap(),
                (span.2.clone(), span.0, span.1),
            );
        }

        self.spans = ctx.props().spans.clone();

        false // never render
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        log::info!("Editor::rendered({self:?}, {ctx:?})");

        // do actual render

        assert!(first_render, "Editor::rendered(..) called more than once");

        let document = web_sys::window().unwrap().document().unwrap();

        let node = self.div.cast::<Node>().unwrap();
        let mut lines = ctx.props().content.lines();
        if let Some(line) = lines.next() {
            let tn = document.create_text_node(line);
            node.append_child(&tn).unwrap();
        }

        //self.merge(&ctx.props().spans);
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        log::info!("render(props={:?})", ctx.props());
        html! {
            <div ref={self.div.clone()} contenteditable="true"></div>
        }
    }
}
