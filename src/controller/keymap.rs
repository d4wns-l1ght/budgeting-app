use std::{collections::HashSet, fmt::Debug, rc::Rc};

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::{controller::ControllerState, model::Model, view::View};

pub(super) trait ActionFn: FnMut(&mut View, &mut Model, &mut ControllerState) {}
impl<T> ActionFn for T where T: FnMut(&mut View, &mut Model, &mut ControllerState) {}
pub(super) trait PredicateFn: Fn(&KeyEvent, &ControllerState) -> bool {}
impl<T> PredicateFn for T where T: Fn(&KeyEvent, &ControllerState) -> bool {}

pub(super) type Action = dyn ActionFn;
pub(super) type Predicate = dyn PredicateFn;

pub(super) struct KeyMap {
	pub keys: HashSet<KeyCode>,
	pub predicate: Option<Pred>,
	pub action: Box<Action>,
}

impl Debug for KeyMap {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("KeyMap")
			.field("keys", &self.keys)
			.field("predicate", &self.predicate)
			.field("action", &"<action>")
			.finish()
	}
}

impl KeyMap {
	pub fn matches(&self, event: &KeyEvent, controller_state: &ControllerState) -> bool {
		self.keys.contains(&event.code)
			&& self
				.predicate
				.as_ref()
				.is_none_or(|pred| pred.evaluate(&event, controller_state))
	}
}

#[derive(Debug, Default)]
pub(super) struct KeyMapBuilder {
	keys: HashSet<KeyCode>,
	predicate: Option<Pred>,
}

impl KeyMapBuilder {
	pub fn new<I>(keys: I) -> Self
	where
		I: IntoIterator<Item = KeyCode>,
	{
		Self {
			keys: keys.into_iter().collect(),
			predicate: None,
		}
	}

	pub fn when(self, pred: Pred) -> Self {
		Self {
			predicate: Some(pred),
			..self
		}
	}

	pub fn do_action<F>(self, f: F) -> KeyMap
	where
		F: ActionFn + 'static,
	{
		KeyMap {
			keys: self.keys,
			predicate: self.predicate,
			action: Box::new(f),
		}
	}
}

pub(super) struct Pred(Rc<Predicate>);

impl Clone for Pred {
	fn clone(&self) -> Self {
		Pred(Rc::clone(&self.0))
	}
}

impl Debug for Pred {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("<predicate>").finish()
	}
}

#[allow(dead_code)]
impl Pred {
	pub fn new<F>(f: F) -> Self
	where
		F: PredicateFn + 'static,
	{
		Self(Rc::new(f))
	}

	pub fn evaluate(&self, event: &KeyEvent, controller_state: &ControllerState) -> bool {
		(self.0)(event, controller_state)
	}

	pub fn and(self, other: Pred) -> Self {
		Self(Rc::new(move |ke, cs| (self.0)(ke, cs) && (other.0)(ke, cs)))
	}

	pub fn or(self, other: Pred) -> Self {
		Self(Rc::new(move |ke, cs| (self.0)(ke, cs) || (other.0)(ke, cs)))
	}

	pub fn not(self) -> Self {
		Self(Rc::new(move |ke, cs| !(self.0)(ke, cs)))
	}

	pub fn into_inner(self) -> Rc<Predicate> {
		self.0
	}
}
