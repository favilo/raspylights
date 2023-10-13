use yew::{html, Component, Properties};

pub(crate) struct ApplyForm {
    name: String,
}

pub(crate) enum Msg {
    ChangedName(String),
    Save,
    Reset,
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub(crate) struct Props {
    #[prop_or_default]
    pub name: String,

    #[prop_or_default]
    pub on_save: yew::Callback<String>,

    #[prop_or_default]
    pub on_reset: yew::Callback<()>,
}

impl Component for ApplyForm {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self {
            name: ctx.props().name.clone(),
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ChangedName(name) => {
                self.name = name;
            }
            Msg::Save => {
                // TODO: Make this be the only point we are sending details to the backend
                ctx.props().on_save.emit(self.name.clone());
            }
            Msg::Reset => {
                ctx.props().on_reset.emit(());
            }
        }
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        html! {
            <>
                <ybc::Input
                    name="model_name"
                    placeholder="Model Name"
                    value={ self.name.clone() }
                    update={ ctx.link().callback(|name: String| Msg::ChangedName(name)) }
                />
                <ybc::Button
                    onclick={ ctx.link().callback(|_| Msg::Save) }>
                    { "Save" }
                </ybc::Button>
                <ybc::Button
                    onclick={ ctx.link().callback(|_| Msg::Reset) }>
                    { "Reset" }
                </ybc::Button>
            </>
        }
    }

    fn changed(&mut self, ctx: &yew::Context<Self>, old_props: &Self::Properties) -> bool {
        self.name = ctx.props().name.clone();
        self.name != old_props.name
    }
}
