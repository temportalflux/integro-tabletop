use crate::{
	components::modal,
	system::dnd5e::{components::{SharedCharacter, validate_uint_only, editor::AutoExchangeSwitch}, data::{CurrencyKind, Wallet, character::Persistent}},
};
use itertools::Itertools;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct CoinIconProps {
	pub kind: CurrencyKind,
	#[prop_or(16)]
	pub size: usize,
}

#[function_component]
pub fn CoinIcon(CoinIconProps { kind, size }: &CoinIconProps) -> Html {
	let inner = match kind {
		CurrencyKind::Platinum => html! {<>
			<path d="m9.95662 3.30735c.16878-.19532.41258-.30735.66898-.30735h2.7488c.2564 0 .5002.11203.669.30735l5.7367 6.63816c.1418.16409.2199.37469.2199.59269v2.9236c0 .218-.0781.4286-.2199.5927l-5.7367 6.6382c-.1688.1953-.4126.3073-.669.3073h-2.7488c-.2564 0-.5002-.112-.66899-.3073l-5.73668-6.6382c-.14178-.1641-.21993-.3747-.21993-.5927v-2.9236c0-.218.07815-.4286.21993-.59269z" fill="#b5b5b5"></path>
			<path d="m10.8356 18.1585-5.08818-5.8151c-.1595-.1823-.24742-.4162-.24742-.6585v1c0 .2423.08792.4762.24742.6585l5.08818 5.8151c.1899.217.4642.3415.7526.3415h.8236c.2884 0 .5627-.1245.7526-.3415l5.0882-5.8151c.1595-.1823.2474-.4162.2474-.6585v-1c0 .2423-.0879.4762-.2474.6585l-5.0882 5.8151c-.1899.217-.4642.3415-.7526.3415h-.8236c-.2884 0-.5627-.1245-.7526-.3415z" fill="#a3a3a3"></path>
			<path clip-rule="evenodd" d="m11.5882 4.5c-.2884 0-.5627.12448-.7526.34149l-5.08818 5.81511c-.1595.1823-.24742.4163-.24742.6585v1.3698c0 .2422.08792.4762.24742.6585l5.08818 5.8151c.1899.217.4642.3415.7526.3415h.8236c.2884 0 .5627-.1245.7526-.3415l5.0882-5.8151c.1595-.1823.2474-.4163.2474-.6585v-1.3698c0-.2423-.0879-.4762-.2474-.6585l-5.0882-5.8151c-.1899-.21702-.4642-.3415-.7526-.3415zm-1.1344-2.5c-.2884 0-.56272.12448-.75261.34149l-6.45377 7.37574c-.1595.18229-.24742.41627-.24742.65847v3.2486c0 .2422.08792.4762.24742.6585l6.45377 7.3757c.18989.217.46421.3415.75261.3415h3.0924c.2884 0 .5627-.1245.7526-.3415l6.4538-7.3757c.1595-.1823.2474-.4163.2474-.6585v-3.2486c0-.2422-.0879-.47618-.2474-.65847l-6.4538-7.37573c-.1899-.21702-.4642-.3415-.7526-.3415z" fill="#949494" fill-rule="evenodd"></path>
			<path d="m10.2929 9.12132c-.39053-.39052-.39053-1.02369 0-1.41421l1.2463-1.41422c.3905-.39052 1.0237-.39052 1.4142 0l3.6679 4.29291c.3905.3905.3905 1.0237 0 1.4142l-1.4142 1.4142c-.3905.3905-1.0237.3905-1.4142 0z" fill="#dcdcdc"></path>
			<path d="m8.34764 10.2442c.42128-.36072 1.05967-.31674 1.42589.0983l1.25207 1.3511c.3662.415.3216 1.0439-.0997 1.4046-.4213.3608-1.05969.3168-1.4259-.0982l-1.25207-1.3511c-.36621-.415-.32157-1.0439.09971-1.4047z" fill="#ccc"></path>
			<path d="m11.9999 2.47315c.3413-.26295 1.5-.37315 2 .02685l4 4.5c.3662.37344.3216.93932-.0997 1.26394s-1.0597.28505-1.4259-.08838c0 0-2.74-3.30604-3.5744-3.98415-.3682-.29925-.7065-.1553-.9997-.45432-.2932-.29901-.2416-1.00098.0997-1.26394z" fill="#ccc"></path>
			<circle cx="18.6001" cy="9.25" fill="#b5b5b5" r="1"></circle>
		</>},
		CurrencyKind::Gold => html! {<>
			<path d="m4 4.5c0-.27615.22385-.5.5-.5h15c.2761 0 .5.22385.5.5v1c0 .27615-.2239.5-.5.5 0 0-2 2.32653-2 6 0 3.6735 2 6 2 6 .2761 0 .5.2239.5.5v1c0 .2761-.2239.5-.5.5h-15c-.27615 0-.5-.2239-.5-.5v-1c0-.2761.22385-.5.5-.5 0 0 2-1.8673 2-6 0-4.13265-2-6-2-6-.27615 0-.5-.22385-.5-.5z" fill="#dd970e"></path>
			<path d="m9.99993 12c0-.6216-.03345-1.2128-.09414-1.7735l4.11311 1.0283c-.002.04-.0039.0802-.0056.1205l-4.07254 2.0363c.03844-.4517.05917-.9222.05917-1.4116z" fill="#eca825"></path>
			<path d="m9.79321 14.6034 4.21509-2.1076c.0516 1.5192.3351 2.8657.7046 4.0042h-5.36158c.17845-.5819.33016-1.214.44189-1.8966z" fill="#eca825"></path>
			<path d="m14.0189 11.2548-4.11311-1.0283c-.10897-1.00676-.30575-1.91549-.55447-2.7265h5.36158c-.3492 1.07581-.6215 2.3373-.694 3.7548z" fill="#ffb72c"></path>
			<path d="m6.26343 5.5h11.54557c-.1777.29323-.3708.64106-.5624 1.0399-.0702.14609-.1404.29953-.2097.4601h-9.99142c-.10248-.24281-.20889-.47037-.31689-.68278-.15543-.30571-.31301-.57786-.46516-.81722z" fill="#eca825"></path>
			<path clip-rule="evenodd" d="m17.8089 5.5h-11.54556c.15215.23936.30973.51151.46516.81722.66607 1.31004 1.2715 3.19617 1.2715 5.68278 0 2.4866-.60543 4.3727-1.2715 5.6828-.15543.3057-.31301.5778-.46516.8172h11.54556c-.1777-.2932-.3708-.6411-.5624-1.0399-.6216-1.2941-1.2465-3.1649-1.2465-5.4601 0-2.29517.6249-4.166 1.2465-5.4601.1916-.39884.3847-.74667.5624-1.0399zm2.6911 12.5s-2-2.3265-2-6c0-3.67347 2-6 2-6 .2761 0 .5-.22385.5-.5v-2c0-.27615-.2239-.5-.5-.5h-17c-.27615 0-.5.22385-.5.5v2c0 .27615.22385.5.5.5 0 0 2 1.86735 2 6 0 4.1327-2 6-2 6-.27615 0-.5.2239-.5.5v2c0 .2761.22385.5.5.5h17c.2761 0 .5-.2239.5-.5v-2c0-.2761-.2239-.5-.5-.5z" fill="#c78727" fill-rule="evenodd"></path>
			<path d="m8 4.25c0-.41421.33579-.75.75-.75h11c.4142 0 .75.33579.75.75s-.3358.75-.75.75h-11c-.41421 0-.75-.33579-.75-.75z" fill="#ffb72c"></path>
			<circle cx="6.75" cy="4.25" fill="#eca825" r=".75"></circle>
		</>},
		CurrencyKind::Electrum => html! {<>
			<path clip-rule="evenodd" d="m7.32745 3c-.42911 0-.84556.13554-1.16546.42156-1.20722 1.07938-4.16199 4.17139-4.16199 8.57844 0 4.407 2.95477 7.4991 4.16199 8.5784.3199.2861.73635.4216 1.16547.4216h9.34504c.4292 0 .8456-.1355 1.1655-.4216 1.2072-1.0793 4.162-4.1714 4.162-8.5784 0-4.40704-2.9548-7.49906-4.162-8.57844-.3199-.28602-.7363-.42156-1.1655-.42156zm4.67255 14.25c.6904 0 1.25-.5596 1.25-1.25s-.5596-1.25-1.25-1.25-1.25.5596-1.25 1.25.5596 1.25 1.25 1.25z" fill="#8a9eac" fill-rule="evenodd"></path>
			<path d="m7.03711 16.9284c.34586.3796.84504.5716 1.35854.5716h7.20875c.5134 0 1.0126-.192 1.3585-.5716 1.0485-1.1506 2.3552-3.0498 2.5198-5.4284.0114.1644.0173.3311.0173.5 0 2.6125-1.4162 4.6983-2.5371 5.9284-.3459.3796-.845.5716-1.3585.5716h-7.20875c-.5135 0-1.01268-.192-1.35854-.5716-1.12095-1.2301-2.53711-3.3159-2.53711-5.9284 0-.1689.00592-.3356.01729-.5.16458 2.3786 1.47133 4.2778 2.51982 5.4284z" fill="#6d7f8c"></path>
			<path d="m14.7246 16.375c-.1829 1.3414-1.333 2.375-2.7246 2.375s-2.54175-1.0336-2.72465-2.375c-.01671.1226-.02535.2478-.02535.375 0 1.5188 1.2312 2.75 2.75 2.75s2.75-1.2312 2.75-2.75c0-.1272-.0086-.2524-.0254-.375z" fill="#6d7f8c"></path>
			<path clip-rule="evenodd" d="m14.75 16c0 1.5188-1.2312 2.75-2.75 2.75s-2.75-1.2312-2.75-2.75 1.2312-2.75 2.75-2.75 2.75 1.2312 2.75 2.75zm-2.75 1.25c.6904 0 1.25-.5596 1.25-1.25s-.5596-1.25-1.25-1.25-1.25.5596-1.25 1.25.5596 1.25 1.25 1.25z" fill="#7c8d99" fill-rule="evenodd"></path>
			<circle cx="12" cy="16" fill="#8697a3" fill-opacity=".5" r="1.25"></circle>
			<path clip-rule="evenodd" d="m8.39565 5.5c-.5135 0-1.01268.19203-1.35854.57159-1.12095 1.23016-2.53711 3.31587-2.53711 5.92841 0 2.6125 1.41616 4.6983 2.53711 5.9284.34586.3796.84504.5716 1.35854.5716h7.20875c.5135 0 1.0126-.192 1.3585-.5716 1.1209-1.2301 2.5371-3.3159 2.5371-5.9284 0-2.61254-1.4162-4.69825-2.5371-5.92841-.3459-.37956-.8451-.57159-1.3585-.57159zm-1.0682-2.5c-.42911 0-.84556.13554-1.16546.42156-1.20722 1.07938-4.16199 4.17139-4.16199 8.57844 0 4.407 2.95477 7.4991 4.16199 8.5784.3199.2861.73635.4216 1.16547.4216h9.34504c.4292 0 .8456-.1355 1.1655-.4216 1.2072-1.0793 4.162-4.1714 4.162-8.5784 0-4.40704-2.9548-7.49906-4.162-8.57844-.3199-.28602-.7363-.42156-1.1655-.42156z" fill="#7c8d99" fill-rule="evenodd"></path>
			<path d="m9.75003 9.13768c.23943.5003.03173 1.10552-.46392 1.35192l-1.62912.8597c-.49566.2463-1.09156.0404-1.33099-.4599s-.03173-1.10555.46392-1.35187l1.62912-.85971c.49566-.24633 1.09156-.04044 1.33099.45986z" fill="#7c8d99"></path>
			<path d="m10 8c0-1.10457.8954-2 2-2h3c1.6569 0 3 1.34315 3 3v1c0 1.1046-.8954 2-2 2h-4c-1.1046 0-2-.8954-2-2z" fill="#9fb3c0"></path>
			<path clip-rule="evenodd" d="m11.8507 3.53389c-.4997.02746-.8798.4106-.8489.85577.0308.44514.4608.78374.9604.75632l.0092-.00049.0303-.00157c.027-.00137.0673-.00336.1193-.00575.1039-.0048.6517.00093.6517.00093l1.3785-.02395s1.0448.01588 1.4768.05451c.2164.01934.3931.04327.5261.07005.1131.02275.1591.04034.1591.04034.8407.47238 1.3364 1.1415 1.969 2.07148.3071.45151.5537.87024.7234 1.17593.0846.15247.1495.27584.1927.35991.0216.04201.0377.07413.0481.09509l.0113.02287.0023.00484c.1974.40967.7304.60007 1.1904.42444.4601-.1757.6733-.65045.4761-1.0604l-.0008-.00162-.0014-.0028-.0044-.00908-.0156-.03164c-.0133-.02693-.0326-.06533-.0576-.1139-.0499-.0971-.1225-.23511-.216-.40351-.1865-.33604-.4578-.797-.7981-1.29727-.6596-.96959-1.3422-1.92667-2.5695-2.61007-.2296-.12785-.4955-.197-.7095-.24006-.2307-.04644-.4866-.07863-.7446-.10169-.5167-.04619-1.6535-.06257-1.6535-.06257s-1.0706.0119-1.4533.02517c-.1918.00665-.5643-.0051-.6749 0-.0553.00255-.0988.00469-.1288.00622l-.0347.00179z" fill="#9fb3c0" fill-rule="evenodd"></path>
			<circle cx="9.75" cy="4.25" fill="#8a9eac" r=".75"></circle>
		</>},
		CurrencyKind::Silver => html! {<>
			<path clip-rule="evenodd" d="m2.49372 21c-.3779 0-.61528-.4138-.42875-.7473l9.50623-16.99932c.189-.33784.6686-.33784.8576 0l9.5062 16.99932c.1866.3335-.0508.7473-.4287.7473zm9.50628-5c.6904 0 1.25-.5596 1.25-1.25s-.5596-1.25-1.25-1.25-1.25.5596-1.25 1.25.5596 1.25 1.25 1.25z" fill="#a59e98" fill-rule="evenodd"></path>
			<path clip-rule="evenodd" d="m2.06497 20.2527c-.18653.3335.05085.7473.42875.7473h19.01258c.3779 0 .6153-.4138.4287-.7473l-9.5062-16.99932c-.189-.33784-.6686-.33784-.8576 0zm10.15323-12.30742c-.0954-.17063-.341-.17063-.4364 0l-5.69332 10.18092c-.09319.1666.02727.372.21819.372h11.38663c.191 0 .3114-.2054.2182-.372z" fill="#99938d" fill-rule="evenodd"></path>
			<path d="m12 17.25c1.3807 0 2.5-1.1193 2.5-2.5 0-.1714-.0173-.3388-.0501-.5005.1919.3752.3001.8002.3001 1.2505 0 1.5188-1.2312 2.75-2.75 2.75s-2.75-1.2312-2.75-2.75c0-.4503.10824-.8753.30011-1.2505-.03286.1617-.05011.3291-.05011.5005 0 1.3807 1.1193 2.5 2.5 2.5z" fill="#857d76"></path>
			<circle cx="12" cy="14.75" fill="#a59e98" fill-opacity=".5" r="1.25"></circle>
			<path d="m11.7819 7.94529c.0954-.17064.341-.17064.4364 0l5.6933 10.18091c.0932.1666-.0273.372-.2182.372h-.133l-5.3421-9.55291c-.0954-.17064-.341-.17064-.4364 0l-5.34216 9.55291h-.13298c-.19093 0-.31139-.2054-.2182-.372z" fill="#b5ada7"></path>
			<path d="m11.6724 11.3586c-.2371-.4211.1496-1.88397.5707-2.12106.1389-.07818.7171.84136.9542 1.26246l.5249 1.1414c.2371.4211.0879.9547-.3331 1.1918-.4211.2371-.9547.0879-1.1918-.3332z" fill="#b5ada7"></path>
			<path d="m11.4292 5.62102c-.2371-.42109.1496-1.88394.5707-2.12103.1389-.07818.7172.84137.9542 1.26246l3.0664 5.44615c.2371.4211.0879.9546-.3332 1.1917s-.9547.0879-1.1917-.3332z" fill="#cec6bf"></path>
		</>},
		CurrencyKind::Copper => html! {<>
			<path clip-rule="evenodd" d="m6.41421 3c-.26521 0-.51957.10536-.7071.29289l-2.41422 2.41422c-.18753.18753-.29289.44189-.29289.7071v11.17159c0 .2652.10536.5196.29289.7071l2.41422 2.4142c.18753.1875.44189.2929.7071.2929h11.17159c.2652 0 .5196-.1054.7071-.2929l2.4142-2.4142c.1875-.1875.2929-.4419.2929-.7071v-11.17159c0-.26521-.1054-.51957-.2929-.7071l-2.4142-2.41422c-.1875-.18753-.4419-.29289-.7071-.29289zm5.58579 10.5c.8284 0 1.5-.6716 1.5-1.5s-.6716-1.5-1.5-1.5-1.5.6716-1.5 1.5.6716 1.5 1.5 1.5z" fill="#ab6e57" fill-rule="evenodd"></path>
			<path d="m10 9c0-1.10457.8954-2 2-2h4c1.1046 0 2 .89543 2 2v1c0 1.1046-.8954 2-2 2h-1c0-1.6569-1.3431-3-3-3-.7684 0-1.4692.28885-2 .76389z" fill="#c2866f"></path>
			<path d="m17.7071 7.94975-.6568-.65686c-.1876-.18753-.4419-.29289-.7072-.29289h-8.68625c-.26521 0-.51957.10536-.7071.29289l-.65686.65686c-.18753.18753-.29289.44189-.29289.7071v-1c0-.26521.10536-.51957.29289-.7071l.65686-.65686c.18753-.18753.44189-.29289.7071-.29289h8.68625c.2653 0 .5196.10536.7072.29289l.6568.65686c.1875.18753.2929.44189.2929.7071v1c0-.26521-.1054-.51957-.2929-.7071z" fill="#9f6854"></path>
			<path d="m14.9585 12.5c-.238 1.4189-1.472 2.5-2.9585 2.5s-2.72048-1.0811-2.95852-2.5c-.02728.1626-.04148.3296-.04148.5 0 1.6569 1.3431 3 3 3s3-1.3431 3-3c0-.1704-.0142-.3374-.0415-.5z" fill="#9f6854"></path>
			<g fill="#c2866f">
				<path clip-rule="evenodd" d="m15 12c0 1.6569-1.3431 3-3 3s-3-1.3431-3-3 1.3431-3 3-3 3 1.3431 3 3zm-3 1.5c.8284 0 1.5-.6716 1.5-1.5s-.6716-1.5-1.5-1.5-1.5.6716-1.5 1.5.6716 1.5 1.5 1.5z" fill-rule="evenodd"></path>
				<circle cx="12" cy="12" fill-opacity=".5" r="1.5"></circle>
				<path clip-rule="evenodd" d="m6.29289 6.94975c-.18753.18753-.29289.44189-.29289.7071v8.68625c0 .2653.10536.5196.29289.7072l.65686.6568c.18753.1875.44189.2929.7071.2929h8.68625c.2653 0 .5196-.1054.7072-.2929l.6568-.6568c.1875-.1876.2929-.4419.2929-.7072v-8.68625c0-.26521-.1054-.51957-.2929-.7071l-.6568-.65686c-.1876-.18753-.4419-.29289-.7072-.29289h-8.68625c-.26521 0-.51957.10536-.7071.29289zm.12132-3.94975c-.26521 0-.51957.10536-.7071.29289l-2.41422 2.41422c-.18753.18753-.29289.44189-.29289.7071v11.17159c0 .2652.10536.5196.29289.7071l2.41422 2.4142c.18753.1875.44189.2929.7071.2929h11.17159c.2652 0 .5196-.1054.7071-.2929l2.4142-2.4142c.1875-.1875.2929-.4419.2929-.7071v-11.17159c0-.26521-.1054-.51957-.2929-.7071l-2.4142-2.41422c-.1875-.18753-.4419-.29289-.7071-.29289z" fill-rule="evenodd"></path>
			</g>
			<path clip-rule="evenodd" d="m10.125 4.5c0-.48325.3918-.875.875-.875h6.0858c.4973 0 .9742.19754 1.3258.54917l1.4142 1.41422c.3517.35163.5492.82854.5492 1.32582v3.58579c0 .4832-.3918.875-.875.875s-.875-.3918-.875-.875v-3.58579c0-.03315-.0132-.06494-.0366-.08838l-1.4142-1.41422c-.0235-.02344-.0553-.03661-.0884-.03661h-6.0858c-.4832 0-.875-.39175-.875-.875z" fill="#e6a58c" fill-rule="evenodd"></path>
			<circle cx="8.375" cy="4.47501" fill="#e5a48c" r=".875"></circle>
		</>},
	};
	let style = format!("width: {size}px; height: {size}px;");
	html! {
		<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="mb-1" style={style}>
			{inner}
		</svg>
	}
}

#[function_component]
pub fn WalletInline() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	let entries = CurrencyKind::all()
		.sorted()
		.rev()
		.filter_map(|coin| {
			let amount = state.persistent().inventory.wallet()[coin];
			match amount {
				0 => None,
				amt => Some(html! {
					<span>{amt} <CoinIcon kind={coin} /></span>
				}),
			}
		})
		.collect::<Vec<_>>();

	let onclick = modal_dispatcher.callback(|_| {
		modal::Action::Open(modal::Props {
			centered: true,
			scrollable: true,
			root_classes: classes!("wallet"),
			content: html! {<Modal />},
			..Default::default()
		})
	});

	html! {
		<span class="wallet-inline ms-auto py-2" {onclick}>
			{match entries.is_empty() {
				true => html! { "Empty Coin Pouch" },
				false => html! {<>{entries}</>},
			}}
		</span>
	}
}

#[function_component]
fn Modal() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let adjustment_wallet = use_state(|| Wallet::default());
	let balance_display = {
		let total_value_gold = state.persistent().inventory.wallet().total_value() / CurrencyKind::Gold.multiplier();
		html! {
			<div>
				<div class="d-flex">
					<h6>{"My Coins"}</h6>
					<span class="ms-2" style="font-size: 0.8rem;">
						{"(est. "}
						{total_value_gold}
						{" GP"}
						<span class="ms-1"><CoinIcon kind={CurrencyKind::Gold}/></span>
						{")"}
					</span>
				</div>
				{CurrencyKind::all().sorted().rev().map(|coin| {
					let amount = state.persistent().inventory.wallet()[coin];
					html! {<>
						<div class="d-flex py-1" style="font-size: 1.25rem;">
							<div class="me-2"><CoinIcon kind={coin} size={24} /></div>
							<div class="my-auto">{coin.to_string()}{" ("}{coin.abbreviation()}{")"}</div>
							<div class="my-auto ms-auto me-3">{amount}</div>
						</div>
						<span class="hr my-1" />
					</>}
				}).collect::<Vec<_>>()}
			</div>
		}
	};
	let adjustment_form = {
		let auto_exchange = state.persistent().settings.currency_auto_exchange;
		let is_empty = adjustment_wallet.is_empty();
		let contains_enough = state.persistent().inventory.wallet().contains(&*adjustment_wallet, auto_exchange);
		let on_change_adj_coin = Callback::from({
			let wallet = adjustment_wallet.clone();
			move |(evt, coin): (web_sys::Event, CurrencyKind)| {
				let Some(target) = evt.target() else { return; };
				let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
				let Ok(value) = input.value().parse::<u64>() else { return; };
				wallet.set({
					let mut wallet = (*wallet).clone();
					wallet[coin] = value;
					wallet
				});
			}
		});
		let onclick_add = Callback::from({
			let adjustments = adjustment_wallet.clone();
			let state = state.clone();
			move |_| {
				let adjustments = {
					let wallet = *adjustments;
					adjustments.set(Wallet::default());
					wallet
				};
				state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
					*persistent.inventory.wallet_mut() += adjustments;
					None
				}));
			}
		});
		let onclick_remove = Callback::from({
			let adjustments = adjustment_wallet.clone();
			let state = state.clone();
			move |_| {
				if !contains_enough {
					return;
				}
				let adjustments = {
					let wallet = *adjustments;
					adjustments.set(Wallet::default());
					wallet
				};
				state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
					let target = persistent.inventory.wallet_mut();
					assert!(target.contains(&adjustments, auto_exchange));
					target.remove(adjustments, auto_exchange);
					None
				}));
			}
		});
		let onclick_clear = Callback::from({
			let wallet = adjustment_wallet.clone();
			move |_| {
				wallet.set(Wallet::default());
			}
		});
		let mut exchange_div_classes = classes!("ms-auto");
		if !auto_exchange {
			exchange_div_classes.push("v-hidden");
		}
		let onclick_exchange = Callback::from({
			let state = state.clone();
			move |_| {
				if !auto_exchange {
					return;
				}
				state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
					persistent.inventory.wallet_mut().normalize();
					None
				}));
			}
		});
		html! {
			<div>
				<div class="d-flex">
					<h6 class="my-auto">{"Adjust Coins"}</h6>
					<div class={exchange_div_classes}>
						<button
							type="button"
							class="btn btn-outline-secondary btn-sm my-1"
							onclick={onclick_exchange}
						>{"Exchange Coins"}</button>
					</div>
				</div>
				<div class="row mb-2 gx-2">
					{CurrencyKind::all().sorted().rev().map(|coin| {
						html! {<>
							<div class="col">
								<div class="d-flex justify-content-center">
									<div class="me-1"><CoinIcon kind={coin} /></div>
									{coin.abbreviation().to_uppercase()}
								</div>
								<input
									type="number" class="form-control text-center p-0"
									min="0"
									value={format!("{}", adjustment_wallet[coin])}
									onkeydown={validate_uint_only()}
									onchange={on_change_adj_coin.reform(move |evt| (evt, coin))}
								/>
							</div>
						</>}
					}).collect::<Vec<_>>()}
				</div>
				<div class="d-flex justify-content-center">
					<button
						type="button" class="btn btn-success btn-sm mx-2"
						disabled={is_empty}
						onclick={onclick_add}
					>{"Add"}</button>
					<button
						type="button" class="btn btn-danger btn-sm mx-2"
						disabled={is_empty || !contains_enough}
						onclick={onclick_remove}
					>{"Remove"}</button>
					<button
						type="button" class="btn btn-secondary btn-sm mx-2"
						disabled={is_empty}
						onclick={onclick_clear}
					>{"Clear"}</button>
				</div>
				<div
					class={contains_enough.then_some("d-none").unwrap_or_default()}
					style="font-size: 0.8rem; font-weight: 650; color: #dc3545;"
				>
					{"Not enough in pouch to remove this amount "}
					{format!("(auto-exchange is {})", match auto_exchange { true => "ON", false => "OFF" })}
				</div>
			</div>
		}
	};
	let settings = {
		html! {
			<div class="collapse" id="settingsCollapse">
				<div class="card card-body mb-3">
					<div class="d-flex">
						<h6>{"Settings"}</h6>
						<button
							type="button"
							class="btn-close ms-auto" aria-label="Close"
							data-bs-toggle="collapse" data-bs-target="#settingsCollapse"
						/>
					</div>
					<AutoExchangeSwitch />
				</div>
			</div>
		}
	};
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Coin Pouch"}</h1>
			<button
				type="button" class="btn btn-secondary btn-sm px-1 py-0 ms-2"
				data-bs-toggle="collapse" data-bs-target="#settingsCollapse"
			>
				<i class="bi bi-gear-fill me-2" />
				{"Settings"}
			</button>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{settings}
			{balance_display}
			{adjustment_form}
		</div>
	</>}
}
