use yew::prelude::*;
use yew_mdc_widgets::Button;
use yewtil::NeqAssign;

#[derive(Clone, Debug)]
pub(crate) struct Balls {
    link: ComponentLink<Self>,

    props: Props,
}

impl Balls {
    fn to_effect(&self) -> lights::effects::Balls {
        self.props.balls.clone()
    }
}

pub(crate) enum Msg {
    SetBall(usize, lights::effects::Ball),
    AddBall,
    RemoveBall(usize),
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub(crate) struct Props {
    #[prop_or_default]
    pub balls: lights::effects::Balls,

    #[prop_or_default]
    pub onupdate: Option<Callback<lights::effects::Balls>>,
}

impl Component for Balls {
    type Message = Msg;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetBall(idx, ball) => self.props.balls.set_ball(idx, ball).expect("correct index"),
            Msg::AddBall => self.props.balls.add_ball(),
            Msg::RemoveBall(idx) => self.props.balls.remove_ball(idx),
        }
        self.props
            .onupdate
            .as_ref()
            .map(|u| u.emit(self.to_effect()));
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let inner: Vec<_> = self
            .props
            .balls
            .balls()
            .iter()
            .enumerate()
            .map(|(idx, b)| {
                html! {
                    <>
                        <super::Ball
                            ball = { b.clone() }
                            onupdate = { Some(self.link.callback(move |ball| {
                                Msg::SetBall(idx, ball)
                            })) }
                         />
                        <ybc::Control>
                            <input type="button"
                                onclick={
                                    self.link.callback(move |_| {
                                        Msg::RemoveBall(idx)
                                    })
                                }
                                value={ "-" }
                            />
                        </ybc::Control>
                        <hr />
                    </>
                }
            })
            .collect();
        let add_button = Button::new().label("+");
        let add_button = add_button.on_click(self.link.callback(move |_| Msg::AddBall));

        html! {
            <>
                { inner }
                { add_button }
            </>
        }
    }
}
