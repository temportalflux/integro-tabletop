use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct SavingThrowProps {
	pub title: String,
	pub value: i32,
	pub proficient: bool,
}

#[function_component]
pub fn SavingThrow(SavingThrowProps { title, value, proficient }: &SavingThrowProps) -> Html {
	let sign = match *value >= 0 {
		true => "+",
		false => "-",
	};
	let proficency_marker = match proficient {
		true => html! { <i class="fa-solid fa-circle" style="color: var(--theme-frame-color);" /> },
		false => html! { <i class="fa-regular fa-circle" /> },
	};
	html! {
		<div class="card" style="border-color: var(--theme-frame-color-muted);">
			<div class="card-body text-center" style="padding: 5px 5px;">
				<div style="display: inline; width: 100%;">
					<span style="font-size: 0.8rem;">
						{proficency_marker}
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
		<div id="saving-throw-container" class="card" style="width: 20em; border-color: var(--theme-frame-color);">
			<div class="card-body text-center">
				<h5 class="card-title">{"Saving Throws"}</h5>
				<div class="container overflow-hidden text-center">
					<div class="row gy-2" style="margin-top: 0;">
						<div class="col gx-2">
							<SavingThrow title={"STR"} value={-1} proficient={false} />
						</div>
						<div class="col gx-2">
							<SavingThrow title={"DEX"} value={0} proficient={false} />
						</div>
						<div class="col gx-2">
							<SavingThrow title={"CON"} value={3} proficient={false} />
						</div>
					</div>
					<div class="row gy-2" style="margin-top: 0;">
						<div class="col gx-2">
							<SavingThrow title={"INT"} value={6} proficient={true} />
						</div>
						<div class="col gx-2">
							<SavingThrow title={"WIS"} value={4} proficient={true} />
						</div>
						<div class="col gx-2">
							<SavingThrow title={"CHA"} value={3} proficient={false} />
						</div>
					</div>
				</div>
			</div>
		</div>
	}
}
