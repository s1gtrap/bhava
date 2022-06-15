# bhava

A text editor written in Rust for the web.

* Changing `props.content` should only rerender if the contents should be displayed differently. TODO: what to do with cursor?
* Changing `props.spans` should never cause a rerender. Modify dom nodes around instead.