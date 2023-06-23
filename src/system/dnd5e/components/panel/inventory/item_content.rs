use crate::{
	system::dnd5e::{
		components::{
			editor::{description, mutator_list},
			validate_uint_only, CharacterHandle, FormulaInline, WalletInline,
		},
		data::{
			item::{self, Item},
			ArmorExtended, WeaponProficiency,
		},
		evaluator::IsProficientWith,
	},
	utility::{Evaluator, InputExt},
};
use yew::prelude::*;

#[derive(Default)]
pub struct ItemBodyProps {
	pub on_quantity_changed: Option<Callback<u32>>,
	pub is_equipped: bool,
	pub set_equipped: Option<Callback<bool>>,
}
pub fn item_body(item: &Item, state: &CharacterHandle, props: Option<ItemBodyProps>) -> Html {
	let props = props.unwrap_or_default();
	let mut sections = Vec::new();
	if IsProficientWith::Tool(item.name.clone()).evaluate(state) {
		sections.push(html! {
			<div class="property">
				<strong>{"Proficient (with tool):"}</strong>
				<span>{"✔ Yes"}</span>
			</div>
		});
	}
	if let Some(rarity) = item.rarity {
		sections.push(html! {
			<div class="property">
				<strong>{"Rarity:"}</strong>
				<span>{rarity}</span>
			</div>
		});
	}
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
		item::Kind::Simple { count } => {
			let inner = match (props.on_quantity_changed, item.can_stack()) {
				(None, _) | (Some(_), false) => html! { <span>{count}</span> },
				(Some(on_changed), true) => {
					let count = *count;
					let increment = Callback::from({
						let on_changed = on_changed.clone();
						move |_| {
							on_changed.emit(count.saturating_add(1));
						}
					});
					let decrement = Callback::from({
						let on_changed = on_changed.clone();
						move |_| {
							on_changed.emit(count.saturating_sub(1));
						}
					});
					let onchange = Callback::from({
						let on_changed = on_changed.clone();
						move |evt: web_sys::Event| {
							let Some(value) = evt.input_value_t::<u32>() else { return; };
							on_changed.emit(value);
						}
					});
					html! {
						<div class="input-group item-quantity-inline">
							<button type="button" class="btn btn-theme" onclick={decrement}><i class="bi bi-dash" /></button>
							<input
								class="form-control text-center"
								type="number"
								min="0" value={count.to_string()}
								onkeydown={validate_uint_only()}
								onchange={onchange}
							/>
							<button type="button" class="btn btn-theme" onclick={increment}><i class="bi bi-plus" /></button>
						</div>
					}
				}
			};
			sections.push(html! {
				<div class="property">
					<strong>{"Quantity:"}</strong>
					{inner}
				</div>
			});
		}
		item::Kind::Equipment(equipment) => {
			let mut equip_sections = Vec::new();
			if let Some(on_equipped) = props.set_equipped {
				let onchange = Callback::from({
					move |evt: web_sys::Event| {
						let Some(checked) = evt.input_checked() else { return; };
						on_equipped.emit(checked);
					}
				});
				equip_sections.push(html! {
					<div class="form-check">
						<input  id="equipItem" class="form-check-input equip" type="checkbox" checked={props.is_equipped} {onchange} />
						<label for="equipItem" class="form-check-label">
							{match props.is_equipped {
								true => format!("Equipped"),
								false => format!("Not Equipped"),
							}}
						</label>
					</div>
				});
			}
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
				equip_sections.push(html! {
					<div class="border-bottom-theme-muted">
						<div>{"You gain the following benefits while this item is equipped:"}</div>
						{mutator_list(&equipment.mutators, None::<&CharacterHandle>)}
						{criteria_html.unwrap_or_default()}
					</div>
				});
			}
			if let Some(shield_bonus) = &equipment.shield {
				equip_sections.push(html! {
					<div class="border-bottom-theme-muted">
						<strong>{"Shield"}</strong>
						<div class="ms-3">
							<div class="property">
								<strong>{"Proficient:"}</strong>
								{match IsProficientWith::Armor(ArmorExtended::Shield).evaluate(state) {
									true => html! { <span>{"✔ Yes"}</span> },
									false => html! { <span>{"❌ No"}</span> },
								}}
							</div>
							<div class="property">
								<strong>{"Armor Class Bonus:"}</strong>
								<span>{format!("{shield_bonus:+}")}</span>
							</div>
						</div>
					</div>
				});
			}
			if let Some(armor) = &equipment.armor {
				let mut armor_sections = Vec::new();
				armor_sections.push(html! {
					<div class="property">
						<strong>{"Type:"}</strong>
						<span>{armor.kind.to_string()}</span>
					</div>
				});
				armor_sections.push(html! {
					<div class="property">
						<strong>{"Proficient:"}</strong>
						{match IsProficientWith::Armor(ArmorExtended::Kind(armor.kind)).evaluate(state) {
							true => html! { <span>{"✔ Yes"}</span> },
							false => html! { <span>{"❌ No"}</span> },
						}}
					</div>
				});
				armor_sections.push(html! {
					<div class="property">
						<strong>{"Armor Class Formula:"}</strong>
						<span><FormulaInline formula={armor.formula.clone()} /></span>
					</div>
				});
				if let Some(min_score) = &armor.min_strength_score {
					armor_sections.push(html! {
						<div class="property">
							<strong>{"Minimum Strength Score:"}</strong>
							<span>{min_score}</span>
						</div>
					});
				}
				equip_sections.push(html! {
					<div class="border-bottom-theme-muted">
						<strong>{"Armor"}</strong>
						<div class="ms-3">
							{armor_sections}
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
				let is_proficient = vec![
					IsProficientWith::Weapon(WeaponProficiency::Kind(weapon.kind)),
					IsProficientWith::Weapon(WeaponProficiency::Classification(
						weapon.classification.clone(),
					)),
				];
				let is_proficient = is_proficient.into_iter().any(|eval| eval.evaluate(state));
				weapon_sections.push(html! {
					<div class="property">
						<strong>{"Proficient:"}</strong>
						{match is_proficient {
							true => html! { <span>{"✔ Yes"}</span> },
							false => html! { <span>{"❌ No"}</span> },
						}}
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
				equip_sections.push(html! {
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
			sections.push(html! {
				<div>
					<strong>{"Equipment"}</strong>
					<div class="ms-3">
						{equip_sections}
					</div>
				</div>
			});
		}
	}
	if !item.description.is_empty() {
		let desc = item.description.clone().evaluate(state);
		sections.push(description(&desc, false));
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
