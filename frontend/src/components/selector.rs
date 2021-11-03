use lights::effects::EffectType;
use yew::prelude::*;
use yew_mdc_widgets::{MdcWidget, Tab, TabBar};

#[derive(Clone, Debug)]
pub(crate) struct Selector {
    link: ComponentLink<Self>,

    props: Props,
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
}

impl Component for Selector {
    type Message = Msg;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        // TODO: We need to figure out what type of message we need to support
        match msg {
            Msg::SetType(ty) => self.props.ty = ty,
        }
        self.props.onclick.as_ref().map(|u| u.emit(self.props.ty));
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        let effects = EffectType::iter_names().collect::<Vec<_>>();
        let options: Vec<_> = effects
            .iter()
            .cloned()
            .enumerate()
            .map(|(idx, i)| -> Tab {
                let id = format!("{}", i);
                let mut t = Tab::new().label(id).tab_index(idx as isize);
                if i == self.props.ty {
                    t = t.active();
                }
                t.on_click(self.link.callback(move |_| Msg::SetType(i)))
            })
            .collect::<_>();

        let id_str = format!("effect_select_{}", self.props.id);

        html! {
            <>
                {
                    TabBar::new().id(&id_str).tabs(options)
                }
            </>
        }
    }
}
