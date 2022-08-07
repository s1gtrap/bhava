use wasm_bindgen::JsCast;

use wasm_bindgen_test::*;

use web_sys::{Element, Node, Text};

use super::Mask;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn to_vec(n: web_sys::NodeList) -> Vec<web_sys::Node> {
    (0..n.length()).map(|i| n.get(i).unwrap()).collect()
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

    //let child2p5 = document.create_text_node("dummy");
    //insert_after(&child1, &child2);
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
            tail.class_list()
                .set_value(&n.dyn_ref::<Element>().unwrap().class_list().value());

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

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_element("SPAN").unwrap();
    child0.class_list().add_1("grey").unwrap();
    child0.set_text_content(Some("Hello, world!"));
    parent.append_child(&child0).unwrap();
    split_at(&child0, 8);
    assert_eq!(
        parent.inner_html(),
        r#"<span class="grey">Hello, w</span><span class="grey">orld!</span>"#,
    );
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
}

#[wasm_bindgen_test]
fn test_upgrade() {
    let document = web_sys::window().unwrap().document().unwrap();

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_text_node("Hello, world!");
    parent.append_child(&child0).unwrap();
    let el = upgrade(&child0, "SPAN");
    assert_eq!(parent.inner_html(), "<span>Hello, world!</span>");
}

fn downgrade<N>(n: N) -> Text
where
    N: AsRef<web_sys::Node>,
{
    web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
        "downgrade({:?})",
        content(&n),
    )));
    let document = web_sys::window().unwrap().document().unwrap();

    let n = n.as_ref();
    match n.node_type() {
        Node::TEXT_NODE => n.clone().dyn_into::<Text>().unwrap().clone(),
        Node::ELEMENT_NODE => {
            let n: &Element = n.dyn_ref::<Element>().unwrap();

            let new = document.create_text_node(&n.text_content().unwrap());
            new.set_text_content(Some(&n.text_content().unwrap()));
            insert_after(&n, &new);

            n.parent_node().unwrap().remove_child(&n).unwrap();

            new
        }
        _ => todo!(),
    }
}

#[wasm_bindgen_test]
fn test_downgrade() {
    let document = web_sys::window().unwrap().document().unwrap();

    let parent = document.create_element("DIV").unwrap();
    let child0 = document.create_element("SPAN").unwrap();
    child0.set_text_content(Some("Hello, world!"));
    parent.append_child(&child0).unwrap();
    let el = downgrade(&child0);
    assert_eq!(parent.inner_html(), "Hello, world!");
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
            if o <= s.floor() && s.floor() < o + l {
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
    assert_eq!(
        find_edge(&div, Mask::Byte(0)),
        Some((0, 0, <Text as AsRef<Node>>::as_ref(&text).clone())),
    );
    assert_eq!(
        find_edge(&div, Mask::Byte(2)),
        Some((0, 0, <Text as AsRef<Node>>::as_ref(&text).clone())),
    );
    assert_eq!(find_edge(&div, Mask::Byte(13)).as_ref(), None);

    let div = document.create_element("DIV").unwrap();
    let text = document.create_text_node("Hello, w");
    div.append_child(&text).unwrap();
    let span = document.create_element("SPAN").unwrap();
    span.set_text_content(Some("orld!"));
    div.append_child(&span).unwrap();
    assert_eq!(
        find_edge(&div, Mask::Byte(0)),
        Some((0, 0, <Text as AsRef<Node>>::as_ref(&text).clone())),
    );
    assert_eq!(
        find_edge(&div, Mask::Byte(5)),
        Some((0, 0, <Text as AsRef<Node>>::as_ref(&text).clone())),
    );
    assert_eq!(
        find_edge(&div, Mask::Byte(8)),
        Some((1, 8, <Element as AsRef<Node>>::as_ref(&span).clone())),
    );
    assert_eq!(
        find_edge(&div, Mask::Byte(10)),
        Some((1, 8, <Element as AsRef<Node>>::as_ref(&span).clone())),
    );
    assert_eq!(find_edge(&div, Mask::Byte(13)), None);
}

pub fn insert_span<N>(n: N, s: (String, Mask, Mask))
where
    N: AsRef<web_sys::Node>,
{
    web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
        "insert_span({:?}, {:?})",
        content(&n),
        s,
    )));
    let n = n.as_ref();
    //let children = to_vec(n.child_nodes());
    let mut inner = vec![];
    if let Some((i, o, m)) = find_edge(n, s.1) {
        if Mask::Byte(i) != s.1 {
            // split starting edge
            split_at(m, s.1.floor() - o);
            inner.push(i + 1); // begin span on next elem
        } else {
            inner.push(i);
        }
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
    for i in inner[0]..=inner[1] {
        let el = upgrade(&children[i], "SPAN");
        el.class_list().add_1(&s.0).unwrap();
        children = to_vec(n.child_nodes());
    }
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

    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Lorem ipsum!"));
    insert_span(&div, ("test2".into(), Mask::Byte(8), Mask::Byte(11)));
    assert_eq!(
        div.inner_html(),
        r#"Lorem ip<span class="test2">sum</span>!"#,
    );

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
    insert_span(&div, ("test5".into(), Mask::Byte(4), Mask::Byte(7)));
    assert_eq!(
        div.inner_html(),
        r#"L<span class="test3">o</span><span class="test3 test4">re</span><span class="test3 test4 test5">m i</span><span class="test3 test4">psu</span><span class="test3">m</span>!"#,
    );

    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Lorem ipsum!"));
    insert_span(&div, ("test3".into(), Mask::Byte(6), Mask::Byte(7)));
    assert_eq!(
        div.inner_html(),
        r#"Lorem <span class="test3">i</span>psum!"#,
    );
    insert_span(&div, ("test2".into(), Mask::Byte(4), Mask::Byte(9)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="test2">m </span><span class="test3 test2">i</span><span class="test2">ps</span>um!"#,
    );
    insert_span(&div, ("test1".into(), Mask::Byte(1), Mask::Byte(11)));
    assert_eq!(
        div.inner_html(),
        r#"L<span class="test1">ore</span><span class="test2 test1">m </span><span class="test3 test2 test1">i</span><span class="test2 test1">ps</span><span class="test1">um</span>!"#,
    );

    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Lorem ipsum dolor sit amet"));
    insert_span(&div, ("a".into(), Mask::Byte(4), Mask::Byte(7)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m i</span>psum dolor sit amet"#,
    );
    insert_span(&div, ("b".into(), Mask::Byte(11), Mask::Byte(15)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m i</span>psum<span class="b"> dol</span>or sit amet"#,
    );
    insert_span(&div, ("c".into(), Mask::Byte(18), Mask::Byte(23)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m i</span>psum<span class="b"> dol</span>or <span class="c">sit a</span>met"#,
    );

    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Lorem ipsum dolor sit amet"));
    insert_span(&div, ("a".into(), Mask::Byte(4), Mask::Byte(11)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m ipsum</span> dolor sit amet"#,
    );
    insert_span(&div, ("b".into(), Mask::Byte(7), Mask::Byte(18)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m i</span><span class="a b">psum</span><span class="b"> dolor </span>sit amet"#,
    );
    insert_span(&div, ("c".into(), Mask::Byte(15), Mask::Byte(23)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m i</span><span class="a b">psum</span><span class="b"> dol</span><span class="b c">or </span><span class="c">sit a</span>met"#,
    );
}

pub fn remove_span<N>(n: N, s: (String, Mask, Mask))
where
    N: AsRef<web_sys::Node>,
{
    web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
        "remove_span({:?}, {:?})",
        content(&n),
        s,
    )));
    let n = n.as_ref();
    //let children = to_vec(n.child_nodes());
    let mut inner = vec![];
    if let Some((i, o, m)) = find_edge(n, s.1) {
        if Mask::Byte(o) != s.1 {
            // split starting edge
            split_at(m, s.1.floor() - o);
            inner.push(i + 1); // begin span on next elem
        } else {
            inner.push(i);
        }
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
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(format!(
            "remove_span({:?}, {:?})",
            content(n),
            s,
        )));
        split_at(m, s.2.floor() - o);
        inner.push(i - 1);
    }

    let mut children = to_vec(n.child_nodes());
    for i in inner[0]..=inner[1] {
        let el = children[i].dyn_ref::<Element>().unwrap();
        el.class_list().remove_1(&s.0).unwrap();
        let el = downgrade(&children[i]);
        children = to_vec(n.child_nodes());
    }
}

#[wasm_bindgen_test]
fn test_remove_span() {
    let document = web_sys::window().unwrap().document().unwrap();

    let div = document.create_element("DIV").unwrap();
    div.set_inner_html(r#"Lor<span class="test">e</span>m ipsum!"#);
    remove_span(&div, ("test".into(), Mask::Byte(3), Mask::Byte(4)));
    assert_eq!(div.inner_html(), "Lorem ipsum!");

    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Lorem ipsum!"));
    insert_span(&div, ("test2".into(), Mask::Byte(8), Mask::Byte(11)));
    assert_eq!(
        div.inner_html(),
        r#"Lorem ip<span class="test2">sum</span>!"#,
    );

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
    insert_span(&div, ("test5".into(), Mask::Byte(4), Mask::Byte(7)));
    assert_eq!(
        div.inner_html(),
        r#"L<span class="test3">o</span><span class="test3 test4">re</span><span class="test3 test4 test5">m i</span><span class="test3 test4">psu</span><span class="test3">m</span>!"#,
    );

    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Lorem ipsum!"));
    insert_span(&div, ("test3".into(), Mask::Byte(6), Mask::Byte(7)));
    assert_eq!(
        div.inner_html(),
        r#"Lorem <span class="test3">i</span>psum!"#,
    );
    insert_span(&div, ("test2".into(), Mask::Byte(4), Mask::Byte(9)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="test2">m </span><span class="test3 test2">i</span><span class="test2">ps</span>um!"#,
    );
    insert_span(&div, ("test1".into(), Mask::Byte(1), Mask::Byte(11)));
    assert_eq!(
        div.inner_html(),
        r#"L<span class="test1">ore</span><span class="test2 test1">m </span><span class="test3 test2 test1">i</span><span class="test2 test1">ps</span><span class="test1">um</span>!"#,
    );

    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Lorem ipsum dolor sit amet"));
    insert_span(&div, ("a".into(), Mask::Byte(4), Mask::Byte(7)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m i</span>psum dolor sit amet"#,
    );
    insert_span(&div, ("b".into(), Mask::Byte(11), Mask::Byte(15)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m i</span>psum<span class="b"> dol</span>or sit amet"#,
    );
    insert_span(&div, ("c".into(), Mask::Byte(18), Mask::Byte(23)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m i</span>psum<span class="b"> dol</span>or <span class="c">sit a</span>met"#,
    );

    let div = document.create_element("DIV").unwrap();
    div.set_text_content(Some("Lorem ipsum dolor sit amet"));
    insert_span(&div, ("a".into(), Mask::Byte(4), Mask::Byte(11)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m ipsum</span> dolor sit amet"#,
    );
    insert_span(&div, ("b".into(), Mask::Byte(7), Mask::Byte(18)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m i</span><span class="a b">psum</span><span class="b"> dolor </span>sit amet"#,
    );
    insert_span(&div, ("c".into(), Mask::Byte(15), Mask::Byte(23)));
    assert_eq!(
        div.inner_html(),
        r#"Lore<span class="a">m i</span><span class="a b">psum</span><span class="b"> dol</span><span class="b c">or </span><span class="c">sit a</span>met"#,
    );
}
