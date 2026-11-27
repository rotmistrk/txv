//! MessageView: yields width while a modal prompt is open.

use super::*;

fn command(id: CommandId, data: Option<Box<dyn Any + Send>>) -> Event {
    Event::Command {
        id,
        data,
        broadcast: true,
    }
}

fn show(view: &mut MessageView, text: &str) {
    let msg = Message::info("build", text);
    view.handle(&command(CM_STATUS_MESSAGE, Some(Box::new(msg))));
}

fn prompt(view: &mut MessageView, id: CommandId, group: u16) {
    view.handle(&command(id, Some(Box::new(group))));
}

const TEXT: &str = "No test command configured";
const WIDTH: u16 = ("[build] ".len() + TEXT.len() + 2) as u16;

#[test]
fn message_takes_its_width() {
    let mut view = MessageView::new(5);
    show(&mut view, TEXT);
    assert_eq!(view.bounds().w(), WIDTH);
}

#[test]
fn message_yields_while_prompt_open_and_returns_after() {
    let mut view = MessageView::new(5);
    show(&mut view, TEXT);
    prompt(&mut view, CM_DEACTIVATE_GROUP, ALL_GROUPS);
    assert_eq!(view.bounds().w(), 0, "prompt open: message must give up its width");
    prompt(&mut view, CM_ACTIVATE_GROUP, ALL_GROUPS);
    assert_eq!(view.bounds().w(), WIDTH, "prompt closed: message comes back");
}

#[test]
fn message_arriving_during_prompt_stays_hidden_until_close() {
    let mut view = MessageView::new(5);
    prompt(&mut view, CM_DEACTIVATE_GROUP, ALL_GROUPS);
    show(&mut view, TEXT);
    assert_eq!(view.bounds().w(), 0);
    prompt(&mut view, CM_ACTIVATE_GROUP, ALL_GROUPS);
    assert_eq!(view.bounds().w(), WIDTH);
}

#[test]
fn specific_group_toggle_does_not_hide_message() {
    let mut view = MessageView::new(5);
    show(&mut view, TEXT);
    prompt(&mut view, CM_DEACTIVATE_GROUP, 7);
    assert_eq!(view.bounds().w(), WIDTH);
}
