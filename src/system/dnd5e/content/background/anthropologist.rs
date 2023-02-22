use crate::system::dnd5e::{
	character::{AddProficiency, Background},
	mutator::{AddSkill, Selector},
	proficiency, Feature, Skill,
};

pub fn anthropologist() -> Background {
	Background {
		name: "Anthropologist".into(),
		description: "You have always been fascinated by other cultures, from the most ancient and \
		primeval lost lands to the most modern civilizations. By studying other cultures' customs, philosophies, \
		laws, rituals, religious beliefs, languages, and art, you have learned how tribes, empires, \
		and all forms of society in between craft their own destinies and doom. This knowledge came to \
		you not only through books and scrolls, but also through first-hand observation—by visiting far-flung \
		settlements and exploring local histories and customs.".into(),
		mutators: vec![
			AddSkill {
				skill: Selector::Specific(Skill::Insight),
				proficiency: proficiency::Level::Full,
			}.into(),
			AddSkill {
				skill: Selector::Specific(Skill::Religion),
				proficiency: proficiency::Level::Full,
			}.into(),
		],
		features: vec![
			Feature {
				name: "Languages".into(),
				description: "You can speak, read, and write two languages of your choice.".into(),
				mutators: vec![
					AddProficiency::Language(Selector::Any { id: Some("langA".into()) }).into(),
					AddProficiency::Language(Selector::Any { id: Some("langB".into()) }).into(),
				],
				..Default::default()
			}.into(),
			Feature {
				name: "Adept Linguist".into(),
				description: "You can communicate with humanoids who don't speak any language you know. \
				You must observe the humanoids interacting with one another for at least 1 day, \
				after which you learn a handful of important words, expressions, and gestures—enough \
				to communicate on a rudimentary level.".into(),
				..Default::default()
			}.into(),
		],
	}
}
