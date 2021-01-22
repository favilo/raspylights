use log::*;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, ToString};
use yew::{
    format::Json,
    prelude::*,
    services::storage::{Area, StorageService},
};

pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    state: State,
}

pub struct State;

pub enum Msg {
   
}

impl Component for App {
    type Message = Msg;

    type Properties = ();

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        todo!()
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        todo!()
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        todo!()
    }

    fn view(&self) -> Html {
        todo!()
    }
}
