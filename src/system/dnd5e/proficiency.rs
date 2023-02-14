use enum_map::Enum;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Level {
	None,
	Half,
	Full,
	Double,
}

impl Default for Level {
	fn default() -> Self {
		Self::None
	}
}

impl Level {
	pub fn as_display_name(&self) -> &'static str {
		match self {
			Self::None => "Not Proficient",
			Self::Half => "Half Proficient",
			Self::Full => "Proficient",
			Self::Double => "Expertise",
		}
	}

	pub fn bonus_multiplier(&self) -> f32 {
		match self {
			Self::None => 0.0,
			Self::Half => 0.5,
			Self::Full => 1.0,
			Self::Double => 2.0,
		}
	}
}

// TODO: Move into components
impl Into<yew::prelude::Html> for Level {
	fn into(self) -> yew::prelude::Html {
		use yew::prelude::*;
		match self {
			Self::None => html! { <i class="fa-regular fa-circle" /> },
			Self::Half => {
				html! { <i class="fa-solid fa-circle-half-stroke" style="color: var(--theme-frame-color);" /> }
			}
			Self::Full => {
				html! { <i class="fa-solid fa-circle" style="color: var(--theme-frame-color);" /> }
			}
			Self::Double => {
				html! { <i class="fa-regular fa-circle-dot" style="color: var(--theme-frame-color);" /> }
			}
		}
	}
}

impl std::ops::Mul<i32> for Level {
	type Output = i32;

	fn mul(self, prof_bonus: i32) -> Self::Output {
		let modified = (prof_bonus as f32) * self.bonus_multiplier();
		modified.floor() as i32
	}
}

/// Non-ability/skill proficiencies
#[derive(Clone, Copy, Debug, PartialEq, Enum)]
pub enum Kind {
	Language,
	Armor,
	Weapon,
	Tool,
}
