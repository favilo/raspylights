use lights::effects::EffectType;
use yew::prelude::*;
// use yew_mdc_widgets::{MdcWidget, Tab, TabBar};

#[derive(Clone, Debug)]
pub(crate) struct Selector {
    ty: &'static str,
}

pub(crate) enum Msg {
    SetType(&'static str),
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub(crate) struct Props {
    #[prop_or("main")]
    pub id: &'static str,

    #[prop_or("Empty")]
    pub ty: &'static str,

    #[prop_or_default]
    pub onclick: Option<Callback<&'static str>>,

    #[prop_or_default]
    pub internal: bool,
}

impl Component for Selector {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self { ty: ctx.props().ty }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        // TODO: We need to figure out what type of message we need to support
        match msg {
            Msg::SetType(ty) => self.ty = ty,
        }
        ctx.props().onclick.as_ref().map(|u| u.emit(self.ty));
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let effects = EffectType::iter_names().collect::<Vec<_>>();
        let options: Vec<_> = effects
            .iter()
            .cloned()
            .filter(|ty| !(ctx.props().internal && ty == &"Rune Script"))
            .map(|i| {
                let id = format!("{}", i);
                let classes = if i == ctx.props().ty {
                    classes!("is-active")
                } else {
                    classes!()
                };
                html! {
                    <li
                        class={ classes }
                        onclick={ ctx.link().callback(move |_| Msg::SetType(i)) }>
                        <a>{id}</a>
                    </li>
                }
            })
            .collect::<_>();

        let id_str = format!("effect_select_{}", ctx.props().id);

        html! {
            <ybc::Tabs
                classes={ classes!(&id_str, "is-center") }
                fullwidth={ true }
                boxed={ true }
                rounded={ true }
            >
                { options }
            </ybc::Tabs>
        }
    }
}
