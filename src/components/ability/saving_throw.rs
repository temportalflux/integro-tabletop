use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct SavingThrowProps {
	pub title: String,
	pub value: i32,
	pub proficient: bool,
}

#[function_component]
pub fn SavingThrow(
	SavingThrowProps {
		title,
		value,
		proficient,
	}: &SavingThrowProps,
) -> Html {
	let sign = match *value >= 0 {
		true => "+",
		false => "-",
	};
	html! {
		<div class="card" style="border-color: var(--theme-frame-color-muted);">
			<div class="card-body text-center" style="padding: 5px 5px;">
				<div style="display: inline; width: 100%;">
					<span style="font-size: 0.8rem;">
						{match*proficient {
							true => crate::data::ProficiencyLevel::Full,
							false => crate::data::ProficiencyLevel::None,
						}}
						<span style="margin-left: 5px; margin-right: 8px;">{title.clone()}</span>
					</span>
					<span style="font-weight: 700; color: var(--theme-roll-modifier);">{sign}{value.abs()}</span>
				</div>
			</div>
		</div>
	}
}

#[function_component]
pub fn SavingThrowContainer() -> Html {
	html! {
		<div id="saving-throw-container" class="card" style="border-color: transparent;">
			<div class="card-body text-center" style="padding: 5px;">
				<h5 class="card-title" style="font-size: 0.8rem;">{"Saving Throws"}</h5>
				<div class="container overflow-hidden text-center" style="--bs-gutter-x: 0;">
					<SavingThrow title={"STR"} value={-1} proficient={false} />
					<SavingThrow title={"DEX"} value={0} proficient={false} />
					<SavingThrow title={"CON"} value={3} proficient={false} />
					<SavingThrow title={"INT"} value={6} proficient={true} />
					<SavingThrow title={"WIS"} value={4} proficient={true} />
					<SavingThrow title={"CHA"} value={3} proficient={false} />
				</div>
			</div>
		</div>
	}
}
