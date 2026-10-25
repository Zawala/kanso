//! `patch-note`, `update-note` and `create-note` for an agent that may
//! suggest edits: they stage a change in the
//! [`ReviewQueue`](crate::review::ReviewQueue) and answer "Queued for
//! review". Nothing here writes the store; it only reads it, through the
//! request's [`Visibility`], so a note the agent may not see is answered as a
//! missing one and no suggestion can target it.
//!
//! The first write release is note text and title only (issue #46, the
//! review queue, decision 8): any other field, and `update-tag` and
//! `update-book` altogether, are refused.

use std::sync::Arc;

use kanso_core::Note;
use serde_json::{Map, Value, json};

use super::schema;
use super::tools::{Occurrence, error_result, find_once};
use crate::AppState;
use crate::error::ApiError;
use crate::proposal::NewChange;
use crate::review::{MAX_TEXT_BYTES, QueueFull, ReviewQueue};
use crate::scope::{AgentScope, Visibility};
use crate::tokens::TokenId;

/// What the agent is told once a change is staged.
pub(crate) const QUEUED: &str = "Queued for review. The person will apply or discard it in Kanso.";

const TEXT_AND_TITLE_ONLY: &str =
    "An agent can suggest changes to a note's text and title only. Other fields aren't changed.";
const QUEUE_FULL: &str =
    "Too many suggested edits are waiting for review. Ask the person to review them first.";
const TOO_LARGE: &str = "That suggested edit is too large to queue for review.";

/// The fields `update-note` may carry for an agent, besides `_id` and `_rev`.
const UPDATE_FIELDS: [&str; 4] = ["_id", "_rev", "title", "body"];
/// The fields `patch-note` carries.
const PATCH_FIELDS: [&str; 4] = ["_id", "_rev", "old_string", "new_string"];
/// The fields `create-note` may carry for an agent.
const CREATE_FIELDS: [&str; 3] = ["bookId", "title", "body"];

type Args = Map<String, Value>;

/// Who is suggesting: the agent's name, its session's token and its scope.
struct Who<'a> {
    agent: &'a str,
    token: TokenId,
    scope: &'a Arc<AgentScope>,
}

/// Runs one suggesting tool for the agent holding `token`. Never writes.
pub(crate) fn call(
    state: &AppState,
    token: TokenId,
    scope: &Arc<AgentScope>,
    vis: &Visibility,
    name: &str,
    arguments: Option<&Value>,
) -> Value {
    let Some(agent) = scope.suggests_as.as_deref() else {
        return error_result(super::READ_ONLY);
    };
    let Some(tool) = schema::tool(name) else {
        return error_result(format!("MCP error -32602: Tool {name} not found"));
    };
    let args = match tool.validate(arguments) {
        Ok(args) => args,
        Err(issues) => {
            return error_result(format!(
                "MCP error -32602: Input validation error: Invalid arguments for tool {name}: {issues}"
            ));
        }
    };
    // Fields the schema doesn't name are dropped by `validate`; look at what
    // the agent actually sent.
    if let Some(sent) = arguments.and_then(Value::as_object) {
        let allowed: &[&str] = match name {
            "create-note" => &CREATE_FIELDS,
            "update-note" => &UPDATE_FIELDS,
            _ => &PATCH_FIELDS,
        };
        if sent.keys().any(|key| !allowed.contains(&key.as_str())) {
            return error_result(TEXT_AND_TITLE_ONLY);
        }
    }
    let who = Who {
        agent,
        token,
        scope,
    };
    let reviews = state.tokens.reviews();
    let staged = match name {
        "patch-note" => patch(reviews, &who, vis, &args),
        "update-note" => update(reviews, &who, vis, &args),
        "create-note" => create(&who, vis, &args),
        _ => Err(super::READ_ONLY.to_owned()),
    };
    let new = match staged {
        Ok(new) => new,
        Err(message) => return error_result(message),
    };
    if new.new_title.len() + new.new_body.len() > MAX_TEXT_BYTES {
        return error_result(TOO_LARGE);
    }
    match reviews.stage(new) {
        Ok(_) => json!({ "content": [{ "type": "text", "text": QUEUED }] }),
        Err(QueueFull) => error_result(QUEUE_FULL),
    }
}

fn str_arg<'a>(args: &'a Args, key: &str) -> &'a str {
    args.get(key).and_then(Value::as_str).unwrap_or_default()
}

/// The note `args` names at the revision it names, when the agent may see it.
/// A stale revision reads as the real API answers it: not found.
fn target(vis: &Visibility, args: &Args) -> Result<Note, String> {
    let missing = || ApiError::not_found().fetch_error_text();
    let id = str_arg(args, "_id");
    let note = vis.note(id).ok_or_else(missing)?;
    if note.rev.as_deref() != Some(str_arg(args, "_rev")) {
        return Err(missing());
    }
    Ok(note)
}

/// A change to `note`. When this agent already has one waiting for the same
/// note at the same revision, the new edit builds on its text and replaces
/// it, so the person reviews one change per note.
fn change_to(
    reviews: &ReviewQueue,
    who: &Who,
    note: &Note,
    title: Option<String>,
    body: impl FnOnce(&str) -> Result<String, String>,
) -> Result<NewChange, String> {
    let rev = note.rev.as_deref().unwrap_or_default();
    let (now_title, now_body) = match reviews.pending_for(who.token, &note.id, rev) {
        Some(pending) => (pending.new_title, pending.new_body),
        None => (note.title.clone(), note.body.clone()),
    };
    let new_body = body(&now_body)?;
    Ok(NewChange {
        note_id: Some(note.id.clone()),
        base_revision: note.rev.clone(),
        book_id: None,
        old_title: note.title.clone(),
        old_body: note.body.clone(),
        new_title: title.unwrap_or(now_title),
        new_body,
        agent: who.agent.to_owned(),
        token: who.token,
        scope: who.scope.clone(),
    })
}

fn patch(
    reviews: &ReviewQueue,
    who: &Who,
    vis: &Visibility,
    args: &Args,
) -> Result<NewChange, String> {
    let note = target(vis, args)?;
    let old = str_arg(args, "old_string").to_owned();
    let new = str_arg(args, "new_string").to_owned();
    change_to(reviews, who, &note, None, move |body| {
        match find_once(body, &old) {
            Occurrence::None => Err(
                "Failed to patch: `old_string` was not found in the note body.".to_owned(),
            ),
            Occurrence::Many => Err(
                "Failed to patch: `old_string` matches multiple locations in the note body. Provide more surrounding context to make it unique.".to_owned(),
            ),
            Occurrence::Once(at) => Ok(format!("{}{new}{}", &body[..at], &body[at + old.len()..])),
        }
    })
}

fn update(
    reviews: &ReviewQueue,
    who: &Who,
    vis: &Visibility,
    args: &Args,
) -> Result<NewChange, String> {
    if !args.contains_key("title") && !args.contains_key("body") {
        return Err("There is nothing to change: give a new `title` or `body`.".to_owned());
    }
    let note = target(vis, args)?;
    let title = args.get("title").and_then(Value::as_str).map(str::to_owned);
    let body = args.get("body").and_then(Value::as_str).map(str::to_owned);
    change_to(reviews, who, &note, title, move |now| {
        Ok(body.unwrap_or_else(|| now.to_owned()))
    })
}

fn create(who: &Who, vis: &Visibility, args: &Args) -> Result<NewChange, String> {
    let book = str_arg(args, "bookId");
    if !vis.book_visible(book) {
        // As a notebook that isn't there.
        return Err(ApiError::not_found().fetch_error_text());
    }
    Ok(NewChange {
        note_id: None,
        base_revision: None,
        book_id: Some(book.to_owned()),
        old_title: String::new(),
        old_body: String::new(),
        new_title: str_arg(args, "title").to_owned(),
        new_body: str_arg(args, "body").to_owned(),
        agent: who.agent.to_owned(),
        token: who.token,
        scope: who.scope.clone(),
    })
}
