
use std::str;
use std::fmt::Debug;
use std::process::{Command, Stdio};
use std::collections::VecDeque;
use std::io::BufReader;
use std::io::prelude::*;

use serde_json as json;

use dom;
use tree;
use tree::Vertex;

pub type Update<M, S> = fn(M, S) -> M;
pub type View<M, S> = fn(&M) -> dom::Object<S>;

// TODO: investigate using Godel numbering of lists to encode 'path' of widget as usize ID
// - might be easier than allocating vectors for each child

type Identifier = String;

#[derive(Serialize, Deserialize, Debug)]
enum Event {
    // TODO: ID, Attributes (e.g. Text), Location (i.e. 'frame')
    // TODO: `text` should really be generic list of 'attributes'
    Create {
        id: Identifier,
        kind: String,
        text: String,
    },

    Update(Identifier, String, String), // ID -> Attribute

    Remove(Identifier), // ID
}

/// Convert 'changeset' to list of events to send to UI 'rendering' process
fn convert<T: PartialEq + Clone>(dom: &dom::Object<T>, set: dom::Changeset) -> Vec<Event> {
    let mut events = vec![];

    fn expand<S>(path: tree::Path, node: &dom::Object<S>, events: &mut Vec<Event>) {
        // TODO: use breadth-first traversal here (using queue) - use path!

        let id = path.to_string();

        match node.widget {
            dom::Widget::Label(ref label) => {
                events.push(Event::Create {
                    id,
                    kind: "Label".into(),
                    text: label.text.clone(),
                })
            }

            dom::Widget::Button(ref button) => {
                events.push(Event::Create {
                    id,
                    kind: "Button".into(),
                    text: button.text.clone(),
                })
            }

            dom::Widget::Field(_) => {
                events.push(Event::Create {
                    id,
                    kind: "Field".into(),
                    text: "".into(),
                })
            }

            _ => {}
        }

        for (n, child) in node.children.iter().enumerate() {
            let mut path = path.clone();
            path.push(n);

            expand(path, child, events);
        }
    }

    for (path, op) in set.into_iter() {
        let node = dom.find(&path).expect("path in nodes");

        match op {
            tree::Operation::Create => expand(path, node, &mut events),
            tree::Operation::Update => {
                let id = path.to_string();
                match node.widget {
                    dom::Widget::Label(ref label) => {
                        events.push(Event::Update(id, "Text".into(), label.text.clone()))
                    }

                    _ => unimplemented!(),
                }
            }

            _ => unimplemented!(),
        }
    }

    events
}

pub fn program<S, M>(mut model: M, update: Update<M, S>, view: View<M, S>)
where
    S: Clone + Send + 'static + PartialEq + Debug,
    M: Send + 'static + Debug,
{
    let mut dom = view(&model);

    // let tree = tree::Tree { children: vec![dom] };

    // println!("model: {:?}", model);
    // println!("view: {:?}", dom);

    // TODO: use `spawn` and listen to stdin/stdout
    // - implement 'quit' event (or just exit when process terminates)

    // TODO: remove hard-coded path to UI subprocess exe
    // - `fork` is another option - only *nix compatible, though.

    let output = Command::new("./cocoa/target/release/cocoa")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    // Create changeset: Create @ 'root'
    let patch = vec![(tree::Path::new(), tree::Operation::Create)];

    let events = convert(&dom, patch);

    let mut stdin = output.stdin.unwrap();
    for event in events.into_iter() {
        writeln!(stdin, "{}", json::to_string(&event).unwrap()).unwrap();
    }

    /// Receive messages from 'renderer' process (via stdout)

    let stdout = BufReader::new(output.stdout.unwrap());
    for line in stdout.lines() {
        // TODO: define/implement this API using JSON

        let line = line.unwrap();
        let mut split = line.split(".");
        let command = split.next().unwrap();
        let path = tree::Path::from_vec(split.map(|s| s.parse().unwrap()).collect());

        let message = match command {
            "click" => {
                // TODO: move 'find' logic into tree/dom module

                dom.find(&path).and_then(|node| match node.widget {
                    dom::Widget::Button(ref button) => button.click.clone(),
                    _ => None,
                })
            }

            _ => None,
        };

        let message = match message {
            Some(m) => m,
            _ => continue,
        };

        // TODO: some events from renderer (e.g. window resize) will not generate 'message' to `update`
        //   but will (potentially) require re-layout
        // - no `update` means call to `view` i.e. no new `dom`

        model = update(model, message);

        let old = dom;
        dom = view(&model);

        let changeset = dom::diff(&old, &dom);

        // TODO: generate layout for `dom`
        // TODO: pass `layout` to `convert` to be associated with events (to renderer)

        // {
        //     let mut root = yoga::Node::new();

        //     let mut queue = VecDeque::new();
        //     queue.push_back((0, &dom));

        //     while let Some((level, node)) = queue.pop_front() {

        //         for child in node.children.iter() {
        //             queue.push_back((level + 1, child));
        //         }

        //         println!("node[{}]: {:?}", level, node);
        //     }
        // }

        let root = layout(&dom);
        root.calculuate();

        let events = convert(&dom, changeset);

        for event in events.into_iter() {
            writeln!(stdin, "{}", json::to_string(&event).unwrap()).unwrap();
        }
    }
}

fn layout<V: Vertex>(tree: &V) -> yoga::Node {
    let mut root = yoga::Node::new();

    // let children: Vec<yoga::Node> = tree.children().iter().map(layout).collect();

    for node in tree.children().iter().map(layout) {
        root.insert(node, 0);
    }

    root
}

mod yoga {
    use layout::yoga::*;

    #[derive(Debug)]
    pub struct Node {
        node: YGNodeRef,
        children: Vec<Node>,
    }

    impl Node {
        pub fn new() -> Self {
            Node {
                node: unsafe { YGNodeNew() },
                children: vec![],
            }
        }

        pub fn insert(&mut self, child: Node, index: u32) {
            unsafe {
                YGNodeStyleSetFlexGrow(child.node, 1.);
                YGNodeInsertChild(self.node, child.node, index);
            }
            self.children.push(child);
        }

        pub fn calculuate(&self) {
            // YGNodeCalculateLayout

            unsafe {
                let node = self.node;
                YGNodeCalculateLayout(node, 500., 400., YGDirection::YGDirectionInherit);

                for child in &self.children {
                    let node = child.node;
                    println!("{}", YGNodeLayoutGetLeft(node));
                    println!("{}", YGNodeLayoutGetTop(node));
                    println!("{}", YGNodeLayoutGetRight(node));
                    println!("{}", YGNodeLayoutGetBottom(node));
                    println!("{}", YGNodeLayoutGetWidth(node));
                    println!("{}", YGNodeLayoutGetHeight(node));

                    println!("");
                }
            }
        }
    }

    impl Drop for Node {
        fn drop(&mut self) {
            unsafe { YGNodeFree(self.node) }
        }
    }
}

// #[derive(Debug)]
// pub struct Layout<'l, S> {
//     pub layout: &'l yoga::Node,
//     pub widget: dom::Widget<S>,
//     pub children: Vec<Layout<S>>,
// }
