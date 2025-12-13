use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div>
            <h1>{ "RustyCanvas" }</h1>
            <p>{ "Frontend is running!" }</p>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}