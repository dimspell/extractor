use dispel_core::references::event_scr::ActionFunction;

#[derive(Debug, Clone)]
pub enum ScriptNode {
    Statement {
        action_index: usize,
        depth: usize,
    },
    Block {
        open_index: usize,
        close_index: usize,
        depth: usize,
        children: Vec<ScriptNode>,
    },
}

pub fn build_act_tree(actions: &[ActionFunction]) -> Vec<ScriptNode> {
    struct Frame {
        open_index: usize,
        children: Vec<ScriptNode>,
    }

    let mut result = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();

    for (i, act) in actions.iter().enumerate() {
        match act.raw_content.as_deref() {
            Some("{") => {
                stack.push(Frame {
                    open_index: i,
                    children: Vec::new(),
                });
            }
            Some("}") => {
                if let Some(frame) = stack.pop() {
                    let depth = stack.len();
                    let block = ScriptNode::Block {
                        open_index: frame.open_index,
                        close_index: i,
                        depth,
                        children: frame.children,
                    };
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(block);
                    } else {
                        result.push(block);
                    }
                } else {
                    result.push(ScriptNode::Statement {
                        action_index: i,
                        depth: 0,
                    });
                }
            }
            _ => {
                let depth = stack.len();
                let node = ScriptNode::Statement {
                    action_index: i,
                    depth,
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    result.push(node);
                }
            }
        }
    }

    while let Some(frame) = stack.pop() {
        let depth = stack.len();
        let block = ScriptNode::Block {
            open_index: frame.open_index,
            close_index: usize::MAX,
            depth,
            children: frame.children,
        };
        if let Some(parent) = stack.last_mut() {
            parent.children.push(block);
        } else {
            result.push(block);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn act(raw: Option<&str>) -> ActionFunction {
        ActionFunction {
            prefix: None,
            function_name: String::new(),
            parameters: vec![],
            raw_content: raw.map(|s| s.to_string()),
        }
    }

    fn func(name: &str) -> ActionFunction {
        ActionFunction {
            prefix: None,
            function_name: name.to_string(),
            parameters: vec![],
            raw_content: None,
        }
    }

    #[test]
    fn test_empty_actions() {
        let tree = build_act_tree(&[]);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_simple_block() {
        let actions = vec![act(Some("{")), func("test"), act(Some("}"))];
        let tree = build_act_tree(&actions);
        assert_eq!(tree.len(), 1);
        match &tree[0] {
            ScriptNode::Block {
                open_index,
                close_index,
                depth,
                children,
            } => {
                assert_eq!(*open_index, 0);
                assert_eq!(*close_index, 2);
                assert_eq!(*depth, 0);
                assert_eq!(children.len(), 1);
                if let ScriptNode::Statement {
                    action_index, depth, ..
                } = &children[0]
                {
                    assert_eq!(*action_index, 1);
                    assert_eq!(*depth, 1);
                } else {
                    panic!("Expected Statement in block children");
                }
            }
            _ => panic!("Expected Block"),
        }
    }

    #[test]
    fn test_nested_blocks() {
        let actions = vec![
            act(Some("{")),
            act(Some("{")),
            func("inner"),
            act(Some("}")),
            act(Some("}")),
        ];
        let tree = build_act_tree(&actions);
        assert_eq!(tree.len(), 1);
        if let ScriptNode::Block { children, .. } = &tree[0] {
            assert_eq!(children.len(), 1);
            if let ScriptNode::Block { children: inner, .. } = &children[0] {
                assert_eq!(inner.len(), 1);
            } else {
                panic!("Expected inner Block");
            }
        } else {
            panic!("Expected outer Block");
        }
    }

    #[test]
    fn test_unclosed_block() {
        let actions = vec![act(Some("{")), func("orphan")];
        let tree = build_act_tree(&actions);
        assert_eq!(tree.len(), 1);
        if let ScriptNode::Block {
            close_index,
            children,
            ..
        } = &tree[0]
        {
            assert_eq!(*close_index, usize::MAX);
            assert_eq!(children.len(), 1);
        } else {
            panic!("Expected Block");
        }
    }

    #[test]
    fn test_orphan_close() {
        let actions = vec![act(Some("}"))];
        let tree = build_act_tree(&actions);
        assert_eq!(tree.len(), 1);
        if let ScriptNode::Statement {
            action_index, ..
        } = &tree[0]
        {
            assert_eq!(*action_index, 0);
        } else {
            panic!("Expected Statement");
        }
    }

    #[test]
    fn test_flat_functions() {
        let actions = vec![func("a"), func("b")];
        let tree = build_act_tree(&actions);
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_if_block() {
        let actions = vec![
            act(Some("if(x)")),
            act(Some("{")),
            func("body"),
            act(Some("}")),
        ];
        let tree = build_act_tree(&actions);
        assert_eq!(tree.len(), 2);
        if let ScriptNode::Statement {
            action_index, ..
        } = &tree[0]
        {
            assert_eq!(*action_index, 0);
        } else {
            panic!("Expected Statement for if");
        }
        if let ScriptNode::Block { children, .. } = &tree[1] {
            assert_eq!(children.len(), 1);
        } else {
            panic!("Expected Block");
        }
    }
}
