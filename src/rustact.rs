use crate::log;
use std::fmt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

#[derive(Default)]
pub struct Element {
    html_type: String,
    props: Props,
}

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:#?}, {:#?} this is a element",
            self.html_type, self.props
        )
    }
}

pub struct Props {
    pub children: Option<Vec<Element>>,
    pub text: Option<String>,
    pub on_click: Option<Box<dyn FnMut() -> ()>>,
    pub class_name: Option<String>,
    pub id: Option<String>,
}

impl fmt::Debug for Props {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?} this is props", self.children)
    }
}

impl Default for Props {
    fn default() -> Self {
        Props {
            children: None,
            text: None,
            on_click: None,
            class_name: None,
            id: None,
        }
    }
}

impl Element {
    pub fn new(html_type: String, props: Props) -> Element {
        Element { html_type, props }
    }
}

pub trait Render: RenderClone {
    fn render(&self) -> Element;
}
pub trait RenderClone {
    fn clone_box(&self) -> Box<dyn Render>;
}
impl<T> RenderClone for T
where
    T: 'static + Render + Clone,
{
    fn clone_box(&self) -> Box<dyn Render> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn Render> {
    fn clone(&self) -> Box<dyn Render> {
        self.clone_box()
    }
}

// pub trait SetState {
//     fn set_state(&self, new_state: &Self) {
//         self = new_state;
//     }
// }

pub fn render(rustact_element: Element, container: &web_sys::Node) {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    if rustact_element.html_type == "TEXT_ELEMENT" {
        match rustact_element.props.text {
            Some(text) => {
                let dom = container
                    .append_child(&document.create_text_node(&text))
                    .expect("couldn't append text node");
                // TODO: Not sure this is needed
                // match rustact_element.props.children {
                //     Some(children) => {
                //         for child in children {
                //             render(child, &dom)
                //         }
                //     }
                //     None => (),
                // }
            }
            None => (),
        };
    } else {
        let dom_el = document.create_element(&rustact_element.html_type).unwrap();

        match rustact_element.props.class_name {
            Some(class_name) => {
                dom_el.set_class_name(&class_name);
            }
            None => (),
        }
        match rustact_element.props.id {
            Some(id) => {
                dom_el.set_id(&id);
            }
            None => (),
        }
        match rustact_element.props.on_click {
            Some(mut on_click) => {
                let closure = Closure::wrap(Box::new(move || on_click()) as Box<dyn FnMut()>);
                dom_el
                    .dyn_ref::<HtmlElement>()
                    .expect("should be an `HtmlElement`")
                    .set_onclick(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }
            None => (),
        }

        let dom = container
            .append_child(&dom_el)
            .expect("couldn't append child");

        match rustact_element.props.children {
            Some(children) => {
                for child in children {
                    render(child, &dom)
                }
            }
            None => (),
        }
    }
}

pub fn create_element(html_type: String, props: Props) -> Element {
    Element::new(html_type, props)
}

pub type Reducer<T> = Box<dyn Fn(&T, &str) -> T>;
#[derive(Debug, Default, Clone, Copy)]
pub struct RustactStore<T> {
    store: T,
}

impl<T> RustactStore<T> {
    pub fn new(store: T) -> Self {
        Self { store }
    }
    pub fn reduce(&mut self, reducer: Reducer<T>, action: &str) {
        let new_store = reducer(&self.store, action);
        self.store = new_store;
    }
    pub fn store(self) -> T {
        self.store
    }
}

pub fn re_render(app: Element, id: Option<String>) {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    if let Some(i) = id {
        let root = document
            .get_element_by_id(&i)
            .expect("should have a root div");

        let parent = root.parent_node().unwrap();

        root.remove();

        //TODO: Determine how to render child vs sibling
        render(app, &parent);
    } else {
        let root = document
            .get_element_by_id("root")
            .expect("should have a root div");
        let node = root.first_child().unwrap();

        root.remove_child(&node).expect("unable to remove child");

        let root_node = root
            .append_child(&document.create_element("div").unwrap())
            .expect("couldn't append child");

        render(app, &root_node);
    }
}

#[derive(Debug, Default, Clone)]
struct StackElement {
    val: String,
    arena_position: usize,
}

// Can I make a parser struct that's not coupled to arena tree?
// Input has to have a wrapper div.
pub fn parse_with_stack(html_string: String) -> ArenaTree {
    let mut tokens = html_string.chars().peekable();
    let mut element_type: String = String::new();
    let mut is_open_tag: bool = false;
    let mut stack: Vec<StackElement> = vec![];
    let mut arena_tree: ArenaTree = ArenaTree::default();

    //if stack has a length we are dealing with have one parent.
    while let Some(character) = tokens.next() {
        let string_character = character.to_string();

        if string_character == "<" && tokens.peek().unwrap().to_string() != "/" {
            is_open_tag = true;
            continue;
        }
        if string_character == "<" && tokens.peek().unwrap().to_string() == "/" {
            is_open_tag = false;
            stack.pop();

            continue;
        }

        if string_character == ">" {
            if element_type != "".to_string() {
                //Let's go back to dealing with Options instead of empty strings.

                let el = element_type.clone();
                if stack.len() >= 1 {
                    arena_tree.set_current_parent_idx(stack.last().unwrap().arena_position);
                } else {
                    arena_tree.set_current_parent_idx(0);
                }

                arena_tree.insert(Node {
                    element_type,
                    ..Default::default()
                });
                stack.push(StackElement {
                    val: el,
                    arena_position: arena_tree.arena.len() - 1,
                });

                element_type = String::new();
            }
            continue;
        }

        if is_open_tag == true {
            element_type.push_str(&string_character);
        }
    }

    arena_tree
}

pub fn html(html_string: String) -> Element {
    let arena_tree = parse_with_stack(html_string);
    let el = arena_tree.create_element_from_tree();
    el
}

#[derive(Debug, Default)]
pub struct ArenaTree {
    current_parent_idx: usize,
    arena: Vec<Node>,
}

impl ArenaTree {
    fn new(current_parent_idx: usize, arena: Vec<Node>) -> Self {
        Self {
            arena,
            current_parent_idx,
        }
    }
    fn set_current_parent_idx(&mut self, idx: usize) {
        self.current_parent_idx = idx;
    }

    fn insert(&mut self, mut node: Node) {
        node.parent = self.current_parent_idx;
        node.idx = self.arena.len();
        self.arena.push(node);
        let child_index = self.arena.len() - 1;
        let parent_node = &mut self.arena[self.current_parent_idx];
        if child_index > 0 {
            parent_node.add_child(child_index);
        }
    }
}

trait CreateElement {
    fn create_element_from_tree(&self) -> Element;
}

impl CreateElement for ArenaTree {
    fn create_element_from_tree(&self) -> Element {
        let arena = &self.arena;

        fn children(node: &Node, arena: &Vec<Node>) -> Option<Vec<Element>> {
            return Some(
                node.children
                    .iter()
                    .map(|child| return create(&arena[child.to_owned()], &arena))
                    .collect::<Vec<Element>>(),
            );
        };

        fn create(node: &Node, arena: &Vec<Node>) -> Element {
            let new_el = Element {
                html_type: node.element_type.clone(),
                props: Props {
                    children: children(node, arena),
                    ..Default::default()
                },
            };

            new_el
        }
        let node = &arena[0];
        let el = create(node, arena);

        el
    }
}

#[derive(Debug, Default)]
struct Node {
    idx: usize,
    element_type: String,
    parent: usize,
    children: Vec<usize>,
}

impl Node {
    fn new(idx: usize, element_type: String, parent: usize, children: Vec<usize>) -> Self {
        Self {
            idx,
            element_type,
            parent,
            children,
        }
    }

    fn add_child(&mut self, child_idx: usize) {
        self.children.push(child_idx);
    }
}

// pub fn use_reducer(initial_state: &'static State) -> (&State, Box<dyn FnMut(&str) -> ()>) {
//     let message_1 = format!("here is state initially {:?}", initial_state);
//     js::log(&message_1);
//     let mut state = initial_state;

//     let dispatch = Box::new(move |action: &str| {
//         state = &state.reduce(action);
//         let message_dispatch = format!("here is state in dispatch {:?}", state);

//         js::log(&message_dispatch);
//         if initial_state.order == false {
//             re_render();
//         }
//         ()
//     });
//     let message_2 = format!("here is state after dispatch {:?}", state);
//     js::log(&message_2);
//     (state, dispatch)
// }
// type UseState = (Rc<RefCell<i32>>, Box<dyn FnMut(i32, Element) -> ()>);
// pub fn rustact() -> Box<dyn FnMut(i32) -> UseState> {
//     let internal_state = Rc::new(RefCell::new(0));
//     let internal_state_copy = internal_state.clone();

//     let use_state = move |initial_state: i32| {
//         let val: i32;

//         if *internal_state_copy.borrow() > 0 {
//             val = *internal_state_copy.borrow();
//             log(&format!("{:?} setting val", internal_state));
//         } else {
//             *internal_state.borrow_mut() = initial_state;
//             val = initial_state;
//             log(&format!("{:?} setting internal", internal_state));
//         }

//         let state = Rc::new(RefCell::new(val));
//         let state_copy = state.clone();
//         let set_state = Box::new(move |new_val: i32, el: Element| {
//             *state_copy.borrow_mut() += new_val;
//         }) as Box<dyn FnMut(i32, Element) -> ()>;

//         (state, set_state)
//     };
//     return Box::new(use_state) as Box<dyn FnMut(i32) -> UseState>;
// }
// pub fn parse_html(html_string: String) {
//     // bytes instead?
//     let mut arena_tree: ArenaTree = ArenaTree::default();
//     let tokens: Vec<char> = html_string.chars().collect();

//     fn recurse(tokens: Vec<char>, mut arena_tree: ArenaTree) -> ArenaTree {
//         if tokens.len() <= 1 {
//             return arena_tree;
//         }
//         let start_position = tokens.iter().position(|x| x.to_string() == "<").unwrap();
//         let close_position = tokens.iter().rposition(|x| x.to_string() == ">").unwrap();
//         let current_element = &tokens[start_position..close_position + 1];
//         let current_element_type_close = current_element
//             .iter()
//             .position(|x| x.to_string() == ">")
//             .unwrap();
//         let children_close = tokens.iter().rposition(|x| x.to_string() == "<").unwrap();
//         let element_type = &current_element[start_position + 1..current_element_type_close];
//         let child_element = &tokens[current_element_type_close + 1..children_close];
//         let new_tokens = child_element.to_vec();
//         log(&format!(
//             "{:?}, {:?},",
//             child_element.to_vec(),
//             element_type
//         ));
//         arena_tree.insert(Node {
//             element_type: element_type.into_iter().collect::<String>(),
//             ..Default::default()
//         });
//         return recurse(new_tokens, arena_tree);
//     }

//     let tree = recurse(tokens, arena_tree);
//     log(&format!("{:?}", tree));
// }
