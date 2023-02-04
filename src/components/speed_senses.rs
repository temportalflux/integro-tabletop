use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
struct SingleValueProps {
	title: String,
	amount: i32,
}

#[function_component]
fn SingleValue(SingleValueProps { title, amount }: &SingleValueProps) -> Html {
	html! {<div>
		<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{title.clone()}</h6>
		<div class="text-center" style="width: 100%;">
			<span style="position: relative; font-size: 26px; font-weight: 500;">
				<span>{*amount}</span>
				<span style="position: absolute; bottom: 2px; font-size: 16px; margin-left: 3px;">{"ft."}</span>
			</span>
		</div>
	</div>}
}

#[function_component]
pub fn SpeedAndSenses() -> Html {
	//let mut speeds = vec![("Walking", 25)];
	let mut speeds = vec![("Walking", 30), ("Flying", 30), ("Burrow", 10)];
	//let mut senses: Vec<(&'static str, i32)> = vec![];
	//let mut senses = vec![("Darkvision", 60)];
	let mut senses = vec![("Darkvision", 60), ("Truesight", 10)];

	let divider = (speeds.len() > 0 && senses.len() > 0).then(|| html! {
		<div class="col-auto p-0"><div class="vr" style="min-height: 100%;" /></div>
	}).unwrap_or_else(|| html! {});
	let speed = match speeds.len() {
		0 => html! {},
		1 => {
			let (title, amount) = speeds.pop().unwrap();
			html! {<div class="col">
				<SingleValue title={format!("{title} Speed")} {amount} />
			</div>}
		},
		_ => html! {<div class="col">
			<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Speeds"}</h6>
			{speeds.into_iter().map(|(title, amount)| html! {
				<span class="d-flex" style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
					<span class="flex-grow-1">{title}</span>
					<span class="ps-2">{amount}{"ft."}</span>
				</span>
			}).collect::<Vec<_>>()}
		</div>},
	};
	let senses_html = match senses.len() {
		0 => html! {},
		1 => {
			let (title, amount) = senses.pop().unwrap();
			html! {<div class="col">
				<SingleValue {title} {amount} />
			</div>}
		},
		_ => html! {<div class="col">
			<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Senses"}</h6>
			{senses.into_iter().map(|(title, amount)| html! {
				<span class="d-flex" style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
					<span class="flex-grow-1">{title}</span>
					<span class="ps-2">{amount}{"ft."}</span>
				</span>
			}).collect::<Vec<_>>()}
		</div>},
	};
	
	html! {
		<div class="card m-2" style="min-width: 150px;">
			<div class="card-body" style="padding: 5px 5px;">
				<div class="row">
					{speed}
					{divider}
					{senses_html}
				</div>
			</div>
		</div>
	}
}
