use std::cell::{Ref, RefCell};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct PoolNode {
  pub ch: char,
  left: Option<usize>,
  right: Option<usize>,
  parent: Option<usize>,
  pub tail: bool,
}

pub struct StringPool {
  pub nodes: RefCell<Vec<PoolNode>>,
  pub strings: RefCell<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringId(usize);

impl PoolNode {
  fn new(ch: char, parent: Option<usize>) -> Self {
    Self {
      ch,
      left: None,
      right: None,
      parent,
      tail: false,
    }
  }
}

impl std::fmt::Debug for StringPool {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // find all the tails -- we manage this in the `insert` function, so even
    // substrings can be identified as distinct from their greater parts
    let tails = self.nodes.borrow()
      .iter()
      .enumerate()
      .filter_map(|(id, node)| {
        if node.tail {
          Some(PoolId(id))
        } else {
          None
        }
      }).collect::<Vec<_>>();

    // collect the strings so we can display them as if they weren't completely
    // illegible in the debug format
    let strings = tails.into_iter()
      .map(|id| self.get(id).collect())
      .collect::<Vec<String>>();

    f.debug_struct("StringPool")
      .field("strings", &strings)
      .finish()
  }
}

impl StringPool {
  pub fn new() -> Self {
    Self {
      nodes: RefCell::new(vec![]),
      strings: RefCell::new(vec![]),
    }
  }

  pub fn insert_string(&self, string: String) -> StringId {
    let id = StringId(self.strings.borrow().len());
    self.strings.borrow_mut().push(string);

    id
  }

  pub fn get_string(&self, StringId(id): StringId) -> Ref<'_, String> {
    Ref::map(
      self.strings.borrow(),
      |strings| strings.get(id).unwrap()
    )
  }

  pub fn insert(&self, str: &str) -> PoolId {
    assert!(!str.is_empty(), "str may not be empty");

    // current node id
    let mut id = 0;
    // last node id
    let mut last_id = None;
    // the node id of the current char's preceding character, if any
    let mut parent = None;

    // borrow our nodes
    let mut nodes = self.nodes.borrow_mut();

    // for each char in the string
    for ch in str.chars() {
      // repeat until we find or create a node that matches the current
      // character
      loop {
        // the length represents the location of the next node, if we are going
        // to create one.  we must get this first because we'll be borrowing
        // `nodes`
        let mut len = nodes.len();

        // find or create the node at `id`
        let node = match nodes.get_mut(id) {
          Some(node) => node,
          None => {
            // if creating a node, we'll have to increment `len` by one
            len += 1;

            // push it and get the mut ref
            nodes.push(PoolNode::new(ch, parent));
            nodes.last_mut().unwrap()
          },
        };

        // whether the current node we got is matching the current string char
        let matches = node.ch == ch;

        // get the next branch, left if matches, right if not
        let next_branch = if matches {
          &mut node.left
        } else {
          &mut node.right
        };

        // save id before we update it
        last_id = Some(id);

        // if next branch already has somewhere for us to go, follow it
        match next_branch {
          Some(next_id) => id = *next_id,
          None => {
            // otherwise, set `id` for next loop iteration to be that value we
            // saved earlier, the value of a node at the end of the vec that
            // will be created
            id = len;

            // since `next_branch` was empty, point it where we'll go next
            *next_branch = Some(id);
          },
        };

        // only update parent if this node is part of this string's binary tree
        if matches {
          parent = last_id;

          // we found our end node, break out
          break;
        };
      };
    };

    // we asserted that the string isn't empty, so there'll have to be at least
    // one node in our vec.  hence, we were somewhere, meaning we'll have a
    // `last_id`
    let last_id = last_id.unwrap();

    // make sure to mark the end node as a tail.  it could be that this string
    // is a substring of something larger and therefore we'll do this so we can
    // dump our table in a debug function.  not particularly necessary, but oh
    // well
    nodes.get_mut(last_id).unwrap().tail = true;

    // wrap it in an arithmetic type to avoid mixing different kinds of indices
    PoolId(last_id)
  }

  pub fn get(&self, PoolId(mut id): PoolId) -> impl Iterator<Item = char> {
    // the nodes are actually traversed backwards in this method, then drained
    // from the end
    let mut deque = VecDeque::new();

    let nodes = self.nodes.borrow();

    loop {
      // get the current node
      let node = nodes.get(id).unwrap();

      // prepend the ch
      deque.push_front(node.ch);

      // follow the parent -- in `insert`, we only update this to include parts
      // of the string in this traversal
      match &node.parent {
        Some(new_id) => id = *new_id,
        // if there's no parent, we're done
        None => break,
      };
    };

    // read the string front-to-back
    deque.into_iter()
  }
}
