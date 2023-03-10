use swayipc::{EventType, WindowEvent, WindowChange, NodeType, Event, WorkspaceEvent, WorkspaceChange};

fn main() {
    let unfocused_opacity = std::env::args()
        .skip(1)
        .next()
        .map(|a| a.parse::<f32>().expect("Unfocused window opacity should be a number"))
        .unwrap_or(0.77);

    if unfocused_opacity < 0.0 || unfocused_opacity > 1.0 {
        panic!("Opacity should be between 0.0 and 1.0")
    }

    let mut sway = swayipc::Connection::new()
        .expect("Cannot connect to Sway");

    let tree = sway.get_tree()
        .expect("Cannot get tree");

    let mut prev_focused = tree
        .find_focused(|n| matches!(n.node_type, NodeType::Con | NodeType::FloatingCon));

    for event in sway
        .subscribe([EventType::Window, EventType::Workspace])
        .expect("Cannot subscribe to Sway events")
    {
        match event.expect("Window event error") {
            Event::Window(e) => match *e {

                WindowEvent { change: WindowChange::Mark, container: marked, .. } => {
                    let mut sway = swayipc::Connection::new()
                        .expect("Cannot connect to Sway to set container border");

                    if marked.marks.contains(&String::from("opaque")) {
                        sway.run_command(format!("[con_id={}] border pixel 6", marked.id))
                            .expect("Cannot set opaque container border");
                    } else {
                        sway.run_command(format!("[con_id={}] border pixel 3", marked.id))
                            .expect("Cannot set non-opaque container border");
                    }
                },

                WindowEvent { change: WindowChange::Focus, container: focused, .. } => {
                    if Some(focused.id) == prev_focused.as_ref().map(|n| n.id) {
                        continue
                    }

                    let mut sway = swayipc::Connection::new()
                        .expect("Cannot connect to Sway to set opacity");
                    sway.run_command(format!("[con_id={}] opacity 1", focused.id))
                        .expect("Cannot set focused window opacity");

                    let tree = sway.get_tree()
                        .expect("Cannot get tree");

                    let unfocused = tree
                        .find(|n| Some(n.id) == prev_focused.as_ref().map(|n| n.id));

                    if let Some(unfocused) = unfocused {
                        if unfocused.marks.contains(&String::from("opaque")) {
                            continue
                        }
                        if unfocused.id == focused.id {
                            continue;
                        }

                        sway.run_command(
                            format!("[con_id={}] opacity {unfocused_opacity}", unfocused.id)
                        )
                        .expect("Cannot set unfocused window opacity");
                    }

                    prev_focused = Some(focused);
                }

                _ => {}
            }

            Event::Workspace(e) => match *e {
                WorkspaceEvent {
                    change: WorkspaceChange::Focus,
                    current: Some(workspace),
                    ..
                } => {
                    let Some(focused) = workspace.find_focused(|n|
                        matches!(n.node_type, NodeType::Con | NodeType::FloatingCon)
                    ) else {
                        continue
                    };

                    let mut sway = swayipc::Connection::new()
                        .expect("Cannot connect to Sway to set opacity on ws change");
                    sway.run_command(format!("[con_id={}] opacity 1", focused.id))
                        .expect("Cannot set focused window opacity");

                    prev_focused = Some(focused)
                }

                _ => {}
            }

            _ => {}
        }
    }
}
