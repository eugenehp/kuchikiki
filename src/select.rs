use std::cell::RefCell;
use std::iter::FilterMap;

use selectors::{self, parser, matching};
use selectors::parser::{AttrSelector, NamespaceConstraint, Selector};
use string_cache::{Atom, Namespace, QualName};

use tree::{Node, NodeRef, NodeData, NodeDataRef, ElementData, Descendants};


impl selectors::Node for NodeRef {
    type Element = NodeDataRef<ElementData>;

    fn parent_node(&self) -> Option<Self> { Node::parent(self) }
    fn first_child(&self) -> Option<Self> { Node::first_child(self) }
    fn last_child(&self) -> Option<Self> { Node::last_child(self) }
    fn prev_sibling(&self) -> Option<Self> { Node::previous_sibling(self) }
    fn next_sibling(&self) -> Option<Self> { Node::next_sibling(self) }
    fn is_document(&self) -> bool { matches!(self.data, NodeData::Document(_)) }
    fn as_element(&self) -> Option<NodeDataRef<ElementData>> { self.clone().into_element_ref() }
}


impl selectors::Element for NodeDataRef<ElementData> {
    type Node = NodeRef;

    fn as_node(&self) -> NodeRef { NodeDataRef::as_node(self).clone() }
    fn is_html_element_in_html_document(&self) -> bool {
        // FIXME: Have a notion of HTML document v.s. XML document?
        self.name.ns == ns!(html)
    }
    fn get_local_name<'a>(&'a self) -> &'a Atom { &self.name.local }
    fn get_namespace<'a>(&'a self) -> &'a Namespace { &self.name.ns }
    fn get_hover_state(&self) -> bool { false }
    fn get_focus_state(&self) -> bool { false }
    fn get_id(&self) -> Option<Atom> {
        self.attributes.borrow().get(&QualName::new(ns!(""), atom!(id))).map(|s| Atom::from_slice(s))
    }
    fn get_disabled_state(&self) -> bool { false }
    fn get_enabled_state(&self) -> bool { false }
    fn get_checked_state(&self) -> bool { false }
    fn get_indeterminate_state(&self) -> bool { false }
    fn has_class(&self, name: &Atom) -> bool {
        !name.is_empty() &&
        if let Some(class_attr) = self.attributes.borrow().get(&QualName::new(ns!(""), atom!(class))) {
            class_attr.split(::selectors::matching::SELECTOR_WHITESPACE)
            .any(|class| name.as_slice() == class )
        } else {
            false
        }
    }
    fn is_link(&self) -> bool {
        self.name.ns == ns!(html) &&
        matches!(self.name.local, atom!(a) | atom!(area) | atom!(link)) &&
        self.attributes.borrow().contains_key(&QualName::new(ns!(""), atom!(href)))
    }
    fn is_visited_link(&self) -> bool { false }
    fn is_unvisited_link(&self) -> bool { self.is_link() }
    fn each_class<F>(&self, mut callback: F) where F: FnMut(&Atom) {
        if let Some(class_attr) = self.attributes.borrow().get(&QualName::new(ns!(""), atom!(class))) {
            for class in class_attr.split(::selectors::matching::SELECTOR_WHITESPACE) {
                if !class.is_empty() {
                    callback(&Atom::from_slice(class))
                }
            }
        }
    }
    fn match_attr<F>(&self, attr: &AttrSelector, test: F) -> bool where F: Fn(&str) -> bool {
        let name = if self.is_html_element_in_html_document() {
            &attr.lower_name
        } else {
            &attr.name
        };
        self.attributes.borrow().iter().any(|(key, value)| {
            !matches!(attr.namespace, NamespaceConstraint::Specific(ref ns) if *ns != key.ns) &&
            key.local == *name &&
            test(value)
        })
    }
}

impl NodeRef {
    pub fn select(&self, css_str: &str) -> Result<SelectNodes<Descendants>, ()> {
        let selectors = try!(parser::parse_author_origin_selector_list_from_str(css_str));
        Ok(SelectNodes{
            iter: self.descendants(),
            filter: selectors,
        })
    }

    pub fn text_iter<'a>(&self) -> FilterMap<Descendants, fn(NodeRef)-> Option<NodeDataRef<RefCell<String>>>> {
        self.descendants().filter_map(NodeRef::into_text_ref)
    }
}

pub struct SelectNodes<T> {
    iter: T,
    filter: Vec<Selector>,
}

impl<'a,T> Iterator for SelectNodes<T> where T: Iterator<Item=NodeRef> {
    type Item = NodeDataRef<ElementData>;

    #[inline]
    fn next(&mut self) -> Option<NodeDataRef<ElementData>> {
        for node in self.iter.by_ref() {
            if let Some(element) = node.into_element_ref() {
                if matching::matches(&self.filter, &element, &None) {
                    return Some(element)
                }
            }
        }
        None
    }
}

