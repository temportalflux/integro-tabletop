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
		<div id="saving-throw-container" class="card" style="">
			<div class="card-body text-center" style="padding: 5px;">
				<h5 class="card-title" style="font-size: 0.8rem;">{"Saving Throws"}</h5>
				<div class="row" style="--bs-gutter-x: 0; margin: 0 0 10px 0;">
					<div class="col">
						<table class="table table-compact" style="margin-bottom: 0;">
							<tbody>
								<tr>
									<td class="text-center">{crate::data::ProficiencyLevel::None}</td>
									<td>{"STR"}</td>
									<td class="text-center"><span style="font-weight: 700; color: var(--theme-roll-modifier);">{"-1"}</span></td>
								</tr>
								<tr>
									<td class="text-center">{crate::data::ProficiencyLevel::None}</td>
									<td>{"DEX"}</td>
									<td class="text-center"><span style="font-weight: 700; color: var(--theme-roll-modifier);">{"+0"}</span></td>
								</tr>
								<tr>
									<td class="text-center">{crate::data::ProficiencyLevel::None}</td>
									<td>{"CON"}</td>
									<td class="text-center"><span style="font-weight: 700; color: var(--theme-roll-modifier);">{"+3"}</span></td>
								</tr>
							</tbody>
						</table>
					</div>
					<div class="col-auto p-0" style="margin: 0 5px;"><div class="vr" style="min-height: 100%;" /></div>
					<div class="col">
						<table class="table table-compact" style="margin-bottom: 0;">
							<tbody>
								<tr>
									<td class="text-center">{crate::data::ProficiencyLevel::Full}</td>
									<td>{"INT"}</td>
									<td class="text-center"><span style="font-weight: 700; color: var(--theme-roll-modifier);">{"+6"}</span></td>
								</tr>
								<tr>
									<td class="text-center">{crate::data::ProficiencyLevel::Full}</td>
									<td>{"WIS"}</td>
									<td class="text-center"><span style="font-weight: 700; color: var(--theme-roll-modifier);">{"+4"}</span></td>
								</tr>
								<tr>
									<td class="text-center">{crate::data::ProficiencyLevel::None}</td>
									<td>{"CHA"}</td>
									<td class="text-center"><span style="font-weight: 700; color: var(--theme-roll-modifier);">{"+3"}</span></td>
								</tr>
							</tbody>
						</table>
					</div>
				</div>
				<div class="container overflow-hidden text-center" style="display: none; --bs-gutter-x: 0;">
					<SavingThrow title={"STR"} value={-1} proficient={false} />
					<SavingThrow title={"DEX"} value={0} proficient={false} />
					<SavingThrow title={"CON"} value={3} proficient={false} />
					<SavingThrow title={"INT"} value={6} proficient={true} />
					<SavingThrow title={"WIS"} value={4} proficient={true} />
					<SavingThrow title={"CHA"} value={3} proficient={false} />
				</div>
				<div>
					<div style="font-size: 11px;">
						<span class="d-inline-flex" aria-label="Advantage" style=" height: 14px; margin-right: 2px; margin-top: -2px; width: 14px; vertical-align: middle;">
							<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
								<g>
									<path d="M13.3665 12.5235L12.009 8.78235L10.6516 12.5235H13.3665Z" fill="#00c680"></path>
									<path fill-rule="evenodd" clip-rule="evenodd" d="M12.241 1.13253C12.0909 1.05 11.9091 1.05 11.759 1.13252L2.25904 6.35753C2.09927 6.4454 2 6.61329 2 6.79563V17.2044C2 17.3867 2.09927 17.5546 2.25904 17.6425L11.759 22.8675C11.9091 22.95 12.0909 22.95 12.241 22.8675L21.741 17.6425C21.9007 17.5546 22 17.3867 22 17.2044V6.79563C22 6.61329 21.9007 6.4454 21.741 6.35753L12.241 1.13253ZM18 17.5H15.1222L14.1991 14.9412H9.80091L8.87783 17.5H6L10.5611 5.5H13.4389L18 17.5Z" fill="#00c680"></path>
								</g>
							</svg>
						</span>
						<span>{"on INT against Magic"}</span>
					</div>
				</div>
			</div>
		</div>
	}
}
