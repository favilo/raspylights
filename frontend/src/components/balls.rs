use yew::prelude::*;
 
#[derive(Clone, Debug)]
pub(crate) struct Balls {
    effect: lights::effects::Balls,
}

impl Balls {
    fn to_effect(&self) -> lights::effects::Balls {
        self.effect.clone()
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

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            effect: ctx.props().balls.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetBall(idx, ball) => self.effect.set_ball(idx, ball).expect("correct index"),
            Msg::AddBall => self.effect.add_ball(),
            Msg::RemoveBall(idx) => self.effect.remove_ball(idx),
        }
        ctx.props()
            .onupdate
            .as_ref()
            .map(|u| u.emit(self.to_effect()));
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let inner: Vec<_> = ctx
            .props()
            .balls
            .balls()
            .iter()
            .enumerate()
            .map(|(idx, b)| {
                html! {
                    <>
                        <super::Ball
                            ball = { b.clone() }
                            onupdate = { Some(ctx.link().callback(move |ball| {
                                Msg::SetBall(idx, ball)
                            })) }
                         />
                        <ybc::Control>
                            <ybc::Button 
                                onclick={
                                    ctx.link().callback(move |_| {
                                        Msg::RemoveBall(idx)
                                    })
                                }
                             >{ "-" }</ybc::Button>
                        </ybc::Control>
                        <hr />
                    </>
                }
            })
            .collect();
        let add_button = html! {
            <ybc::Button onclick={ ctx.link().callback(|_: MouseEvent| Msg::AddBall) }>{ "+" }</ybc::Button>
        };
        // let add_button = add_button.on_click(move |_: MouseEvent| {
            // Msg::AddBall;
        // });

        html! {
            <>
                { inner }
                { add_button }
            </>
        }
    }
}
