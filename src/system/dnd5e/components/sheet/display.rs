use crate::{
	auth,
	components::{Nav, NavDisplay, TabContent},
	database::app::{Database, Entry},
	system::{
		core::{ModuleId, SourceId},
		dnd5e::{
			components::{
				ability, panel, ArmorClass, CharacterHandle, ConditionsCard, DefensesCard,
				HitPointMgmtCard, InitiativeBonus, Inspiration, ProfBonus, Proficiencies,
				SpeedAndSenses,
			},
			data::Ability,
		},
	},
};
use yew::prelude::*;
use yewdux::prelude::use_store;

mod header;
use header::*;

#[derive(Clone, PartialEq, Properties)]
pub struct SheetDisplayProps {
	pub open_editor: Callback<()>,
}

#[function_component]
pub fn SheetDisplay(SheetDisplayProps { open_editor }: &SheetDisplayProps) -> Html {
	let database = use_context::<Database>().unwrap();
	let state = use_context::<CharacterHandle>().unwrap();
	let (auth_status, _dispatch) = use_store::<auth::Status>();
	let task_dispatch = use_context::<crate::task::Dispatch>().unwrap();
	let save_to_storage = Callback::from({
		let id = state.id().unversioned();
		move |_| {
			let Some(client) = auth_status.storage() else {
				log::debug!("no storage client");
				return;
			};
			let SourceId { module: Some(ModuleId::Github { user_org, repository }), system, path, ..} = &id else {
				log::debug!("non-github source id");
				return;
			};
			let path = match system {
				None => path.clone(),
				Some(system) => std::path::Path::new(&system).join(&path),
			};
			let message = format!("Manually save character");
			let content = {
				let doc = state.export_as_kdl();
				let doc = doc.to_string();
				let doc = doc.replace("\\r", "");
				let doc = doc.replace("\\n", "\n");
				let doc = doc.replace("\\t", "\t");
				let doc = doc.replace("    ", "\t");
				doc
			};
			let id_str = id.to_string();
			let repo_org = user_org.clone();
			let repo_name = repository.clone();
			let database = database.clone();
			task_dispatch.spawn("Update File", None, async move {
				let Some(Entry { file_id: Some(file_id), .. }) = database.get::<Entry>(id_str).await? else {
						log::debug!("missing file id");
						return Ok(());
					};
				let args = crate::storage::github::CreateOrUpdateFileArgs {
					repo_org: &repo_org,
					repo_name: &repo_name,
					path_in_repo: &path,
					commit_message: &message,
					content: &content,
					file_id: Some(&file_id),
					branch: None,
				};
				log::debug!("executing update character request");
				client.create_or_update_file(args).await?;
				log::debug!("finished update character request");
				Ok(()) as anyhow::Result<()>
			});
		}
	});
	html! {
		<div class="container overflow-hidden">
			<div class="d-flex border-bottom-theme-muted mt-1 mb-2 px-3 pb-1">
				<Header />
				<div class="ms-auto">
					<a class="icon forge" onclick={open_editor.reform(|_| ())} />
					<button class="btn btn-success btn-xs" onclick={save_to_storage}>{"Save"}</button>
				</div>
			</div>
			<div class="row" style="--bs-gutter-x: 10px;">
				<div class="col-md-auto" style="max-width: 210px;">

					<div class="row m-0" style="--bs-gutter-x: 0;">
						<div class="col">
							<ability::Score ability={Ability::Strength} />
							<ability::Score ability={Ability::Dexterity} />
							<ability::Score ability={Ability::Constitution} />
						</div>
						<div class="col">
							<ability::Score ability={Ability::Intelligence} />
							<ability::Score ability={Ability::Wisdom} />
							<ability::Score ability={Ability::Charisma} />
						</div>
					</div>

					<ability::SavingThrowContainer />
					<Proficiencies />

				</div>
				<div class="col-md-auto">

					<div class="d-flex justify-content-center">
						<SpeedAndSenses />
					</div>

					<div id="skills-container" class="card" style="min-width: 320px; border-color: var(--theme-frame-color);">
						<div class="card-body" style="padding: 5px;">
							<ability::SkillTable />
						</div>
					</div>

				</div>
				<div class="col">
					<div class="row m-0" style="--bs-gutter-x: 0;">
						<div class="col-auto">
							<div class="d-flex align-items-center" style="height: 100%;">
								<InitiativeBonus />
								<ArmorClass />
								<Inspiration />
							</div>
						</div>
						<div class="col">
							<HitPointMgmtCard />
						</div>
					</div>
					<div class="row m-0" style="--bs-gutter-x: 0;">
						<div class="col-auto">
							<div class="d-flex align-items-center" style="height: 100%;">
								<ProfBonus />
							</div>
						</div>
						<div class="col">
							<DefensesCard />
						</div>
						<div class="col">
							<ConditionsCard />
						</div>
					</div>

					<div class="card m-1" style="height: 550px;">
						<div class="card-body" style="padding: 5px;">
							<Nav root_classes={"onesheet-tabs"} disp={NavDisplay::Tabs} default_tab_id={"actions"}>
								<TabContent id="actions" title={html! {{"Actions"}}}>
									<panel::Actions />
								</TabContent>
								<TabContent id="spells" title={html! {{"Spells"}}}>
									<panel::Spells />
								</TabContent>
								<TabContent id="inventory" title={html! {{"Inventory"}}}>
									<panel::Inventory />
								</TabContent>
								<TabContent id="features" title={html! {{"Features & Traits"}}}>
									<panel::Features />
								</TabContent>
								<TabContent id="description" title={html! {{"Description"}}}>
									{"Description"}
								</TabContent>
							</Nav>
						</div>
					</div>
				</div>
			</div>
		</div>
	}
}
