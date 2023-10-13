use yew::{Component, Properties, html};

pub(crate) struct HistoryList {
    selected: String,
    history: Vec<String>,
}

pub(crate) enum Msg {
    Selected(usize),
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub(crate) struct Props {
    #[prop_or_default]
    pub selected: String,
}

impl Component for HistoryList {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        // TODO: Fetch history from backend
        Self {
            selected: ctx.props().selected.clone(),
            history: vec![],
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        html! {
            <>
                <label for="model_name">{ "Model Name" }</label>
                <input name="model_name" value={ self.selected.clone() } />
            </>
        }
    }

    fn changed(&mut self, ctx: &yew::Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }
}
