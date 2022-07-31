use wasm_bindgen::JsCast;

use wasm_bindgen_test::*;

use web_sys::{Element, Node, Text};

use super::Mask;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn to_vec(n: web_sys::NodeList) -> Vec<web_sys::Node> {
    (0..n.length()).map(|i| n.get(i).unwrap()).collect()
}

pub fn children<N>(n: N) -> Vec<(web_sys::Node, usize, usize)>
where
    N: AsRef<web_sys::Node>,
{
    let n = n.as_ref().child_nodes();
    let mut o = 0;
    (0..n.length())
        .map(|i| {
            let n = n.item(i).unwrap();
            let mut s = o;
            o += match n.node_type() {
                web_sys::Node::ELEMENT_NODE if n.node_name() == "DIV" => {
                    s += 1;
                    n.text_content().unwrap().len() + 1
                }
                _ => n.text_content().unwrap().len(),
            };
            (n, s, o)
        })
        .collect()
}

fn content<N>(n: N) -> Result<String, String>
where
    N: AsRef<Node>,
{
    n.as_ref()
        .dyn_ref::<Element>()
        .map(Element::outer_html)
        .ok_or_else(|| n.as_ref().text_content().unwrap())
}

fn insert_after<N, M>(n: N, m: M)
where
    N: AsRef<web_sys::Node>,
    M: AsRef<web_sys::Node>,
{
    web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
        "insert_after({:?}, {:?})",
        content(&n),
        content(&m),
    )));
    n.as_ref()
        .parent_node()
        .expect(&format!(
            "{:?} has no parent!",
            (n.as_ref().text_content(), n.as_ref().parent_node())
        ))
        .insert_before(m.as_ref(), n.as_ref().next_sibling().as_ref())
        .unwrap();
}

#[wasm_bindgen_test]
fn test_insert_after() {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_element("SPAN").unwrap();
    parent.append_child(&child0).unwrap();

    let child1 = document.create_element("SPAN").unwrap();
    insert_after(&child0, &child1);
    assert_eq!(
        &parent.child_nodes().get(0).unwrap(),
        <web_sys::Element as AsRef<web_sys::Node>>::as_ref(&child0),
    );
    assert_eq!(
        &parent.child_nodes().get(1).unwrap(),
        <web_sys::Element as AsRef<web_sys::Node>>::as_ref(&child1),
    );

    let child2 = document.create_text_node("dummy");
    insert_after(&child1, &child2);
    assert_eq!(
        &parent.child_nodes().get(0).unwrap(),
        <web_sys::Element as AsRef<web_sys::Node>>::as_ref(&child0),
    );
    assert_eq!(
        &parent.child_nodes().get(1).unwrap(),
        <web_sys::Element as AsRef<web_sys::Node>>::as_ref(&child1),
    );
    assert_eq!(
        &parent.child_nodes().get(2).unwrap(),
        <Text as AsRef<Node>>::as_ref(child2.as_ref()),
    );

    let child2p5 = document.create_text_node("dummy");
    insert_after(&child1, &child2);
}

fn depth<N>(n: N) -> usize
where
    N: AsRef<web_sys::Node>,
{
    match n.as_ref().node_type() {
        web_sys::Node::TEXT_NODE => 0,
        web_sys::Node::ELEMENT_NODE => to_vec(n.as_ref().child_nodes())
            .iter()
            .map(depth)
            .max()
            .map(|d| d + 1)
            .unwrap_or(0),
        t => todo!("nodeType {t}"),
    }
}

#[wasm_bindgen_test]
fn test_depth() {
    let document = web_sys::window().unwrap().document().unwrap();

    let text = document.create_text_node("Hello, world!");
    assert_eq!(depth(text), 0);

    let div = document.create_element("DIV").unwrap();
    assert_eq!(depth(div), 0);

    let div = document.create_element("DIV").unwrap();
    div.set_inner_html("Hello, world!");
    assert_eq!(depth(div), 1);

    let div = document.create_element("DIV").unwrap();
    div.set_inner_html("<span>Hello, world!</span>");

    let div = document.create_element("DIV").unwrap();
    div.set_inner_html("<span>Hell</span><span>o, wor</span><span>ld!</span>");
    assert_eq!(depth(div), 2);

    let div = document.create_element("DIV").unwrap();
    div.set_inner_html("<span>Hell<span>o, wor</span>ld!</span>");
    assert_eq!(depth(div), 3);
}

fn split_at<N>(n: N, o: usize) -> Vec<web_sys::Node>
where
    N: AsRef<web_sys::Node>,
{
    web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
        "split_at({:?}, {:?})",
        content(&n),
        o
    )));
    let document = web_sys::window().unwrap().document().unwrap();

    let n = n.as_ref();

    if o == 0 || o >= n.text_content().unwrap().len() {
        let ret = vec![n.clone()];
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
            "return {:?}",
            ret.iter().map(content).collect::<Vec<_>>()
        )));
        return ret;
    }

    let ret = match n.node_type() {
        web_sys::Node::TEXT_NODE => {
            let tail_str = &n.text_content().unwrap()[o..];

            n.set_text_content(Some(&n.text_content().unwrap()[..o]));

            let tail = document.create_text_node(tail_str);
            web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
                "split_at({:?}, {:?}) calling insert_at({:?}, {:?}) b/c TEXT_NODE",
                content(&n),
                o,
                content(&n),
                content(&tail),
            )));
            insert_after(n, &tail);

            let tail: &web_sys::Node = web_sys::Node::as_ref(&tail);
            vec![n.clone(), tail.clone()]
        }
        web_sys::Node::ELEMENT_NODE => {
            assert!(depth(n) <= 1);

            let head_str = &n.text_content().unwrap()[..o];
            let tail_str = &n.text_content().unwrap()[o..];

            n.set_text_content(Some(head_str));

            let tail = document.create_element(&n.node_name()).unwrap();
            tail.set_text_content(Some(tail_str));
            web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
                "split_at({:?}, {:?}) calling insert_at({:?}, {:?}) b/c ELEMENT_NODE",
                content(&n),
                o,
                content(&n),
                content(&tail),
            )));
            insert_after(n, &tail);

            let tail: &web_sys::Node = web_sys::Node::as_ref(&tail);
            vec![n.clone(), tail.clone()]
        }
        t => todo!("nodeType {t}"),
    };
    web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
        "return {:?}",
        ret.iter().map(content).collect::<Vec<_>>()
    )));
    ret
}

#[wasm_bindgen_test]
fn test_split_at() {
    let document = web_sys::window().unwrap().document().unwrap();

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello, world!");
    parent.append_child(&child0).unwrap();
    assert_eq!(split_at(&child0, 0).len(), 1);
    assert_eq!(&split_at(&child0, 1)[0], child0.as_ref());

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello, world!");
    parent.append_child(&child0).unwrap();
    assert_eq!(
        split_at(&child0, 0),
        vec![<Text as AsRef<Node>>::as_ref(&child0).clone()],
    );

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello, world!");
    parent.append_child(&child0).unwrap();
    assert_eq!(
        split_at(&child0, 1),
        vec![
            <Text as AsRef<Node>>::as_ref(&child0).clone(),
            parent.child_nodes().get(1).unwrap(),
        ],
    );

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello, world!");
    parent.append_child(&child0).unwrap();
    assert_eq!(
        split_at(&child0, 12),
        vec![
            <Text as AsRef<Node>>::as_ref(&child0).clone(),
            parent.child_nodes().get(1).unwrap(),
        ],
    );

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello, world!");
    parent.append_child(&child0).unwrap();
    assert_eq!(
        split_at(&child0, 13),
        vec![<Text as AsRef<Node>>::as_ref(&child0).clone()],
    );
}

#[wasm_bindgen_test]
fn test_split_at_dom() {
    let document = web_sys::window().unwrap().document().unwrap();

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello, world!");
    parent.append_child(&child0).unwrap();
    assert_eq!(&split_at(&child0, 1)[0], child0.as_ref());
    let v = to_vec(parent.child_nodes());
    assert_eq!(v[0].text_content().unwrap(), "H");
    assert_eq!(v[1].text_content().unwrap(), "ello, world!");

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello, world!");
    parent.append_child(&child0).unwrap();
    split_at(&child0, 6);
    let v = to_vec(parent.child_nodes());
    assert_eq!(v[0].text_content().unwrap(), "Hello,");
    assert_eq!(v[1].text_content().unwrap(), " world!");

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_element("SPAN").unwrap();
    child0.set_text_content(Some("Hello, world!"));
    parent.append_child(&child0).unwrap();
    split_at(&child0, 9);
    let v = to_vec(parent.child_nodes());
    assert_eq!(v[0].text_content().unwrap(), "Hello, wo");
    assert_eq!(v[1].text_content().unwrap(), "rld!");

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_element("SPAN").unwrap();
    child0.set_text_content(Some("Hello, world!"));
    parent.append_child(&child0).unwrap();
    split_at(&child0, 12);
    let v = to_vec(parent.child_nodes());
    assert_eq!(v[0].text_content().unwrap(), "Hello, world");
    assert_eq!(v[1].text_content().unwrap(), "!");
}

#[wasm_bindgen_test]
fn test_split_at_zero() {
    let document = web_sys::window().unwrap().document().unwrap();

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello, world!");
    parent.append_child(&child0).unwrap();
    split_at(&child0, 0);
    assert_eq!(
        to_vec(parent.child_nodes())
            .iter()
            .filter_map(web_sys::Node::text_content)
            .collect::<Vec<_>>(),
        vec!["Hello, world!".to_owned()],
    );
}

#[wasm_bindgen_test]
fn test_split_at_len() {
    let document = web_sys::window().unwrap().document().unwrap();

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello, world!");
    parent.append_child(&child0).unwrap();
    split_at(&child0, 13);
    assert_eq!(
        to_vec(parent.child_nodes())
            .iter()
            .filter_map(web_sys::Node::text_content)
            .collect::<Vec<_>>(),
        vec!["Hello, world!".to_owned()],
    );
}

fn upgrade<N>(n: N, s: &str) -> Element
where
    N: AsRef<web_sys::Node>,
{
    web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
        "upgrade({:?})",
        content(&n),
    )));
    let document = web_sys::window().unwrap().document().unwrap();

    let n = n.as_ref();
    match n.node_type() {
        Node::ELEMENT_NODE => n.clone().dyn_into::<Element>().unwrap().clone(),
        Node::TEXT_NODE => {
            let n: &Text = n.dyn_ref::<Text>().unwrap();

            let new = document.create_element(&s).unwrap();
            new.set_text_content(Some(&n.text_content().unwrap()));
            insert_after(&n, &new);

            n.parent_node().unwrap().remove_child(&n).unwrap();

            new
        }
        _ => todo!(),
    }
    /*if n.node_type() != Node::TEXT_NODE {
        panic!("trying to upgrade {} node", n.node_type());
    }

    let n: &Text = n.dyn_ref::<Text>().unwrap();

    let new = document.create_element(&s).unwrap();
    new.set_text_content(Some(&n.text_content().unwrap()));
    insert_after(&n, &new);

    n.parent_node().unwrap().remove_child(&n).unwrap();

    new*/
}

#[wasm_bindgen_test]
fn test_upgrade() {
    let document = web_sys::window().unwrap().document().unwrap();

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello, world!");
    parent.append_child(&child0).unwrap();
    let el = upgrade(&child0, "SPAN");
    assert_eq!(parent.inner_html(), "<span>Hello, world!</span>");

    el.dyn_ref::<Element>().unwrap().class_list();
}

fn insert_span_split_at<N>(n: N, m: Mask) -> Node
where
    N: AsRef<web_sys::Node>,
{
    let n = n.as_ref().child_nodes();
    let mut o = 0;

    for i in 0..n.length() {
        let n = n.get(i).unwrap();
        match m {
            Mask::Byte(m) => {
                if o < m && m < o + n.text_content().unwrap().len() {
                    // split n
                    let v = split_at(n, m);
                    assert_eq!(v.len(), 2, "tried to split on edge");
                    let tail = upgrade(&v[1], "span");
                    return <Element as AsRef<Node>>::as_ref(&tail).clone();
                }
            }
            Mask::Bit(..) => todo!("split bit mask"),
        }
    }

    unreachable!()
}

#[wasm_bindgen_test]
fn test_insert_span_split_at() {
    fn to_vec(n: web_sys::NodeList) -> Vec<web_sys::Node> {
        (0..n.length()).map(|i| n.get(i).unwrap()).collect()
    }

    let document = web_sys::window().unwrap().document().unwrap();

    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Hello, world!"));
    insert_span_split_at(&div, Mask::Byte(8));
    assert_eq!(
        div.child_nodes().get(0).unwrap().node_type(),
        web_sys::Node::TEXT_NODE,
    );
    assert_eq!(
        div.child_nodes().get(0).unwrap().text_content().unwrap(),
        "Hello, w",
    );
    assert_eq!(
        div.child_nodes().get(1).unwrap().node_type(),
        web_sys::Node::ELEMENT_NODE,
    );
    assert_eq!(
        div.child_nodes().get(1).unwrap().text_content().unwrap(),
        "orld!",
    );

    let div = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello");
    div.append_child(&child0).unwrap();
    let child1 = document.create_text_node(", world!");
    div.append_child(&child1).unwrap();
    let v = to_vec(div.child_nodes());
    assert_eq!(v[0].node_type(), web_sys::Node::TEXT_NODE);
    assert_eq!(v[0].text_content().unwrap(), "Hello");
    assert_eq!(v[1].node_type(), web_sys::Node::TEXT_NODE);
    assert_eq!(v[1].text_content().unwrap(), ", world!");

    assert_eq!(div.inner_html(), "Hello, world!");
    insert_span_split_at(&div, Mask::Byte(2));
    assert_eq!(div.inner_html(), "He<span>llo</span>, world!");
    let v = to_vec(div.child_nodes());
    assert_eq!(v[0].node_type(), web_sys::Node::TEXT_NODE);
    assert_eq!(v[0].text_content().unwrap(), "He");
    assert_eq!(v[1].node_type(), web_sys::Node::ELEMENT_NODE);
    assert_eq!(v[1].text_content().unwrap(), "llo");
    assert_eq!(v[2].node_type(), web_sys::Node::TEXT_NODE);
    assert_eq!(v[2].text_content().unwrap(), ", world!");
}

fn find_edge<N>(n: N, s: Mask) -> Option<(usize, usize, Node)>
where
    N: AsRef<web_sys::Node>,
{
    let n = n.as_ref();
    let children = to_vec(n.child_nodes());
    children.iter().enumerate().find_map({
        let mut o = 0;
        move |(i, n)| {
            let l = n.text_content().unwrap().len();
            if o < s.floor() && s.floor() < o + l {
                Some((i, o, n.clone()))
            } else {
                o += l;
                None
            }
        }
    })
}

#[wasm_bindgen_test]
fn test_find_edge() {
    let document = web_sys::window().unwrap().document().unwrap();

    let div = document.create_element("DIV").unwrap();
    let text = document.create_text_node("Hello, world!");
    div.append_child(&text).unwrap();
    assert_eq!(find_edge(&div, Mask::Byte(0)).as_ref(), None);
    assert_eq!(
        find_edge(&div, Mask::Byte(2)),
        Some((0, 0, <Text as AsRef<Node>>::as_ref(&text).clone()))
    );
    assert_eq!(find_edge(&div, Mask::Byte(13)).as_ref(), None);

    let div = document.create_element("DIV").unwrap();
    let text = document.create_text_node("Hello, w");
    div.append_child(&text).unwrap();
    let span = document.create_element("SPAN").unwrap();
    span.set_text_content(Some("orld!"));
    div.append_child(&span).unwrap();
    assert_eq!(find_edge(&div, Mask::Byte(0)), None);
    assert_eq!(
        find_edge(&div, Mask::Byte(5)),
        Some((0, 0, <Text as AsRef<Node>>::as_ref(&text).clone()))
    );
    assert_eq!(find_edge(&div, Mask::Byte(8)), None);
    assert_eq!(
        find_edge(&div, Mask::Byte(10)),
        Some((1, 8, <Element as AsRef<Node>>::as_ref(&span).clone())),
    );
    assert_eq!(find_edge(&div, Mask::Byte(13)), None);
}

fn insert_span<N>(n: N, s: (String, Mask, Mask))
where
    N: AsRef<web_sys::Node>,
{
    web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
        "insert_span({:?}, {:?})",
        content(&n),
        s,
    )));
    let n = n.as_ref();
    let children = to_vec(n.child_nodes());
    let mut inner = vec![];
    if let Some((i, o, m)) = find_edge(n, s.1) {
        // split starting edge
        split_at(m, s.1.floor());
        inner.push(i + 1); // begin span on next elem
    }

    println!(
        "{:?}",
        (
            n.dyn_ref::<Element>().unwrap().outer_html(),
            to_vec(n.child_nodes())
        )
    );
    /*panic!(
        "{:?}",
        (
            n.dyn_ref::<Element>().unwrap().outer_html(),
            to_vec(n.child_nodes())
        )
    );*/
    if let Some((i, o, m)) = find_edge(n, s.2) {
        // split starting edge
        /*panic!(
            "{:?}",
            (
                m.parent_node()
                    .unwrap()
                    .dyn_ref::<Element>()
                    .map(Element::outer_html)
                    .or_else(|| m.text_content()),
                m.node_type(),
                m.dyn_ref::<Element>()
                    .map(Element::outer_html)
                    .or_else(|| m.text_content()),
                m,
                s.2.floor()
            )
        );*/
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
            "insert_span({:?}, {:?})",
            content(n),
            s,
        )));
        split_at(m, s.2.floor() - o);
        inner.push(i);
    }

    let mut children = to_vec(n.child_nodes());
    web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
        "upgrading {:?}",
        inner
    )));
    for i in inner {
        let el = upgrade(&children[i], "SPAN");
        el.class_list().add_1(&s.0);
        children = to_vec(n.child_nodes());
    }

    /*let n = n.as_ref();
    let mut o = 0;
    let mut e = n.child_nodes().length();
    let mut i = 0;

    while i < e {
        let c = n.child_nodes().get(i).unwrap();
        if o < s.1.floor() && s.1.floor() < o + c.text_content().unwrap().len() {
            // span begins inside n => split n
            if c.node_type() == web_sys::Node::TEXT_NODE {
                // split text node
                insert_span_split_at(n, s.1);
                e = n.child_nodes().length();
            } else if c.node_type() == web_sys::Node::ELEMENT_NODE && c.node_name() == "SPAN" {
                insert_span_split_at(c, s.2 - o);
                e = n.child_nodes().length();
            } else {
                // not ending in this elem
            }
        }
        i += 1;
    }*/
}

#[wasm_bindgen_test]
fn test_insert_span() {
    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Lorem ipsum!"));
    insert_span(&div, ("test".into(), Mask::Byte(3), Mask::Byte(4)));
    assert_eq!(
        div.inner_html(),
        r#"Lor<span class="test">e</span>m ipsum!"#,
    );

    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Lorem ipsum!"));
    insert_span(&div, ("test2".into(), Mask::Byte(8), Mask::Byte(11)));
    assert_eq!(
        div.inner_html(),
        r#"Lorem ip<span class="test2">sum</span>!"#,
    );

    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Lorem ipsum!"));
    insert_span(&div, ("test3".into(), Mask::Byte(1), Mask::Byte(11)));
    assert_eq!(
        div.inner_html(),
        r#"L<span class="test3">orem ipsum</span>!"#,
    );
    insert_span(&div, ("test4".into(), Mask::Byte(2), Mask::Byte(10)));
    assert_eq!(
        div.inner_html(),
        r#"L<span class="test3">o</span><span class="test3 test4">rem ipsu</span><span class="test3">m</span>!"#,
    );
}

/*fn tree_head_count(div: &web_sys::HtmlDivElement) -> usize {
    let n: &web_sys::Node = &div;
    let l = n.child_nodes();

    for i in 0..l.length() {
        let e = l.get(i).unwrap();
        match e.node_type() {
            web_sys::Node::TEXT_NODE => {} // FIXME(s1g): are sequential text nodes allowed?
            web_sys::Node::ELEMENT_NODE if e.node_name() == "SPAN" => {} // FIXME(s1g): should
            // check if sequential
            // spans are same class
            web_sys::Node::ELEMENT_NODE if e.node_name() == "DIV" => break i,
            web_sys::Node::ELEMENT_NODE => panic!("illegal element node name: {:?}", e.node_name()),
            t => panic!("illegal element node type: {t:?}"),
        }
    }

    l.length()
}*/

/*fn validate_tree_depth(div: &web_sys::HtmlDivElement) {
    let n: &web_sys::Node = &div;
    let l = n.child_nodes();

    let head_len = tree_head_count(div);

    for
        if n0.node_type() != web_sys::Node::TEXT_NODE {
            panic!("first element of editor should always be text node");
        }

        for i in 1..n.child_nodes().length() {
            if n.child_nodes().item(i).unwrap().node_type() != web_sys::Node::ELEMENT_NODE
                || n.child_nodes().item(i).unwrap().node_name() != "DIV"
            {
                panic!("tail elements of editor should always be div element");
            }
        }
    }
}*/

/*#[wasm_bindgen_test]
fn test_validate_tree_depth() {
    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("DIV").unwrap();
    validate_tree_lines(div.dyn_ref().unwrap());

    div.set_inner_html("Lorem ipsum dolor sit amet,<div>consectetur adipiscing elit,</div><div>sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</div>");
    validate_tree_lines(div.dyn_ref().unwrap());
}*/

pub fn validate_tree_lines(div: &web_sys::HtmlDivElement) {
    let n: &web_sys::Node = &div;

    if let Some(n0) = n.child_nodes().get(0) {
        if n0.node_type() != web_sys::Node::TEXT_NODE {
            panic!("first element of editor should always be text node");
        }

        for i in 1..n.child_nodes().length() {
            if n.child_nodes().item(i).unwrap().node_type() != web_sys::Node::ELEMENT_NODE
                || n.child_nodes().item(i).unwrap().node_name() != "DIV"
            {
                panic!("tail elements of editor should always be div element");
            }
        }
    }
}

#[wasm_bindgen_test]
fn test_validate_tree_lines() {
    use wasm_bindgen::JsCast;

    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("DIV").unwrap();
    validate_tree_lines(div.dyn_ref().unwrap());

    div.set_inner_html("Lorem ipsum dolor sit amet,<div>consectetur adipiscing elit,</div><div>sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</div>");
    validate_tree_lines(div.dyn_ref().unwrap());
}

/*#[wasm_bindgen_test]
#[should_panic]
fn test_validate_tree_lines_first_div() {
    use wasm_bindgen::JsCast;

    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("DIV").unwrap();
    div.set_inner_html("<div>Lorem ipsum dolor sit amet,</div><div>consectetur adipiscing elit,</div><div>sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</div>");
    validate_tree_lines(div.dyn_ref().unwrap());
}*/

pub fn split<N>(n: N, o: usize) -> Vec<(web_sys::Node, usize, usize)>
where
    N: AsRef<web_sys::Node>,
{
    let n = n.as_ref().child_nodes();
    let mut o = 0;
    (0..n.length())
        .map(|i| {
            let n = n.item(i).unwrap();
            let mut s = o;
            o += match n.node_type() {
                web_sys::Node::ELEMENT_NODE if n.node_name() == "DIV" => {
                    s += 1;
                    n.text_content().unwrap().len() + 1
                }
                _ => n.text_content().unwrap().len(),
            };
            (n, s, o)
        })
        .collect()
}

#[wasm_bindgen_test]
fn test_children() {
    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("div").unwrap();
    let span: &web_sys::Node = &document.create_element("span").unwrap();
    span.set_text_content(Some("Hel"));
    let b: &web_sys::Node = &document.create_element("b").unwrap();
    b.set_text_content(Some("lo,"));
    div.append_child(&span).unwrap();
    div.append_child(&b).unwrap();
    assert_eq!(
        children(div.clone()),
        vec![(span.clone(), 0, 3), (b.clone(), 3, 6)],
    );
    let a: &web_sys::Node = &document.create_element("a").unwrap();
    a.set_text_content(Some(" world!"));
    div.append_child(&a).unwrap();
    assert_eq!(
        children(div),
        vec![(span.clone(), 0, 3), (b.clone(), 3, 6), (a.clone(), 6, 13)],
    );

    let div = document.create_element("div").unwrap();
    let line1: &web_sys::Node = &document.create_text_node("Hello,");
    let line2: &web_sys::Node = &document.create_element("div").unwrap();
    line2.set_text_content(Some("Best Regards,"));
    div.append_child(&line1).unwrap();
    div.append_child(&line2).unwrap();
    assert_eq!(
        children(div.clone()),
        vec![(line1.clone(), 0, 6), (line2.clone(), 7, 20)],
    );
    let line3: &web_sys::Node = &document.create_element("div").unwrap();
    line3.set_text_content(Some("Wallace"));
    div.append_child(&line3).unwrap();
    assert_eq!(
        children(div.clone()),
        vec![
            (line1.clone(), 0, 6),
            (line2.clone(), 7, 20),
            (line3.clone(), 21, 28),
        ],
    );
}

/*#[wasm_bindgen_test]
fn test_children_split() {
    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("div").unwrap();
    div.set_inner_html("<p>Lorem ipsum</p><p>dolor and so on</p>");
    let c = children(div.clone());
    //c[1].split(3);
    assert_eq!(div.inner_html(), "assdasd");
}*/
