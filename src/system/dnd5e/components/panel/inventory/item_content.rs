use crate::system::dnd5e::{
	components::{editor::mutator_list, FormulaInline, WalletInline},
	data::item::{Item, ItemKind},
};
use yew::prelude::*;

pub fn item_body(item: &Item) -> Html {
	// TODO: Proficient (weaponry, armor, etc)
	// TODO: rarity
	let mut sections = Vec::new();
	if !item.worth.is_empty() {
		sections.push(html! {
			<div class="property">
				<strong>{"Worth:"}</strong>
				<span><WalletInline wallet={item.worth} /></span>
			</div>
		});
	}
	if item.weight > 0.0 {
		sections.push(html! {
			<div class="property">
				<strong>{"Weight:"}</strong>
				<span>{item.weight * item.quantity() as f32}{" lb."}</span>
			</div>
		});
	}
	match &item.kind {
		ItemKind::Simple { count } => {
			sections.push(html! {
				<div class="property">
					<strong>{"Quantity:"}</strong>
					<span>{count}</span>
				</div>
			});
		}
		ItemKind::Equipment(equipment) => {
			if !equipment.mutators.is_empty() {
				let mut criteria_html = None;
				if let Some(criteria) = &equipment.criteria {
					criteria_html = Some(html! {
						<div>
							<div>{"Only if:"}</div>
							<span>{criteria.description().unwrap_or_else(|| format!("criteria missing description"))}</span>
						</div>
					});
				}
				sections.push(html! {
					<div class="border-bottom-theme-muted">
						<div>{"You gain the following benefits while this item is equipped:"}</div>
						{mutator_list(&equipment.mutators, false)}
						{criteria_html.unwrap_or_default()}
					</div>
				});
			}
			if let Some(shield_bonus) = &equipment.shield {
				sections.push(html! {
					<div class="border-bottom-theme-muted">
						<strong>{"Shield"}</strong>
						<div class="ms-3">
							<div class="property">
								<strong>{"Armor Class Bonus:"}</strong>
								<span>{format!("{shield_bonus:+}")}</span>
							</div>
						</div>
					</div>
				});
			}
			if let Some(armor) = &equipment.armor {
				let prop_kind = html! {
					<div class="property">
						<strong>{"Type:"}</strong>
						<span>{armor.kind.to_string()}</span>
					</div>
				};
				let prop_formula = html! {
					<div class="property">
						<strong>{"Armor Class Formula:"}</strong>
						<span><FormulaInline formula={armor.formula.clone()} /></span>
					</div>
				};
				let prop_min_score = match &armor.min_strength_score {
					None => None,
					Some(min_score) => Some(html! {
						<div class="property">
							<strong>{"Minimum Strength Score:"}</strong>
							<span>{min_score}</span>
						</div>
					}),
				};
				sections.push(html! {
					<div class="border-bottom-theme-muted">
						<strong>{"Armor"}</strong>
						<div class="ms-3">
							{prop_kind}
							{prop_formula}
							{prop_min_score.unwrap_or_default()}
						</div>
					</div>
				});
			}
			if let Some(weapon) = &equipment.weapon {
				let mut weapon_sections = Vec::new();
				weapon_sections.push(html! {
					<div class="property">
						<strong>{"Type:"}</strong>
						<span>{weapon.kind}</span>
					</div>
				});
				weapon_sections.push(html! {
					<div class="property">
						<strong>{"Classification:"}</strong>
						<span>{weapon.classification.clone()}</span>
					</div>
				});
				if let Some(reach) = weapon.melee_reach() {
					weapon_sections.push(html! {
						<div class="property">
							<strong>{"Melee Attack Reach:"}</strong>
							<span>{reach}{" ft."}</span>
						</div>
					});
				}
				if let Some((short, long)) = weapon.range() {
					// TODO: find a way to communicate attack range better:
					// - normal if the target is at or closer than `short`
					// - made a disadvantage when the target is father than `short`, but closer than `long`
					// - impossible beyond the `long` range
					weapon_sections.push(html! {
						<div class="property">
							<strong>{"Range:"}</strong>
							<span>{format!("{short} ft. / {long} ft.")}</span>
						</div>
					});
				}
				if let Some(damage) = &weapon.damage {
					weapon_sections.push(html! {
						<div class="property">
							<strong>{"Damage:"}</strong>
							<span>
								{match (&damage.roll, damage.bonus) {
									(None, bonus) => bonus.to_string(),
									(Some(roll), 0) => roll.to_string(),
									(Some(roll), bonus) => format!("{}{bonus:+}", roll.to_string()),
								}}
								<span style="margin-left: 0.5rem;">{damage.damage_type.display_name()}</span>
							</span>
						</div>
					});
				}
				if !weapon.properties.is_empty() {
					weapon_sections.push(html! {
						<div class="property">
							<strong>{"Properties:"}</strong>
							<ul>
								{weapon.properties.iter().map(|property| html! {
									<li>
										<div class="property">
											<strong>{property.display_name()}{":"}</strong>
											<span>{property.description()}</span>
										</div>
									</li>
								}).collect::<Vec<_>>()}
							</ul>
						</div>
					});
				}
				sections.push(html! {
					<div class="border-bottom-theme-muted">
						<strong>{"Weapon"}</strong>
						<div class="ms-3">
							{weapon_sections}
						</div>
					</div>
				});
			}
			if let Some(_attunement) = &equipment.attunement {
				// TODO: Display attunement
				// (if mutable) (un)attune button: disabled when all slots filled and not currently attuned
				// mutators & criteria applied when attuned
				// warning if attuned and not currently equipped
			}
		}
	}
	if let Some(desc) = &item.description {
		sections.push(html! { <div class="text-block">{desc.clone()}</div> });
	}
	if let Some(notes) = &item.notes {
		sections.push(html! {
			<div class="property">
				<strong>{"Notes."}</strong>
				<span class="text-block">{notes.clone()}</span>
			</div>
		});
	}
	if !item.tags.is_empty() {
		sections.push(html! {
			<div class="property">
				<strong>{"Tags:"}</strong>
				<span>{item.tags.join(", ")}</span>
			</div>
		});
	}
	html! {<>
		{sections}
	</>}
}
