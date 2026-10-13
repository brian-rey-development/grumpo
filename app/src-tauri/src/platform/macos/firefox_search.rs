//! Finds Firefox's URL field in the browser chrome of a window. Page content lives under
//! `AXWebArea` and `AXScrollArea`, which are never entered, so the walk touches a few dozen
//! interface elements and never the (possibly huge) document.

use std::collections::VecDeque;

use objc2_application_services::AXUIElement;
use objc2_core_foundation::CFRetained;

use super::ax::Ax;
use crate::platform::PlatformError;

const MAX_DEPTH: usize = 12;
const MAX_NODES: usize = 400;

const ATTR_CHILDREN: &str = "AXChildren";
const ATTR_ROLE: &str = "AXRole";
const ATTR_DOM_ID: &str = "AXDOMIdentifier";
const URLBAR_DOM_ID: &str = "urlbar-input";
const NAV_BAR_DOM_ID: &str = "nav-bar";
const TOOLBAR_ROLE: &str = "AXToolbar";
const SKIPPED_ROLES: [&str; 2] = ["AXWebArea", "AXScrollArea"];
const URL_FIELD_ROLES: [&str; 2] = ["AXTextField", "AXComboBox"];

struct Node {
    element: CFRetained<AXUIElement>,
    depth: usize,
    in_nav_bar: bool,
}

struct Search<'a> {
    ax: &'a Ax,
    queue: VecDeque<Node>,
    fallback: Option<CFRetained<AXUIElement>>,
}

/// Breadth first from the toolbars of `window`. The element with the URL bar's DOM id wins,
/// and otherwise the first text field inside the nav bar is used.
pub(super) fn find_url_field(
    ax: &Ax,
    window: &AXUIElement,
) -> Result<Option<CFRetained<AXUIElement>>, PlatformError> {
    let mut search = Search {
        ax,
        queue: toolbars_first(ax, window)?,
        fallback: None,
    };
    for _ in 0..MAX_NODES {
        let Some(node) = search.queue.pop_front() else {
            break;
        };
        if let Some(field) = search.visit(node)? {
            return Ok(Some(field));
        }
    }
    Ok(search.fallback)
}

impl Search<'_> {
    /// Returns the field when `node` is the URL bar, and otherwise queues its children.
    fn visit(&mut self, node: Node) -> Result<Option<CFRetained<AXUIElement>>, PlatformError> {
        let role = self.ax.string(&node.element, ATTR_ROLE)?;
        if has_role(role.as_deref(), &SKIPPED_ROLES) {
            return Ok(None);
        }
        let dom_id = self.ax.string(&node.element, ATTR_DOM_ID)?;
        if dom_id.as_deref() == Some(URLBAR_DOM_ID) {
            return Ok(Some(node.element));
        }
        let in_nav_bar = node.in_nav_bar || dom_id.as_deref() == Some(NAV_BAR_DOM_ID);
        if in_nav_bar && has_role(role.as_deref(), &URL_FIELD_ROLES) && self.fallback.is_none() {
            self.fallback = Some(node.element.clone());
        }
        if node.depth < MAX_DEPTH {
            self.enqueue_children(&node, in_nav_bar)?;
        }
        Ok(None)
    }

    fn enqueue_children(&mut self, node: &Node, in_nav_bar: bool) -> Result<(), PlatformError> {
        for element in self.ax.children(&node.element, ATTR_CHILDREN)? {
            self.queue.push_back(Node {
                element,
                depth: node.depth + 1,
                in_nav_bar,
            });
        }
        Ok(())
    }
}

fn has_role(role: Option<&str>, accepted: &[&str]) -> bool {
    role.is_some_and(|role| accepted.contains(&role))
}

fn toolbars_first(ax: &Ax, window: &AXUIElement) -> Result<VecDeque<Node>, PlatformError> {
    let mut toolbars = VecDeque::new();
    let mut others = VecDeque::new();
    for element in ax.children(window, ATTR_CHILDREN)? {
        let is_toolbar = ax.string(&element, ATTR_ROLE)?.as_deref() == Some(TOOLBAR_ROLE);
        let node = Node {
            element,
            depth: 1,
            in_nav_bar: false,
        };
        if is_toolbar {
            toolbars.push_back(node);
        } else {
            others.push_back(node);
        }
    }
    toolbars.append(&mut others);
    Ok(toolbars)
}
