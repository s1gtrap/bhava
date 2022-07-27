use wasm_bindgen_test::*;

use super::Mask;

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

fn insert_after<N, M>(n: N, m: M)
where
    N: AsRef<web_sys::Node>,
    M: AsRef<web_sys::Node>,
{
    n.as_ref()
        .parent_node()
        .unwrap()
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
}

fn insert_span_split_at<N>(n: N, m: Mask)
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
                    match n.node_type() {
                        web_sys::Node::TEXT_NODE => {
                            let document = web_sys::window().unwrap().document().unwrap();
                            let tail = document.create_text_node(&n.text_content().unwrap()[m..]);
                            insert_after(&n, tail);
                            n.set_text_content(Some(&n.text_content().unwrap()[..m]));
                        }
                        t => todo!("split node type {t:?}"),
                    }
                }
            }
            Mask::Bit(..) => todo!("split bit mask"),
        }
    }
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
        web_sys::Node::TEXT_NODE,
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

    insert_span_split_at(&div, Mask::Byte(2));
    let v = to_vec(div.child_nodes());
    assert_eq!(v[0].node_type(), web_sys::Node::TEXT_NODE);
    assert_eq!(v[0].text_content().unwrap(), "He");
    assert_eq!(v[1].node_type(), web_sys::Node::TEXT_NODE);
    assert_eq!(v[1].text_content().unwrap(), "llo");
    assert_eq!(v[2].node_type(), web_sys::Node::TEXT_NODE);
    assert_eq!(v[2].text_content().unwrap(), ", world!");
}

fn insert_span_split<N>(n: N, s: (Mask, Mask))
where
    N: AsRef<web_sys::Node>,
{
    let document = web_sys::window().unwrap().document().unwrap();

    let n = n.as_ref().child_nodes();
    let mut o = 0;

    for i in 0..n.length() {
        let n = n.get(i).unwrap();
        if o < s.1.floor() && s.1.floor() < o + n.text_content().unwrap().len() {
            // span begins inside n => split n
            if n.node_type() == web_sys::Node::TEXT_NODE {
                // split text node
            } else if n.node_type() == web_sys::Node::ELEMENT_NODE && n.node_name() == "SPAN" {
            }
        }
    }
}

#[wasm_bindgen_test]
fn test_insert_span_split() {
    /*let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Hello, world!"));

    insert_span_split(&div, (Mask::Byte(3), Mask::Byte(10)));

    assert_eq!(
        div.child_nodes().get(0).unwrap().text_content().unwrap(),
        "Hel",
    );
    assert_eq!(
        div.child_nodes().get(1).unwrap().text_content().unwrap(),
        "lo, wor",
    );
    assert_eq!(
        div.child_nodes().get(2).unwrap().text_content().unwrap(),
        "ld!",
    );*/
}

fn insert_span<N>(n: N, s: (String, Mask, Mask))
where
    N: AsRef<web_sys::Node>,
{
    let n = n.as_ref().child_nodes();
    let mut o = 0;

    for i in 0..n.length() {
        let n = n.get(i).unwrap();
        if o < s.1.floor() && s.1.floor() < o + n.text_content().unwrap().len() {
            // span begins inside n => split n
            if n.node_type() == web_sys::Node::TEXT_NODE {
                // split text node
            } else if n.node_type() == web_sys::Node::ELEMENT_NODE && n.node_name() == "SPAN" {
            }
        }
    }
}

#[wasm_bindgen_test]
fn test_insert_span() {
    /*let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Hello, world!"));

    insert_span(&div, ("test".into(), Mask::Byte(2), Mask::Byte(4)));

    assert_eq!(
        div.inner_html(),
        r#"He<span class="test">ll</span>o, world!"#,
    );*/
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
