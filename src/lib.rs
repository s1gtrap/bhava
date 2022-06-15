use std::{cmp::Ordering, collections::VecDeque, ops::Sub};

use wasm_bindgen::JsCast;

use web_sys::{Element, Node};

use yew::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mask {
    Byte(usize),
    Bit(usize, usize, u8),
}

impl Mask {
    fn ceil(&self) -> usize {
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

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    pub content: String,
    pub spans: Vec<(Mask, Mask, String)>,
}

#[derive(Debug)]
pub struct Editor {
    div: NodeRef,
}

impl Editor {
    fn merge(&mut self, spans: &[(Mask, Mask, String)]) {
        log::trace!("Editor::merge({self:?}, {spans:?})");

        let mut edges: Vec<(Mask, Vec<(bool, String)>)> = vec![];
        for span in spans {
            match edges.binary_search_by_key(&span.0, |(k, _)| *k) {
                Ok(pos) => {
                    log::debug!("push {} to {:?} at idx {}", span.2, edges[pos], pos);
                    edges.get_mut(pos).unwrap().1.push((true, span.2.clone()));
                }
                Err(pos) => {
                    log::debug!("insert {:?} at idx {}", vec![span.2.clone()], pos);
                    edges.insert(pos, (span.0, vec![(true, span.2.clone())]));
                }
            }

            match edges.binary_search_by_key(&span.1, |(k, _)| *k) {
                Ok(pos) => {
                    log::debug!("push {} to {:?} at idx {}", span.2, edges[pos], pos);
                    edges.get_mut(pos).unwrap().1.push((false, span.2.clone()));
                }
                Err(pos) => {
                    log::debug!("insert {:?} at idx {}", vec![span.2.clone()], pos);
                    edges.insert(pos, (span.1, vec![(false, span.2.clone())]));
                }
            }
        }

        let mut q: VecDeque<_> = edges.iter().collect();

        let mut last = 0;
        let mut o = 0;
        let mut ni = self.contents();
        while !ni.is_empty() {
            let (s, e, _c) = ni.current();

            if let Some(ed) = q.pop_front() {
                assert!(Mask::Byte(s) < ed.0, "{:?} >= {:?}", Mask::Byte(s), ed.0);

                fn upgrade(n: &Node) -> Element {
                    if n.node_type() == Node::ELEMENT_NODE {
                        return n.dyn_ref::<Element>().unwrap().clone();
                    }

                    let document = web_sys::window().unwrap().document().unwrap();
                    let span = document.create_element("span").unwrap();

                    span.set_text_content(Some(&n.text_content().unwrap()));

                    n.parent_node()
                        .expect("no parent")
                        .replace_child(&span, n)
                        .unwrap();

                    span
                }

                if ed.0 < Mask::Byte(o + e) {
                    let (head, tail) = ni.split(ed.0 - last);

                    let head = upgrade(&head); // FIXME(s1g)
                    let tail = upgrade(&tail);

                    for c in (0..head.class_list().length())
                        .into_iter()
                        .map(|i| head.class_list().item(i).unwrap())
                    {
                        tail.class_list().add_1(&c).unwrap();
                    }

                    for change in &ed.1 {
                        if change.0 {
                            // if edge begins, append class
                            tail.class_list().add_1(&change.1).unwrap();
                        } else {
                            // if it ends, remove it from classList
                            // FIXME(s1g): could be more than one span
                            let tail = upgrade(&tail);
                            tail.class_list().remove_1(&change.1).unwrap();
                        }
                    }

                    last = ed.0.ceil();
                }
            }

            o = e;

            ni.next();
        }
    }

    fn contents(&self) -> NodeIter {
        let node = self.div.cast::<Node>().unwrap();
        NodeIter {
            current: 0,
            offset: 0,
            node,
        }
    }

    //fn add(&mut self, h: (Mask, Mask, String), ctx: &Context<Self>) {}
}

impl Component for Editor {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Editor {
            div: NodeRef::default(),
            //spans: vec![],
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.merge(&ctx.props().spans);

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

        self.merge(&ctx.props().spans);
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div ref={self.div.clone()} contenteditable="true"></div>
        }
    }
}
